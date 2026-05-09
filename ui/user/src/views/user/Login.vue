<template>
  <div class="login-page">
    <el-card class="login-card">
      <template #header>
        <div class="card-header">登录</div>
      </template>

      <div class="login-tabs">
        <el-radio-group v-model="loginType" size="small">
          <el-radio-button value="password">密码登录</el-radio-button>
          <el-radio-button value="code">验证码登录</el-radio-button>
        </el-radio-group>
      </div>

      <el-form ref="formRef" :model="form" :rules="rules" @submit.prevent="handleLogin">
        <el-form-item prop="username">
          <el-input
            v-model="form.username"
            :placeholder="loginType === 'password' ? '用户名' : '邮箱'"
            prefix-icon="User"
            size="large"
          />
        </el-form-item>

        <template v-if="loginType === 'password'">
          <el-form-item prop="password">
            <el-input
              v-model="form.password"
              type="password"
              placeholder="密码"
              prefix-icon="Lock"
              size="large"
              show-password
              @keyup.enter="handleLogin"
            />
          </el-form-item>
        </template>

        <template v-else>
          <el-form-item prop="code">
            <el-input
              v-model="form.code"
              placeholder="验证码"
              prefix-icon="Key"
              size="large"
              maxlength="6"
              @keyup.enter="handleLogin"
            >
              <template #append>
                <el-button
                  :disabled="codeCountdown > 0"
                  @click="handleSendCode"
                  style="padding: 0 8px; min-width: 80px;"
                >
                  {{ codeCountdown > 0 ? `${codeCountdown}s` : '获取验证码' }}
                </el-button>
              </template>
            </el-input>
          </el-form-item>
        </template>

        <el-form-item>
          <el-button
            type="primary"
            size="large"
            style="width: 100%"
            :loading="loading"
            @click="handleLogin"
          >
            登录
          </el-button>
        </el-form-item>
      </el-form>

      <div class="footer-link">
        没有账号？<router-link to="/register">立即注册</router-link>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { sendVerificationCode } from '@/api/user'

const router = useRouter()
const userStore = useUserStore()
const formRef = ref<FormInstance>()
const loading = ref(false)
const loginType = ref<'password' | 'code'>('password')
const codeCountdown = ref(0)
let codeTimer: ReturnType<typeof setInterval> | null = null

const form = reactive({
  username: '',
  password: '',
  code: '',
})

const rules: FormRules = {
  username: [{ required: true, message: '请输入', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }],
  code: [{ required: true, message: '请输入验证码', trigger: 'blur' }],
}

watch(loginType, () => {
  form.password = ''
  form.code = ''
  formRef.value?.clearValidate()
})

async function handleSendCode() {
  const valid = await formRef.value?.validateField('username').catch(() => false)
  if (!valid) return
  codeCountdown.value = 60
  codeTimer = setInterval(() => {
    codeCountdown.value--
    if (codeCountdown.value <= 0 && codeTimer) clearInterval(codeTimer)
  }, 1000)
  try {
    await sendVerificationCode({ email: form.username, scene: 'login' })
    ElMessage.success('验证码已发送')
  } catch {
    ElMessage.error('发送失败，请稍后重试')
  }
}

async function handleLogin() {
  const valid = await formRef.value?.validate()
  if (!valid) return
  loading.value = true
  try {
    const res = await userStore.login(form.username, loginType.value === 'password' ? form.password : form.code, loginType.value)
    if (res.success) {
      ElMessage.success('登录成功')
      router.push('/')
    } else {
      ElMessage.error(res.message || '登录失败')
    }
  } catch (err: any) {
    ElMessage.error(err?.message || '登录失败')
  } finally {
    loading.value = false
  }
}

function setUserInfo(userInfo: any) {
  if (!userInfo) return
  userStore.userInfo = {
    id: userInfo.id,
    email: userInfo.email,
    username: userInfo.username,
    nickname: userInfo.nickname,
    avatar_url: userInfo.avatar_url,
    is_active: userInfo.is_active,
    created_at: '',
    updated_at: '',
  }
}
</script>

<style scoped>
.login-page {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}
.login-card { width: 420px; }
.card-header { text-align: center; font-size: 20px; font-weight: bold; }
.login-tabs { display: flex; justify-content: center; margin-bottom: 20px; }
.footer-link { text-align: center; margin-top: 10px; }
</style>