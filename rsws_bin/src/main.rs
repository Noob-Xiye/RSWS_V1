//! RSWS 主程序

use salvo::prelude::*;
use rsws_api::router;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建路由
    let router = router::create_router();

    // 创建服务
    let _service = Service::new(router);

    // 启动服务器
    println!("Server starting on http://0.0.0.0:3000");
    let _acceptor = TcpListener::new("0.0.0.0:3000").bind().await;
    std::future::pending::<()>().await;
}
