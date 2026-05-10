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

        <!-- 注册表单 -->
        <el-form ref="formRef" :model="form" :rules="rules" @submit.prevent="handleRegister">
          <el-form-item prop="email">
            <el-input
              v-model="form.email"
              placeholder="邮箱地址"
              size="large"
              class="form-input"
            >
              <template #prefix>
                <el-icon><Message /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item prop="username">
            <el-input
              v-model="form.username"
              placeholder="用户名"
              size="large"
              class="form-input"
            >
              <template #prefix>
                <el-icon><User /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item prop="password">
            <el-input
              v-model="form.password"
              type="password"
              placeholder="密码"
              size="large"
              show-password
              class="form-input"
            >
              <template #prefix>
                <el-icon><Lock /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item prop="confirmPassword">
            <el-input
              v-model="form.confirmPassword"
              type="password"
              placeholder="确认密码"
              size="large"
              show-password
              class="form-input"
              @keyup.enter="handleRegister"
            >
              <template #prefix>
                <el-icon><Lock /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item>
            <el-button
              type="primary"
              size="large"
              class="register-btn"
              :loading="loading"
              @click="handleRegister"
            >
              注 册
            </el-button>
          </el-form-item>
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
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { useUserStore } from '@/stores/user'

const router = useRouter()
const userStore = useUserStore()
const formRef = ref<FormInstance>()
const loading = ref(false)
const form = reactive({ email: '', username: '', password: '', confirmPassword: '' })

const validatePass = (_rule: unknown, value: string, callback: (error?: Error) => void) => {
  if (value !== form.password) callback(new Error('两次密码不一致'))
  else callback()
}

const rules: FormRules = {
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '邮箱格式不正确', trigger: 'blur' }
  ],
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 2, max: 20, message: '用户名长度 2-20 个字符', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码至少6位', trigger: 'blur' }
  ],
  confirmPassword: [
    { required: true, message: '请确认密码', trigger: 'blur' },
    { validator: validatePass, trigger: 'blur' }
  ]
}

async function handleRegister() {
  const valid = await formRef.value?.validate()
  if (!valid) return
  loading.value = true
  try {
    const result = await userStore.register(form.email, form.password, form.username)
    if (result.success) {
      ElMessage.success('注册成功')
      router.push('/')
    } else {
      ElMessage.error(result.message || '注册失败')
    }
  } finally {
    loading.value = false
  }
}
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

/* 注册容器 */
.register-container {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 420px;
}

/* 注册卡片 */
.register-card {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 24px;
  padding: 40px;
}

/* Logo 和标题 */
.register-header {
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

.register-title {
  font-size: 28px;
  font-weight: 700;
  color: #fff;
  margin-bottom: 8px;
}

.register-subtitle {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.5);
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

/* 注册按钮 */
.register-btn {
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

.register-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(102, 126, 234, 0.4);
}

/* 底部链接 */
.register-footer {
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
  .register-card {
    padding: 24px;
  }
  
  .register-title {
    font-size: 24px;
  }
}
</style>
