use chrono::Utc;
use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use rsws_model::log::{CreateRequestLogRequest, RequestLog, UpdateRequestLogRequest};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

pub struct RequestService {
    db_pool: PgPool,
    log_service: Arc<crate::log_service::LogService>,
}

impl RequestService {
    pub fn new(db_pool: PgPool, log_service: Arc<crate::log_service::LogService>) -> Self {
        Self {
            db_pool,
            log_service,
        }
    }

    // 生成请求ID - 使用雪花ID替代UUID
    pub fn generate_request_id() -> String {
        snowflake::next_id().to_string()
    }

    // 记录请求开始
    pub async fn log_request_start(
        &self,
        request: CreateRequestLogRequest,
    ) -> Result<i64, ServiceError> {
        self.log_service.log_request(request).await
    }

    // 更新请求结束信息
    pub async fn log_request_end(
        &self,
        log_id: i64,
        request: UpdateRequestLogRequest,
    ) -> Result<(), ServiceError> {
        self.log_service.update_request_log(log_id, request).await
    }

    // 获取请求统计信息
    pub async fn get_request_stats(
        &self,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
    ) -> Result<RequestStats, ServiceError> {
        let start = start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));
        let end = end_time.unwrap_or_else(|| Utc::now());

        // 总请求数
        let total_requests = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM request_logs WHERE created_at BETWEEN $1 AND $2",
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        // 成功请求数（2xx状态码）
        let successful_requests = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM request_logs WHERE created_at BETWEEN $1 AND $2 AND response_status BETWEEN 200 AND 299",
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        // 错误请求数（4xx, 5xx状态码）
        let error_requests = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM request_logs WHERE created_at BETWEEN $1 AND $2 AND response_status >= 400",
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        // 平均响应时间
        let avg_response_time = sqlx::query_scalar!(
            "SELECT AVG(duration_ms) FROM request_logs WHERE created_at BETWEEN $1 AND $2 AND duration_ms IS NOT NULL",
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(Some(0.0))
        .unwrap_or(0.0);

        // 最慢的请求
        let slowest_requests = sqlx::query_as::<_, SlowRequest>(
            r#"
            SELECT path, method, duration_ms, created_at
            FROM request_logs 
            WHERE created_at BETWEEN $1 AND $2 AND duration_ms IS NOT NULL
            ORDER BY duration_ms DESC
            LIMIT 10
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.db_pool)
        .await?;

        // 热门路径统计
        let popular_paths = sqlx::query_as::<_, PathStats>(
            r#"
            SELECT path, COUNT(*) as request_count, AVG(duration_ms) as avg_duration
            FROM request_logs 
            WHERE created_at BETWEEN $1 AND $2
            GROUP BY path
            ORDER BY request_count DESC
            LIMIT 20
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.db_pool)
        .await?;

        // 状态码分布
        let status_distribution = sqlx::query_as::<_, StatusStats>(
            r#"
            SELECT response_status, COUNT(*) as count
            FROM request_logs 
            WHERE created_at BETWEEN $1 AND $2 AND response_status IS NOT NULL
            GROUP BY response_status
            ORDER BY count DESC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(RequestStats {
            total_requests,
            successful_requests,
            error_requests,
            avg_response_time,
            slowest_requests,
            popular_paths,
            status_distribution,
            start_time: start,
            end_time: end,
        })
    }

    // 获取用户请求历史
    pub async fn get_user_request_history(
        &self,
        user_id: i64,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<RequestLog>, ServiceError> {
        let page_size = page_size.unwrap_or(50).min(200) as i64;
        let offset = (page.unwrap_or(1) - 1) as i64 * page_size;

        let logs = sqlx::query_as::<_, RequestLog>(
            r#"
            SELECT * FROM request_logs 
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(logs)
    }

    // 检测异常请求模式
    pub async fn detect_anomalies(&self) -> Result<Vec<AnomalyReport>, ServiceError> {
        let mut anomalies = Vec::new();
        let now = Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let one_day_ago = now - chrono::Duration::days(1);

        // 检测高频请求IP
        let high_frequency_ips = sqlx::query_as::<_, IpFrequency>(
            r#"
            SELECT ip_address, COUNT(*) as request_count
            FROM request_logs 
            WHERE created_at >= $1 AND ip_address IS NOT NULL
            GROUP BY ip_address
            HAVING COUNT(*) > 1000
            ORDER BY request_count DESC
            "#,
        )
        .bind(one_hour_ago)
        .fetch_all(&self.db_pool)
        .await?;

        for ip_freq in high_frequency_ips {
            anomalies.push(AnomalyReport {
                anomaly_type: "high_frequency_ip".to_string(),
                description: format!(
                    "IP {} made {} requests in the last hour",
                    ip_freq.ip_address, ip_freq.request_count
                ),
                severity: if ip_freq.request_count > 5000 {
                    "high"
                } else {
                    "medium"
                }
                .to_string(),
                detected_at: now,
                metadata: serde_json::json!({
                    "ip_address": ip_freq.ip_address,
                    "request_count": ip_freq.request_count,
                    "time_window": "1_hour"
                }),
            });
        }

        // 检测高错误率路径
        let high_error_paths = sqlx::query_as::<_, PathErrorRate>(
            r#"
            SELECT path, 
                   COUNT(*) as total_requests,
                   COUNT(CASE WHEN response_status >= 400 THEN 1 END) as error_requests,
                   (COUNT(CASE WHEN response_status >= 400 THEN 1 END) * 100.0 / COUNT(*)) as error_rate
            FROM request_logs 
            WHERE created_at >= $1
            GROUP BY path
            HAVING COUNT(*) > 100 AND (COUNT(CASE WHEN response_status >= 400 THEN 1 END) * 100.0 / COUNT(*)) > 50
            ORDER BY error_rate DESC
            "#
        )
        .bind(one_day_ago)
        .fetch_all(&self.db_pool)
        .await?;

        for path_error in high_error_paths {
            anomalies.push(AnomalyReport {
                anomaly_type: "high_error_rate".to_string(),
                description: format!(
                    "Path {} has {:.1}% error rate ({}/{} requests)",
                    path_error.path,
                    path_error.error_rate,
                    path_error.error_requests,
                    path_error.total_requests
                ),
                severity: if path_error.error_rate > 80.0 {
                    "high"
                } else {
                    "medium"
                }
                .to_string(),
                detected_at: now,
                metadata: serde_json::json!({
                    "path": path_error.path,
                    "error_rate": path_error.error_rate,
                    "total_requests": path_error.total_requests,
                    "error_requests": path_error.error_requests
                }),
            });
        }

        Ok(anomalies)
    }

    // 清理旧的请求日志
    pub async fn cleanup_old_logs(&self, retention_days: i32) -> Result<u64, ServiceError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);

        let result = sqlx::query!(
            "DELETE FROM request_logs WHERE created_at < $1",
            cutoff_date
        )
        .execute(&self.db_pool)
        .await?;

        info!("Cleaned up {} old request logs", result.rows_affected());
        Ok(result.rows_affected())
    }
}

// 统计数据结构
#[derive(Debug, serde::Serialize)]
pub struct RequestStats {
    pub total_requests: i64,
    pub successful_requests: i64,
    pub error_requests: i64,
    pub avg_response_time: f64,
    pub slowest_requests: Vec<SlowRequest>,
    pub popular_paths: Vec<PathStats>,
    pub status_distribution: Vec<StatusStats>,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: chrono::DateTime<Utc>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct SlowRequest {
    pub path: String,
    pub method: String,
    pub duration_ms: Option<i32>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PathStats {
    pub path: String,
    pub request_count: i64,
    pub avg_duration: Option<f64>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct StatusStats {
    pub response_status: Option<i32>,
    pub count: i64,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct IpFrequency {
    pub ip_address: String,
    pub request_count: i64,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PathErrorRate {
    pub path: String,
    pub total_requests: i64,
    pub error_requests: i64,
    pub error_rate: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct AnomalyReport {
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub detected_at: chrono::DateTime<Utc>,
    pub metadata: serde_json::Value,
}
