# RSWS V1 - 数字内容交易平台

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
| 数据库 | PostgreSQL |
| 缓存 | Redis |
| 异步运行时 | Tokio |
| USDT 监听 | 内置 (TronGrid/Etherscan API) |

### 前端

| 组件 | 技术 |
|------|------|
| 框架 | React 18 + TypeScript |
| 构建 | Vite |
| UI | Ant Design |
| 状态管理 | Context API + useReducer |
| HTTP | Axios |

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
├── rsws_usdt/         # USDT 监听服务 (NEW in v0.1.0)
├── ui/
│   ├── user/          # 用户端前端
│   └── admin/         # 管理端前端
├── sql/               # 数据库脚本
├── static/            # 静态资源
└── config.toml        # 配置文件
```

---

## 快速开始

### 1. 环境准备

- Rust 1.70+
- Node.js 18+
- PostgreSQL 14+
- Redis 6+

### 2. 数据库初始化

```bash
# 创建数据库
createdb rsws

# 执行迁移脚本
psql -d rsws -f sql/unified_schema.sql
psql -d rsws -f sql/usdt_tables.sql
psql -d rsws -f sql/missing_tables.sql
```

### 3. 配置

编辑 `config.toml`：

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://user:pass@localhost:5432/rsws"

[redis]
url = "redis://localhost:6379"
```

### 4. 启动后端

```bash
cargo run --release
```

### 5. 启动前端

```bash
# 用户端
cd ui/user
npm install
npm run dev

# 管理端
cd ui/admin
npm install
npm run dev
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
- [x] USDT 监听配置 (NEW)
- [x] 数据报表

### 支付

- [x] USDT (TRC20) 支付
- [x] USDT (ERC20) 支付
- [ ] PayPal 支付 (计划中)

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

## 版本规划

### v0.1.0 (当前)

- [x] 项目骨架重构
- [x] USDT 监听服务
- [x] 基础支付流程
- [x] 管理后台基础

### v0.2.0

- [ ] PayPal Webhook
- [ ] 用户上传审核流程
- [ ] 佣金自动结算

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

---

## 部署

详见 [DEPLOYMENT.md](./DEPLOYMENT.md)

---

## 许可证

MIT License

---

## 贡献

欢迎提交 Issue 和 Pull Request。
