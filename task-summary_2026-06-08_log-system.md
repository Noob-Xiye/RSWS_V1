# RSWS 日志系统增强 (2026-06-08 22:09-22:??)

## 用户需求
1. log_configs 表增加 `level` 字段（可配置级别过滤）
2. 集成 tracing 日志中间件（每个请求带 trace_id）

## 完成改动

### 1. SQL 迁移文件
- **新文件**: `migrations/20260608000001_add_log_level.sql`
- `log_configs` 表新增 `config_type` (string/bool/number) 和 `level` (trace/debug/info/warn/error) 字段
- 插入 9 条默认配置（全局级别 + 5 个模块级别 + 3 个功能开关）
- `system_logs` 表新增索引

### 2. log_service.rs 重构
- 新增 `LogConfig` 结构体（含 `config_type`、`level` 字段）
- 新增 `UpdateLogConfigRequest` DTO
- 新增 `LogQueryParams` 查询参数结构体
- **模块级别过滤**: `is_log_level_enabled` 和 `is_module_log_enabled` 支持按 log_configs 的 level 动态过滤
- 四种记录方法 (`log_system`, `log_error`, `log_payment`, `log_request`) 在插入前先检查对应模块的级别配置
- 新增 `query_system_logs` 分页查询（支持按 level、module、user_id、时间范围筛选）

### 3. log handler 扩展
- 新增 6 个 API 端点：
  - `GET /admin/log-configs` - 列出所有日志配置
  - `GET /admin/log-configs/{key}` - 获取指定配置
  - `POST /admin/log-configs/{key}` - 创建/更新配置
  - `PUT /admin/log-configs/{key}` - 更新配置
  - `DELETE /admin/log-configs/{key}` - 删除配置
  - `GET /admin/logs/system` - 查询系统日志（带分页和筛选）
- 注意：路由已注册为 `logs/system`（而非 `log-configs` 下）

### 4. tracing 中间件
- **新文件**: `rsws_api/src/middleware/tracing.rs`
- 自定义 `tracing_logger` 中间件，与已有的 `request_id_middleware` 配合
- 记录每个 HTTP 请求的：request_id, method, path, status, duration_ms, client_ip, user_agent
- 按状态码分级别：5xx=ERROR, 4xx=WARN, 其他=INFO
- 慢请求告警（>1s）

### 5. router.rs 集成
- 注册顺序：`request_id_middleware` → `tracing_logger` → CORS → AffixState → 路由
- 日志配置路由已整合到 `/api/v1/admin/log-configs` 和 `/api/v1/admin/logs/system`

## 编译状态
- `cargo check -p rsws_service` 零错误零警告 ✅
- `cargo check -p rsws_api` 零错误零警告 ✅

## 下一步操作
已执行部署的 SQL 迁移脚本（需要 Docker 内执行）：
```bash
# 在 PostgreSQL 容器内执行迁移
docker exec -i rsws-postgres psql -U rsws -d rsws < migrations/20260608000001_add_log_level.sql
```
或者直接复制到 Docker 内，然后重启容器。