use crate::commission_service::CommissionService;
use crate::config_service::ConfigService;
use chrono::Utc;
use rsws_common::error::ServiceError;
use rsws_common::snowflake;
use rsws_db::{OrderRepository, PaymentRepository, ResourceRepository};
use rsws_model::payment::*;
use rsws_model::resource::Resource; // 确保导入 Resource
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

pub struct PaymentService {
    order_repo: Arc<OrderRepository>,
    payment_repo: Arc<PaymentRepository>,
    resource_repo: Arc<ResourceRepository>,
    config_service: Arc<ConfigService>,
    commission_service: Arc<CommissionService>,
    db_pool: Arc<PgPool>, // 添加数据库连接池
}

impl PaymentService {
    pub fn new(
        order_repo: Arc<OrderRepository>,
        payment_repo: Arc<PaymentRepository>,
        resource_repo: Arc<ResourceRepository>,
        config_service: Arc<ConfigService>,
        commission_service: Arc<CommissionService>,
        db_pool: Arc<PgPool>, // 添加参数
    ) -> Self {
        Self {
            order_repo,
            payment_repo,
            resource_repo,
            config_service,
            commission_service,
            db_pool, // 初始化字段
        }
    }

    // 获取支付方式列表（从数据库动态获取）
    pub async fn get_payment_methods(&self) -> Result<Vec<PaymentMethod>, ServiceError> {
        let method_configs = self.config_service.get_active_payment_methods().await?;
        let mut methods = Vec::new();

        for config in method_configs {
            let (min_amount, max_amount, fee_rate) = match config.method_id.as_str() {
                "paypal" => {
                    if let Some(paypal_config) = self.config_service.get_paypal_config().await? {
                        (
                            Some(paypal_config.min_amount),
                            Some(paypal_config.max_amount),
                            Some(paypal_config.fee_rate),
                        )
                    } else {
                        continue; // 跳过未配置的支付方式
                    }
                }
                "usdt_tron" => {
                    if let Some(tron_config) =
                        self.config_service.get_blockchain_config("tron").await?
                    {
                        (
                            Some(tron_config.min_amount),
                            Some(tron_config.max_amount),
                            Some(tron_config.fee_rate),
                        )
                    } else {
                        continue;
                    }
                }
                "usdt_eth" => {
                    if let Some(eth_config) = self
                        .config_service
                        .get_blockchain_config("ethereum")
                        .await?
                    {
                        (
                            Some(eth_config.min_amount),
                            Some(eth_config.max_amount),
                            Some(eth_config.fee_rate),
                        )
                    } else {
                        continue;
                    }
                }
                _ => (None, None, None),
            };

            methods.push(PaymentMethod {
                id: config.method_id,
                name: config.method_name,
                icon: config.icon_url,
                enabled: config.is_active,
                min_amount,
                max_amount,
                fee_rate,
                description: config.description,
            });
        }

        Ok(methods)
    }

    // 创建支付（使用动态配置）
    pub async fn create_payment(
        &self,
        order_id: i64,
        payment_method: &str,
        return_url: Option<String>,
        cancel_url: Option<String>,
        user_id: i64,
    ) -> Result<PayOrderResponse, ServiceError> {
        // 获取订单信息
        let order = self
            .order_repo
            .get_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("订单不存在".to_string()))?;

        // 检查权限
        if order.user_id != user_id {
            return Err(ServiceError::Forbidden("无权操作该订单".to_string()));
        }

        // 检查订单状态
        if order.status != OrderStatus::Pending {
            return Err(ServiceError::BusinessError(
                "订单状态不允许支付".to_string(),
            ));
        }

        // 检查订单是否过期
        if let Some(expired_at) = order.expired_at {
            if Utc::now() > expired_at {
                // 自动取消过期订单
                self.order_repo
                    .update_order_status(order_id, OrderStatus::Cancelled)
                    .await?;
                return Err(ServiceError::BusinessError("订单已过期".to_string()));
            }
        }

        // 根据支付方式选择支付提供商
        let provider = match request.payment_method.as_str() {
            "paypal" => &self.paypal_service,
            "usdt" => &self.blockchain_service,
            _ => return Err(ServiceError::BusinessError("不支持的支付方式".to_string())),
        };

        // 创建支付
        let payment_result = provider
            .create_payment(
                order_id,
                order.amount,
                "USD", // 默认美元
                request.return_url.as_deref(),
                request.cancel_url.as_deref(),
            )
            .await?;

        // 创建支付交易记录
        let transaction = PaymentTransaction {
            id: snowflake::next_id(),
            order_id,
            user_id,
            payment_method: request.payment_method.clone(),
            payment_provider: request.payment_method.clone(),
            external_transaction_id: payment_result.payment_id.clone(),
            amount: order.amount,
            currency: "USD".to_string(),
            status: payment_result.status,
            gateway_response: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };

        self.payment_repo.create_transaction(&transaction).await?;

        // 更新订单支付信息
        self.order_repo
            .update_payment_info(
                order_id,
                &request.payment_method,
                &payment_result.payment_id,
            )
            .await?;

        Ok(PayOrderResponse {
            success: true,
            message: "支付创建成功".to_string(),
            payment_url: payment_result.payment_url,
            payment_id: Some(payment_result.payment_id),
            qr_code: payment_result.qr_code,
        })
    }

    // 验证支付
    pub async fn verify_payment(
        &self,
        payment_id: &str,
    ) -> Result<VerifyPaymentResponse, ServiceError> {
        // 获取支付交易记录
        let transaction = self
            .payment_repo
            .get_transaction_by_payment_id(payment_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("支付记录不存在".to_string()))?;

        // 根据支付方式选择验证提供商
        let provider = match transaction.payment_provider.as_str() {
            "paypal" => &self.paypal_service,
            "usdt" => &self.blockchain_service,
            _ => return Err(ServiceError::BusinessError("不支持的支付方式".to_string())),
        };

        // 验证支付状态
        let verification = provider.verify_payment(payment_id).await?;

        // 更新交易状态
        self.payment_repo
            .update_transaction_status(
                transaction.id,
                verification.status.clone(),
                verification.external_transaction_id.as_deref(),
            )
            .await?;

        // 如果支付成功，更新订单状态
        if verification.status == TransactionStatus::Completed {
            self.order_repo
                .update_order_status(transaction.order_id, OrderStatus::Completed)
                .await?;

            self.order_repo
                .update_completed_at(transaction.order_id, Utc::now())
                .await?;
        }

        Ok(VerifyPaymentResponse {
            success: verification.status == TransactionStatus::Completed,
            status: verification.status,
            message: match verification.status {
                TransactionStatus::Completed => "支付成功".to_string(),
                TransactionStatus::Failed => "支付失败".to_string(),
                TransactionStatus::Pending => "支付处理中".to_string(),
                _ => "支付状态未知".to_string(),
            },
            order_id: Some(transaction.order_id),
        })
    }

    // 智能支付处理 - 根据资源提供者类型处理不同的收款逻辑
    pub async fn process_payment_completion(
        &self,
        payment_id: &str,
        external_transaction_id: &str,
    ) -> Result<(), ServiceError> {
        // 获取支付交易记录
        let transaction = self
            .payment_repo
            .get_transaction_by_payment_id(payment_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("支付记录不存在".to_string()))?;

        // 获取订单信息
        let order = self
            .order_repo
            .get_by_id(transaction.order_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("订单不存在".to_string()))?;

        // 获取资源信息
        let resource = self
            .resource_repo
            .get_by_id(order.resource_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("资源不存在".to_string()))?;

        // 根据资源提供者类型处理支付
        match resource.provider_type.as_str() {
            "admin" => {
                // 管理员资源：直接收款到系统账户
                self.process_admin_resource_payment(&transaction, &order, &resource)
                    .await?
            }
            "user" => {
                // 用户资源：扣除佣金后支付给用户
                self.process_user_resource_payment(&transaction, &order, &resource)
                    .await?
            }
            _ => {
                error!("Unknown provider type: {}", resource.provider_type);
                return Err(ServiceError::BusinessError(
                    "未知的资源提供者类型".to_string(),
                ));
            }
        }

        // 更新交易状态
        self.payment_repo
            .update_transaction_status(
                transaction.id,
                TransactionStatus::Completed,
                Some(external_transaction_id),
            )
            .await?;

        // 更新订单状态
        self.order_repo
            .update_order_status(transaction.order_id, OrderStatus::Completed)
            .await?;

        self.order_repo
            .update_completed_at(transaction.order_id, Utc::now())
            .await?;

        info!(
            "Payment completed successfully for order: {}",
            transaction.order_id
        );
        Ok(())
    }

    // 处理管理员资源支付
    async fn process_admin_resource_payment(
        &self,
        transaction: &PaymentTransaction,
        order: &Order,
        resource: &Resource,
    ) -> Result<(), ServiceError> {
        info!("Processing admin resource payment for order: {}", order.id);

        // 管理员资源直接收款到系统配置的账户
        // 根据支付方式获取系统收款配置
        let system_account = match transaction.payment_method.as_str() {
            "paypal" => {
                let paypal_config = self
                    .config_service
                    .get_paypal_config()
                    .await?
                    .ok_or_else(|| ServiceError::ConfigError("PayPal配置不存在".to_string()))?;
                paypal_config.client_id // 使用系统PayPal账户
            }
            "usdt_tron" | "usdt_eth" => {
                let blockchain_config = self
                    .config_service
                    .get_blockchain_config(if transaction.payment_method == "usdt_tron" {
                        "tron"
                    } else {
                        "ethereum"
                    })
                    .await?
                    .ok_or_else(|| ServiceError::ConfigError("区块链配置不存在".to_string()))?;
                blockchain_config.wallet_address // 使用系统USDT地址
            }
            _ => return Err(ServiceError::BusinessError("不支持的支付方式".to_string())),
        };

        // 记录收款日志
        self.log_payment_receipt(
            transaction.order_id,
            "system",
            &system_account,
            transaction.amount,
            "Admin resource payment - full amount to system",
        )
        .await?;

        Ok(())
    }

    // 处理用户资源支付
    async fn process_user_resource_payment(
        &self,
        transaction: &PaymentTransaction,
        order: &Order,
        resource: &Resource,
    ) -> Result<(), ServiceError> {
        info!("Processing user resource payment for order: {}", order.id);

        // 计算佣金
        let commission_record = self
            .commission_service
            .calculate_commission(
                order.id,
                order.user_id,
                transaction.amount,
                None, // 暂时不支持推荐人
            )
            .await?;

        let commission_amount = commission_record
            .as_ref()
            .map(|r| r.commission_amount)
            .unwrap_or(Decimal::ZERO);

        let user_amount = transaction.amount - commission_amount;

        // 获取用户收款配置
        let user_payment_config = self
            .get_user_payment_config(
                resource.provider_id.unwrap_or(0),
                &transaction.payment_method,
            )
            .await?;

        // 支付给用户（扣除佣金后的金额）
        if user_amount > Decimal::ZERO {
            self.transfer_to_user(
                &user_payment_config,
                user_amount,
                &transaction.payment_method,
                order.id,
            )
            .await?;

            // 记录用户收款日志
            self.log_payment_receipt(
                transaction.order_id,
                "user",
                &user_payment_config.account_address,
                user_amount,
                &format!(
                    "User resource payment - amount after commission: {}",
                    commission_amount
                ),
            )
            .await?;
        }

        // 收取佣金到系统账户
        if commission_amount > Decimal::ZERO {
            let system_account = self.get_system_account(&transaction.payment_method).await?;

            self.transfer_commission(
                &system_account,
                commission_amount,
                &transaction.payment_method,
                order.id,
            )
            .await?;

            // 记录佣金收款日志
            self.log_payment_receipt(
                transaction.order_id,
                "commission",
                &system_account,
                commission_amount,
                "Commission collection from user resource",
            )
            .await?;

            // 标记佣金为已支付
            if let Some(commission) = commission_record {
                self.commission_service
                    .pay_commission(commission.id)
                    .await?;
            }
        }

        Ok(())
    }

    // 获取用户收款配置（支持多配置选择）
    async fn get_user_payment_config(
        &self,
        user_id: i64,
        payment_method: &str,
    ) -> Result<UserPaymentConfig, ServiceError> {
        // 优先获取用户的默认配置（最新创建的）
        let config = sqlx::query_as!(
            UserPaymentConfig,
            r#"
            SELECT id, user_id, payment_method, account_address, 
                   account_name, is_active, created_at, updated_at
            FROM user_payment_configs 
            WHERE user_id = $1 AND payment_method = $2 AND is_active = true
            ORDER BY updated_at DESC, created_at DESC
            LIMIT 1
            "#,
            user_id,
            payment_method
        )
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
        .ok_or_else(|| ServiceError::NotFound(format!("用户{}收款配置不存在", payment_method)))?;

        Ok(config)
    }

    // 获取用户所有可用的收款配置
    pub async fn get_user_available_payment_configs(
        &self,
        user_id: i64,
    ) -> Result<std::collections::HashMap<String, Vec<UserPaymentConfig>>, ServiceError> {
        let configs = sqlx::query_as!(
            UserPaymentConfig,
            r#"
            SELECT id, user_id, payment_method, account_address, 
                   account_name, is_active, created_at, updated_at
            FROM user_payment_configs 
            WHERE user_id = $1 AND is_active = true
            ORDER BY payment_method, updated_at DESC, created_at DESC
            "#,
            user_id
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let mut grouped_configs = std::collections::HashMap::new();
        for config in configs {
            grouped_configs
                .entry(config.payment_method.clone())
                .or_insert_with(Vec::new)
                .push(config);
        }

        Ok(grouped_configs)
    }

    // 获取系统收款账户
    async fn get_system_account(&self, payment_method: &str) -> Result<String, ServiceError> {
        match payment_method {
            "paypal" => {
                let config = self
                    .config_service
                    .get_paypal_config()
                    .await?
                    .ok_or_else(|| ServiceError::ConfigError("PayPal配置不存在".to_string()))?;
                Ok(config.client_id)
            }
            "usdt_tron" => {
                let config = self
                    .config_service
                    .get_blockchain_config("tron")
                    .await?
                    .ok_or_else(|| ServiceError::ConfigError("TRON配置不存在".to_string()))?;
                Ok(config.wallet_address)
            }
            "usdt_eth" => {
                let config = self
                    .config_service
                    .get_blockchain_config("ethereum")
                    .await?
                    .ok_or_else(|| ServiceError::ConfigError("以太坊配置不存在".to_string()))?;
                Ok(config.wallet_address)
            }
            _ => Err(ServiceError::BusinessError("不支持的支付方式".to_string())),
        }
    }

    // 转账给用户
    async fn transfer_to_user(
        &self,
        user_config: &UserPaymentConfig,
        amount: Decimal,
        payment_method: &str,
        order_id: i64,
    ) -> Result<(), ServiceError> {
        // 这里应该调用实际的支付接口进行转账
        // 目前只记录转账意图
        info!(
            "Transfer to user: {} {} to {} via {} for order {}",
            amount, payment_method, user_config.account_address, payment_method, order_id
        );

        // TODO: 实现实际的转账逻辑
        // - PayPal: 使用PayPal API进行转账
        // - USDT: 使用区块链API进行转账

        Ok(())
    }

    // 转账佣金到系统账户
    async fn transfer_commission(
        &self,
        system_account: &str,
        amount: Decimal,
        payment_method: &str,
        order_id: i64,
    ) -> Result<(), ServiceError> {
        info!(
            "Transfer commission: {} {} to system account {} for order {}",
            amount, payment_method, system_account, order_id
        );

        // 佣金通常是从已收到的款项中分配，不需要额外转账
        // 这里主要是记录佣金分配

        Ok(())
    }

    // 记录收款日志
    async fn log_payment_receipt(
        &self,
        order_id: i64,
        recipient_type: &str,
        account: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            r#"
            INSERT INTO payment_logs 
            (id, order_id, recipient_type, account, amount, description, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            snowflake::next_id(),
            order_id,
            recipient_type,
            account,
            amount,
            description,
            Utc::now()
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
