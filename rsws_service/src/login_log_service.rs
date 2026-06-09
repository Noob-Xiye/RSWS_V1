//! Login log service for tracking authentication events

use chrono::{DateTime, Utc};
use rsws_common::error::RswsError;
use serde::Serialize;
use sqlx::{FromRow, PgPool, Row};
use std::net::IpAddr;

/// Login log entry
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct LoginLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub login_type: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_info: Option<serde_json::Value>,
    pub fail_reason: Option<String>,
    pub request_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Login type variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginType {
    Password,
    ApiKey,
    OAuth,
    EmailLink,
}

impl LoginType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoginType::Password => "password",
            LoginType::ApiKey => "api_key",
            LoginType::OAuth => "oauth",
            LoginType::EmailLink => "email_link",
        }
    }
}

/// Login status variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginStatus {
    Success,
    Failed,
    Locked,
    Expired,
}

impl LoginStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoginStatus::Success => "success",
            LoginStatus::Failed => "failed",
            LoginStatus::Locked => "locked",
            LoginStatus::Expired => "expired",
        }
    }
}

/// Request to create a login log
#[derive(Debug, Clone)]
pub struct CreateLoginLogRequest {
    pub user_id: Option<i64>,
    pub login_type: LoginType,
    pub status: LoginStatus,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub device_info: Option<serde_json::Value>,
    pub fail_reason: Option<String>,
    pub request_id: Option<String>,
}

/// Query parameters for login logs
#[derive(Debug, Clone, Default)]
pub struct LoginLogQuery {
    pub user_id: Option<i64>,
    pub status: Option<String>,
    pub login_type: Option<String>,
    pub ip_address: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: i64,
    pub page_size: i64,
}

/// Paginated login log response
#[derive(Debug, Clone, Serialize)]
pub struct LoginLogPage {
    pub items: Vec<LoginLog>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Login log service
#[derive(Debug, Clone)]
pub struct LoginLogService {
    pool: PgPool,
}

impl LoginLogService {
    /// Create a new login log service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record a login attempt
    pub async fn record_login(&self, req: CreateLoginLogRequest) -> Result<LoginLog, RswsError> {
        let ip_str = req.ip_address.map(|ip| ip.to_string());
        
        let row = sqlx::query(
            r#"
            INSERT INTO login_logs 
                (user_id, login_type, status, ip_address, user_agent, device_info, fail_reason, request_id)
            VALUES 
                ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, user_id, login_type, status, ip_address, user_agent, device_info, 
                fail_reason, request_id, created_at
            "#
        )
        .bind(req.user_id)
        .bind(req.login_type.as_str())
        .bind(req.status.as_str())
        .bind(ip_str)
        .bind(req.user_agent)
        .bind(req.device_info)
        .bind(req.fail_reason)
        .bind(req.request_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to record login: {}", e)))?;

        Ok(LoginLog {
            id: row.try_get("id").unwrap_or_default(),
            user_id: row.try_get("user_id").ok(),
            login_type: row.try_get("login_type").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            ip_address: row.try_get("ip_address").ok(),
            user_agent: row.try_get("user_agent").ok(),
            device_info: row.try_get("device_info").ok(),
            fail_reason: row.try_get("fail_reason").ok(),
            request_id: row.try_get("request_id").ok(),
            created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
        })
    }

    /// Query login logs with pagination
    pub async fn query_logs(&self, query: LoginLogQuery) -> Result<LoginLogPage, RswsError> {
        let offset = (query.page - 1) * query.page_size;
        
        let mut conditions: Vec<String> = vec!["1=1".to_string()];
        let mut param_idx = 1;

        let user_id_param = if let Some(uid) = query.user_id {
            conditions.push(format!("user_id = ${}", param_idx));
            param_idx += 1;
            Some(uid)
        } else {
            None
        };

        let status_param = if let Some(ref status) = query.status {
            conditions.push(format!("status = ${}", param_idx));
            param_idx += 1;
            Some(status.clone())
        } else {
            None
        };

        let login_type_param = if let Some(ref login_type) = query.login_type {
            conditions.push(format!("login_type = ${}", param_idx));
            param_idx += 1;
            Some(login_type.clone())
        } else {
            None
        };

        let ip_param = if let Some(ref ip) = query.ip_address {
            conditions.push(format!("ip_address = ${}", param_idx));
            param_idx += 1;
            Some(ip.clone())
        } else {
            None
        };

        let from_date_param = if let Some(from) = query.from_date {
            conditions.push(format!("created_at >= ${}", param_idx));
            param_idx += 1;
            Some(from)
        } else {
            None
        };

        let to_date_param = if let Some(to) = query.to_date {
            conditions.push(format!("created_at <= ${}", param_idx));
            param_idx += 1;
            Some(to)
        } else {
            None
        };
        
        let where_clause = conditions.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) FROM login_logs WHERE {}", where_clause);
        let data_sql = format!(
            "SELECT id, user_id, login_type, status, ip_address, user_agent, device_info, \
             fail_reason, request_id, created_at FROM login_logs WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_idx, param_idx + 1
        );
        
        // Build and execute count query
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        if let Some(uid) = user_id_param { count_query = count_query.bind(uid); }
        if let Some(ref s) = status_param { count_query = count_query.bind(s); }
        if let Some(ref lt) = login_type_param { count_query = count_query.bind(lt); }
        if let Some(ref ip) = ip_param { count_query = count_query.bind(ip); }
        if let Some(ref from) = from_date_param { count_query = count_query.bind(from); }
        if let Some(ref to) = to_date_param { count_query = count_query.bind(to); }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count login logs: {}", e)))?;
        
        // Build and execute data query
        let mut data_query = sqlx::query_as::<_, LoginLog>(&data_sql);
        if let Some(uid) = user_id_param { data_query = data_query.bind(uid); }
        if let Some(ref s) = status_param { data_query = data_query.bind(s); }
        if let Some(ref lt) = login_type_param { data_query = data_query.bind(lt); }
        if let Some(ref ip) = ip_param { data_query = data_query.bind(ip); }
        if let Some(ref from) = from_date_param { data_query = data_query.bind(from); }
        if let Some(ref to) = to_date_param { data_query = data_query.bind(to); }

        let items = data_query
            .bind(query.page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to fetch login logs: {}", e)))?;
        
        Ok(LoginLogPage {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// Get recent login attempts for a user
    pub async fn get_recent_logins(&self, user_id: i64, limit: i64) -> Result<Vec<LoginLog>, RswsError> {
        let logs = sqlx::query_as::<_, LoginLog>(
            r#"
            SELECT id, user_id, login_type, status, ip_address, user_agent, device_info,
                   fail_reason, request_id, created_at
            FROM login_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to fetch recent logins: {}", e)))?;
        
        Ok(logs)
    }

    /// Check for suspicious login patterns (brute force detection)
    pub async fn check_suspicious_activity(&self, ip: &str, minutes: i64) -> Result<(i64, i64), RswsError> {
        let result = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_attempts,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_attempts
            FROM login_logs
            WHERE ip_address = $1
              AND created_at > NOW() - INTERVAL '1 minute' * $2
            "#
        )
        .bind(ip)
        .bind(minutes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to check suspicious activity: {}", e)))?;
        
        let total: i64 = result.try_get("total_attempts").unwrap_or(0);
        let failed: i64 = result.try_get("failed_attempts").unwrap_or(0);
        
        Ok((total, failed))
    }
}
