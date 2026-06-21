# Resource Sharing Web System (RSWS)

资源分享 Web 系统，支持资源展示、用户认证、订单支付（USDT/PayPal）、用户端与管理后台分离架构。

## 项目结构

```
RSWS_V1/
├── frontend/
│   ├── admin/        # 管理后台前端（Vue3 + Element Plus）
│   └── user/         # 用户前端（Vue3 + Element Plus）
├── rsws_api/         # API 路由层
├── rsws_bin/         # 应用入口
├── rsws_common/      # 通用工具库
├── rsws_db/          # 数据库访问层
├── rsws_model/       # 数据模型
├── rsws_service/     # 业务逻辑层
├── rsws_usdt/        # USDT 支付监听
├── migrations/       # 数据库迁移
└── scripts/          # 部署与测试脚本
```

## 前端主题定制

用户前端和管理后台前端均支持主题切换，采用 **CSS 变量 + localStorage 持久化** 方案。

### 支持的主题

| 主题 | 名称 | 说明 |
| --- | --- | --- |
| 🌙 暗色主题 | `dark` | 默认主题，VS Code 风格深色背景 |
| ☀️ 亮色主题 | `light` | 清新亮色界面 |
| 🔲 高对比暗色 | `high-contrast-dark` | 高对比度暗色主题 |

### 使用方式

- **用户前端**：登录后在用户中心 → 主题设置选择
- **管理后台**：系统设置 → 外观设置选择

### 技术实现

- `theme.css`：定义所有主题的 CSS 自定义属性（`--theme-*` 变量）
- `useTheme.ts`：Vue 组合式函数，管理主题状态、localStorage 读写、`data-theme` 属性设置
- 所有组件使用 `var(--theme-*)` 引用变量，无硬编码颜色

## 快速开始

```bash
# 启动前端开发服务器
cd frontend/user
npm install
npm run dev

cd frontend/admin
npm install
npm run dev

# 启动后端（需 Rust 环境）
cargo run
```

## 架构要点

1. **数据库配置管理模式**：动态数据库配置，支持运行时更新
2. **API Key 加密存储**：AES-GCM-256 加密，密钥从 APP_SECRET HMAC-SHA256 派生
3. **Cregis 签名认证**：所有受保护接口使用签名验证
4. **分层架构**：api → service → db → model，职责清晰
5. **付费墙系统**：后端根据购买状态裁剪内容，前端仅作展示
6. **USDT 支付监听**：TronGrid 轮询监听 USDT 交易
7. **日志系统**：支持级别过滤、tracing 中间件、四节点扩展（login/error/audit）

## License

MIT
