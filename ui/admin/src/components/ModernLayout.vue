<template>
  <div class="layout">
    <!-- 导航栏 -->
    <header class="navbar">
      <div class="navbar-container">
        <router-link to="/dashboard" class="logo">
          <span class="logo-icon">⚙️</span>
          <span class="logo-text">RSWS Admin</span>
        </router-link>

        <nav class="nav-links">
          <router-link to="/dashboard" class="nav-link" :class="{ active: $route.path === '/dashboard' }">数据概览</router-link>
          <router-link to="/user" class="nav-link" :class="{ active: $route.path.startsWith('/user') }">用户管理</router-link>
          <router-link to="/resource" class="nav-link" :class="{ active: $route.path.startsWith('/resource') }">资源管理</router-link>
          <router-link to="/order" class="nav-link" :class="{ active: $route.path.startsWith('/order') }">订单管理</router-link>
          <router-link to="/payment" class="nav-link" :class="{ active: $route.path.startsWith('/payment') }">支付配置</router-link>
          <router-link to="/log" class="nav-link" :class="{ active: $route.path.startsWith('/log') }">日志查询</router-link>
          <router-link to="/admin" class="nav-link" :class="{ active: $route.path.startsWith('/admin') }">管理员</router-link>
        </nav>

        <div class="user-area">
          <el-dropdown trigger="click">
            <div class="user-info">
              <el-avatar :size="32" class="user-avatar">
                {{ authStore.username?.charAt(0)?.toUpperCase() || 'A' }}
              </el-avatar>
              <span class="user-name">{{ authStore.username || 'Admin' }}</span>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="handleLogout">
                  <el-icon><SwitchButton /></el-icon>退出登录
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </header>

    <!-- 主内容 -->
    <main class="main-content">
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

withDefaults(defineProps<{ showFooter?: boolean }>(), { showFooter: false })

const router = useRouter()
const authStore = useAuthStore()

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>

<style scoped>
.layout {
  min-height: 100vh;
  background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16213e 100%);
  color: #fff;
}

/* 导航栏 */
.navbar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 100;
  background: rgba(15, 15, 26, 0.9);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.navbar-container {
  max-width: 1600px;
  margin: 0 auto;
  padding: 0 24px;
  height: 64px;
  display: flex;
  align-items: center;
  gap: 32px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 700;
  text-decoration: none;
  flex-shrink: 0;
}

.logo-icon {
  font-size: 22px;
}

.logo-text {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.nav-links {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  flex: 1;
}

.nav-link {
  color: rgba(255, 255, 255, 0.6);
  text-decoration: none;
  font-size: 14px;
  font-weight: 500;
  padding: 8px 14px;
  border-radius: 8px;
  transition: all 0.3s;
  white-space: nowrap;
}

.nav-link:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.05);
}

.nav-link.active {
  color: #fff;
  background: rgba(102, 126, 234, 0.15);
}

.user-area {
  flex-shrink: 0;
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
  font-size: 14px;
}

.user-name {
  color: #fff;
  font-size: 14px;
}

/* 主内容 */
.main-content {
  padding-top: 64px;
  min-height: 100vh;
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
@media (max-width: 1024px) {
  .nav-links {
    display: none;
  }
}
</style>
