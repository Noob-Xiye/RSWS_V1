<template>
  <div class="oss-config-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>OSS 对象存储配置</span>
          <el-button type="primary" @click="handleTest" :loading="testing">
            测试连接
          </el-button>
        </div>
      </template>

      <el-form
        ref="formRef"
        :model="config"
        :rules="rules"
        label-width="140px"
        label-position="right"
      >
        <!-- 存储提供商 -->
        <el-form-item label="存储提供商" prop="provider">
          <el-select v-model="config.provider" placeholder="请选择存储提供商">
            <el-option label="本地存储" value="local" />
            <el-option label="AWS S3" value="s3" />
            <el-option label="MinIO" value="minio" />
            <el-option label="阿里云 OSS" value="aliyun-oss" />
            <el-option label="腾讯云 COS" value="tencent-cos" />
          </el-select>
        </el-form-item>

        <!-- 启用状态 -->
        <el-form-item label="启用状态" prop="is_active">
          <el-switch v-model="config.is_active" />
        </el-form-item>

        <!-- 本地存储路径（仅 local） -->
        <el-form-item
          v-if="config.provider === 'local'"
          label="本地存储路径"
          prop="endpoint"
        >
          <el-input
            v-model="config.endpoint"
            placeholder="例如：/data/rsws/uploads"
          />
          <div class="form-tip">本地文件系统路径，用于存放上传的文件</div>
        </el-form-item>

        <!-- Endpoint -->
        <el-form-item
          v-if="config.provider !== 'local'"
          label="Endpoint"
          prop="endpoint"
        >
          <el-input
            v-model="config.endpoint"
            :placeholder="endpointPlaceholder"
          />
          <div class="form-tip">{{ endpointTip }}</div>
        </el-form-item>

        <!-- Bucket -->
        <el-form-item
          v-if="config.provider !== 'local'"
          label="Bucket 名称"
          prop="bucket"
        >
          <el-input v-model="config.bucket" placeholder="例如：my-bucket" />
        </el-form-item>

        <!-- Access Key -->
        <el-form-item
          v-if="config.provider !== 'local'"
          label="Access Key ID"
          prop="access_key"
        >
          <el-input
            v-model="config.access_key"
            placeholder="请输入 Access Key ID"
            show-password
          />
        </el-form-item>

        <!-- Secret Key -->
        <el-form-item
          v-if="config.provider !== 'local'"
          label="Secret Access Key"
          prop="secret_key"
        >
          <el-input
            v-model="config.secret_key"
            placeholder="请输入 Secret Access Key"
            show-password
          />
        </el-form-item>

        <!-- Region -->
        <el-form-item
          v-if="config.provider !== 'local'"
          label="Region"
          prop="region"
        >
          <el-input
            v-model="config.region"
            :placeholder="regionPlaceholder"
          />
          <div class="form-tip">{{ regionTip }}</div>
        </el-form-item>

        <!-- 存储前缀 -->
        <el-form-item label="存储前缀" prop="prefix">
          <el-input v-model="config.prefix" placeholder="例如：resources/" />
          <div class="form-tip">对象存储中的路径前缀，默认 resources/</div>
        </el-form-item>

        <!-- 自定义域名 -->
        <el-form-item label="自定义域名" prop="custom_domain">
          <el-input
            v-model="config.custom_domain"
            placeholder="例如：https://cdn.example.com"
          />
          <div class="form-tip">可选，用于 CDN 加速访问</div>
        </el-form-item>

        <!-- 操作按钮 -->
        <el-form-item>
          <el-button type="primary" @click="handleSave" :loading="saving">
            保存配置
          </el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import {
  getOssConfig,
  saveOssConfig,
  testOssConnection
} from '@/api/oss-config'
import type { OssStorageConfig } from '@/api/oss-config'

// 表单引用
const formRef = ref<FormInstance>()

// 加载状态
const saving = ref(false)
const testing = ref(false)

// 配置数据
const config = reactive<OssStorageConfig>({
  provider: 'local',
  endpoint: '',
  bucket: '',
  access_key: '',
  secret_key: '',
  region: '',
  prefix: 'resources/',
  custom_domain: '',
  is_active: false
})

// 表单验证规则
const rules = reactive<FormRules>({
  provider: [{ required: true, message: '请选择存储提供商', trigger: 'change' }],
  endpoint: [{ required: true, message: '请输入 Endpoint', trigger: 'blur' }],
  bucket: [{ required: true, message: '请输入 Bucket 名称', trigger: 'blur' }],
  access_key: [{ required: true, message: '请输入 Access Key ID', trigger: 'blur' }],
  secret_key: [{ required: true, message: '请输入 Secret Access Key', trigger: 'blur' }],
  region: [{ required: true, message: '请输入 Region', trigger: 'blur' }]
})

// 计算属性：动态 placeholder 和提示
const endpointPlaceholder = computed(() => {
  switch (config.provider) {
    case 's3':
      return '留空使用 AWS S3 默认 Endpoint'
    case 'minio':
      return '例如：http://localhost:9000'
    case 'aliyun-oss':
      return '例如：https://oss-cn-hangzhou.aliyuncs.com'
    case 'tencent-cos':
      return '例如：https://cos.ap-guangzhou.myqcloud.com'
    default:
      return ''
  }
})

const endpointTip = computed(() => {
  switch (config.provider) {
    case 's3':
      return 'AWS S3 留空使用默认 Endpoint，或填写自定义 Endpoint'
    case 'minio':
      return 'MinIO 服务地址，例如 http://localhost:9000'
    case 'aliyun-oss':
      return '阿里云 OSS Endpoint，格式：https://oss-cn-region.aliyuncs.com'
    case 'tencent-cos':
      return '腾讯云 COS Endpoint，格式：https://cos.ap-region.myqcloud.com'
    default:
      return ''
  }
})

const regionPlaceholder = computed(() => {
  switch (config.provider) {
    case 's3':
      return '例如：us-east-1'
    case 'minio':
      return '例如：us-east-1（MinIO 可随意填写）'
    case 'aliyun-oss':
      return '例如：cn-hangzhou'
    case 'tencent-cos':
      return '例如：ap-guangzhou'
    default:
      return ''
  }
})

const regionTip = computed(() => {
  switch (config.provider) {
    case 's3':
      return 'AWS S3 Region，例如 us-east-1, ap-southeast-1'
    case 'minio':
      return 'MinIO 对 Region 要求不严格，可随意填写'
    case 'aliyun-oss':
      return '阿里云 OSS Region，例如 cn-hangzhou, cn-beijing'
    case 'tencent-cos':
      return '腾讯云 COS Region，例如 ap-guangzhou, ap-shanghai'
    default:
      return ''
  }
})

// 加载配置
async function loadConfig() {
  try {
    const res = await getOssConfig()
    if (res.data) {
      Object.assign(config, res.data)
    }
  } catch (error: any) {
    ElMessage.error(error.message || '加载配置失败')
  }
}

// 保存配置
async function handleSave() {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  saving.value = true
  try {
    await saveOssConfig(config)
    ElMessage.success('配置保存成功')
  } catch (error: any) {
    ElMessage.error(error.message || '保存失败')
  } finally {
    saving.value = false
  }
}

// 测试连接
async function handleTest() {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  testing.value = true
  try {
    const res = await testOssConnection(config)
    if (res.data.success) {
      ElMessage.success(res.data.message || '连接测试成功')
    } else {
      ElMessage.error(res.data.message || '连接测试失败')
    }
  } catch (error: any) {
    ElMessage.error(error.message || '连接测试失败')
  } finally {
    testing.value = false
  }
}

// 重置表单
function handleReset() {
  loadConfig()
}

// 初始化
onMounted(() => {
  loadConfig()
})
</script>

<style scoped>
.oss-config-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}
</style>
