use salvo::prelude::*;
use rsws_service::config_service::ConfigService;
use rsws_model::payment::*;
use rsws_common::error::ServiceError;
use std::sync::Arc;

#[derive(Clone)]
pub struct PaymentConfigHandler {
    config_service: Arc<ConfigService>,
}

impl PaymentConfigHandler {
    pub fn new(config_service: Arc<ConfigService>) -> Self {
        Self { config_service }
    }

    // 获取PayPal配置
    #[handler]
    pub async fn get_paypal_config(&self, res: &mut Response) {
        match self.config_service.get_paypal_config().await {
            Ok(Some(config)) => {
                // 不返回敏感信息
                let safe_config = serde_json::json!({
                    "client_id": config.client_id,
                    "sandbox": config.sandbox,
                    "webhook_id": config.webhook_id,
                    "return_url": config.return_url,
                    "cancel_url": config.cancel_url,
                    "brand_name": config.brand_name,
                    "min_amount": config.min_amount,
                    "max_amount": config.max_amount,
                    "fee_rate": config.fee_rate,
                    "is_active": config.is_active,
                });
                res.render(Json(safe_config));
            },
            Ok(None) => res.status_code(StatusCode::NOT_FOUND),
            Err(e) => res.status_code(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    // 更新PayPal配置
    #[handler]
    pub async fn update_paypal_config(&self, req: &mut Request, res: &mut Response) {
        let request: UpdatePayPalConfigRequest = match req.parse_json().await {
            Ok(r) => r,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                return;
            }
        };

        // TODO: 从JWT中获取admin_id
        let admin_id = 1;

        match self.config_service.update_paypal_config(request, admin_id).await {
            Ok(_) => res.render(Json(serde_json::json!({"message": "PayPal config updated successfully"})));
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({"error": e.to_string()})));
            }
        }
    }

    // 获取区块链配置
    #[handler]
    pub async fn get_blockchain_config(&self, req: &mut Request, res: &mut Response) {
        let network = req.param::<String>("network").unwrap_or_default();
        
        match self.config_service.get_blockchain_config(&network).await {
            Ok(Some(config)) => {
                // 不返回敏感信息
                let safe_config = serde_json::json!({
                    "network": config.network,
                    "network_name": config.network_name,
                    "api_url": config.api_url,
                    "usdt_contract": config.usdt_contract,
                    "wallet_addresses": config.wallet_addresses,
                    "min_confirmations": config.min_confirmations,
                    "min_amount": config.min_amount,
                    "max_amount": config.max_amount,
                    "fee_rate": config.fee_rate,
                    "is_active": config.is_active,
                });
                res.render(Json(safe_config));
            },
            Ok(None) => res.status_code(StatusCode::NOT_FOUND),
            Err(e) => res.status_code(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    // 更新区块链配置
    #[handler]
    pub async fn update_blockchain_config(&self, req: &mut Request, res: &mut Response) {
        let network = req.param::<String>("network").unwrap_or_default();
        let request: UpdateBlockchainConfigRequest = match req.parse_json().await {
            Ok(r) => r,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                return;
            }
        };

        // TODO: 从JWT中获取admin_id
        let admin_id = 1;

        match self.config_service.update_blockchain_config(&network, request, admin_id).await {
            Ok(_) => res.render(Json(serde_json::json!({"message": "Blockchain config updated successfully"})));
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({"error": e.to_string()})));
            }
        }
    }

    // 获取支付方式列表
    #[handler]
    pub async fn get_payment_methods(&self, res: &mut Response) {
        match self.config_service.get_active_payment_methods().await {
            Ok(methods) => res.render(Json(methods)),
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({"error": e.to_string()})));
            }
        }
    }

    // 更新支付方式
    #[handler]
    pub async fn update_payment_method(&self, req: &mut Request, res: &mut Response) {
        let method_id = req.param::<String>("method_id").unwrap_or_default();
        let request: UpdatePaymentMethodRequest = match req.parse_json().await {
            Ok(r) => r,
            Err(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
                return;
            }
        };

        // TODO: 从JWT中获取admin_id
        let admin_id = 1;

        match self.config_service.update_payment_method(&method_id, request, admin_id).await {
            Ok(_) => res.render(Json(serde_json::json!({"message": "Payment method updated successfully"})));
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({"error": e.to_string()})));
            }
        }
    }
}