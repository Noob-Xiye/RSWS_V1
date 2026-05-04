# RSWS 项目结构说明

**版本**: v0.1.0
**更新时间**: 2026-05-05

---

## 项目概览

RSWS (Resource Sale Web System) 是一个付费阅读/下载网站系统，支持：
- PayPal 支付
- USDT (TRC20/ERC20) 支付
- C2C 模式 (用户可上传销售资源)
- 多语言支持 (面向海外用户)

---

## 目录结构

```
RSWS_V1/
├── Cargo.toml              # Workspace 配置
├── Cargo.lock              # 依赖锁定
├── config.toml             # 应用配置
├── database.sql            # 数据库架构 (整合版)
├── LICENSE                 # MIT 许可证
├── README.md               # 项目说明
├── VERSION.json            # 版本信息
├── FRONTEND_REBUILD.md     # 前端重构计划 (Vue 3 + Bun)
│
├── rsws_bin/               # 可执行程序入口
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # 程序入口，启动服务
│
├── rsws_api/               # API 层 (HTTP 处理)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # 模块导出
│       ├── router.rs       # 路由定义
│       ├── admin_handler.rs    # 管理员 API
│       ├── user_handler.rs     # 用户 API
│       ├── config_handler.rs   # 配置 API
│       ├── order.rs        # 订单 API
│       ├── resource.rs     # 资源 API
│       ├── middleware/     # 中间件
│       │   ├── mod.rs
│       │   ├── auth.rs         # 认证中间件
│       │   ├── signature_auth.rs   # 签名认证
│       │   └── unified_auth.rs     # 统一认证
│       ├── user/           # 用户子模块
│       │   ├── mod.rs
│       │   ├── avatar.rs
│       │   ├── email.rs
│       │   ├── password.rs
│       │   ├── profile.rs
│       │   └── security.rs
│       ├── admin/          # 管理员子模块
│       │   └── payment_config_handler.rs
│       └── webhook/        # Webhook 处理
│           ├── blockchain.rs
│           └── paypal.rs
│
├── rsws_service/           # 业务逻辑层
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # 模块导出
│       ├── admin_service.rs    # 管理员服务
│       ├── user_service.rs     # 用户服务
│       ├── auth_service.rs     # 认证服务
│       ├── api_key_service.rs  # API Key 服务
│       ├── config_service.rs   # 配置服务
│       ├── order_service.rs    # 订单服务
│       ├── payment_service.rs  # 支付服务
│       ├── paypal_service.rs   # PayPal 服务
│       ├── resource_service.rs # 资源服务
│       ├── blockchain_service.rs   # 区块链服务
│       ├── commission_service.rs   # 佣金服务
│       ├── cross_platform_service.rs  # 跨平台服务
│       ├── log_service.rs      # 日志服务
│       ├── request_service.rs  # 请求服务
│       ├── user_payment_service.rs  # 用户支付配置服务
│       └── webhook_service.rs  # Webhook 服务
│
├── rsws_db/                # 数据库访问层
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # 模块导出
│       ├── admin.rs        # 管理员数据访问
│       ├── user.rs         # 用户数据访问
│       ├── order.rs        # 订单数据访问
│       ├── payment.rs      # 支付数据访问
│       ├── resource.rs     # 资源数据访问
│       ├── api_key.rs      # API Key 数据访问
│       ├── postgres/       # PostgreSQL 实现
│       │   ├── mod.rs
│       │   └── ...
│       └── redis/          # Redis 实现
│           ├── mod.rs
│           └── api_key.rs
│
├── rsws_model/             # 数据模型层
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # 模块导出
│       ├── api_key.rs      # API Key 模型
│       ├── auth.rs         # 认证模型
│       ├── config.rs       # 配置模型
│       ├── payment.rs      # 支付模型
│       ├── resource.rs     # 资源模型
│       ├── response.rs     # 响应模型
│       ├── user/           # 用户模型
│       │   ├── mod.rs
│       │   ├── admin.rs
│       │   ├── user.rs
│       │   ├── role.rs
│       │   ├── avatar.rs
│       │   ├── email.rs
│       │   ├── password.rs
│       │   └── profile.rs
│       └── log/            # 日志模型
│           ├── mod.rs
│           ├── error_log.rs
│           ├── operation_log.rs
│           ├── payment_log.rs
│           ├── request_log.rs
│           ├── system_log.rs
│           └── webhook_log.rs
│
├── rsws_common/            # 核心公共模块 ⭐
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # 模块导出
│       ├── error_code.rs   # 统一错误码 ⭐
│       ├── error.rs        # 统一错误类型 ⭐
│       ├── response.rs     # 统一响应格式 ⭐
│       ├── config.rs       # 配置管理
│       ├── encryption.rs   # 加密解密
│       ├── signature.rs    # 签名验证
│       ├── password.rs     # 密码处理
│       ├── snowflake.rs    # ID 生成
│       ├── email.rs        # 邮件发送
│       └── utils/          # 工具函数
│           └── mod.rs
│
├── rsws_usdt/              # USDT 监听服务
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # 模块导出
│       ├── config.rs       # USDT 配置
│       ├── listener.rs     # 监听服务
│       ├── matcher.rs      # 交易匹配
│       ├── processor.rs    # 交易处理
│       ├── tron.rs         # TRON 网络
│       └── ethereum.rs     # Ethereum 网络
│
└── static/                 # 静态资源
    ├── public/             # 公共资源
    ├── resources/          # 资源文件
    └── uploads/            # 上传文件
```

---

## 模块职责

| 模块 | 职责 | 依赖 |
|------|------|------|
| `rsws_bin` | 程序入口，启动服务 | all |
| `rsws_api` | HTTP API 处理，路由定义 | service, common |
| `rsws_service` | 业务逻辑，服务编排 | db, model, common |
| `rsws_db` | 数据库访问，SQL 执行 | model, common |
| `rsws_model` | 数据结构定义 | common |
| `rsws_common` | 核心公共功能 | 无 |
| `rsws_usdt` | USDT 监听服务 | model, common |

---

## 核心公共模块 (rsws_common)

### 错误码体系 (error_code.rs)

```rust
// 系统级错误 (1xxxx)
ErrorCode::SUCCESS              // 0
ErrorCode::BAD_REQUEST          // 10001
ErrorCode::UNAUTHORIZED         // 10002
ErrorCode::FORBIDDEN            // 10003
ErrorCode::NOT_FOUND            // 10004

// 认证错误 (2xxxx)
ErrorCode::AUTH_INVALID_CREDENTIALS  // 20001
ErrorCode::AUTH_TOKEN_EXPIRED        // 20002
ErrorCode::AUTH_SIGNATURE_INVALID    // 20004

// 用户错误 (3xxxx)
ErrorCode::USER_NOT_FOUND       // 30001
ErrorCode::USER_EMAIL_EXISTS    // 30003

// 资源错误 (4xxxx)
ErrorCode::RESOURCE_NOT_FOUND   // 40001

// 订单错误 (5xxxx)
ErrorCode::ORDER_NOT_FOUND      // 50001
ErrorCode::ORDER_EXPIRED        // 50005

// 支付错误 (6xxxx)
ErrorCode::PAYMENT_TRANSACTION_FAILED  // 60006
ErrorCode::USDT_ADDRESS_INVALID        // 60201
```

### 响应格式 (response.rs)

```rust
// 成功响应
ApiResponse::success(data)

// 错误响应
ApiResponse::error(ErrorCode::NOT_FOUND)

// 快捷方法
ApiResponse::ok()
ApiResponse::bad_request("参数错误")
ApiResponse::unauthorized("未授权")
ApiResponse::not_found("资源不存在")

// 分页响应
PaginatedData::new(items, total, page, page_size)
```

### 错误类型 (error.rs)

```rust
// 业务错误
RswsError::business(ErrorCode::USER_NOT_FOUND)

// 快捷构造
RswsError::bad_request("参数错误")
RswsError::not_found("用户不存在")

// 类型别名
type RswsResult<T> = Result<T, RswsError>;
```

---

## 技术栈

| 类别 | 技术 |
|------|------|
| **Web 框架** | Salvo (rustls) |
| **数据库** | PostgreSQL + SQLx |
| **缓存** | Redis |
| **异步运行时** | Tokio |
| **序列化** | serde + serde_json |
| **错误处理** | thiserror + anyhow |
| **加密** | argon2 + aes-gcm + sha2 |
| **邮件** | lettre (rustls) |
| **HTTP 客户端** | reqwest (rustls) |

---

## API 认证

使用 **API Key + 签名** 认证：

```
X-Api-Key: ak_xxxxxxxxxxxx
X-Timestamp: 1714848000000
X-Nonce: abc123
X-Signature: HMAC-SHA256(api_secret, method + path + timestamp + nonce + body)
```

---

## 数据库

- **文件**: `database.sql`
- **表数量**: 26
- **索引数量**: 60+
- **初始配置**: 40+

### 主要表

| 表 | 说明 |
|----|------|
| `users` | 用户表 |
| `admins` | 管理员表 |
| `resources` | 资源表 (支持 C2C) |
| `orders` | 订单表 |
| `payment_transactions` | 支付交易表 |
| `usdt_transactions` | USDT 交易记录 |
| `system_configs` | 系统配置 |
| `paypal_configs` | PayPal 配置 |
| `blockchain_configs` | 区块链配置 |

---

## 开发状态

| 模块 | 状态 | 说明 |
|------|------|------|
| rsws_common | ✅ 完成 | 核心功能已实现 |
| rsws_model | ✅ 完成 | 数据模型已定义 |
| rsws_db | 🔄 80% | 主要数据访问已实现 |
| rsws_service | 🔄 80% | 主要服务已实现 |
| rsws_api | 🔄 70% | 主要 API 已实现 |
| rsws_usdt | ✅ 完成 | USDT 监听已实现 |
| rsws_bin | ✅ 完成 | 程序入口已实现 |
| frontend | ❌ 待重构 | Vue 3 + Bun |

---

## 下一步

1. 完善数据库访问层
2. 完善业务服务层
3. 完善 API 层
4. 重构前端 (Vue 3 + Bun)
5. 集成测试
6. 部署上线
