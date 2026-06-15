<template>
  <div class="register-page">
    <!-- 背景装饰 -->
    <div class="bg-decoration">
      <div class="bg-blob blob-1"></div>
      <div class="bg-blob blob-2"></div>
      <div class="bg-blob blob-3"></div>
    </div>

    <!-- 注册卡片 -->
    <div class="register-container">
      <div class="register-card">
        <!-- Logo -->
        <div class="register-header">
          <div class="logo">
            <span class="logo-icon">💎</span>
            <span class="logo-text">RSWS</span>
          </div>
          <h1 class="register-title">创建账户</h1>
          <p class="register-subtitle">加入我们，探索优质资源</p>
        </div>

        <!-- 步骤指示器 -->
        <div class="steps">
          <div class="step" :class="{ active: step === 1, done: step > 1 }">
            <div class="step-num">{{ step > 1 ? '✓' : '1' }}</div>
            <span>填写信息</span>
          </div>
          <div class="step-line" :class="{ active: step > 1 }"></div>
          <div class="step" :class="{ active: step === 2 }">
            <div class="step-num">2</div>
            <span>验证邮箱</span>
          </div>
        </div>

        <!-- 第一步：填写基本信息 -->
        <el-form v-show="step === 1" ref="form1Ref" :model="form" :rules="rules1" @submit.prevent>
          <el-form-item prop="email">
            <el-input v-model="form.email" placeholder="邮箱地址" size="large" class="form-input">
              <template #prefix><el-icon><Message /></el-icon></template>
            </el-input>
          </el-form-item>

          <el-form-item prop="username">
            <el-input v-model="form.username" placeholder="用户名（登录用）" size="large" class="form-input">
              <template #prefix><el-icon><User /></el-icon></template>
            </el-input>
          </el-form-item>

          <el-form-item prop="nickname">
            <el-input v-model="form.nickname" placeholder="昵称（显示名称）" size="large" class="form-input">
              <template #prefix><el-icon><Star /></el-icon></template>
            </el-input>
          </el-form-item>

          <el-form-item>
            <el-button type="primary" size="large" class="register-btn" @click="goToStep2" :loading="sendingCode">
              发送验证码
            </el-button>
          </el-form-item>
        </el-form>

        <!-- 第二步：验证码 + 密码 -->
        <el-form v-show="step === 2" ref="form2Ref" :model="form" :rules="rules2" @submit.prevent="handleRegister">
          <div class="email-hint">
            验证码已发送至 <strong>{{ maskedEmail }}</strong>
          </div>

          <el-form-item prop="verification_code">
            <div class="code-input-wrapper">
              <el-input v-model="form.verification_code" placeholder="6位验证码" size="large" class="form-input code-input" maxlength="6">
                <template #prefix><el-icon><Key /></el-icon></template>
              </el-input>
              <el-button
                size="large"
                :disabled="cooldown > 0"
                @click="resendCode"
                :loading="sendingCode"
                class="resend-btn"
              >
                {{ cooldown > 0 ? `${cooldown}s` : '重新发送' }}
              </el-button>
            </div>
          </el-form-item>

          <el-form-item prop="password">
            <el-input v-model="form.password" type="password" placeholder="设置密码" size="large" show-password class="form-input">
              <template #prefix><el-icon><Lock /></el-icon></template>
            </el-input>
          </el-form-item>

          <el-form-item prop="confirmPassword">
            <el-input v-model="form.confirmPassword" type="password" placeholder="确认密码" size="large" show-password class="form-input" @keyup.enter="handleRegister">
              <template #prefix><el-icon><Lock /></el-icon></template>
            </el-input>
          </el-form-item>

          <el-form-item>
            <el-button type="primary" size="large" class="register-btn" :loading="loading" @click="handleRegister">
              注 册
            </el-button>
          </el-form-item>

          <div class="back-link" @click="step = 1">
            <el-icon><ArrowLeft /></el-icon> 返回修改信息
          </div>
        </el-form>

        <!-- 底部链接 -->
        <div class="register-footer">
          <span class="footer-text">已有账号？</span>
          <router-link to="/login" class="footer-link">立即登录</router-link>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { sendVerificationCode } from '@/api/user'

const router = useRouter()
const userStore = useUserStore()
const form1Ref = ref<FormInstance>()
const form2Ref = ref<FormInstance>()
const loading = ref(false)
const sendingCode = ref(false)
const step = ref(1)
const cooldown = ref(0)
let cooldownTimer: ReturnType<typeof setInterval> | null = null

const form = reactive({
  email: '',
  username: '',
  nickname: '',
  verification_code: '',
  password: '',
  confirmPassword: '',
})

const maskedEmail = computed(() => {
  const e = form.email
  if (!e || !e.includes('@')) return e
  const [local, domain] = e.split('@')
  const masked = local.length > 2
    ? local[0] + '***' + local[local.length - 1]
    : local
  return `${masked}@${domain}`
})

const validatePass = (_rule: unknown, value: string, callback: (error?: Error) => void) => {
  if (value !== form.password) callback(new Error('两次密码不一致'))
  else callback()
}

const rules1: FormRules = {
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '邮箱格式不正确', trigger: 'blur' },
  ],
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 2, max: 20, message: '用户名长度 2-20 个字符', trigger: 'blur' },
  ],
  nickname: [
    { required: true, message: '请输入昵称', trigger: 'blur' },
    { min: 1, max: 30, message: '昵称长度 1-30 个字符', trigger: 'blur' },
  ],
}

const rules2: FormRules = {
  verification_code: [
    { required: true, message: '请输入验证码', trigger: 'blur' },
    { len: 6, message: '验证码为6位', trigger: 'blur' },
  ],
  password: [
    { required: true, message: '请设置密码', trigger: 'blur' },
    { min: 6, message: '密码至少6位', trigger: 'blur' },
  ],
  confirmPassword: [
    { required: true, message: '请确认密码', trigger: 'blur' },
    { validator: validatePass, trigger: 'blur' },
  ],
}

function startCooldown(seconds: number) {
  cooldown.value = seconds
  if (cooldownTimer) clearInterval(cooldownTimer)
  cooldownTimer = setInterval(() => {
    cooldown.value--
    if (cooldown.value <= 0) {
      clearInterval(cooldownTimer!)
      cooldownTimer = null
    }
  }, 1000)
}

async function goToStep2() {
  const valid = await form1Ref.value?.validate()
  if (!valid) return
  await sendCode()
  if (cooldown.value > 0) {
    step.value = 2
  }
}

async function sendCode() {
  sendingCode.value = true
  try {
    const res = await sendVerificationCode({ email: form.email, code_type: 'register' })
    if (res.code === 0) {
      ElMessage.success('验证码已发送')
      startCooldown(60)
    } else {
      ElMessage.error(res.msg || '发送失败')
    }
  } catch (err: any) {
    ElMessage.error(err?.msg || err?.message || '发送失败')
  } finally {
    sendingCode.value = false
  }
}

async function resendCode() {
  if (cooldown.value > 0) return
  await sendCode()
}

async function handleRegister() {
  const valid = await form2Ref.value?.validate()
  if (!valid) return
  loading.value = true
  try {
    const result = await userStore.register(
      form.email,
      form.password,
      form.username,
      form.nickname,
      form.verification_code,
    )
    if (result.code === 0) {
      ElMessage.success('注册成功，正在跳转...')
      router.push('/')
    } else {
      ElMessage.error(result.msg || '注册失败')
    }
  } finally {
    loading.value = false
  }
}

onUnmounted(() => {
  if (cooldownTimer) clearInterval(cooldownTimer)
})
</script>

<style scoped>
.register-page {
  position: relative;
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16213e 100%);
  overflow: hidden;
  padding: 40px 20px;
}

.bg-decoration { position: absolute; inset: 0; pointer-events: none; }
.bg-blob { position: absolute; border-radius: 50%; filter: blur(100px); opacity: 0.4; }
.blob-1 { width: 500px; height: 500px; background: #667eea; top: -200px; right: -100px; animation: float 10s ease-in-out infinite; }
.blob-2 { width: 400px; height: 400px; background: #764ba2; bottom: -150px; left: -100px; animation: float 12s ease-in-out infinite reverse; }
.blob-3 { width: 300px; height: 300px; background: #f093fb; top: 50%; left: 50%; transform: translate(-50%, -50%); animation: pulse 8s ease-in-out infinite; }
@keyframes float { 0%, 100% { transform: translate(0, 0); } 50% { transform: translate(40px, -40px); } }
@keyframes pulse { 0%, 100% { opacity: 0.3; transform: translate(-50%, -50%) scale(1); } 50% { opacity: 0.5; transform: translate(-50%, -50%) scale(1.1); } }

.register-container { position: relative; z-index: 1; width: 100%; max-width: 420px; }
.register-card {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 24px;
  padding: 40px;
}
.register-header { text-align: center; margin-bottom: 24px; }
.logo { display: inline-flex; align-items: center; gap: 8px; margin-bottom: 20px; }
.logo-icon { font-size: 32px; }
.logo-text {
  font-size: 28px; font-weight: 800;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text; -webkit-text-fill-color: transparent;
}
.register-title { font-size: 28px; font-weight: 700; color: #fff; margin-bottom: 8px; }
.register-subtitle { font-size: 14px; color: rgba(255, 255, 255, 0.5); }

/* Steps */
.steps {
  display: flex; align-items: center; justify-content: center; gap: 12px; margin-bottom: 28px;
}
.step {
  display: flex; align-items: center; gap: 6px;
  color: rgba(255, 255, 255, 0.4); font-size: 13px; transition: color 0.3s;
}
.step.active { color: #667eea; }
.step.done { color: #67c23a; }
.step-num {
  width: 28px; height: 28px; border-radius: 50%; display: flex; align-items: center; justify-content: center;
  font-size: 13px; font-weight: 600;
  background: rgba(255, 255, 255, 0.1); color: rgba(255, 255, 255, 0.4); transition: all 0.3s;
}
.step.active .step-num { background: #667eea; color: #fff; }
.step.done .step-num { background: #67c23a; color: #fff; }
.step-line {
  width: 40px; height: 2px; background: rgba(255, 255, 255, 0.1); transition: background 0.3s;
}
.step-line.active { background: #67c23a; }

/* Email hint */
.email-hint {
  text-align: center; color: rgba(255, 255, 255, 0.6); font-size: 13px; margin-bottom: 20px;
}
.email-hint strong { color: #667eea; }

/* Code input + resend */
.code-input-wrapper { display: flex; gap: 10px; width: 100%; }
.code-input { flex: 1; }
.resend-btn {
  flex-shrink: 0; min-width: 100px;
  background: rgba(102, 126, 234, 0.15); border-color: rgba(102, 126, 234, 0.3);
  color: #667eea;
}
.resend-btn:hover:not(:disabled) { background: rgba(102, 126, 234, 0.25); border-color: #667eea; }
.resend-btn:disabled { opacity: 0.5; }

/* Back link */
.back-link {
  text-align: center; color: rgba(255, 255, 255, 0.5); font-size: 13px;
  cursor: pointer; margin-top: 8px; transition: color 0.3s;
}
.back-link:hover { color: #667eea; }
.back-link .el-icon { vertical-align: middle; margin-right: 2px; }

/* Form styles */
.form-input :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px; box-shadow: none; transition: all 0.3s;
}
.form-input :deep(.el-input__wrapper:hover) { border-color: rgba(255, 255, 255, 0.2); }
.form-input :deep(.el-input__wrapper.is-focus) { border-color: #667eea; background: rgba(102, 126, 234, 0.1); }
.form-input :deep(.el-input__inner) { color: #fff; }
.form-input :deep(.el-input__inner::placeholder) { color: rgba(255, 255, 255, 0.4); }
.form-input :deep(.el-input__prefix) { color: rgba(255, 255, 255, 0.5); }

.register-btn {
  width: 100%; height: 48px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none; border-radius: 12px; font-size: 16px; font-weight: 600;
  letter-spacing: 4px; transition: all 0.3s;
}
.register-btn:hover { transform: translateY(-2px); box-shadow: 0 10px 30px rgba(102, 126, 234, 0.4); }

.register-footer { text-align: center; margin-top: 24px; }
.footer-text { color: rgba(255, 255, 255, 0.5); font-size: 14px; }
.footer-link {
  color: #667eea; font-size: 14px; font-weight: 500;
  text-decoration: none; margin-left: 4px; transition: color 0.3s;
}
.footer-link:hover { color: #764ba2; }

@media (max-width: 480px) {
  .register-card { padding: 24px; }
  .register-title { font-size: 24px; }
  .code-input-wrapper { flex-direction: column; }
  .resend-btn { min-width: unset; width: 100%; }
}
</style>
