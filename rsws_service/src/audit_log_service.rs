//! Audit log service for tracking sensitive operations

use chrono::{DateTime, Utc};
use rsws_common::error::RswsError;
use serde::Serialize;
use serde_json::Value;
use sqlx::FromRow;
use sqlx::PgPool;
use std::net::IpAddr;

/// Audit log entry
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<i64>,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub change_summary: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub verified_by: Option<String>,
    pub risk_level: String,
    pub created_at: DateTime<Utc>,
}

/// Action types for audit logs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    // User actions
    UserLogin,
    UserLogout,
    UserRegister,
    UserUpdate,
    PasswordChange,
    // Wallet actions
    WalletCreate,
    WalletUpdate,
    Withdraw,
    Deposit,
    // Order actions
    OrderCreate,
    OrderCancel,
    OrderComplete,
    OrderRefund,
    // Resource actions
    ResourceCreate,
    ResourceUpdate,
    ResourceDelete,
    // Admin actions
    PermissionChange,
    RoleChange,
    ConfigUpdate,
    // API Key actions
    ApiKeyCreate,
    ApiKeyRevoke,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::UserLogin => "user_login",
            AuditAction::UserLogout => "user_logout",
            AuditAction::UserRegister => "user_register",
            AuditAction::UserUpdate => "user_update",
            AuditAction::PasswordChange => "password_change",
            AuditAction::WalletCreate => "wallet_create",
            AuditAction::WalletUpdate => "wallet_update",
            AuditAction::Withdraw => "withdraw",
            AuditAction::Deposit => "deposit",
            AuditAction::OrderCreate => "order_create",
            AuditAction::OrderCancel => "order_cancel",
            AuditAction::OrderComplete => "order_complete",
            AuditAction::OrderRefund => "order_refund",
            AuditAction::ResourceCreate => "resource_create",
            AuditAction::ResourceUpdate => "resource_update",
            AuditAction::ResourceDelete => "resource_delete",
            AuditAction::PermissionChange => "permission_change",
            AuditAction::RoleChange => "role_change",
            AuditAction::ConfigUpdate => "config_update",
            AuditAction::ApiKeyCreate => "api_key_create",
            AuditAction::ApiKeyRevoke => "api_key_revoke",
        }
    }
}

/// Resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    User,
    Admin,
    Order,
    Wallet,
    Resource,
    Config,
    ApiKey,
    System,
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::User => "user",
            ResourceType::Admin => "admin",
            ResourceType::Order => "order",
            ResourceType::Wallet => "wallet",
            ResourceType::Resource => "resource",
            ResourceType::Config => "config",
            ResourceType::ApiKey => "api_key",
            ResourceType::System => "system",
        }
    }
}

/// Risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

/// Verification methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMethod {
    TwoFactor,
    Email,
    Sms,
    Password,
    ApiKey,
}

impl VerificationMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationMethod::TwoFactor => "2fa",
            VerificationMethod::Email => "email",
            VerificationMethod::Sms => "sms",
            VerificationMethod::Password => "password",
            VerificationMethod::ApiKey => "api_key",
        }
    }
}

/// Request to create an audit log
#[derive(Debug, Clone)]
pub struct CreateAuditLogRequest {
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub action: AuditAction,
    pub resource_type: ResourceType,
    pub resource_id: Option<i64>,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub change_summary: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub verified_by: Option<VerificationMethod>,
    pub risk_level: RiskLevel,
}

/// Query parameters for audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditLogQuery {
    pub user_id: Option<i64>,
    pub admin_id: Option<i64>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub risk_level: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: i64,
    pub page_size: i64,
}

/// Paginated audit log response
#[derive(Debug, Clone, Serialize)]
pub struct AuditLogPage {
    pub items: Vec<AuditLog>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize)]
pub struct AuditStats {
    pub total_operations: i64,
    pub high_risk_operations: i64,
    pub operations_by_action: Vec<ActionCount>,
    pub recent_high_risk: Vec<AuditLog>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ActionCount {
    pub action: String,
    pub count: i64,
}

/// Audit log service
#[derive(Debug, Clone)]
pub struct AuditLogService {
    pool: PgPool,
}

impl AuditLogService {
    /// Create a new audit log service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record an audit event
    pub async fn record(&self, req: CreateAuditLogRequest) -> Result<AuditLog, RswsError> {
        let ip_str = req.ip_address.map(|ip| ip.to_string());
        let verified_str = req.verified_by.map(|v| v.as_str().to_string());

        let log = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs 
                (user_id, admin_id, action, resource_type, resource_id, old_value, new_value, 
                 change_summary, ip_address, user_agent, verified_by, risk_level)
            VALUES 
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING 
                id, user_id, admin_id, action, resource_type, resource_id, old_value, new_value,
                change_summary, ip_address, user_agent, verified_by, risk_level, created_at
            "#,
        )
        .bind(req.user_id)
        .bind(req.admin_id)
        .bind(req.action.as_str())
        .bind(req.resource_type.as_str())
        .bind(req.resource_id)
        .bind(req.old_value)
        .bind(req.new_value)
        .bind(req.change_summary)
        .bind(ip_str)
        .bind(req.user_agent)
        .bind(verified_str)
        .bind(req.risk_level.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to record audit: {}", e)))?;

        Ok(log)
    }

    /// Quick record for simple actions
    pub async fn record_simple(
        &self,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: Option<i64>,
        user_id: Option<i64>,
        summary: &str,
    ) -> Result<AuditLog, RswsError> {
        self.record(CreateAuditLogRequest {
            action,
            resource_type,
            resource_id,
            user_id,
            change_summary: Some(summary.to_string()),
            risk_level: RiskLevel::Low,
            admin_id: None,
            old_value: None,
            new_value: None,
            ip_address: None,
            user_agent: None,
            verified_by: None,
        })
        .await
    }

    /// Query audit logs with pagination
    pub async fn query(&self, query: AuditLogQuery) -> Result<AuditLogPage, RswsError> {
        let offset = (query.page - 1) * query.page_size;

        let mut conditions = vec!["1=1".to_string()];

        if query.user_id.is_some() {
            conditions.push(format!("user_id = ${}", conditions.len()));
        }
        if query.admin_id.is_some() {
            conditions.push(format!("admin_id = ${}", conditions.len()));
        }
        if query.action.is_some() {
            conditions.push(format!("action = ${}", conditions.len()));
        }
        if query.resource_type.is_some() {
            conditions.push(format!("resource_type = ${}", conditions.len()));
        }
        if query.resource_id.is_some() {
            conditions.push(format!("resource_id = ${}", conditions.len()));
        }
        if query.risk_level.is_some() {
            conditions.push(format!("risk_level = ${}", conditions.len()));
        }
        if query.from_date.is_some() {
            conditions.push(format!("created_at >= ${}", conditions.len()));
        }
        if query.to_date.is_some() {
            conditions.push(format!("created_at <= ${}", conditions.len()));
        }

        let where_clause = conditions.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) FROM audit_logs WHERE {}", where_clause);
        let data_sql = format!(
            "SELECT id, user_id, admin_id, action, resource_type, resource_id, old_value, \
             new_value, change_summary, ip_address, user_agent, verified_by, risk_level, created_at \
             FROM audit_logs WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            conditions.len(),
            conditions.len() + 1
        );
        // Build and execute count query
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        if let Some(uid) = query.user_id {
            count_query = count_query.bind(uid);
        }
        if let Some(aid) = query.admin_id {
            count_query = count_query.bind(aid);
        }
        if let Some(a) = &query.action {
            count_query = count_query.bind(a);
        }
        if let Some(rt) = &query.resource_type {
            count_query = count_query.bind(rt);
        }
        if let Some(rid) = query.resource_id {
            count_query = count_query.bind(rid);
        }
        if let Some(rl) = &query.risk_level {
            count_query = count_query.bind(rl);
        }
        if let Some(from) = query.from_date {
            count_query = count_query.bind(from);
        }
        if let Some(to) = query.to_date {
            count_query = count_query.bind(to);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count audit logs: {}", e)))?;

        // Build and execute data query
        let mut data_query = sqlx::query_as::<_, AuditLog>(&data_sql);
        if let Some(uid) = query.user_id {
            data_query = data_query.bind(uid);
        }
        if let Some(aid) = query.admin_id {
            data_query = data_query.bind(aid);
        }
        if let Some(a) = &query.action {
            data_query = data_query.bind(a);
        }
        if let Some(rt) = &query.resource_type {
            data_query = data_query.bind(rt);
        }
        if let Some(rid) = query.resource_id {
            data_query = data_query.bind(rid);
        }
        if let Some(rl) = &query.risk_level {
            data_query = data_query.bind(rl);
        }
        if let Some(from) = query.from_date {
            data_query = data_query.bind(from);
        }
        if let Some(to) = query.to_date {
            data_query = data_query.bind(to);
        }

        let items = data_query
            .bind(query.page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to fetch audit logs: {}", e)))?;

        Ok(AuditLogPage {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// Get audit statistics
    pub async fn get_stats(&self, hours: i64) -> Result<AuditStats, RswsError> {
        let total_operations: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs WHERE created_at > NOW() - INTERVAL '1 hour' * $1",
        )
        .bind(hours)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let high_risk_operations: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs WHERE risk_level IN ('high', 'critical') AND created_at > NOW() - INTERVAL '1 hour' * $1"
        )
        .bind(hours)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let operations_by_action = sqlx::query_as::<_, ActionCount>(
            r#"
            SELECT action, COUNT(*) as count
            FROM audit_logs
            WHERE created_at > NOW() - INTERVAL '1 hour' * $1
            GROUP BY action
            ORDER BY count DESC
            "#,
        )
        .bind(hours)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let recent_high_risk = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT id, user_id, admin_id, action, resource_type, resource_id, old_value, new_value,
                   change_summary, ip_address, user_agent, verified_by, risk_level, created_at
            FROM audit_logs
            WHERE risk_level IN ('high', 'critical')
            ORDER BY created_at DESC
            LIMIT 10
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(AuditStats {
            total_operations,
            high_risk_operations,
            operations_by_action,
            recent_high_risk,
        })
    }

    /// Get resource history
    pub async fn get_resource_history(
        &self,
        resource_type: ResourceType,
        resource_id: i64,
        limit: i64,
    ) -> Result<Vec<AuditLog>, RswsError> {
        let logs = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT id, user_id, admin_id, action, resource_type, resource_id, old_value, new_value,
                   change_summary, ip_address, user_agent, verified_by, risk_level, created_at
            FROM audit_logs
            WHERE resource_type = $1 AND resource_id = $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(resource_type.as_str())
        .bind(resource_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to fetch resource history: {}", e)))?;

        Ok(logs)
    }
}
