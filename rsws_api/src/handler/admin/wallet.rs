//! USDT 钱包管理
//!
//! 列出、创建/更新 USDT 钱包地址

use crate::state::get_state;
use rsws_common::ResponseExt;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use serde::Deserialize;

/// USDT 钱包请求体
#[derive(Debug, Deserialize, salvo_oapi::ToSchema)]
pub struct UsdtWalletRequest {
    pub address: String,
    pub name: Option<String>,
}

/// 列出所有 USDT 钱包
#[endpoint(
    responses(
        (status_code = 200, description = "成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn list_usdt_wallets(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    match state.blockchain_service.list_usdt_wallets().await {
        Ok(wallets) => res.success(serde_json::json!({ "items": wallets })),
        Err(e) => res.error(e),
    }
}

/// 更新或创建 USDT 钱包
#[endpoint(
    request_body = UsdtWalletRequest,
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 403, description = "非管理员"),
    )
)]
pub async fn update_usdt_wallet(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let network: String = req.param("network").unwrap_or_else(|| "tron".to_string());
    if network != "tron" && network != "ethereum" {
        res.http_error(
            StatusCode::BAD_REQUEST,
            "Invalid network, use 'tron' or 'ethereum'",
        );
        return;
    }

    let body = req.parse_json::<UsdtWalletRequest>().await;
    let state = get_state(depot);
    match body {
        Ok(data) => {
            let valid = if network == "tron" {
                state
                    .blockchain_service
                    .validate_trc20_address(&data.address)
            } else {
                state
                    .blockchain_service
                    .validate_erc20_address(&data.address)
            };
            if !valid {
                res.http_error(StatusCode::BAD_REQUEST, "Invalid address format");
                return;
            }

            match state
                .blockchain_service
                .upsert_usdt_wallet(&network, &data.address, data.name.as_deref())
                .await
            {
                Ok(wallet) => res.success(wallet),
                Err(e) => res.error(e),
            }
        }
        Err(e) => {
            res.http_error(StatusCode::BAD_REQUEST, format!("Invalid request: {}", e));
        }
    }
}
