use chrono::Utc;
use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use rsws_model::config::LogConfig;
use rsws_model::log::*;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub struct LogService {
    db_pool: PgPool,
    config: Arc<tokio::sync::RwLock<LogConfig>>,
}

impl LogService {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            config: Arc::new(tokio::sync::RwLock::new(LogConfig::default())),
        }
    }

    // 更新日志配置
    pub async fn update_config(&self, config: LogConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    // 获取日志配置
    pub async fn get_config(&self) -> LogConfig {
        self.config.read().await.clone()
    }

    // 记录系统日志
    pub async fn log_system(&self, request: CreateSystemLogRequest) -> Result<i64, ServiceError> {
        let config = self.config.read().await;

        // 检查是否启用数据库日志
        if !config.enable_database_logging {
            return Ok(0);
        }

        let log_id = snowflake::next_id();

        sqlx::query!(
            r#"
            INSERT INTO system_logs (
                id, log_level, module, message, context, user_id, admin_id,
                ip_address, user_agent, request_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            log_id,
            request.log_level.to_string(),
            request.module,
            request.message,
            request.context,
            request.user_id,
            request.admin_id,
            request.ip_address,
            request.user_agent,
            request.request_id,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        // 同时输出到控制台
        match request.log_level {
            LogLevel::Debug => debug!(
                "[{}] {}: {}",
                request.module,
                request.message,
                request.context.unwrap_or_default()
            ),
            LogLevel::Info => info!(
                "[{}] {}: {}",
                request.module,
                request.message,
                request.context.unwrap_or_default()
            ),
            LogLevel::Warn => warn!(
                "[{}] {}: {}",
                request.module,
                request.message,
                request.context.unwrap_or_default()
            ),
            LogLevel::Error => error!(
                "[{}] {}: {}",
                request.module,
                request.message,
                request.context.unwrap_or_default()
            ),
            LogLevel::Fatal => error!(
                "[FATAL][{}] {}: {}",
                request.module,
                request.message,
                request.context.unwrap_or_default()
            ),
        }

        Ok(log_id)
    }

    // 记录错误日志
    pub async fn log_error(&self, request: CreateErrorLogRequest) -> Result<i64, ServiceError> {
        let config = self.config.read().await;

        if !config.enable_error_logging {
            return Ok(0);
        }

        let log_id = snowflake::next_id();

        sqlx::query!(
            r#"
            INSERT INTO error_logs (
                id, error_type, error_message, stack_trace, module, function_name,
                user_id, admin_id, request_id, context, ip_address, user_agent, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            log_id,
            request.error_type,
            request.error_message,
            request.stack_trace,
            request.module,
            request.function_name,
            request.user_id,
            request.admin_id,
            request.request_id,
            request.context,
            request.ip_address,
            request.user_agent,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        error!(
            "[ERROR][{}] {}: {}",
            request.module, request.error_type, request.error_message
        );

        Ok(log_id)
    }

    // 记录支付日志
    pub async fn log_payment(&self, request: CreatePaymentLogRequest) -> Result<i64, ServiceError> {
        let config = self.config.read().await;

        if !config.enable_payment_logging {
            return Ok(0);
        }

        let log_id = snowflake::next_id();

        sqlx::query!(
            r#"
            INSERT INTO payment_logs (
                id, transaction_id, order_id, user_id, payment_method, amount, currency,
                status, provider_response, gateway_transaction_id, ip_address, user_agent, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            log_id,
            request.transaction_id,
            request.order_id,
            request.user_id,
            request.payment_method,
            request.amount,
            request.currency,
            request.status,
            request.provider_response,
            request.gateway_transaction_id,
            request.ip_address,
            request.user_agent,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        info!(
            "[PAYMENT] User {} payment {} with {}: {}",
            request.user_id, request.amount, request.payment_method, request.status
        );

        Ok(log_id)
    }

    // 记录请求日志
    pub async fn log_request(&self, request: CreateRequestLogRequest) -> Result<i64, ServiceError> {
        let config = self.config.read().await;

        if !config.enable_request_logging {
            return Ok(0);
        }

        let log_id = snowflake::next_id();

        sqlx::query!(
            r#"
            INSERT INTO request_logs (
                id, request_id, method, path, query_params, headers, body_size,
                user_id, admin_id, ip_address, user_agent, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            log_id,
            request.request_id,
            request.method,
            request.path,
            request.query_params,
            request.headers,
            request.body_size,
            request.user_id,
            request.admin_id,
            request.ip_address,
            request.user_agent,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(log_id)
    }

    // 更新请求日志（添加响应信息）
    pub async fn update_request_log(
        &self,
        log_id: i64,
        request: UpdateRequestLogRequest,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            r#"
            UPDATE request_logs 
            SET response_status = $2, response_size = $3, duration_ms = $4
            WHERE id = $1
            "#,
            log_id,
            request.response_status,
            request.response_size,
            request.duration_ms
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // 查询系统日志
    pub async fn query_system_logs(
        &self,
        request: LogQueryRequest,
    ) -> Result<Vec<SystemLog>, ServiceError> {
        let mut query = "SELECT * FROM system_logs WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(level) = &request.log_level {
            param_count += 1;
            query.push_str(&format!(" AND log_level = ${}", param_count));
            params.push(Box::new(level.to_string()));
        }

        if let Some(module) = &request.module {
            param_count += 1;
            query.push_str(&format!(" AND module = ${}", param_count));
            params.push(Box::new(module.clone()));
        }

        if let Some(user_id) = request.user_id {
            param_count += 1;
            query.push_str(&format!(" AND user_id = ${}", param_count));
            params.push(Box::new(user_id));
        }

        if let Some(start_time) = request.start_time {
            param_count += 1;
            query.push_str(&format!(" AND created_at >= ${}", param_count));
            params.push(Box::new(start_time));
        }

        if let Some(end_time) = request.end_time {
            param_count += 1;
            query.push_str(&format!(" AND created_at <= ${}", param_count));
            params.push(Box::new(end_time));
        }

        query.push_str(" ORDER BY created_at DESC");

        let page_size = request.page_size.unwrap_or(50).min(200) as i64;
        let offset = (request.page.unwrap_or(1) - 1) as i64 * page_size;

        param_count += 1;
        query.push_str(&format!(" LIMIT ${}", param_count));
        params.push(Box::new(page_size));

        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));
        params.push(Box::new(offset));

        // 这里需要使用动态查询，简化版本
        let logs = sqlx::query_as::<_, SystemLog>(&query)
            .fetch_all(&self.db_pool)
            .await?;

        Ok(logs)
    }

    // 清理过期日志
    pub async fn cleanup_old_logs(&self) -> Result<(), ServiceError> {
        let config = self.config.read().await;
        let retention_days = config.log_retention_days;

        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);

        // 清理系统日志
        sqlx::query!("DELETE FROM system_logs WHERE created_at < $1", cutoff_date)
            .execute(&self.db_pool)
            .await?;

        // 清理错误日志
        sqlx::query!("DELETE FROM error_logs WHERE created_at < $1", cutoff_date)
            .execute(&self.db_pool)
            .await?;

        // 清理支付日志（保留更长时间）
        let payment_cutoff = Utc::now() - chrono::Duration::days((retention_days * 3));
        sqlx::query!(
            "DELETE FROM payment_logs WHERE created_at < $1",
            payment_cutoff
        )
        .execute(&self.db_pool)
        .await?;

        // 清理请求日志
        sqlx::query!(
            "DELETE FROM request_logs WHERE created_at < $1",
            cutoff_date
        )
        .execute(&self.db_pool)
        .await?;

        // 清理Webhook日志
        sqlx::query!(
            "DELETE FROM webhook_logs WHERE created_at < $1",
            cutoff_date
        )
        .execute(&self.db_pool)
        .await?;

        info!("Cleaned up logs older than {} days", retention_days);
        Ok(())
    }
}
