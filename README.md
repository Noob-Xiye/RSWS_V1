# RSWS V1 - 数字内容交易平台

[![CI](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/ci.yml/badge.svg)](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/ci.yml)
[![Release](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/release.yml/badge.svg)](https://github.com/Noob-Xiye/RSWS_V1/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **版本**: 0.1.0  
> **状态**: 开发中

## 项目简介

RSWS (Resource Sharing Web System) 是一个数字内容付费交易平台，支持：

- **平台自营**: 平台提供付费内容（文档、软件、资源包等）
- **用户上传**: 创作者上传内容，设置价格销售
- **平台抽佣**: 从每笔交易中收取平台服务费
- **多币支付**: USDT (TRC20/ERC20) + PayPal

### 典型应用场景

- Minecraft MOD 整合包销售
- 设计素材、文档模板
- 软件工具、插件资源
- 教程课程、知识付费

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
- [x] PayPal 支付（Webhook 真实签名验证）

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

| 参数 | 说明 |
|------|------|
| `api_key` | API Key (登录后获取) |
| `timestamp` | 时间戳 (毫秒) |
| `nonce` | 随机字符串 |
| `sign` | MD5 签名 |

### 签名算法

```
1. 排除 sign 参数，按 key ASCII 升序排序
2. 拼接: api_secret + key1 + value1 + key2 + value2 + ...
3. MD5 计算并转小写 hex
```

### 示例

```
参数: api_key=test&timestamp=1234567890&nonce=abc123
API Secret: my_secret

排序后: api_key, nonce, timestamp
拼接: my_secret + api_key + test + nonce + abc123 + timestamp + 1234567890
MD5: f6b5f8c3e2a1d4c7b9e0f1a2b3c4d5e6
```

### 时间戳验证

- 允许 ±5 分钟偏差
- 防止请求被重放攻击

### 前端实现

```typescript
import CryptoJS from 'crypto-js';

function generateSignature(params: Record<string, string>, apiSecret: string): string {
  const keys = Object.keys(params)
    .filter(key => key !== 'sign')
    .sort();
  
  const paramStr = keys.map(key => key + params[key]).join('');
  const signStr = apiSecret + paramStr;
  return CryptoJS.MD5(signStr).toString();
}
```

### 后端验证

Rust 实现使用 `md5` crate:

```rust
use std::collections::HashMap;
use md5;

fn compute_signature(params: &HashMap<String, String>, api_secret: &str) -> String {
    let mut keys: Vec<&String> = params.keys()
        .filter(|k| (*k).as_str() != "sign")
        .collect();
    keys.sort();
    
    let param_str: String = keys.iter()
        .map(|k| format!("{}{}", k, params[*k]))
        .collect();
    
    let sign_str = format!("{}{}", api_secret, param_str);
    format!("{:x}", md5::compute(sign_str.as_bytes()))
}
```

---

## 版本规划

### v0.1.0 (当前) ✅

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
