# RSWS_V1 Admin Frontend Integration Spec

> 前端 Admin 与后端 Rust API 的精确对照表（基于源码分析）
> 生成时间：2026-05-09

---

## 1. 认证流程

### 登录
- **前端**：`POST /api/v1/admin/login` `{email, password}`
- **后端响应**：
  ```json
  {
    "success": true,
    "data": {
      "token": "...",
      "api_key": "adm_ak_...",
      "admin": { "id": 1, "email": "...", "username": "...", "role": "...", ... }
    }
  }
  ```
- **注意**：后端 `login` handler 返回的是 `AdminInfo`（不含 token/api_key），但 middleware 在 Redis 中设置了 session。**实际登录返回结构需确认**。

### AuthStore 问题
- 当前 `login` 期望 `res.data.token` 和 `res.data.api_key`
- 但后端登录返回的可能是 `AdminInfo` 本身
- 需对 `adminLogin` 后端返回进行确认

### 受保护接口
- 所有 `/api/v1/admin/*` 需带 `X-API-Key` header（admin_api_key）
- 非管理员访问 → 403 Forbidden

---

## 2. 管理员管理

| 前端操作 | 后端端点 | 方法 | 请求体 | 响应 |
|---------|---------|------|--------|------|
| 登录 | `/admin/login` | POST | `{email, password}` | `AdminInfo` |
| 获取当前管理员 | `/admin` | GET | — | `AdminInfo` |
| 获取管理员列表 | `/admin/list` | GET | `?page=&page_size=&role=` | `{items: AdminInfo[], total, page, page_size, total_pages}` |
| 创建管理员 | `/admin/create` | POST | `{email, username, password, role}` | `AdminInfo` |
| 获取单个管理员 | `/admin/<id>` | GET | — | `AdminInfo` |
| 停用管理员 | `/admin/<id>/deactivate` | POST | — | `{id, message}` |

**AdminInfo 结构**（后端）：
```typescript
interface AdminInfo {
  id: number
  email: string
  username: string
  role: string
  avatar_url?: string
  is_active: boolean
  permissions: string[]
  created_at: string
  last_login_at: string | null
}
```

**关键修复**：前端 `listAdmins` 应正确解析 `{items: AdminInfo[], total, page, ...}` 结构。

---

## 3. 管理员 API Key 管理

| 前端操作 | 后端端点 | 方法 | 请求体 | 响应 |
|---------|---------|------|--------|------|
| 获取 API Key 列表 | `/admin/api-keys` | GET | — | `AdminApiKey[]`（数组，非分页） |
| 创建 API Key | `/admin/api-keys` | POST | `{name, permissions?, rate_limit?, expires_in_days?}` | `AdminApiKeyResponse` |
| 删除 API Key | `/admin/<id>/api-keys/<key_id>` | DELETE | — | `{deleted: bool}` |

**AdminApiKey 结构**（后端）：
```typescript
interface AdminApiKey {
  id: number
  admin_id: number
  name: string
  api_key: string
  api_secret_encrypted: string  // 不暴露给前端
  permissions: string[]
  rate_limit: number | null
  last_used_at: string | null
  expires_at: string | null
  is_active: boolean
  created_at: string
  updated_at: string
}
```

**AdminApiKeyResponse**（创建后返回）：
```typescript
interface AdminApiKeyResponse {
  id: number
  name: string
  api_key: string
  api_secret: string | null  // 仅创建时返回一次明文
  permissions: string[]
  rate_limit: number | null
  last_used_at: string | null
  expires_at: string | null
  is_active: boolean
  created_at: string
}
```

---

## 4. 支付配置（USDT/PayPal）

### 现状
后端**没有**独立的 config CRUD 端点。支付配置存储在：
- `blockchain_configs` 表（USDT 网络配置，含 usdt_contract、api_url 等）
- `paypal_configs` 表（PayPal client_id、secret、mode 等）
- `usdt_wallets` 表（USDT 收款地址）

### USDT 地址获取
- `GET /api/v1/payment/usdt/<network>`（用户级，需登录）
- 响应：`{network, address, contract}`

### 需要新增后端端点
```
GET  /api/v1/admin/blockchain-configs       → 获取区块链配置列表
PUT  /api/v1/admin/blockchain-configs/<id> → 更新区块链配置
GET  /api/v1/admin/paypal-config            → 获取 PayPal 配置
PUT  /api/v1/admin/paypal-config            → 更新 PayPal 配置
```

**临时方案**：前端支付配置页面 USDT/PayPal 表单暂时只能展示已存储的值（通过数据库直接读或新增 admin endpoints）。

---

## 5. 资源/订单/日志（已有 API）

### 资源（通过 `/api/v1/resource`）
- `GET /api/v1/resource?page=&page_size=&category_id=&search=` → `{items: Resource[], total, ...}`
- `POST /api/v1/resource` → 创建资源（需认证）
- `GET /api/v1/resource/<id>` → 详情

### 订单（通过 `/api/v1/order`）
- `GET /api/v1/order?page=&limit=` → `{items: Order[], total, ...}`
- `POST /api/v1/order` → 创建订单
- `GET /api/v1/order/<id>` → 详情
- `POST /api/v1/order/<id>/cancel` → 取消订单

### 日志（通过 `/api/v1/admin/logs/system`）
- `GET /api/v1/admin/logs/system?level=&page=&page_size=` → `{items: SystemLog[], total, ...}`

### 日志配置（通过 `/api/v1/admin/log-configs`）
- `GET /api/v1/admin/log-configs` → `LogConfig[]`
- `POST /api/v1/admin/log-configs` → 创建
- `GET /api/v1/admin/log-configs/<key>` → 详情
- `PUT /api/v1/admin/log-configs/<key>` → 更新
- `DELETE /api/v1/admin/log-configs/<key>` → 删除

---

## 6. 路由前缀

- 所有业务 API：`/api/v1/...`
- Admin 管理接口：`/api/v1/admin/...`
- Swagger 文档：`/api-doc/openapi.json`

---

## 7. 已知问题 & 待修复

1. **Admin list 分页解析**：`listAdmins` 前端代码需正确从 `{items, total, ...}` 中提取
2. **登录响应结构**：需确认后端 `login` 实际返回 `AdminInfo` 还是有 `token`/`api_key` 字段
3. **支付配置无 CRUD API**：需后端补充或通过数据库直接管理
4. **API Key 列表无分页**：后端返回 `Vec<AdminApiKey>` 数组，前端按数组处理即可

---

## 8. 前端环境配置

```env
# .env
VITE_API_URL=http://localhost:8080/api/v1
```

后端当前端口 8080（Docker 配置），前端 dev 默认 5173。