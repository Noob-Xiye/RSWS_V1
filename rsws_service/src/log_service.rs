//! 日志服务
//!
//! 日志记录 + 日志配置管理
//! Log + Log Config 存 DB，后台可动态管理

use rsws_common::error::RswsError;
use rsws_common::snowflake;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// 日志级别（按严重程度排序）
const LOG_LEVELS: [&str; 5] = ["trace", "debug", "info", "warn", "error"];

fn parse_level_index(level: &str) -> usize {
    LOG_LEVELS.iter().position(|&l| l == level).unwrap_or(2)
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LogConfig {
    pub id: i64,
    pub config_key: String,
    pub config_value: String,
    /// 配置类型: string, bool, number
    #[sqlx(default)]
    pub config_type: Option<String>,
    /// 日志级别: trace, debug, info, warn, error（仅当 config_key 为日志节点时有效）
    #[sqlx(default)]
    pub level: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 更新日志配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogConfigRequest {
    pub config_key: String,
    pub config_value: String,
    pub config_type: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub level: Option<String>,
}

/// 日志服务
pub struct LogService {
    pool: PgPool,
}

/// 查询系统日志的参数
pub struct LogQueryParams {
    pub level: Option<String>,
    pub module: Option<String>,
    pub user_id: Option<i64>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl LogService {
    /// 创建日志服务实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ==================== 日志记录 ====================

    /// 检查全局日志级别是否启用
    pub async fn is_log_level_enabled(&self, level: &str) -> bool {
        self.check_module_level("log.level", level)
            .await
            .unwrap_or_else(|| {
                // 默认 info 级别以上都记录
                parse_level_index(level) >= parse_level_index("info")
            })
    }

    /// 检查指定模块的日志级别是否启用
    pub async fn is_module_log_enabled(&self, module_key: &str, level: &str) -> bool {
        match self.check_module_level(module_key, level).await {
            Some(v) => v,
            None => self.is_log_level_enabled(level).await,
        }
    }

    /// 内部：检查某个 log_configs key 的 level 配置是否通过
    /// - 如果配置为 bool 类型且为 "false"，返回 false（模块关闭）
    /// - 如果配置为日志级别字符串，比较当前 level 是否 >= 配置级别
    /// - 如果配置不存在，返回 None（走默认逻辑）
    async fn check_module_level(&self, key: &str, level: &str) -> Option<bool> {
        let config = self.get_log_config(key).await.ok().flatten()?;

        if !config.is_active {
            return Some(false);
        }

        let config_type = config.config_type.as_deref().unwrap_or("string");

        if config_type == "bool" {
            return Some(config.config_value == "true");
        }

        // 按日志级别比较
        let config_level = config.level.as_deref().unwrap_or("info");
        let config_idx = parse_level_index(config_level);
        let level_idx = parse_level_index(level);
        Some(level_idx >= config_idx)
    }

    /// 记录系统日志（按模块级别过滤）
    #[allow(clippy::too_many_arguments)]
    pub async fn log_system(
        &self,
        level: &str,
        module: &str,
        message: &str,
        context: Option<Value>,
        user_id: Option<i64>,
        admin_id: Option<i64>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        request_id: Option<&str>,
    ) -> Result<(), RswsError> {
        // 按模块级别过滤
        let module_key = format!("log.module.{}", module);
        if !self.is_module_log_enabled(&module_key, level).await {
            return Ok(());
        }

        let id = snowflake::next_id();
        sqlx::query(
            r#"INSERT INTO system_logs (id, log_level, module, message, context, user_id, admin_id, ip_address, user_agent, request_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
        )
        .bind(id)
        .bind(level)
        .bind(module)
        .bind(message)
        .bind(context)
        .bind(user_id)
        .bind(admin_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(request_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to log: {}", e)))?;

        Ok(())
    }

    /// 记录错误日志（总是记录，不受过滤影响）
    #[allow(clippy::too_many_arguments)]
    pub async fn log_error(
        &self,
        error_type: &str,
        error_message: &str,
        stack_trace: Option<&str>,
        module: &str,
        function_name: Option<&str>,
        user_id: Option<i64>,
        admin_id: Option<i64>,
        request_id: Option<&str>,
        context: Option<Value>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), RswsError> {
        // 检查 log.error 开关和模块级别
        if !self.is_module_log_enabled("log.error", "error").await {
            return Ok(());
        }

        let id = snowflake::next_id();
        sqlx::query(
            r#"INSERT INTO error_logs (id, error_type, error_message, stack_trace, module, function_name, user_id, admin_id, request_id, context, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(id)
        .bind(error_type)
        .bind(error_message)
        .bind(stack_trace)
        .bind(module)
        .bind(function_name)
        .bind(user_id)
        .bind(admin_id)
        .bind(request_id)
        .bind(context)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to log error: {}", e)))?;

        Ok(())
    }

    /// 记录支付日志
    #[allow(clippy::too_many_arguments)]
    pub async fn log_payment(
        &self,
        transaction_id: Option<&str>,
        order_id: Option<i64>,
        user_id: i64,
        payment_method: &str,
        amount: i64,
        currency: &str,
        status: &str,
        provider_response: Option<Value>,
        gateway_transaction_id: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), RswsError> {
        // 按 log.payment 和 log.module.payment 过滤
        if !self.is_module_log_enabled("log.payment", "info").await {
            return Ok(());
        }

        let id = snowflake::next_id();
        sqlx::query(
            r#"INSERT INTO payment_logs (id, transaction_id, order_id, user_id, payment_method, amount, currency, status, provider_response, gateway_transaction_id, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(id)
        .bind(transaction_id)
        .bind(order_id)
        .bind(user_id)
        .bind(payment_method)
        .bind(amount)
        .bind(currency)
        .bind(status)
        .bind(provider_response)
        .bind(gateway_transaction_id)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to log payment: {}", e)))?;

        Ok(())
    }

    /// 记录请求日志
    #[allow(clippy::too_many_arguments)]
    pub async fn log_request(
        &self,
        request_id: &str,
        method: &str,
        path: &str,
        query_params: Option<Value>,
        user_id: Option<i64>,
        admin_id: Option<i64>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        response_status: Option<i32>,
        duration_ms: Option<i32>,
    ) -> Result<(), RswsError> {
        // 按 log.request 过滤
        if !self.is_module_log_enabled("log.request", "info").await {
            return Ok(());
        }

        let id = snowflake::next_id();
        sqlx::query(
            r#"INSERT INTO request_logs (id, request_id, method, path, query_params, user_id, admin_id, ip_address, user_agent, response_status, duration_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
        )
        .bind(id)
        .bind(request_id)
        .bind(method)
        .bind(path)
        .bind(query_params)
        .bind(user_id)
        .bind(admin_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(response_status)
        .bind(duration_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to log request: {}", e)))?;

        Ok(())
    }

    // ==================== 日志配置管理 ====================

    /// 获取日志配置
    pub async fn get_log_config(&self, key: &str) -> Result<Option<LogConfig>, RswsError> {
        sqlx::query_as::<_, LogConfig>("SELECT * FROM log_configs WHERE config_key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to get log config: {}", e)))
    }

    /// 列出所有日志配置
    pub async fn list_log_configs(&self) -> Result<Vec<LogConfig>, RswsError> {
        sqlx::query_as::<_, LogConfig>("SELECT * FROM log_configs ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to list log configs: {}", e)))
    }

    /// 设置日志配置（upsert）
    pub async fn set_log_config(
        &self,
        key: &str,
        value: &str,
        config_type: &str,
        description: Option<&str>,
        level: Option<&str>,
    ) -> Result<LogConfig, RswsError> {
        sqlx::query_as::<_, LogConfig>(
            r#"INSERT INTO log_configs (config_key, config_value, config_type, description, level, is_active)
            VALUES ($1, $2, $3, $4, $5, true)
            ON CONFLICT (config_key) DO UPDATE SET
                config_value = EXCLUDED.config_value,
                config_type = EXCLUDED.config_type,
                description = EXCLUDED.description,
                level = COALESCE(EXCLUDED.level, log_configs.level),
                updated_at = NOW()
            RETURNING *"#,
        )
        .bind(key)
        .bind(value)
        .bind(config_type)
        .bind(description)
        .bind(level)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to set log config: {}", e)))
    }

    /// 更新日志配置
    pub async fn update_log_config(
        &self,
        request: &UpdateLogConfigRequest,
    ) -> Result<LogConfig, RswsError> {
        let config_type = request.config_type.as_deref().unwrap_or("string");
        sqlx::query_as::<_, LogConfig>(
            r#"UPDATE log_configs SET
                config_value = $2,
                config_type = $3,
                description = COALESCE($4, description),
                is_active = COALESCE($5, is_active),
                level = COALESCE($6, level),
                updated_at = NOW()
            WHERE config_key = $1
            RETURNING *"#,
        )
        .bind(&request.config_key)
        .bind(&request.config_value)
        .bind(config_type)
        .bind(&request.description)
        .bind(request.is_active)
        .bind(&request.level)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RswsError::internal(format!("Failed to update log config: {}", e)))
    }

    /// 删除日志配置
    pub async fn delete_log_config(&self, key: &str) -> Result<bool, RswsError> {
        let result = sqlx::query("DELETE FROM log_configs WHERE config_key = $1")
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to delete log config: {}", e)))?;
        Ok(result.rows_affected() > 0)
    }

    /// 获取日志配置值（便捷方法）
    pub async fn get_log_config_value(&self, key: &str) -> Result<Option<String>, RswsError> {
        let config = self.get_log_config(key).await?;
        Ok(config.filter(|c| c.is_active).map(|c| c.config_value))
    }

    /// 查询系统日志（分页）
    pub async fn query_system_logs(
        &self,
        params: LogQueryParams,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Value>, i64), RswsError> {
        let mut conditions: Vec<String> = vec!["1=1".to_string()];
        let mut param_idx = 1;

        let level_param = if let Some(ref lvl) = params.level {
            conditions.push(format!("log_level = ${}", param_idx));
            param_idx += 1;
            Some(lvl.to_string())
        } else {
            None
        };

        let module_param = if let Some(ref mod_name) = params.module {
            conditions.push(format!("module = ${}", param_idx));
            param_idx += 1;
            Some(mod_name.to_string())
        } else {
            None
        };

        let user_id_param = if let Some(uid) = params.user_id {
            conditions.push(format!("user_id = ${}", param_idx));
            param_idx += 1;
            Some(uid)
        } else {
            None
        };

        let start_time_param = if let Some(ref st) = params.start_time {
            conditions.push(format!("created_at >= ${}", param_idx));
            param_idx += 1;
            Some(st)
        } else {
            None
        };

        let end_time_param = if let Some(ref et) = params.end_time {
            conditions.push(format!("created_at <= ${}", param_idx));
            param_idx += 1;
            Some(et)
        } else {
            None
        };

        let where_clause = conditions.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) FROM system_logs WHERE {}", where_clause);
        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql);

        if let Some(ref lvl) = level_param {
            count_query = count_query.bind(lvl);
        }
        if let Some(ref mod_name) = module_param {
            count_query = count_query.bind(mod_name);
        }
        if let Some(uid) = user_id_param {
            count_query = count_query.bind(uid);
        }
        if let Some(ref st) = start_time_param {
            count_query = count_query.bind(st);
        }
        if let Some(ref et) = end_time_param {
            count_query = count_query.bind(et);
        }

        let count: (i64,) = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to count system logs: {}", e)))?;

        let offset = (page - 1) * page_size;
        let select_sql = format!(
            "SELECT to_jsonb(t) FROM (SELECT * FROM system_logs WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}) t",
            where_clause, param_idx, param_idx + 1
        );

        let mut select_query = sqlx::query_as::<_, (Value,)>(&select_sql);

        if let Some(ref lvl) = level_param {
            select_query = select_query.bind(lvl);
        }
        if let Some(ref mod_name) = module_param {
            select_query = select_query.bind(mod_name);
        }
        if let Some(uid) = user_id_param {
            select_query = select_query.bind(uid);
        }
        if let Some(ref st) = start_time_param {
            select_query = select_query.bind(st);
        }
        if let Some(ref et) = end_time_param {
            select_query = select_query.bind(et);
        }
        select_query = select_query.bind(page_size).bind(offset);

        let logs: Vec<Value> = select_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RswsError::internal(format!("Failed to query system logs: {}", e)))?
            .into_iter()
            .map(|(v,)| v)
            .collect();

        Ok((logs, count.0))
    }
}
