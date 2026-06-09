# RSWS - Resource Sharing Web System

[![CI](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/ci.yml/badge.svg)](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/ci.yml)
[![Release](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/release.yml/badge.svg)](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **当前版本**: v0.1.1  
> **状态**: 编译通过，CI 全绿，Docker 部署待优化

## 项目简介

RSWS (Resource Sharing Web System) 是一个基于 Rust 构建的**数字内容交易平台**，支持创作者上传、定价、销售数字内容，并提供完整的订单管理、支付集成和后台管理功能。

### 核心功能

- **双币支付**: 支持 USDT (TRC20/ERC20) 自动监听 + PayPal 支付
- **创作者经济**: 用户可以上传数字内容（文档、软件、资源包等）并自由定价销售
- **平台抽佣**: 每笔交易自动结算平台服务费，支持灵活的佣金比例配置
- **自动监听**: 后台服务自动轮询链上交易，支付到账后自动开放下载权限
- **完善的后台管理**: 用户管理、资源审核、订单管理、支付配置、日志审计

---

## 技术栈

### 后端

| 组件 | 技术 |
|------|------|
| Web 框架 | Salvo 0.93 (Rust) |
| 数据库 | PostgreSQL 14+ + SQLx 0.8 |
| 缓存 | Redis 6+ |
| 异步运行时 | Tokio 1.52 |
| 密码哈希 | Argon2 |
| USDT 监听 | 内置 (TronGrid/Etherscan API) |
| 对象存储 | AWS S3 SDK (支持阿里云 OSS 等) |
| 容器化 | Docker + docker-compose |

### 前端

| 组件 | 技术 |
|------|------|
| 框架 | Vue 3 + TypeScript |
| 构建工具 | Vite |
| UI 组件库 | Element Plus |
| 状态管理 | Pinia |
| HTTP 客户端 | Axios |
| 包管理 | Bun / npm |

---

## 项目结构

```
RSWS_V1/
├── rsws_bin/              # 启动入口
├── rsws_api/              # HTTP API 层
│   └── src/
│       ├── handler/        # 请求处理器（已按职责拆分）
│       │   ├── admin/      # 管理后台接口（14 个子模块）
│       │   ├── custom/     # 用户端接口（4 个子模块）
│       │   ├── common/     # 共享接口（3 个子模块）
│       │   └── middleware/ # 中间件（认证、日志、限流等）
│       ├── router.rs       # 路由配置
│       └── state.rs        # 应用状态管理
├── rsws_service/          # 业务逻辑层
│   └── src/
│       ├── user_service.rs
│       ├── order_service.rs
│       ├── resource_service.rs
│       ├── payment_service.rs
│       ├── api_key_manager.rs  # API Key 统一管理（Redis）
│       ├── config_service.rs    # 动态配置管理
│       ├── log_service.rs       # 日志服务
│       └── *                   # 其他服务...
├── rsws_model/            # 数据模型层
├── rsws_db/               # 数据库访问层（已移除 API Key 数据库依赖）
├── rsws_common/           # 公共工具（错误处理、密码哈希、加密等）
├── rsws_usdt/            # USDT 链上监听服务
├── migrations/            # 数据库迁移文件
│   └── 20260608000000_initial_schema.sql  # 初始 schema
├── ui/                   # 前端项目
│   ├── user/              # 用户端前端
│   └── admin/             # 管理端前端
├── static/                # 静态资源目录
├── docker-compose.yml     # Docker 编排配置
├── Dockerfile             # 多阶段构建（编译阶段 + 运行阶段）
└── config.toml           # 配置文件模板
```

---

## 快速开始

### 1. 环境准备

**必需**:
- Rust 1.85+ (edition 2024)
- Docker Desktop 4.x+ (用于数据库和 Redis)
- PostgreSQL 14+ (如不使用 Docker)
- Redis 6+ (如不使用 Docker)

**可选**:
- WSL2 Ubuntu 24.04 (Windows 用户推荐，用于编译和运行)
- Visual Studio Build Tools 2022 (Windows 原生编译，需要额外配置)

### 2. 使用 Docker 启动数据库和 Redis（推荐）

```bash
# 启动数据库和 Redis
docker compose up -d postgres redis

# 查看运行状态
docker compose ps
```

### 3. 数据库初始化

```bash
# 手动执行初始 schema
docker exec -i rsws-postgres psql -U rsws -d rsws < migrations/20260608000000_initial_schema.sql

# 插入初始数据（管理员、测试用户、配置等）
docker exec -i rsws-postgres psql -U rsws -d rsws < scripts/initial_data.sql
```

### 4. 配置环境变量

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑 .env
# 设置数据库连接、Redis 连接、APP_SECRET 等
```

关键配置项：
```bash
# 数据库
RSWS_DATABASE_URL=postgresql://rsws:rsws_secret@127.0.0.1:5432/rsws

# Redis
RSWS_REDIS_URL=redis://:rsws_redis@127.0.0.1:6379

# 应用密钥（用于 API Key 加密）
RSWS_APP_SECRET=your-strong-secret-key-here

# 服务器端口
RSWS_API_PORT=5170
```

### 5. 编译和运行

**方案 A: WSL2 编译和运行（推荐，避免 Docker OOM）**

```bash
# 在 WSL2 Ubuntu 24.04 中
cd /mnt/f/GitRepo/RSWS_V1
source ~/.cargo/env
cargo build --release --bin resource-sharing-web-system

# 运行
export RSWS_DATABASE_URL="postgresql://rsws:rsws_secret@127.0.0.1:5432/rsws"
export RSWS_REDIS_URL="redis://:rsws_redis@127.0.0.1:6379"
./target/release/resource-sharing-web-system
```

**方案 B: Docker 内编译（需要 ≥16GB 内存，可能 OOM）**

```bash
# 构建并启动所有服务
docker compose up -d

# 查看日志
docker compose logs -f rsws
```

> **注意**: Docker 内编译 `aws-lc-sys` 时容易 OOM。如果失败，请使用方案 A。

### 6. 验证运行

```bash
# 健康检查
curl http://localhost:5170/health

# 管理员登录
curl -X POST http://localhost:5170/api/v1/admin/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@rsws.local","password":"Admin123!@#"}'
```

---

## 架构要点

### 1. 分层架构

```
rsws_bin (启动入口)
    ↓
rsws_api (HTTP API 层 - Handler/Router/Middleware)
    ↓
rsws_service (业务逻辑层 - Service)
    ↓
rsws_db (数据库访问层 - Repository)
    ↓
rsws_model (数据模型层 - Entity/DTO)
```

### 2. Handler 三层拆分

为清晰分离管理后台、用户端和共享逻辑，Handler 已按职责拆分为三层：

- **`handler/admin/`** - 管理后台接口（14 个子模块）
  - `auth.rs` - 管理员登录/登出
  - `user.rs` - 用户管理
  - `resource.rs` - 资源管理
  - `order.rs` - 订单管理
  - `category.rs` - 分类管理
  - `log.rs` - 日志查询
  - `log_config.rs` - 日志配置
  - `payment_method.rs` - 支付方式配置
  - `dashboard.rs` - 控制台数据
  - `wallet.rs` - 钱包管理
  - `api_key.rs` - API Key 管理
  - `login_log.rs` - 登录日志
  - `error_log.rs` - 错误日志
  - `audit_log.rs` - 审计日志

- **`handler/custom/`** - 用户端接口（4 个子模块）
  - `auth.rs` - 用户注册/登录
  - `resource.rs` - 资源浏览/购买
  - `order.rs` - 订单管理
  - `wallet.rs` - 钱包管理

- **`handler/common/`** - 共享接口（3 个子模块）
  - `category.rs` - 分类查询
  - `upload.rs` - 文件上传
  - `health.rs` - 健康检查

### 3. API Key 管理（纯 Redis 存储）

**设计决策**: API Key 采用 **纯 Redis 存储**，不落库。

- **优点**: 高性能、无需持久化、安全隔离
- **键格式**: `rsws:api_key:{key_id}`
- **实现**: `rsws_service/src/api_key_manager.rs` 统一管理 Admin/User API Key

**已删除的数据库依赖** (2026-06-07):
- `rsws_db/src/user_api_key.rs` - 已删除
- `rsws_db/src/admin.rs` 中的 8 个 `admin_api_keys` DB 函数 - 已删除
- `user_api_keys` 和 `admin_api_keys` 表 - 已废弃

### 4. 配置管理模式

**核心理念**: 配置存储在数据库中，支持动态更新无需重启服务。

- **实现**: `rsws_service/src/config_service.rs`
- **存储**: PostgreSQL + Redis 缓存
- **配置表**: `paypal_configs`, `blockchain_configs`, `email_configs`, `log_configs`, `usdt_listen_configs`

### 5. 日志系统（四节点）

系统实现了完整的日志审计体系：

| 日志类型 | 表名 | 用途 |
|---------|------|------|
| 系统日志 | `system_logs` | API 调用记录（复用） |
| 登录日志 | `login_logs` | 用户登录成功/失败记录 |
| 错误日志 | `error_logs` | 系统异常记录 |
| 审计日志 | `audit_logs` | 管理员操作审计 |

**日志级别动态配置**: 通过 `log_configs` 表配置不同操作类型（`operation_type`）的日志级别和是否启用。

**Tracing 中间件**: `rsws_api/src/middleware/tracing.rs` 自动记录所有 API 请求，包括：
- 请求 ID（UlidGenerator，Salvo 内置）
- 状态码（按级别分级）
- 慢请求告警（>1s）

### 6. 密码哈希（Argon2）

**已修复**: 数据库中的密码哈希已从 bcrypt 更新为 Argon2。

- **实现**: `rsws_common/src/password.rs`
- **算法**: Argon2id (内存 19MB, 2 次迭代, 1 个并行度)
- **管理员密码**: `Admin123!@#` → `$argon2id$v=19$m=19456,t=2,p=1$...`

### 7. 请求链路 ID（Salvo 内置）

**已优化**: 使用 Salvo 内置的 `RequestId::new()` 中间件，替代手写的 UUID 实现。

- **生成器**: UlidGenerator（时间戳排序 + 随机性）
- **优势**: 比 UUID 更适合分布式日志追踪
- **中间件顺序**: `affix_state` → `RequestId::new()` → `CORS` → `tracing`

### 8. 中间件体系

```
请求 → affix_state (注入 AppState)
    ↓
  RequestId (生成请求 ID)
    ↓
  CORS (跨域处理)
    ↓
  tracing (请求日志)
    ↓
  rate_limit (限流)
    ↓
  api_key_auth (API Key 认证)
    ↓
  Handler (业务处理)
```

---

## API 认证

### 当前状态

⚠️ **已知问题**: Admin API 的 X-API-Key 认证未生效，需要签名认证（`user_id`, `timestamp`, `nonce`, `sign`）。

### 签名认证方案（Cregis 单密钥）

所有受保护的 API 端点需要在 Query 参数中携带签名：

| 参数 | 说明 | 是否传输 |
|------|------|----------|
| `user_id` | 用户 ID（公开标识符） | ✅ 传输 |
| `timestamp` | 时间戳 (毫秒) | ✅ 传输 |
| `nonce` | 随机字符串 (6位) | ✅ 传输 |
| `sign` | MD5 签名 | ✅ 传输 |
| `api_key` | API 密钥（签名密钥） | ❌ **不传输** |

#### 签名算法

1. 排除 `sign` 参数，按 key ASCII 升序排序
2. 拼接: `key1 + value1 + key2 + value2 + ...`
3. 将 `api_key` 拼接到字符串最前面
4. MD5 计算并转小写 hex

#### 示例

```
API Key（密钥，不传输）: f502a9ac9ca54327986f29c03b271491

签名输入:
f502a9ac9ca54327986f29c03b271491  ← API Key（密钥）
+ address=TXsmKpEuW7qWnXzJLGP9eDLvWWPR2GRn1FS
+ amount=1.1
+ nonce=hwlkk6
+ timestamp=1688004243314

→ MD5 → sign = d6eef2de79e39f434a38efb910213ba6
```

---

## API 文档

启动后访问：

- **Swagger UI**: `http://localhost:5170/swagger-ui/`
- **OpenAPI JSON**: `http://localhost:5170/openapi.json`

---

## 开发指南

### 添加新 API

1. 在 `rsws_model` 定义数据结构
2. 在 `rsws_db` 实现 Repository（如需要数据库访问）
3. 在 `rsws_service` 实现业务逻辑
4. 在 `rsws_api/src/handler/` 添加 Handler
5. 在 `rsws_api/src/router.rs` 注册路由

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
# 格式化
cargo fmt --all

# 编译检查
cargo check --workspace

# Clippy 检查
cargo clippy --all-targets --all-features -- -D warnings
```

---

## CI/CD

### CI 流水线 (`.github/workflows/ci.yml`)

**触发条件**: Push 或 Pull Request 到 `main` 分支

**执行步骤**:
1. 代码格式化检查 (`cargo fmt`)
2. Clippy 静态分析 (`cargo clippy`)
3. 编译检查 (`cargo build --all`)
4. 单元测试 (`cargo test --all`)
5. 安全审计 (`cargo audit`)

**服务依赖**:
- PostgreSQL 14 (用于集成测试)
- Redis 6 (用于集成测试)

### Release 流水线 (`.github/workflows/release.yml`)

**触发条件**: 推送 `v*` 标签

**执行步骤**:
1. 构建 Docker 镜像 (amd64)
2. 推送到 GitHub Container Registry (GHCR)
3. 生成 SBOM (Software Bill of Materials)
4. 创建 GitHub Release

**镜像名称**: `ghcr.io/{owner}/rsws-v1:{version}`

---

## 部署

### 当前限制

❌ **Docker 内编译**: 容易 OOM（尤其是 `aws-lc-sys` 编译）  
❌ **GLIBC 版本**: WSL2 Ubuntu 24.04 编译的二进制需要 GLIBC 2.38，但 `debian:bookworm-slim` 只有 2.36

### 推荐部署方案

**方案 A: WSL2 直接运行（当前使用）**

```bash
# 在 WSL2 Ubuntu 24.04 中编译
cargo build --release

# 设置环境变量
export RSWS_DATABASE_URL="postgresql://rsws:rsws_secret@127.0.0.1:5432/rsws"
export RSWS_REDIS_URL="redis://:rsws_redis@127.0.0.1:6379"

# 运行
./target/release/resource-sharing-web-system
```

**方案 B: Docker 部署（需要修复）**

待解决问题：
1. Docker 内编译 OOM → 使用预编译二进制 + 多阶段构建
2. GLIBC 版本不兼容 → 使用 `ubuntu:24.04` 作为运行时镜像

---

## 版本规划

### v0.1.1 (当前) ✅

- [x] 项目骨架重构
- [x] Handler 三层拆分（admin/custom/common）
- [x] API Key 统一管理系统（纯 Redis 存储）
- [x] 日志系统四节点（system/login/error/audit）
- [x] Tracing 中间件（请求日志、慢请求告警）
- [x] RequestId 中间件（Salvo 内置 UlidGenerator）
- [x] 密码哈希修复（Argon2）
- [x] API 集成测试框架
- [x] GitHub Actions CI/CD
- [x] Docker 部署配置（待优化）
- [x] CORS 中间件
- [x] 速率限制中间件

### v0.2.0

- [ ] 修复 Admin API 认证（X-API-Key 中间件）
- [ ] 用户上传审核流程
- [ ] Email 通知服务（注册验证、密码重置）
- [ ] Docker 部署优化（多阶段构建、预编译二进制）
- [ ] 静态文件服务（上传文件下载）

### v0.3.0

- [ ] 资源预览功能
- [ ] 提现流程
- [ ] 数据报表完善
- [ ] 佣金自动结算优化

### v1.0.0

- [ ] 完整功能闭环
- [ ] 性能优化（缓存策略、数据库索引）
- [ ] 安全加固（SQL 注入防护、XSS 防护、CSRF 防护）
- [ ] 生产环境部署文档

---

## 已知问题

### 1. Docker 部署

- **问题**: Docker 内编译 OOM（15.5GB 内存不足）
- **临时方案**: WSL2 直接运行
- **待修复**: 使用预编译二进制 + 多阶段构建

### 2. GLIBC 版本不兼容

- **问题**: WSL2 Ubuntu 24.04 编译的二进制需要 GLIBC 2.38，`debian:bookworm-slim` 只有 2.36
- **临时方案**: WSL2 直接运行
- **待修复**: 使用 `ubuntu:24.04` 作为运行时镜像

### 3. Admin API 认证

- **问题**: X-API-Key 认证未生效，需要签名认证
- **状态**: 待修复
- **影响**: 管理后台 API 无法使用 API Key 认证

### 4. 数据库迁移

- **问题**: `sqlx::migrate!()` 已注释（迁移文件已删除）
- **状态**: 使用初始 schema 手动执行
- **待修复**: 重新生成准确的迁移文件

---

## 贡献

欢迎提交 Issue 和 Pull Request。

**开发规范**:
1. 所有新功能必须包含单元测试
2. 所有 API 变更必须更新 OpenAPI 文档
3. 所有数据库变更必须包含迁移文件
4. 代码必须通过 `cargo fmt` 和 `cargo clippy` 检查

---

## 许可证

MIT License

---

## 联系方式

- **作者**: Noob-Xiye
- **GitHub**: https://github.com/Noob-Xiye/RSWS_V1
- **Issues**: https://github.com/Noob-Xiye/RSWS_V1/issues

---

**最后更新**: 2026-06-09
