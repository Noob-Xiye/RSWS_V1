# RSWS V1 - 数字内容交易平台

[![CI](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/ci.yml/badge.svg)](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/ci.yml)
[![Release](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/release.yml/badge.svg)](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **版本**: 0.1.1  
> **状态**: 生产可用

## 项目简介

RSWS (Resource Sharing Web System) 是一个功能完整的**数字内容付费交易平台**，采用 Rust 高性能后端 + Vue 3 现代前端构建。

### 核心功能

- **双币支付**: 原生支持 USDT (TRC20/ERC20) 自动监听 + PayPal 支付
- **创作者经济**: 用户可上传数字内容（文档、软件、资源包等）并自由定价销售
- **平台抽佣**: 每笔交易自动结算平台服务费，支持灵活的佣金比例配置
- **自动监听**: 后台服务自动轮询链上交易，支付到账后秒级开放下载权限
- **管理后台**: 全面的运营管理套件——用户、资源、订单、支付配置、数据报表

### 典型应用场景

- 游戏 MOD / 整合包销售平台
- 设计素材 / 文档模板交易
- 软件工具 / 插件分发
- 教程课程 / 知识付费
- 数字艺术品交易

---

## 技术栈

### 后端

| 组件 | 技术 |
|------|------|
| Web 框架 | Salvo (Rust) |
| 数据库 | PostgreSQL + SQLx |
| 缓存 | Redis |
| 异步运行时 | Tokio |
| USDT 监听 | 内置 (TronGrid/Etherscan API) |
| 容器化 | Docker + docker-compose |

### 前端

| 组件 | 技术 |
|------|------|
| 框架 | Vue 3 + TypeScript |
| 构建 | Vite |
| UI | Element Plus |
| 状态管理 | Pinia |
| HTTP | Axios |
| 包管理 | Bun |

---

## 项目结构

```
RSWS_V1/
├── rsws_bin/          # 启动入口
├── rsws_api/          # HTTP API 层
├── rsws_service/      # 业务逻辑层
├── rsws_model/        # 数据模型层
├── rsws_db/           # 数据库访问层
├── rsws_common/       # 公共工具
├── rsws_usdt/         # USDT 监听服务
├── migrations/        # 数据库迁移文件
├── ui/
│   ├── user/          # 用户端前端
│   └── admin/         # 管理端前端
├── static/            # 静态资源
└── config.toml        # 配置文件
```

---

## 快速开始

### 1. 环境准备

- Rust 1.85+
- Docker & Docker Compose（推荐）
- PostgreSQL 14+（如不使用 Docker）
- Redis 6+（如不使用 Docker）

### 2. 使用 Docker 启动（推荐）

```bash
# 启动开发环境（仅数据库和 Redis）
docker compose -f docker-compose.dev.yml up -d

# 或启动完整生产环境
docker compose up -d
```

### 3. 数据库初始化

项目启动时自动运行数据库迁移：

```bash
# 首次启动会自动创建表结构和初始数据
cargo run --release
```

迁移文件位于 `migrations/` 目录。

### 4. 配置

复制环境变量模板：

```bash
cp .env.example .env
```

编辑 `.env` 或通过环境变量覆盖 `config.toml`：

```bash
export RSWS_DATABASE_URL="postgresql://user:pass@localhost:5432/rsws"
export RSWS_REDIS_URL="redis://localhost:6379"
```

### 5. 启动后端

```bash
cargo run --release
```

### 6. 启动前端

```bash
# 用户端
cd ui/user
bun install
bun run dev

# 管理端
cd ui/admin
bun install
bun run dev
```

---

## 核心功能

### 用户端

- [x] 注册/登录/找回密码
- [x] 浏览资源列表
- [x] 查看资源详情
- [x] 付费购买资源
- [x] 下载已购资源
- [x] 上传资源（创作者）
- [x] 钱包管理
- [x] 订单管理

### 管理端

- [x] 用户管理
- [x] 资源审核
- [x] 订单管理
- [x] 支付配置
- [x] USDT 监听配置
- [x] 数据报表
- [x] 日志配置管理

### 支付

- [x] USDT (TRC20) 支付
- [x] USDT (ERC20) 支付
- [x] PayPal 支付（带 HMAC 签名验证）

---

## USDT 支付流程

```
用户下单 → 生成收款地址 → 展示支付二维码
                                    ↓
用户转账 USDT ← ← ← ← ← ← ← ← ← ← ←
                                    ↓
USDT 监听服务检测交易 → 匹配订单 → 确认支付
                                    ↓
开放下载权限 ← ← ← ← ← ← ← ← ← ← ←
```

### USDT 监听配置

在管理后台配置：

1. **收款地址**: 添加 TRC20/ERC20 收款地址
2. **监听参数**: 轮询间隔、最小确认数
3. **API 配置**: TronGrid/Etherscan API Key

---

## API 文档

启动后访问：

- Swagger UI: `http://localhost:8080/swagger-ui/`
- OpenAPI JSON: `http://localhost:8080/openapi.json`

---

## API 认证

### Cregis 签名方案

所有受保护的 API 端点需要在 Query 参数中携带签名：

| 参数 | 说明 | 传输？ |
|------|------|--------|
| `user_id` | 用户 ID（公开标识符） | ✅ 传输 |
| `timestamp` | 时间戳 (毫秒) | ✅ 传输 |
| `nonce` | 随机字符串 (6位) | ✅ 传输 |
| `sign` | MD5 签名 | ✅ 传输 |
| `api_key` | API 密钥（签名密钥） | ❌ **不传输** |

### 签名算法（Cregis 单密钥方案）

```
1. 排除 sign 参数，按 key ASCII 升序排序
2. 拼接: key1 + value1 + key2 + value2 + ...
3. 将 api_key 拼接到字符串最前面（Cregis 方案）
4. MD5 计算并转小写 hex
```

> **注意**: `api_key` 是签名密钥，**不传输**；`user_id` 是公开标识符，**传输**，用于查库获取 `api_key`。

### 示例（Cregis 方案）

```
API Key（密钥，不传输）: f502a9ac9ca54327986f29c03b271491

签名输入（API Key 前置到排序参数）:
f502a9ac9ca54327986f29c03b271491  ← API Key（密钥）
+ address=TXsmKpEuW7qWnXzJLGP9eDLvWPR2GRn1FS
+ amount=1.1
+ nonce=hwlkk6
+ timestamp=1688004243314
→ MD5 → sign = d6eef2de79e39f434a38efb910213ba6
```

**最终发送的请求**（包含 `sign`，**不包含 `api_key`**）:
```json
{
  "user_id": 123,
  "address": "TXsmKpEuW7qWnXzJLGP9eDLvWWPR2GRn1FS",
  "amount": "1.1",
  "nonce": "hwlkk6",
  "timestamp": 1688004243314,
  "sign": "d6eef2de79e39f434a38efb910213ba6"
}
```

### 时间戳验证

- 允许 ±5 分钟偏差
- 防止请求被重放攻击

### 前端实现（Cregis 单密钥方案）

```typescript
import { MD5 } from 'crypto-js';

interface SignParams {
  [key: string]: string;
}

function generateSignParams(params: SignParams, apiKey: string): SignParams {
  // 1. 添加防重放字段
  params['timestamp'] = Date.now().toString();
  params['nonce'] = Math.random().toString(36).substring(2, 8);
  
  // 2. 排除 sign，按 key ASCII 升序排序
  const keys = Object.keys(params)
    .filter(key => key !== 'sign')
    .sort();
  
  // 3. 拼接参数字符串（key + value）
  const paramStr = keys.map(key => key + params[key]).join('');
  
  // 4. 将 apiKey 拼到最前面（Cregis 方案）
  const signStr = apiKey + paramStr;
  
  // 5. MD5 + 小写 hex
  params['sign'] = MD5(signStr).toString();
  
  return params;
}

// 使用示例
const apiKey = localStorage.getItem('apiKey'); // api_key，不传输
const params = { user_id: '123', page: '1', size: '20' };
const signedParams = generateSignParams(params, apiKey);
// signedParams 包含 sign，不包含 apiKey
```

### 后端验证（Cregis 单密钥方案）

Rust 实现使用 `md5` crate:

```rust
use std::collections::HashMap;
use md5;

/// 计算签名（Cregis 方案：api_key 前置）
fn compute_signature(params: &HashMap<String, String>, api_key: &str) -> String {
    // 1. 获取所有 key（排除 sign），排序
    let mut keys: Vec<&String> = params.keys()
        .filter(|k| (*k).as_str() != "sign")
        .collect();
    keys.sort();
    
    // 2. 按 ASCII 顺序拼接 key + value
    let param_str: String = keys
        .iter()
        .map(|k| format!("{}{}", k, params[*k]))
        .collect();
    
    // 3. api_key 拼在最前面（Cregis 方案）
    let sign_str = format!("{}{}", api_key, param_str);
    
    // 4. MD5 + 小写 hex
    format!("{:x}", md5::compute(sign_str.as_bytes()))
}

/// 验证签名
async fn verify_signature(
    user_id: i64,
    params: &HashMap<String, String>,
    sign: &str,
) -> Result<i64, RswsError> {
    // 1. 通过 user_id 查找 api_key
    let api_key_record = api_key_repo
        .get_active_key_by_user_id(user_id)
        .await?
        .ok_or_else(|| RswsError::business(ErrorCode::AUTH_INVALID_API_KEY))?;
    
    // 2. 重算签名
    let computed_sign = compute_signature(params, &api_key_record.api_key);
    
    // 3. 对比签名
    if computed_sign != sign {
        return Err(RswsError::business(ErrorCode::AUTH_INVALID_SIGNATURE));
    }
    
    // 4. 更新最后使用时间
    api_key_repo.update_last_used(api_key_record.id).await?;
    
    Ok(user_id)
}
```

---

## 版本规划

### v0.1.1 (当前) ✅

- [x] 项目骨架重构
- [x] USDT 监听服务
- [x] 基础支付流程
- [x] 管理后台基础
- [x] API Key 统一认证
- [x] Redis 速率限制
- [x] PayPal Webhook 签名验证
- [x] 佣金结算系统
- [x] 数据库迁移体系 (sqlx migrate)
- [x] Docker 部署配置
- [x] CORS 中间件
- [x] API 集成测试框架
- [x] GitHub Actions CI/CD

### v0.2.0

- [ ] 用户上传审核流程
- [ ] 佣金自动结算优化
- [ ] Email 通知服务

### v0.3.0

- [ ] 资源预览功能
- [ ] 提现流程
- [ ] 数据报表完善

### v1.0.0

- [ ] 完整功能闭环
- [ ] 性能优化
- [ ] 安全加固

---

## 开发指南

### 添加新 API

1. 在 `rsws_model` 定义数据结构
2. 在 `rsws_db` 实现 Repository
3. 在 `rsws_service` 实现业务逻辑
4. 在 `rsws_api` 添加路由和 Handler

### 添加新支付方式

1. 在 `rsws_service/payment_service.rs` 扩展
2. 在 `rsws_api` 添加回调接口
3. 在前端添加支付选项

### 运行测试

```bash
# 单元测试
cargo test --lib

# 集成测试（需要数据库）
cargo test --test api_integration
```

### 代码检查

```bash
# 编译检查
cargo check

# Clippy 检查
cargo clippy
```

---

## 部署

详见 [DEPLOYMENT.md](./DEPLOYMENT.md)

---

## 许可证

MIT License

---

## 贡献

欢迎提交 Issue 和 Pull Request。

---

## 架构要点与核心理念

> **重要**: 本节记录项目的关键架构决策和实现细节，确保后续接手的开发者能快速理解项目设计。

### 1. 数据库配置管理模式

**核心理念**: 配置存储在数据库中，而非配置文件，实现动态配置更新无需重启服务。

```
┌─────────────────────────────────────────────────────────────────┐
│                        ConfigService                            │
│  (rsws_service/src/config_service.rs)                          │
├─────────────────────────────────────────────────────────────────┤
│  职责: 从数据库读取各类配置（PayPal、Blockchain、Email 等）       │
│  存储: PostgreSQL + Redis 缓存                                  │
│  特点: 支持热更新，无需重启服务                                  │
└─────────────────────────────────────────────────────────────────┘
```

**配置表结构**:
- `paypal_configs` - PayPal 支付配置
- `blockchain_configs` - 区块链监听配置
- `email_configs` - 邮件服务配置
- `usdt_listen_configs` - USDT 监听地址配置
- `encryption_configs` - 加密配置（预留）

**配置安全方案** (2026-06-05 决策):
- 采用 **方案B: 明文存储 + 数据库权限保护**
- `encrypted` 字段为历史遗留，数据库实际存储明文
- 生产环境通过 Docker 内部网络隔离数据库
- `encryption.rs` 和 `encryption_configs` 表保留待 v1.0.0 清理

### 2. API Key 存储架构

**核心理念**: API Key 采用 **纯 Redis 存储**，不落库。

```
┌─────────────────────────────────────────────────────────────────┐
│                      API Key 架构                               │
├─────────────────────────────────────────────────────────────────┤
│  存储位置: Redis Only (标记: "API Key Only Redis")              │
│  键格式:   rsws:api_key:{key_id}                                │
│  原因:    高频访问、无需持久化、安全隔离                         │
└─────────────────────────────────────────────────────────────────┘
```

**已删除的废弃代码** (2026-06-07):
- `rsws_db/src/user_api_key.rs` - 删除
- `rsws_db/src/admin.rs` 中的 8 个 `admin_api_keys` DB 函数 - 删除
- `rsws_api/src/handler/admin.rs` 中的 4 个用户 API Key 管理函数 - 删除
- 对应路由 - 删除

**保留代码**:
- `list_api_keys` 等纯 Redis API Key 管理函数

### 3. 应用状态管理

**核心理念**: 通过 `AppState` 统一管理所有服务实例，使用 `Arc` 实现线程安全共享。

```rust
// 位置: rsws_api/src/state.rs
pub struct AppState {
    pub pool: PgPool,                                    // 数据库连接池
    pub config: AppConfig,                               // 应用配置
    pub user_service: Arc<UserService>,                  // 用户服务
    pub order_service: Arc<OrderService>,                // 订单服务
    pub resource_service: Arc<ResourceService>,          // 资源服务
    pub api_key_service: Arc<ApiKeyService>,             // API Key 服务 (Redis)
    pub paypal_service: Arc<PayPalService>,              // PayPal 服务
    pub payment_service: Arc<PaymentService>,            // 支付服务
    pub blockchain_service: Arc<BlockchainService>,      // 区块链服务
    pub webhook_service: Arc<WebhookService>,            // Webhook 服务
    pub config_service: Arc<ConfigService>,              // 配置服务
    pub admin_service: Arc<AdminService>,                // 管理服务
    pub log_service: Arc<LogService>,                    // 日志服务
    pub category_service: Arc<CategoryRepository>,       // 分类服务
}
```

**依赖关系**:
- `ConfigService` 需要 `Clone` trait，用于多处共享
- 所有服务通过 `Arc` 包装，支持跨线程共享
- `AppState::new()` 接收各服务实例，统一初始化

### 4. 错误处理体系

**核心理念**: 分层错误处理，领域错误独立定义，统一转换为 `RswsError`。

```
┌─────────────────────────────────────────────────────────────────┐
│                      错误处理层次                               │
├─────────────────────────────────────────────────────────────────┤
│  Layer 1: 领域错误 (UploadError, etc.)                          │
│           - 使用 thiserror 派生                                 │
│           - 中文错误信息，符合项目风格                           │
│                                                                 │
│  Layer 2: 通用错误 RswsError (rsws_common)                      │
│           - BadRequest, Unauthorized, Forbidden, NotFound, etc. │
│           - 提供 bad_request(), unauthorized() 等便捷方法        │
│                                                                 │
│  Layer 3: HTTP 响应 (handler 层)                               │
│           - res.error(RswsError::xxx())                         │
│           - 自动转换为 HTTP 状态码和 JSON 响应                  │
└─────────────────────────────────────────────────────────────────┘
```

**UploadError 设计** (2026-06-07 实现):
```rust
// 位置: rsws_api/src/handler/upload.rs
#[derive(Error, Debug)]
pub enum UploadError {
    #[error("缺少 content-type 头")]
    MissingContentType,
    
    #[error("Multipart boundary 解析失败: {0}")]
    BoundaryParseFailed(String),
    
    #[error("请求体读取失败: {0}")]
    BodyReadFailed(String),
    
    #[error("文件大小超过限制: {0}")]
    FileSizeExceeded(String),
    
    // ... 更多变体
}

// multer::Error → UploadError 转换
impl From<multer::Error> for UploadError { ... }
```

**注意事项**:
- `RswsError` 有 blanket 实现 `impl<T: Into<String>> From<T>`，避免冲突
- 领域错误调用时显式转换: `e.to_string()` 或 `RswsError::bad_request(e.to_string())`

### 5. 文件上传架构

**核心理念**: 支持 OSS 分块上传和单文件上传，使用 `multer` 解析 multipart 请求。

```
┌─────────────────────────────────────────────────────────────────┐
│                      文件上传流程                               │
├─────────────────────────────────────────────────────────────────┤
│  1. 客户端发起 multipart/form-data 请求                         │
│  2. Salvo ReqBody → http_body_util::BodyStream                 │
│  3. BodyStream → multer::Multipart 解析                        │
│  4. 提取文件字段 → 上传到 OSS                                   │
│  5. 返回文件 URL                                               │
└─────────────────────────────────────────────────────────────────┘
```

**关键技术点**:
- 使用 `http_body_util::BodyStream` 转换 Salvo 请求体
- `multer::Multipart::new()` 需要 `Stream<Item = Result<Bytes, multer::Error>>`
- 错误处理统一到 `UploadError`

**依赖**:
- `multer` - multipart 解析
- `http-body-util` - Body 流转换
- `futures-util` - Stream 扩展
- `thiserror` - 错误派生

### 6. USDT 支付监听服务

**核心理念**: 后台服务自动轮询链上交易，支付到账后自动确认订单。

```
用户下单 → 生成收款地址 → 展示支付二维码
                                    ↓
用户转账 USDT ← ← ← ← ← ← ← ← ← ← ←
                                    ↓
USDT 监听服务检测交易 → 匹配订单 → 确认支付
                                    ↓
开放下载权限 ← ← ← ← ← ← ← ← ← ← ←
```

**监听服务**:
- Tron 监听: 使用 TronGrid API
- ERC20 监听: 使用 Etherscan API
- 配置来源: 数据库 `usdt_listen_configs` 表

### 7. 分层架构

```
┌─────────────────────────────────────────────────────────────────┐
│ rsws_bin/        启动入口，组装所有依赖                          │
│                  - 读取配置                                     │
│                  - 初始化数据库和 Redis                          │
│                  - 创建所有 Service 实例                         │
│                  - 启动 USDT 监听服务                           │
├─────────────────────────────────────────────────────────────────┤
│ rsws_api/        HTTP API 层，路由和 Handler                     │
│                  - 请求解析                                     │
│                  - 响应序列化                                   │
│                  - 认证中间件                                   │
│                  - 错误处理                                     │
├─────────────────────────────────────────────────────────────────┤
│ rsws_service/    业务逻辑层                                       │
│                  - 领域服务                                     │
│                  - 业务规则验证                                 │
│                  - 跨服务协调                                   │
├─────────────────────────────────────────────────────────────────┤
│ rsws_db/         数据库访问层                                     │
│                  - Repository 模式                              │
│                  - SQL 查询封装                                 │
├─────────────────────────────────────────────────────────────────┤
│ rsws_model/      数据模型层                                       │
│                  - 实体定义                                     │
│                  - DTO                                         │
├─────────────────────────────────────────────────────────────────┤
│ rsws_common/     公共工具                                         │
│                  - 配置结构                                     │
│                  - 错误类型                                     │
│                  - 响应扩展                                     │
└─────────────────────────────────────────────────────────────────┘
```

### 8. 关键依赖版本

| Crate | 版本 | 用途 |
|-------|------|------|
| salvo | 0.93 | Web 框架 |
| sqlx | 0.8 | 数据库 |
| multer | 3.1 | Multipart 解析 |
| thiserror | 2.0 | 错误派生 |
| http-body-util | 0.1 | Body 流转换 |
| tokio | 1.x | 异步运行时 |
