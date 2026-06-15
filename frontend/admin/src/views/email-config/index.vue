<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <span>邮件配置</span>
      </template>

      <el-form :model="form" label-width="120px" style="max-width: 600px">
        <el-form-item label="SMTP 服务商">
          <el-input v-model="form.provider" placeholder="例：QQ邮箱、163邮箱、Gmail" />
        </el-form-item>
        <el-form-item label="SMTP 服务器">
          <el-input v-model="form.host" placeholder="例：smtp.qq.com" />
        </el-form-item>
        <el-form-item label="端口">
          <el-input-number v-model="form.port" :min="1" :max="65535" />
        </el-form-item>
        <el-form-item label="用户名">
          <el-input v-model="form.username" placeholder="发件邮箱地址" />
        </el-form-item>
        <el-form-item label="密码">
          <el-input v-model="form.password" type="password" placeholder="留空表示不修改" />
        </el-form-item>
        <el-form-item label="使用 TLS">
          <el-switch v-model="form.use_tls" />
        </el-form-item>
        <el-form-item label="发件人邮箱">
          <el-input v-model="form.from_email" placeholder="例：noreply@example.com" />
        </el-form-item>
        <el-form-item label="发件人名称">
          <el-input v-model="form.from_name" placeholder="例：RSWS 系统通知" />
        </el-form-item>
        <el-form-item label="回复地址">
          <el-input v-model="form.reply_to" placeholder="留空表示不设置" />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="handleSave" :loading="saving">保存配置</el-button>
          <el-button @click="handleTest" :loading="testing">发送测试邮件</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { getEmailConfig, updateEmailConfig } from '@/api/email'

const saving = ref(false)
const testing = ref(false)

const form = reactive({
  provider: '',
  host: '',
  port: 465,
  username: '',
  password: '',
  use_tls: true,
  from_email: '',
  from_name: '',
  reply_to: '',
})

async function fetchConfig() {
  try {
    const res = await getEmailConfig()
    if (res.code === 0 && res.data) {
      form.provider = res.data.provider || ''
      form.host = res.data.host || ''
      form.port = res.data.port || 465
      form.username = res.data.username || ''
      form.use_tls = res.data.use_tls ?? true
      form.from_email = res.data.from_email || ''
      form.from_name = res.data.from_name || ''
      form.reply_to = res.data.reply_to || ''
    }
  } catch {}
}

async function handleSave() {
  saving.value = true
  try {
    const data = {
      provider: form.provider || undefined,
      host: form.host || undefined,
      port: form.port || undefined,
      username: form.username || undefined,
      use_tls: form.use_tls,
      from_email: form.from_email || undefined,
      from_name: form.from_name || undefined,
      reply_to: form.reply_to || undefined,
    }
    if (form.password) data.password = form.password
    const res = await updateEmailConfig(data)
    if (res.code === 0) {
      ElMessage.success('保存成功')
      form.password = ''
    }
  } catch {
    ElMessage.error('保存失败')
  } finally {
    saving.value = false
  }
}

function handleTest() {
  ElMessage.info('测试邮件功能待后端实现')
}

onMounted(() => fetchConfig())
</script>

<style scoped>
.page-container { padding: 20px; }
</style>