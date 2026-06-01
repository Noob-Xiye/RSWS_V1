# RSWS_V1 项目结构

**实际项目文件统计**（已排除 target, node_modules, .git 等目录）

---

## 目录结构

```
RSWS_V1/
├── docs/                    # 文档目录 (0 个文件)
├── migrations/              # 数据库迁移文件 (4 个文件)
├── rsws_api/               # Rust API 模块 (15 个文件)
├── rsws_bin/               # Rust 二进制模块 (3 个文件)
├── rsws_common/            # Rust 公共模块 (14 个文件)
├── rsws_db/                # Rust 数据库模块 (13 个文件)
├── rsws_model/             # Rust 模型模块 (24 个文件)
├── rsws_service/           # Rust 服务模块 (18 个文件)
├── rsws_usdt/              # Rust USDT 模块 (8 个文件)
├── scripts/                 # 脚本目录 (9 个文件)
├── static/                 # 静态文件 (3 个文件)
├── ui/                      # 前端 UI (99 个文件)
└── target/                  # Rust 编译输出 (已排除)
```

---

## 文件统计

### 按目录统计
| 目录 | 文件数 |
|------|--------|
| ui | 99 |
| rsws_model | 24 |
| rsws_api | 15 |
| rsws_common | 14 |
| rsws_service | 18 |
| rsws_db | 13 |
| migrations | 4 |
| scripts | 9 |
| rsws_usdt | 8 |
| rsws_bin | 3 |
| static | 3 |
| docs | 0 |
| **总计** | **~210** |

### 按文件类型统计
| 扩展名 | 文件数 | 说明 |
|--------|--------|------|
| .rs | 87 | Rust 源代码 |
| .json | 13 | JSON 配置文件 |
| .sql | 13 | SQL 脚本 |
| .py | 5 | Python 脚本 |
| .toml | 9 | TOML 配置文件 |
| .yml | 2 | YAML 配置文件 |
| .md | 2 | Markdown 文档 |
| **总计** | **131** | **文本文件** |

---

## 编码状态

### ✅ 已修复的 UTF-16 文件（4 个）
1. `backup_20260531_2232.sql` → UTF-8 无 BOM ✓
2. `backup_20260531_2235.sql` → UTF-8 无 BOM ✓
3. `build.log` → UTF-8 无 BOM ✓
4. `build_error.log` → UTF-8 无 BOM ✓

### ✅ 编码正确的文件
- **102 个 Rust 源文件** (.rs) - 全部 UTF-8 无 BOM
- **所有配置文件** - 全部 UTF-8 无 BOM
- **所有文档文件** - 全部 UTF-8 无 BOM

---

## 项目类型

**Rust Web 服务项目**
- 使用 Rust 编程语言
- 包含前端 UI（可能使用 Node.js/React 等）
- 使用 PostgreSQL 数据库
- 使用 Docker 容器化部署
- 遵循 Rust 编码规范（UTF-8 强制要求）

---

## 排除的目录

以下目录**不包含在项目文件统计中**：
- `target/` - Rust 编译输出（二进制文件，不检查编码）
- `node_modules/` - Node.js 依赖（第三方库，不检查编码）
- `.git/` - Git 版本控制（版本历史，不检查编码）
- `.github/` - GitHub Actions 工作流（可选检查）
- `dist/` - 分发/构建输出（生成的文件）
- `build/` - 构建临时文件（生成的文件）
- `vendor/` - 依赖供应商（第三方库）

**实际项目源代码和配置文件数量：~210 个**

---

## 编码规范

项目已配置 `.editorconfig`，统一使用：
- **字符编码**: UTF-8 无 BOM
- **换行符**: LF (Unix 风格)
- **缩进**: 4 空格 (Rust), 2 空格 (其他)

---

**更新时间**: 2026-05-31 23:20
