use chrono::{DateTime, Utc};
use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use rsws_model::response::PaginatedResponse;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct CommissionRule {
    pub id: i64,
    pub name: String,
    pub rule_type: String, // percentage, fixed, tiered
    pub rate: Decimal,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CommissionRecord {
    pub id: i64,
    pub order_id: i64,
    pub user_id: i64,
    pub referrer_id: Option<i64>,
    pub rule_id: i64,
    pub order_amount: Decimal,
    pub commission_amount: Decimal,
    pub commission_rate: Decimal,
    pub status: String, // pending, paid, cancelled
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateCommissionRuleRequest {
    pub name: String,
    pub rule_type: String,
    pub rate: Decimal,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
}

pub struct CommissionService {
    db_pool: Arc<PgPool>,
}

impl CommissionService {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }

    // 创建佣金规则
    pub async fn create_commission_rule(
        &self,
        request: CreateCommissionRuleRequest,
    ) -> Result<i64, ServiceError> {
        let rule_id = snowflake::next_id();

        sqlx::query!(
            r#"
            INSERT INTO commission_rules (
                id, name, rule_type, rate, min_amount, max_amount, is_active
            ) VALUES ($1, $2, $3, $4, $5, $6, true)
            "#,
            rule_id,
            request.name,
            request.rule_type,
            request.rate,
            request.min_amount,
            request.max_amount
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        info!("Commission rule created: {} ({})", request.name, rule_id);
        Ok(rule_id)
    }

    // 计算佣金
    pub async fn calculate_commission(
        &self,
        order_id: i64,
        user_id: i64,
        order_amount: Decimal,
        referrer_id: Option<i64>,
    ) -> Result<Option<CommissionRecord>, ServiceError> {
        // 获取适用的佣金规则
        let rule = sqlx::query_as!(
            CommissionRule,
            r#"
            SELECT id, name, rule_type, rate, min_amount, max_amount, 
                   is_active, created_at, updated_at
            FROM commission_rules 
            WHERE is_active = true 
              AND (min_amount IS NULL OR $1 >= min_amount)
              AND (max_amount IS NULL OR $1 <= max_amount)
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            order_amount
        )
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let Some(rule) = rule else {
            return Ok(None);
        };

        // 计算佣金金额
        let commission_amount = match rule.rule_type.as_str() {
            "percentage" => order_amount * rule.rate / Decimal::from(100),
            "fixed" => rule.rate,
            _ => {
                return Err(ServiceError::ValidationError(
                    "Invalid commission rule type".to_string(),
                ))
            }
        };

        // 创建佣金记录
        let commission_id = snowflake::next_id();

        sqlx::query!(
            r#"
            INSERT INTO commission_records (
                id, order_id, user_id, referrer_id, rule_id,
                order_amount, commission_amount, commission_rate, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending')
            "#,
            commission_id,
            order_id,
            user_id,
            referrer_id,
            rule.id,
            order_amount,
            commission_amount,
            rule.rate
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let commission_record = CommissionRecord {
            id: commission_id,
            order_id,
            user_id,
            referrer_id,
            rule_id: rule.id,
            order_amount,
            commission_amount,
            commission_rate: rule.rate,
            status: "pending".to_string(),
            paid_at: None,
            created_at: Utc::now(),
        };

        info!(
            "Commission calculated: {} for order {}",
            commission_amount, order_id
        );
        Ok(Some(commission_record))
    }

    // 支付佣金
    pub async fn pay_commission(&self, commission_id: i64) -> Result<(), ServiceError> {
        sqlx::query!(
            "UPDATE commission_records SET status = 'paid', paid_at = NOW() WHERE id = $1",
            commission_id
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        info!("Commission paid: {}", commission_id);
        Ok(())
    }

    // 获取用户佣金记录
    pub async fn get_user_commissions(
        &self,
        user_id: i64,
        page: u32,
        page_size: u32,
    ) -> Result<PaginatedResponse<CommissionRecord>, ServiceError> {
        let offset = (page - 1) * page_size;

        let commissions = sqlx::query_as!(
            CommissionRecord,
            r#"
            SELECT id, order_id, user_id, referrer_id, rule_id,
                   order_amount, commission_amount, commission_rate,
                   status, paid_at, created_at
            FROM commission_records 
            WHERE referrer_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            page_size as i64,
            offset as i64
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM commission_records WHERE referrer_id = $1",
            user_id
        )
        .fetch_one(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(PaginatedResponse {
            data: commissions,
            total: total as u64,
            page,
            page_size,
            total_pages: ((total as f64) / (page_size as f64)).ceil() as u32,
        })
    }
}
