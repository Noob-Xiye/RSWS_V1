<template>
  <div class="page-container">
    <el-row :gutter="20">
      <!-- 左侧：个人信息 -->
      <el-col :span="8">
        <el-card>
          <template #header><span>个人信息</span></template>
          <div class="profile-info">
            <div class="avatar-section">
              <el-avatar :size="72" :src="adminInfo.avatar_url || undefined">
                {{ adminInfo.nickname?.charAt(0) || adminInfo.username?.charAt(0) || 'A' }}
              </el-avatar>
              <div class="profile-name">
                <div class="nickname">{{ adminInfo.nickname || adminInfo.username }}</div>
                <div class="role-badge">
                  <el-tag size="small" :type="adminInfo.role === 'super_admin' ? 'danger' : 'primary'">
                    {{ adminInfo.role === 'super_admin' ? '超级管理员' : '管理员' }}
                  </el-tag>
                </div>
              </div>
            </div>
            <el-divider />
            <div class="info-list">
              <div class="info-row">
                <span class="label">用户名</span>
                <span class="value">{{ adminInfo.username || '-' }}</span>
              </div>
              <div class="info-row">
                <span class="label">邮箱</span>
                <span class="value">{{ adminInfo.email || '-' }}</span>
              </div>
              <div class="info-row">
                <span class="label">账号状态</span>
                <el-tag size="small" :type="adminInfo.is_active ? 'success' : 'danger'">
                  {{ adminInfo.is_active ? '正常' : '已禁用' }}
                </el-tag>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 右侧：安全设置 -->
      <el-col :span="16">
        <el-card class="mb-16">
          <template #header><span>安全设置</span></template>
          <el-form ref="pwdFormRef" :model="pwdForm" :rules="pwdRules" label-width="120px">
            <el-form-item label="当前密码" prop="old_password">
              <el-input v-model="pwdForm.old_password" type="password" placeholder="请输入当前密码" show-password />
            </el-form-item>
            <el-form-item label="新密码" prop="new_password">
              <el-input v-model="pwdForm.new_password" type="password" placeholder="6-32位字母/数字" show-password />
            </el-form-item>
            <el-form-item label="确认新密码" prop="confirm_password">
              <el-input v-model="pwdForm.confirm_password" type="password" placeholder="再次输入新密码" show-password />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" :loading="pwdLoading" @click="handleChangePassword">
                修改密码
              </el-button>
            </el-form-item>
          </el-form>
        </el-card>

        <el-card>
          <template #header><span>系统信息</span></template>
          <el-descriptions :column="1" border size="small">
            <el-descriptions-item label="系统版本">RSWS V1.0.0</el-descriptions-item>
            <el-descriptions-item label="前端版本">Vue 3 + Vite</el-descriptions-item>
            <el-descriptions-item label="运行环境">{{ runtimeEnv }}</el-descriptions-item>
            <el-descriptions-item label="登录时间">{{ formatDate(currentLoginTime) }}</el-descriptions-item>
          </el-descriptions>
        </el-card>

        <el-card>
          <template #header><span>存储配置</span></template>
          <el-form :model="storageForm" label-width="120px">
            <el-form-item label="存储方式">
              <el-radio-group v-model="storageForm.provider">
                <el-radio value="local">本地存储</el-radio>
                <el-radio value="s3">AWS S3</el-radio>
                <el-radio value="minio">MinIO</el-radio>
                <el-radio value="aliyun-oss">阿里云 OSS</el-radio>
                <el-radio value="tencent-cos">腾讯云 COS</el-radio>
              </el-radio-group>
            </el-form-item>
            <template v-if="storageForm.provider !== 'local'">
              <el-form-item label="Endpoint" required>
                <el-input v-model="storageForm.endpoint" placeholder="https://oss-cn-hangzhou.aliyuncs.com" />
              </el-form-item>
              <el-form-item label="Bucket" required>
                <el-input v-model="storageForm.bucket" placeholder="my-bucket" />
              </el-form-item>
              <el-form-item label="Access Key" required>
                <el-input v-model="storageForm.access_key" placeholder="Access Key ID" />
              </el-form-item>
              <el-form-item label="Secret Key" required>
                <el-input v-model="storageForm.secret_key" type="password" placeholder="Secret Access Key" show-password />
              </el-form-item>
              <el-form-item label="Region">
                <el-input v-model="storageForm.region" placeholder="oss-cn-hangzhou" />
              </el-form-item>
              <el-form-item label="存储前缀">
                <el-input v-model="storageForm.prefix" placeholder="resources/" />
              </el-form-item>
              <el-form-item label="CDN 域名">
                <el-input v-model="storageForm.custom_domain" placeholder="https://cdn.example.com (可选)" />
              </el-form-item>
            </template>
            <el-form-item>
              <el-button type="primary" :loading="storageSaving" @click="handleSaveStorage">
                保存配置
              </el-button>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { getAdminInfo } from '@/api/admin'
import { resetAdminPassword } from '@/api/admin'
import { getStorageConfig, updateStorageConfig, type StorageConfig } from '@/api/admin'

const adminInfo = ref<Record<string, any>>({})
const pwdLoading = ref(false)
const pwdFormRef = ref<FormInstance>()
const currentLoginTime = ref(new Date().toISOString())
const runtimeEnv = import.meta.env.MODE || 'development'

// 存储配置
const storageForm = reactive<StorageConfig>({
  provider: 'local',
  enabled: false,
  endpoint: '',
  bucket: '',
  access_key: '',
  secret_key: '',
  region: '',
  prefix: 'resources/',
  custom_domain: null,
})
const storageSaving = ref(false)

const pwdForm = reactive({
  old_password: '',
  new_password: '',
  confirm_password: '',
})

const validateConfirm = (rule: any, value: string, callback: any) => {
  if (value !== pwdForm.new_password) {
    callback(new Error('两次输入的密码不一致'))
  } else {
    callback()
  }
}

const pwdRules: FormRules = {
  old_password: [
    { required: true, message: '请输入当前密码', trigger: 'blur' },
  ],
  new_password: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { min: 6, max: 32, message: '密码长度为 6-32 位', trigger: 'blur' },
  ],
  confirm_password: [
    { required: true, message: '请确认新密码', trigger: 'blur' },
    { validator: validateConfirm, trigger: 'blur' },
  ],
}

async function fetchAdminInfo() {
  try {
    const res = await getAdminInfo()
    if (res.code === 0 && res.data) {
      const info = res.data.admins?.[0] || res.data.admin || res.data
      adminInfo.value = info
    }
  } catch {
    ElMessage.error('获取管理员信息失败')
  }
}

async function handleChangePassword() {
  if (!pwdFormRef.value) return
  const valid = await pwdFormRef.value.validate().catch(() => false)
  if (!valid) return

  pwdLoading.value = true
  try {
    const res = await resetAdminPassword(adminInfo.value.id, pwdForm.new_password)
    if (res.code === 0) {
      ElMessage.success('密码修改成功')
      pwdForm.old_password = ''
      pwdForm.new_password = ''
      pwdForm.confirm_password = ''
    } else {
      ElMessage.error(res.msg || '修改失败')
    }
  } catch {
    ElMessage.error('修改失败，请检查当前密码是否正确')
  } finally {
    pwdLoading.value = false
  }
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

onMounted(() => {
  fetchAdminInfo()
  fetchStorageConfig()
})

async function fetchStorageConfig() {
  try {
    const res = await getStorageConfig()
    if (res.code === 0 && res.data) {
      Object.assign(storageForm, res.data)
    }
  } catch {
    // 没有配置时使用默认值
  }
}

async function handleSaveStorage() {
  storageSaving.value = true
  try {
    storageForm.enabled = storageForm.provider !== 'local'
    const res = await updateStorageConfig(storageForm)
    if (res.code === 0) {
      ElMessage.success('存储配置已保存')
    } else {
      ElMessage.error(res.msg || '保存失败')
    }
  } catch {
    ElMessage.error('保存失败')
  } finally {
    storageSaving.value = false
  }
}
</script>

<style scoped>
.page-container {
  padding: 20px;
}

.mb-16 {
  margin-bottom: 16px;
}

.profile-info {
  padding: 8px 0;
}

.avatar-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.profile-name {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.nickname {
  font-size: 18px;
  font-weight: 600;
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.info-row .label {
  color: #999;
  font-size: 13px;
}

.info-row .value {
  font-size: 13px;
  color: #333;
}
</style>
