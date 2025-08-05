use chrono::{DateTime, Utc};
use reqwest::Client;
use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub platform_name: String,
    pub api_endpoint: String,
    pub api_key: String,
    pub api_secret: Option<String>,
    pub is_active: bool,
    pub sync_interval_minutes: u32,
    pub supported_operations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SyncRecord {
    pub id: i64,
    pub platform_name: String,
    pub operation_type: String,
    pub local_id: String,
    pub remote_id: Option<String>,
    pub sync_status: String, // pending, success, failed
    pub sync_data: Value,
    pub error_message: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

use std::sync::Arc;
use sqlx::PgPool;
use rsws_common::error::ServiceError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub platform_name: String,
    pub api_endpoint: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub webhook_url: Option<String>,
    pub is_active: bool,
    pub sync_interval_minutes: i32,
    pub supported_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub platform: String,
    pub sync_type: String,
    pub success_count: i32,
    pub error_count: i32,
    pub last_sync_at: chrono::DateTime<chrono::Utc>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformOrder {
    pub platform_order_id: String,
    pub platform_name: String,
    pub local_order_id: Option<i64>,
    pub user_email: String,
    pub product_sku: String,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

pub struct CrossPlatformService {
    db_pool: PgPool,
    platforms: Arc<tokio::sync::RwLock<HashMap<String, PlatformConfig>>>,
    log_service: Arc<crate::log_service::LogService>,
}

impl CrossPlatformService {
    pub fn new(db_pool: PgPool, log_service: Arc<crate::log_service::LogService>) -> Self {
        Self {
            db_pool,
            platforms: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            log_service,
        }
    }

    // 注册平台配置
    pub async fn register_platform(&self, config: PlatformConfig) -> Result<(), ServiceError> {
        let mut platforms = self.platforms.write().await;
        platforms.insert(config.platform_name.clone(), config.clone());
        
        info!("Registered platform: {}", config.platform_name);
        Ok(())
    }

    // 获取平台配置
    pub async fn get_platform_config(&self, platform_name: &str) -> Option<PlatformConfig> {
        let platforms = self.platforms.read().await;
        platforms.get(platform_name).cloned()
    }

    // 同步所有平台数据
    pub async fn sync_all_platforms(&self) -> Result<Vec<SyncResult>, ServiceError> {
        let platforms = self.platforms.read().await;
        let mut results = Vec::new();

        for (name, config) in platforms.iter() {
            if config.is_active {
                match self.sync_platform_data(name, config).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        error!("Failed to sync platform {}: {}", name, e);
                        results.push(SyncResult {
                            platform: name.clone(),
                            sync_type: "full".to_string(),
                            success_count: 0,
                            error_count: 1,
                            last_sync_at: chrono::Utc::now(),
                            errors: vec![e.to_string()],
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    // 同步单个平台数据
    async fn sync_platform_data(&self, platform_name: &str, config: &PlatformConfig) -> Result<SyncResult, ServiceError> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        // 根据平台类型执行不同的同步逻辑
        match platform_name {
            "shopify" => {
                match self.sync_shopify_orders(config).await {
                    Ok(count) => success_count += count,
                    Err(e) => {
                        error_count += 1;
                        errors.push(format!("Shopify sync error: {}", e));
                    }
                }
            },
            "woocommerce" => {
                match self.sync_woocommerce_orders(config).await {
                    Ok(count) => success_count += count,
                    Err(e) => {
                        error_count += 1;
                        errors.push(format!("WooCommerce sync error: {}", e));
                    }
                }
            },
            "magento" => {
                match self.sync_magento_orders(config).await {
                    Ok(count) => success_count += count,
                    Err(e) => {
                        error_count += 1;
                        errors.push(format!("Magento sync error: {}", e));
                    }
                }
            },
            _ => {
                error_count += 1;
                errors.push(format!("Unsupported platform: {}", platform_name));
            }
        }

        Ok(SyncResult {
            platform: platform_name.to_string(),
            sync_type: "orders".to_string(),
            success_count,
            error_count,
            last_sync_at: chrono::Utc::now(),
            errors,
        })
    }

    // 同步Shopify订单
    async fn sync_shopify_orders(&self, config: &PlatformConfig) -> Result<i32, ServiceError> {
        // 实现Shopify API调用逻辑
        let client = reqwest::Client::new();
        let api_key = config.api_key.as_ref()
            .ok_or_else(|| ServiceError::ConfigError("Missing Shopify API key".to_string()))?;
        
        let url = format!("{}/admin/api/2023-10/orders.json", config.api_endpoint);
        
        let response = client
            .get(&url)
            .header("X-Shopify-Access-Token", api_key)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("Shopify API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::ExternalApiError(
                format!("Shopify API returned status: {}", response.status())
            ));
        }

        let orders_data: serde_json::Value = response.json().await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to parse Shopify response: {}", e)))?;

        let orders = orders_data.get("orders")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ServiceError::ExternalApiError("Invalid Shopify orders response".to_string()))?;

        let mut synced_count = 0;
        for order in orders {
            if let Ok(cross_order) = self.parse_shopify_order(order) {
                if self.save_cross_platform_order(&cross_order).await.is_ok() {
                    synced_count += 1;
                }
            }
        }

        info!("Synced {} Shopify orders", synced_count);
        Ok(synced_count)
    }

    // 同步WooCommerce订单
    async fn sync_woocommerce_orders(&self, config: &PlatformConfig) -> Result<i32, ServiceError> {
        // 实现WooCommerce REST API调用逻辑
        let client = reqwest::Client::new();
        let api_key = config.api_key.as_ref()
            .ok_or_else(|| ServiceError::ConfigError("Missing WooCommerce API key".to_string()))?;
        let api_secret = config.api_secret.as_ref()
            .ok_or_else(|| ServiceError::ConfigError("Missing WooCommerce API secret".to_string()))?;
        
        let url = format!("{}/wp-json/wc/v3/orders", config.api_endpoint);
        
        let response = client
            .get(&url)
            .basic_auth(api_key, Some(api_secret))
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("WooCommerce API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::ExternalApiError(
                format!("WooCommerce API returned status: {}", response.status())
            ));
        }

        let orders: Vec<serde_json::Value> = response.json().await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to parse WooCommerce response: {}", e)))?;

        let mut synced_count = 0;
        for order in orders {
            if let Ok(cross_order) = self.parse_woocommerce_order(&order) {
                if self.save_cross_platform_order(&cross_order).await.is_ok() {
                    synced_count += 1;
                }
            }
        }

        info!("Synced {} WooCommerce orders", synced_count);
        Ok(synced_count)
    }

    // 同步Magento订单
    async fn sync_magento_orders(&self, config: &PlatformConfig) -> Result<i32, ServiceError> {
        // 实现Magento REST API调用逻辑
        let client = reqwest::Client::new();
        let api_key = config.api_key.as_ref()
            .ok_or_else(|| ServiceError::ConfigError("Missing Magento API key".to_string()))?;
        
        let url = format!("{}/rest/V1/orders", config.api_endpoint);
        
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("Magento API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::ExternalApiError(
                format!("Magento API returned status: {}", response.status())
            ));
        }

        let orders_data: serde_json::Value = response.json().await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to parse Magento response: {}", e)))?;

        let orders = orders_data.get("items")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ServiceError::ExternalApiError("Invalid Magento orders response".to_string()))?;

        let mut synced_count = 0;
        for order in orders {
            if let Ok(cross_order) = self.parse_magento_order(order) {
                if self.save_cross_platform_order(&cross_order).await.is_ok() {
                    synced_count += 1;
                }
            }
        }

        info!("Synced {} Magento orders", synced_count);
        Ok(synced_count)
    }

    // 解析Shopify订单
    fn parse_shopify_order(&self, order: &serde_json::Value) -> Result<CrossPlatformOrder, ServiceError> {
        Ok(CrossPlatformOrder {
            platform_order_id: order.get("id")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .ok_or_else(|| ServiceError::ValidationError("Missing Shopify order ID".to_string()))?,
            platform_name: "shopify".to_string(),
            local_order_id: None,
            user_email: order.get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown@example.com")
                .to_string(),
            product_sku: order.get("line_items")
                .and_then(|items| items.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("sku"))
                .and_then(|sku| sku.as_str())
                .unwrap_or("unknown")
                .to_string(),
            amount: order.get("total_price")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            currency: order.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_string(),
            status: order.get("financial_status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            created_at: order.get("created_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|| chrono::Utc::now()),
            metadata: Some(order.clone()),
        })
    }

    // 解析WooCommerce订单
    fn parse_woocommerce_order(&self, order: &serde_json::Value) -> Result<CrossPlatformOrder, ServiceError> {
        Ok(CrossPlatformOrder {
            platform_order_id: order.get("id")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .ok_or_else(|| ServiceError::ValidationError("Missing WooCommerce order ID".to_string()))?,
            platform_name: "woocommerce".to_string(),
            local_order_id: None,
            user_email: order.get("billing")
                .and_then(|billing| billing.get("email"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown@example.com")
                .to_string(),
            product_sku: order.get("line_items")
                .and_then(|items| items.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("sku"))
                .and_then(|sku| sku.as_str())
                .unwrap_or("unknown")
                .to_string(),
            amount: order.get("total")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            currency: order.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_string(),
            status: order.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            created_at: order.get("date_created")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|| chrono::Utc::now()),
            metadata: Some(order.clone()),
        })
    }

    // 解析Magento订单
    fn parse_magento_order(&self, order: &serde_json::Value) -> Result<CrossPlatformOrder, ServiceError> {
        Ok(CrossPlatformOrder {
            platform_order_id: order.get("entity_id")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .ok_or_else(|| ServiceError::ValidationError("Missing Magento order ID".to_string()))?,
            platform_name: "magento".to_string(),
            local_order_id: None,
            user_email: order.get("customer_email")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown@example.com")
                .to_string(),
            product_sku: order.get("items")
                .and_then(|items| items.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("sku"))
                .and_then(|sku| sku.as_str())
                .unwrap_or("unknown")
                .to_string(),
            amount: order.get("grand_total")
                .and_then(|v| v.as_f64())
                .map(|f| rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default())
                .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            currency: order.get("order_currency_code")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_string(),
            status: order.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            created_at: order.get("created_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|| chrono::Utc::now()),
            metadata: Some(order.clone()),
        })
    }

    // 保存跨平台订单
    async fn save_cross_platform_order(&self, order: &CrossPlatformOrder) -> Result<(), ServiceError> {
        // 检查是否已存在
        let existing = sqlx::query_scalar!(
            "SELECT id FROM cross_platform_orders WHERE platform_order_id = $1 AND platform_name = $2",
            order.platform_order_id,
            order.platform_name
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing.is_some() {
            return Ok(()); // 已存在，跳过
        }

        // 插入新订单
        sqlx::query!(
            r#"
            INSERT INTO cross_platform_orders (
                id, platform_order_id, platform_name, local_order_id, user_email,
                product_sku, amount, currency, status, metadata, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            rsws_common::snowflake::next_id(),
            order.platform_order_id,
            order.platform_name,
            order.local_order_id,
            order.user_email,
            order.product_sku,
            order.amount,
            order.currency,
            order.status,
            order.metadata,
            order.created_at
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // 获取跨平台订单统计
    pub async fn get_platform_stats(&self) -> Result<HashMap<String, PlatformStats>, ServiceError> {
        let stats = sqlx::query_as::<_, PlatformStatsRow>(
            r#"
            SELECT platform_name, 
                   COUNT(*) as total_orders,
                   SUM(amount) as total_revenue,
                   COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_orders
            FROM cross_platform_orders
            GROUP BY platform_name
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut result = HashMap::new();
        for stat in stats {
            result.insert(stat.platform_name.clone(), PlatformStats {
                platform_name: stat.platform_name,
                total_orders: stat.total_orders,
                total_revenue: stat.total_revenue.unwrap_or_default(),
                completed_orders: stat.completed_orders,
            });
        }

        Ok(result)
    }
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
struct PlatformStatsRow {
    platform_name: String,
    total_orders: i64,
    total_revenue: Option<rust_decimal::Decimal>,
    completed_orders: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct PlatformStats {
    pub platform_name: String,
    pub total_orders: i64,
    pub total_revenue: rust_decimal::Decimal,
    pub completed_orders: i64,
}
