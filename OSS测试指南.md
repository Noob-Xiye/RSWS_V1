# RSWS_V1 OSS 支持功能 - 测试指南

## 📋 功能概述

已实现完整的 OSS（对象存储）支持，包括：
- ✅ 统一的存储抽象层（支持本地、AWS S3、MinIO、阿里云 OSS、腾讯云 COS）
- ✅ 前端 OSS 配置页面（表单验证、测试连接）
- ✅ 后端 OSS 配置 API（读取/保存/测试）
- ✅ 文件分块上传 API（支持大文件）
- ✅ 资源删除时自动删除 OSS 文件
- ✅ 前端资源上传组件已对接 OSS 接口

---

## 🔧 第一步：编译检查

### 1.1 执行编译检查

在 `F:\GitRepo\RSWS_V1` 目录执行：

```powershell
# 进入项目目录
Set-Location -Path "F:\GitRepo\RSWS_V1"

# 执行编译检查（只检查库代码，不运行二进制）
cargo check --lib 2>&1 | Select-Object -First 100
```

### 1.2 常见编译错误及修复

#### 错误 1: `async_trait` 未导入

**错误信息**：
```
error[E0432]: unresolved import `async_trait`
 --> rsws_service\src\oss_service.rs:1:5
  |
1 | use async_trait::async_trait;
  |     ^^^^^^^^^^^^
  |     |
  |     unresolved import
```

**修复**：
确保 `rsws_service/Cargo.toml` 包含：
```toml
async-trait = { workspace = true }
```

#### 错误 2: `StorageBackend` trait 方法签名不匹配

**错误信息**：
```
error[E0049]: method `upload` has 4 parameters but the declaration in trait `StorageBackend` has 3
```

**原因**：trait 定义和实现不匹配

**修复**：检查 `oss_service.rs` 中的 trait 定义和各结构体实现是否一致

#### 错误 3: `OssStorageConfig` 字段访问错误

**错误信息**：
```
error[E0609]: no field `is_local` on type `OssStorageConfig`
```

**修复**：我已经修复了这个问题，使用 `match config.provider.as_str()` 代替

#### 错误 4: `ResourceService` 缺少 `config_service` 字段

**错误信息**：
```
error[E0063]: missing field `config_service` in initializer of `ResourceService`
```

**修复**：我已经添加了 `config_service: Option<crate::config_service::ConfigService>` 字段

#### 错误 5: 前端 API 路径错误

**错误信息**：前端控制台显示 404 错误

**修复**：确认 `rsws_api/src/router.rs` 中路由配置正确

---

## 🚀 第二步：本地测试（使用 MinIO）

### 2.1 启动 MinIO

```powershell
# 拉取 MinIO 镜像
docker pull minio/minio

# 启动 MinIO（控制台端口 9001，API 端口 9000）
docker run -d `
  -p 9000:9000 `
  -p 9001:9001 `
  --name minio `
  -e "MINIO_ROOT_USER=minioadmin" `
  -e "MINIO_ROOT_PASSWORD=minioadmin" `
  -v "F:\minio-data:/data" `
  minio/minio server /data --console-address ":9001"
```

### 2.2 创建 Bucket

1. 访问 MinIO 控制台：http://localhost:9001
2. 登录（用户名/密码：`minioadmin` / `minioadmin`）
3. 点击 "Create Bucket"
4. 输入 Bucket 名称：`rsws`
5. 点击 "Create Bucket"

### 2.3 配置 RSWS 使用 MinIO

1. 启动 RSWS 后端服务
2. 访问管理员端：http://localhost:5170/admin
3. 进入 "系统设置" → "OSS 存储配置"
4. 填写配置：
   - **存储提供商**：`MinIO`
   - **是否启用**：`是`
   - **本地存储路径 / Endpoint**：`http://localhost:9000`
   - **Bucket 名称**：`rsws`
   - **Access Key ID**：`minioadmin`
   - **Secret Access Key**：`minioadmin`
   - **Region**：`us-east-1`
   - **存储前缀**：`resources`
   - **自定义域名**：留空
5. 点击 "测试连接"
6. 如果测试通过，点击 "保存配置"

---

## 🧪 第三步：功能测试

### 3.1 测试文件上传

1. 进入 "资源管理" → "创建资源"
2. 填写资源信息（标题、价格、分类等）
3. 在 "资源文件" 区域：
   - 拖拽文件到上传区域，或点击选择文件
   - 观察上传进度条
   - 上传完成后，文件 URL 会自动填充到输入框
4. 点击 "确定" 创建资源

### 3.2 验证文件上传成功

**本地存储模式**：
- 检查 `uploads/resources/` 目录是否有文件
- 文件 URL 类似：`http://localhost:5170/uploads/resources/20240601/12345678.zip`

**MinIO 模式**：
- 访问 MinIO 控制台：http://localhost:9001
- 进入 `rsws` Bucket
- 检查 `resources/` 前缀下是否有文件

### 3.3 测试资源删除（同步删除 OSS 文件）

1. 在资源列表中点击 "删除" 按钮
2. 确认删除
3. 验证：
   - 数据库记录已删除
   - OSS 中的文件也已删除（检查本地目录或 MinIO 控制台）

---

## ☁️ 第四步：测试云存储

### 4.1 阿里云 OSS 配置

1. 登录阿里云 OSS 控制台：https://oss.console.aliyun.com
2. 创建 Bucket：
   - Bucket 名称：`rsws-yourname`
   - 地域：`oss-cn-hangzhou`
   - 存储类型：`标准存储`
   - 读写权限：`私有`（推荐）或 `公共读`
3. 获取 AccessKey：
   - 访问 https://ram.console.aliyun.com/manage/ak
   - 创建 AccessKey 和 AccessKey Secret
4. 在 RSWS 管理员端配置：
   - **存储提供商**：`阿里云 OSS`
   - **Endpoint**：`https://oss-cn-hangzhou.aliyuncs.com`
   - **Bucket 名称**：`rsws-yourname`
   - **Access Key ID**：`你的 AccessKey`
   - **Secret Access Key**：`你的 AccessKey Secret`
   - **Region**：`cn-hangzhou`
   - **存储前缀**：`resources`

### 4.2 腾讯云 COS 配置

1. 登录腾讯云 COS 控制台：https://console.cloud.tencent.com/cos
2. 创建 Bucket：
   - Bucket 名称：`rsws-yourname`
   - 所属地域：`ap-guangzhou`
   - 访问权限：`私有读写`（推荐）或 `公有读私有写`
3. 获取 SecretId 和 SecretKey：
   - 访问 https://console.cloud.tencent.com/cam/capi
   - 创建 API 密钥
4. 在 RSWS 管理员端配置：
   - **存储提供商**：`腾讯云 COS`
   - **Endpoint**：`https://cos.ap-guangzhou.myqcloud.com`
   - **Bucket 名称**：`rsws-yourname`
   - **Access Key ID**：`你的 SecretId`
   - **Secret Access Key**：`你的 SecretKey`
   - **Region**：`ap-guangzhou`
   - **存储前缀**：`resources`

### 4.3 AWS S3 配置

1. 登录 AWS S3 控制台：https://s3.console.aws.amazon.com
2. 创建 Bucket：
   - Bucket 名称：`rsws-yourname`
   - 地域：`us-east-1`
   - 访问权限：`私有`（推荐）
3. 获取 AccessKey 和 SecretAccessKey：
   - 访问 https://console.aws.amazon.com/iam/home#/security_credentials
   - 创建访问密钥
4. 在 RSWS 管理员端配置：
   - **存储提供商**：`AWS S3`
   - **Endpoint**：`https://s3.us-east-1.amazonaws.com`
   - **Bucket 名称**：`rsws-yourname`
   - **Access Key ID**：`你的 AccessKey`
   - **Secret Access Key**：`你的 SecretAccessKey`
   - **Region**：`us-east-1`
   - **存储前缀**：`resources`

---

## 🐛 常见问题排查

### 问题 1: 编译失败

**排查步骤**：
1. 检查 `cargo check` 的完整错误输出
2. 确认所有依赖已添加到 `Cargo.toml`
3. 确认 `rsws_service/src/lib.rs` 导出了 `oss_service` 模块
4. 确认 `rsws_api/src/handler/mod.rs` 导出了 `admin_oss` 和 `upload` 模块

### 问题 2: 上传文件失败

**排查步骤**：
1. 检查后端日志（查看错误信息）
2. 检查 OSS 配置是否正确（在管理员端测试连接）
3. 检查 Bucket 权限（是否允许上传）
4. 检查网络连通性（能否访问 OSS endpoint）

### 问题 3: 前端显示 404 错误

**排查步骤**：
1. 检查路由配置是否正确（`router.rs`）
2. 检查前端 API 路径是否正确（`resource.ts`）
3. 检查后端服务是否启动
4. 使用浏览器开发者工具查看网络请求

### 问题 4: 文件 URL 无法访问

**本地存储模式**：
- 检查 `server.upload_dir` 配置是否正确
- 检查静态文件服务是否启动（`salvo-serve-static`）

**云存储模式**：
- 检查 Bucket 权限（私有 vs 公有读）
- 检查自定义域名配置（如果使用 CDN）
- 使用预签名 URL 访问私有文件

---

## 📊 测试检查清单

### 后端编译
- [ ] `cargo check --lib` 通过（无错误）
- [ ] `cargo build` 成功（生成二进制）

### 配置功能
- [ ] 访问 OSS 配置页面（管理员端）
- [ ] 填写配置并保存
- [ ] 测试连接功能正常
- [ ] 配置正确保存到数据库（`system_configs` 表）

### 上传功能
- [ ] 单文件上传成功（小文件）
- [ ] 分块上传成功（大文件）
- [ ] 上传进度条显示正常
- [ ] 文件 URL 自动填充

### 删除功能
- [ ] 删除资源时同步删除 OSS 文件
- [ ] 删除失败时不阻断资源删除操作

### 云存储测试
- [ ] MinIO 上传/下载/删除成功
- [ ] 阿里云 OSS 上传/下载/删除成功（可选）
- [ ] 腾讯云 COS 上传/下载/删除成功（可选）
- [ ] AWS S3 上传/下载/删除成功（可选）

---

## 🎯 性能优化建议

### 1. 使用 CDN 加速

在 OSS 配置中填写 `custom_domain`（自定义域名），例如：
- 阿里云 OSS：配置自定义域名 + CDN 加速
- 腾讯云 COS：配置自定义源站 + CDN 加速
- AWS S3：配置 CloudFront 分发

### 2. 大文件分块上传

当前实现已支持分块上传（默认分块大小 5MB），可根据网络情况调整：
- 修改 `upload.rs` 中的 `CHUNK_SIZE`
- 前端根据文件大小自动选择分块上传或单文件上传

### 3. 异步删除

当前删除资源时是同步删除 OSS 文件，可优化为：
- 将删除任务推送到消息队列（Redis/RabbitMQ）
- 后台 worker 异步删除 OSS 文件
- 提高用户体验（删除操作立即返回）

---

## 📝 后续工作

### 1. 添加文件访问接口

创建 `/api/v1/resource/{id}/file` 端点：
- 私有文件：生成预签名 URL（有时效性）
- 公有文件：直接返回 CDN/ OSS URL

### 2. 添加存储使用情况统计

在管理员端显示：
- 已用存储空间
- 文件数量
- 存储成本估算

### 3. 添加图片缩略图生成

上传图片时自动生成缩略图：
- 使用 `image` crate 处理图片
- 生成多种尺寸的缩略图（可选）
- 保存到 OSS（不同 key 前缀）

### 4. 添加文件预览功能

支持在线预览：
- 图片：直接显示
- 文档：转换为 PDF 预览
- 视频：HLS 流媒体播放

---

## 🎉 总结

完成上述测试后，RSWS_V1 的 OSS 支持功能就完整实现了！

**核心功能**：
- ✅ 支持本地、AWS S3、MinIO、阿里云 OSS、腾讯云 COS
- ✅ 统一存储抽象层（易于扩展其他存储后端）
- ✅ 前端配置页面（表单验证、测试连接）
- ✅ 后端配置 API（读取/保存/测试）
- ✅ 文件分块上传（支持大文件）
- ✅ 资源删除时同步删除 OSS 文件

**下一步**：
1. 执行编译检查（告诉我任何错误）
2. 本地测试（使用 MinIO）
3. 可选：测试云存储（阿里云/腾讯云/AWS）
4. 可选：添加文件访问接口、存储统计等功能

---

**测试过程中遇到任何问题，请随时告诉我！** 🚀
