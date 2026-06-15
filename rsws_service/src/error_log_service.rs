//! Error log service for tracking application errors and panics

use chrono::{DateTime, Utc};
use rsws_common::error::RswsError;
use serde::Serialize;
use sqlx::{FromRow, PgPool};

/// Error log entry
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ErrorLog {
    pub id: i64,
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<i64>,
    pub context: Option<serde_json::Value>,
    pub source_file: Option<String>,
    pub line_number: Option<i32>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// Error type variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    Panic,
    Exception,
    Timeout,
    Validation,
    Database,
    ExternalApi,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorType::Panic => "panic",
            ErrorType::Exception => "exception",
            ErrorType::Timeout => "timeout",
            ErrorType::Validation => "validation",
            ErrorType::Database => "database",
            ErrorType::ExternalApi => "external_api",
        }
    }
}

/// Request to create an error log
#[derive(Debug, Clone)]
pub struct CreateErrorLogRequest {
    pub error_type: ErrorType,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<i64>,
    pub context: Option<serde_json::Value>,
    pub source_file: Option<String>,
    pub line_number: Option<i32>,
}

/// Query parameters for error logs
#[derive(Debug, Clone, Default)]
pub struct ErrorLogQuery {
    pub error_type: Option<String>,
    pub resolved: Option<bool>,
    pub user_id: Option<i64>,
    pub request_id: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: i64,
    pub page_size: i64,
}

/// Paginated error log response
#[derive(Debug, Clone, Serialize)]
pub struct ErrorLogPage {
    pub items: Vec<ErrorLog>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Request to resolve an error
#[derive(Debug, Clone)]
pub struct ResolveErrorRequest {
    pub error_id: i64,
    pub resolved_by: i64,
}

/// Error statistics
#[derive(Debug, Clone, Serialize)]
pub struct ErrorStats {
    pub total_errors: i64,
    pub unresolved_errors: i64,
    pub errors_by_type: Vec<ErrorTypeCount>,
    pub recent_errors: Vec<ErrorLog>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ErrorTypeCount {
    pub error_type: String,
    pub count: i64,
}

/// Error log service
#[derive(Debug, Clone)]
pub struct ErrorLogService {
    pool: PgPool,
}

impl ErrorLogService {
    /// Create a new error log service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record an error
    pub async fn record_error(&self, req: CreateErrorLogRequest) -> Result<ErrorLog, RswsError> {
        let log = sqlx::query_as::<_, ErrorLog>(
            r#"
            INSERT INTO error_logs 
                (error_type, error_message, stack_trace, request_id, user_id, context, source_file, line_number)
            VALUES 
                ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, error_type, error_message, stack_trace, request_id, user_id, context,
                source_file, line_number, resolved, resolved_at, resolved_by, created_at
            "#
        )
        .bind(req.error_type.as_str())
        .bind(req.error_message)
        .bind(req.stack_trace)
        .bind(req.request_id)
        .bind(req.user_id)
        .bind(req.context)
        .bind(req.source_file)
        .bind(req.line_number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to record error: {}", e)))?;

        Ok(log)
    }

    /// Record error from a standard error type
    pub async fn record_error_from<E: std::fmt::Display>(
        &self,
        error: &E,
        error_type: ErrorType,
        request_id: Option<String>,
        user_id: Option<i64>,
    ) -> Result<ErrorLog, RswsError> {
        self.record_error(CreateErrorLogRequest {
            error_type,
            error_message: error.to_string(),
            request_id,
            user_id,
            stack_trace: None,
            context: None,
            source_file: None,
            line_number: None,
        })
        .await
    }

    /// Query error logs with pagination
    pub async fn query_errors(&self, query: ErrorLogQuery) -> Result<ErrorLogPage, RswsError> {
        let offset = (query.page - 1) * query.page_size;

        let mut conditions = vec!["1=1".to_string()];

        if query.error_type.is_some() {
            conditions.push(format!("error_type = ${}", conditions.len()));
        }
        if query.resolved.is_some() {
            conditions.push(format!("resolved = ${}", conditions.len()));
        }
        if query.user_id.is_some() {
            conditions.push(format!("user_id = ${}", conditions.len()));
        }
        if query.request_id.is_some() {
            conditions.push(format!("request_id = ${}", conditions.len()));
        }
        if query.from_date.is_some() {
            conditions.push(format!("created_at >= ${}", conditions.len()));
        }
        if query.to_date.is_some() {
            conditions.push(format!("created_at <= ${}", conditions.len()));
        }

        let where_clause = conditions.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) FROM error_logs WHERE {}", where_clause);
        let data_sql = format!(
            "SELECT id, error_type, error_message, stack_trace, request_id, user_id, context, \
             source_file, line_number, resolved, resolved_at, resolved_by, created_at \
             FROM error_logs WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            conditions.len(),
            conditions.len() + 1
        );

        // Build and execute count query
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        if let Some(et) = &query.error_type {
            count_query = count_query.bind(et);
        }
        if let Some(r) = query.resolved {
            count_query = count_query.bind(r);
        }
        if let Some(uid) = query.user_id {
            count_query = count_query.bind(uid);
        }
        if let Some(rid) = &query.request_id {
            count_query = count_query.bind(rid);
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
            .map_err(|e| RswsError::internal(format!("Failed to count errors: {}", e)))?;

        // Build and execute data query
        let mut data_query = sqlx::query_as::<_, ErrorLog>(&data_sql);
        if let Some(et) = &query.error_type {
            data_query = data_query.bind(et);
        }
        if let Some(r) = query.resolved {
            data_query = data_query.bind(r);
        }
        if let Some(uid) = query.user_id {
            data_query = data_query.bind(uid);
        }
        if let Some(rid) = &query.request_id {
            data_query = data_query.bind(rid);
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
            .map_err(|e| RswsError::internal(format!("Failed to fetch errors: {}", e)))?;

        Ok(ErrorLogPage {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// Mark an error as resolved
    pub async fn resolve_error(&self, req: ResolveErrorRequest) -> Result<ErrorLog, RswsError> {
        let log = sqlx::query_as::<_, ErrorLog>(
            r#"
            UPDATE error_logs
            SET resolved = TRUE, resolved_at = NOW(), resolved_by = $2
            WHERE id = $1
            RETURNING 
                id, error_type, error_message, stack_trace, request_id, user_id, context,
                source_file, line_number, resolved, resolved_at, resolved_by, created_at
            "#,
        )
        .bind(req.error_id)
        .bind(req.resolved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to resolve error: {}", e)))?;

        Ok(log)
    }

    /// Get error statistics
    pub async fn get_stats(&self, hours: i64) -> Result<ErrorStats, RswsError> {
        let total_errors: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM error_logs WHERE created_at > NOW() - INTERVAL '1 hour' * $1",
        )
        .bind(hours)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let unresolved_errors: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM error_logs WHERE resolved = FALSE")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let errors_by_type = sqlx::query_as::<_, ErrorTypeCount>(
            r#"
            SELECT error_type, COUNT(*) as count
            FROM error_logs
            WHERE created_at > NOW() - INTERVAL '1 hour' * $1
            GROUP BY error_type
            ORDER BY count DESC
            "#,
        )
        .bind(hours)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let recent_errors = sqlx::query_as::<_, ErrorLog>(
            r#"
            SELECT id, error_type, error_message, stack_trace, request_id, user_id, context,
                   source_file, line_number, resolved, resolved_at, resolved_by, created_at
            FROM error_logs
            WHERE resolved = FALSE
            ORDER BY created_at DESC
            LIMIT 10
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(ErrorStats {
            total_errors,
            unresolved_errors,
            errors_by_type,
            recent_errors,
        })
    }

    /// Get error by ID
    pub async fn get_by_id(&self, id: i64) -> Result<ErrorLog, RswsError> {
        let log = sqlx::query_as::<_, ErrorLog>(
            r#"
            SELECT id, error_type, error_message, stack_trace, request_id, user_id, context,
                   source_file, line_number, resolved, resolved_at, resolved_by, created_at
            FROM error_logs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to fetch error: {}", e)))?
        .ok_or_else(|| RswsError::not_found("Error log not found"))?;

        Ok(log)
    }
}
