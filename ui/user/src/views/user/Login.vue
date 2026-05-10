<template>
  <div class="login-page">
    <!-- 背景装饰 -->
    <div class="bg-decoration">
      <div class="bg-blob blob-1"></div>
      <div class="bg-blob blob-2"></div>
      <div class="bg-blob blob-3"></div>
    </div>

    <!-- 登录卡片 -->
    <div class="login-container">
      <div class="login-card">
        <!-- Logo -->
        <div class="login-header">
          <div class="logo">
            <span class="logo-icon">💎</span>
            <span class="logo-text">RSWS</span>
          </div>
          <h1 class="login-title">欢迎回来</h1>
          <p class="login-subtitle">登录您的账户继续</p>
        </div>

        <!-- 登录方式切换 -->
        <div class="login-tabs">
          <button 
            class="tab-btn" 
            :class="{ active: loginType === 'password' }"
            @click="loginType = 'password'"
          >密码登录</button>
          <button 
            class="tab-btn" 
            :class="{ active: loginType === 'code' }"
            @click="loginType = 'code'"
          >验证码登录</button>
        </div>

        <!-- 登录表单 -->
        <el-form ref="formRef" :model="form" :rules="rules" @submit.prevent="handleLogin">
          <el-form-item prop="username">
            <el-input
              v-model="form.username"
              :placeholder="loginType === 'password' ? '用户名' : '邮箱地址'"
              size="large"
              class="form-input"
            >
              <template #prefix>
                <el-icon><User /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <template v-if="loginType === 'password'">
            <el-form-item prop="password">
              <el-input
                v-model="form.password"
                type="password"
                placeholder="密码"
                size="large"
                show-password
                class="form-input"
                @keyup.enter="handleLogin"
              >
                <template #prefix>
                  <el-icon><Lock /></el-icon>
                </template>
              </el-input>
            </el-form-item>
          </template>

          <template v-else>
            <el-form-item prop="code">
              <el-input
                v-model="form.code"
                placeholder="验证码"
                size="large"
                maxlength="6"
                class="form-input"
                @keyup.enter="handleLogin"
              >
                <template #prefix>
                  <el-icon><Key /></el-icon>
                </template>
                <template #append>
                  <el-button
                    :disabled="codeCountdown > 0"
                    class="code-btn"
                    @click="handleSendCode"
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
              class="login-btn"
              :loading="loading"
              @click="handleLogin"
            >
              登 录
            </el-button>
          </el-form-item>
        </el-form>

        <!-- 底部链接 -->
        <div class="login-footer">
          <span class="footer-text">还没有账号？</span>
          <router-link to="/register" class="footer-link">立即注册</router-link>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { sendVerificationCode } from '@/api/user'

const router = useRouter()
const route = useRoute()
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
      const redirect = route.query.redirect as string
      router.push(redirect || '/user')
    } else {
      ElMessage.error(res.message || '登录失败')
    }
  } catch (err: any) {
    ElMessage.error(err?.message || '登录失败')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-page {
  position: relative;
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16213e 100%);
  overflow: hidden;
}

/* 背景装饰 */
.bg-decoration {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.bg-blob {
  position: absolute;
  border-radius: 50%;
  filter: blur(100px);
  opacity: 0.4;
}

.blob-1 {
  width: 500px;
  height: 500px;
  background: #667eea;
  top: -200px;
  right: -100px;
  animation: float 10s ease-in-out infinite;
}

.blob-2 {
  width: 400px;
  height: 400px;
  background: #764ba2;
  bottom: -150px;
  left: -100px;
  animation: float 12s ease-in-out infinite reverse;
}

.blob-3 {
  width: 300px;
  height: 300px;
  background: #f093fb;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation: pulse 8s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translate(0, 0); }
  50% { transform: translate(40px, -40px); }
}

@keyframes pulse {
  0%, 100% { opacity: 0.3; transform: translate(-50%, -50%) scale(1); }
  50% { opacity: 0.5; transform: translate(-50%, -50%) scale(1.1); }
}

/* 登录容器 */
.login-container {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 420px;
  padding: 20px;
}

/* 登录卡片 */
.login-card {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 24px;
  padding: 40px;
}

/* Logo 和标题 */
.login-header {
  text-align: center;
  margin-bottom: 32px;
}

.logo {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 20px;
}

.logo-icon {
  font-size: 32px;
}

.logo-text {
  font-size: 28px;
  font-weight: 800;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.login-title {
  font-size: 28px;
  font-weight: 700;
  color: #fff;
  margin-bottom: 8px;
}

.login-subtitle {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.5);
}

/* 登录方式切换 */
.login-tabs {
  display: flex;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 4px;
  margin-bottom: 24px;
}

.tab-btn {
  flex: 1;
  padding: 10px 16px;
  border: none;
  background: transparent;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  transition: all 0.3s;
}

.tab-btn:hover {
  color: rgba(255, 255, 255, 0.8);
}

.tab-btn.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

/* 表单样式 */
.form-input :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  box-shadow: none;
  transition: all 0.3s;
}

.form-input :deep(.el-input__wrapper:hover) {
  border-color: rgba(255, 255, 255, 0.2);
}

.form-input :deep(.el-input__wrapper.is-focus) {
  border-color: #667eea;
  background: rgba(102, 126, 234, 0.1);
}

.form-input :deep(.el-input__inner) {
  color: #fff;
}

.form-input :deep(.el-input__inner::placeholder) {
  color: rgba(255, 255, 255, 0.4);
}

.form-input :deep(.el-input__prefix) {
  color: rgba(255, 255, 255, 0.5);
}

.form-input :deep(.el-input-group__append) {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-left: none;
  border-radius: 0 12px 12px 0;
  padding: 0;
}

.code-btn {
  background: transparent;
  border: none;
  color: #667eea;
  font-size: 13px;
  padding: 0 16px;
  min-width: 90px;
}

.code-btn:hover {
  color: #764ba2;
}

.code-btn:disabled {
  color: rgba(255, 255, 255, 0.3);
}

/* 登录按钮 */
.login-btn {
  width: 100%;
  height: 48px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 4px;
  transition: all 0.3s;
}

.login-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(102, 126, 234, 0.4);
}

/* 底部链接 */
.login-footer {
  text-align: center;
  margin-top: 24px;
}

.footer-text {
  color: rgba(255, 255, 255, 0.5);
  font-size: 14px;
}

.footer-link {
  color: #667eea;
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  margin-left: 4px;
  transition: color 0.3s;
}

.footer-link:hover {
  color: #764ba2;
}

/* 响应式 */
@media (max-width: 480px) {
  .login-card {
    padding: 24px;
  }
  
  .login-title {
    font-size: 24px;
  }
}
</style>
