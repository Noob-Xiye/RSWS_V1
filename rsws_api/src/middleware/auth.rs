//! 认证中间件

use salvo::prelude::*;

/// API Key 认证
#[handler]
pub async fn api_key_auth(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    // TODO: 实现 API Key 认证

    ctrl.call_next(req, depot, res).await;
}
