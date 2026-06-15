<template>
  <div class="layout">
    <!-- 导航栏 -->
    <header class="navbar">
      <div class="navbar-container">
        <router-link to="/" class="logo">
          <span class="logo-icon">💎</span>
          <span class="logo-text">RSWS</span>
        </router-link>

        <nav class="nav-links">
          <router-link to="/" class="nav-link" :class="{ active: $route.path === '/' }">首页</router-link>
          <router-link v-if="userStore.isLoggedIn" to="/orders" class="nav-link" :class="{ active: $route.path === '/orders' }">我的订单</router-link>
        </nav>

        <div class="user-area">
          <template v-if="userStore.isLoggedIn">
            <el-dropdown trigger="click">
              <div class="user-info">
                <el-avatar :size="32" class="user-avatar">
                  {{ userStore.username?.charAt(0)?.toUpperCase() }}
                </el-avatar>
                <span class="user-name">{{ userStore.username }}</span>
              </div>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item @click="$router.push('/user')">
                    <el-icon><User /></el-icon>用户中心
                  </el-dropdown-item>
                  <el-dropdown-item divided @click="handleLogout">
                    <el-icon><SwitchButton /></el-icon>退出登录
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </template>
          <template v-else>
            <router-link to="/login" class="btn-login">登录</router-link>
            <router-link to="/register" class="btn-register">注册</router-link>
          </template>
        </div>
      </div>
    </header>

    <!-- 主内容 -->
    <main class="main-content">
      <slot />
    </main>

    <!-- 页脚 -->
    <footer class="footer" v-if="showFooter">
      <div class="footer-container">
        <div class="footer-brand">
          <span class="logo-icon">💎</span>
          <span class="logo-text">RSWS</span>
        </div>
        <p class="footer-copyright">© 2024 RSWS. All rights reserved.</p>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useUserStore } from '@/stores/user'

withDefaults(defineProps<{ showFooter?: boolean }>(), { showFooter: true })

const router = useRouter()
const userStore = useUserStore()

function handleLogout() {
  userStore.logout()
  router.push('/')
}
</script>

<style scoped>
.layout {
  min-height: 100vh;
  background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16213e 100%);
  color: #fff;
  display: flex;
  flex-direction: column;
}

/* 导航栏 */
.navbar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 100;
  background: rgba(15, 15, 26, 0.8);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.navbar-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 24px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.logo {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 24px;
  font-weight: 700;
  text-decoration: none;
}

.logo-icon {
  font-size: 28px;
}

.logo-text {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.nav-links {
  display: flex;
  gap: 32px;
}

.nav-link {
  color: rgba(255, 255, 255, 0.7);
  text-decoration: none;
  font-size: 15px;
  font-weight: 500;
  padding: 8px 0;
  position: relative;
  transition: color 0.3s;
}

.nav-link:hover,
.nav-link.active {
  color: #fff;
}

.nav-link.active::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, #667eea, #764ba2);
  border-radius: 2px;
}

.user-area {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 4px 12px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.1);
  transition: background 0.3s;
}

.user-info:hover {
  background: rgba(255, 255, 255, 0.15);
}

.user-avatar {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
  font-weight: 600;
}

.user-name {
  color: #fff;
  font-size: 14px;
}

.btn-login,
.btn-register {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  transition: all 0.3s;
}

.btn-login {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.btn-login:hover {
  background: rgba(255, 255, 255, 0.15);
}

.btn-register {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
  border: none;
}

.btn-register:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
}

/* 主内容 */
.main-content {
  flex: 1;
  padding-top: 64px;
}

/* 页脚 */
.footer {
  background: rgba(15, 15, 26, 0.8);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding: 40px 24px;
}

.footer-container {
  max-width: 1400px;
  margin: 0 auto;
  text-align: center;
}

.footer-brand {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 12px;
}

.footer-copyright {
  color: rgba(255, 255, 255, 0.4);
  font-size: 14px;
}

/* 下拉菜单 */
:deep(.el-dropdown-menu) {
  background: rgba(26, 26, 46, 0.95);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

:deep(.el-dropdown-menu__item) {
  color: rgba(255, 255, 255, 0.8);
}

:deep(.el-dropdown-menu__item:hover) {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

/* 响应式 */
@media (max-width: 768px) {
  .nav-links {
    display: none;
  }
}
</style>
