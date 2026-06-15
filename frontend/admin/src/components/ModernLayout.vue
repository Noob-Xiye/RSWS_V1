<template>
  <div class="layout">
    <!-- 左侧边栏 -->
    <aside class="sidebar" :class="{ collapsed: isCollapsed }">
      <div class="sidebar-header">
        <router-link to="/dashboard" class="logo">
          <span class="logo-icon">💎</span>
          <span v-show="!isCollapsed" class="logo-text">RSWS Admin</span>
        </router-link>
        <button class="collapse-btn" @click="isCollapsed = !isCollapsed">
          <el-icon :size="16"><Fold v-if="!isCollapsed" /><Expand v-else /></el-icon>
        </button>
      </div>

      <el-scrollbar class="sidebar-menu-wrap">
        <el-menu
          :default-active="activeMenu"
          :collapse="isCollapsed"
          :collapse-transition="false"
          background-color="transparent"
          text-color="rgba(255,255,255,0.65)"
          active-text-color="#fff"
          router
        >
          <el-menu-item index="/dashboard">
            <el-icon><DataAnalysis /></el-icon>
            <template #title>数据概览</template>
          </el-menu-item>

          <el-sub-menu index="user-group">
            <template #title>
              <el-icon><User /></el-icon>
              <span>用户管理</span>
            </template>
            <el-menu-item index="/users">用户账号</el-menu-item>
            <el-menu-item index="/user-api-keys">用户 API Key</el-menu-item>
            <el-menu-item index="/user-resources">用户资源</el-menu-item>
            <el-menu-item index="/user-orders">用户订单</el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="admin-group">
            <template #title>
              <el-icon><UserFilled /></el-icon>
              <span>管理员管理</span>
            </template>
            <el-menu-item index="/admins">管理员账号</el-menu-item>
            <el-menu-item index="/admin-api-keys">管理员 API Key</el-menu-item>
            <el-menu-item index="/platform-resources">平台资源</el-menu-item>
            <el-menu-item index="/platform-orders">平台订单</el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="settings-group">
            <template #title>
              <el-icon><Setting /></el-icon>
              <span>系统设置</span>
            </template>
            <el-menu-item index="/email-config">邮件配置</el-menu-item>
            <el-menu-item index="/logs">日志管理</el-menu-item>
            <el-menu-item index="/payment-config">支付配置</el-menu-item>
            <el-menu-item index="/settings">系统设置</el-menu-item>
          </el-sub-menu>
        </el-menu>
      </el-scrollbar>
    </aside>

    <div class="main-wrapper" :style="{ marginLeft: isCollapsed ? '64px' : '240px' }">
      <header class="topbar">
        <div class="topbar-left">
          <el-breadcrumb separator="/">
            <el-breadcrumb-item :to="{ path: '/dashboard' }">首页</el-breadcrumb-item>
            <el-breadcrumb-item v-if="currentTitle">{{ currentTitle }}</el-breadcrumb-item>
          </el-breadcrumb>
        </div>
        <div class="topbar-right">
          <el-dropdown trigger="click">
            <div class="user-info">
              <el-avatar :size="28" class="user-avatar">
                {{ authStore.adminName?.charAt(0)?.toUpperCase() || 'A' }}
              </el-avatar>
              <span class="user-name">{{ authStore.adminName || '未登录' }}</span>
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
      </header>

      <main class="main-content">
        <slot />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  DataAnalysis, User, UserFilled, Setting, SwitchButton, Fold, Expand
} from '@element-plus/icons-vue'

withDefaults(defineProps<{ showFooter?: boolean }>(), { showFooter: false })

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const isCollapsed = ref(false)

const activeMenu = computed(() => route.path)

const currentTitle = computed(() => {
  const map: Record<string, string> = {
    '/dashboard': '数据概览',
    '/users': '用户账号',
    '/user-api-keys': '用户 API Key',
    '/user-resources': '用户资源',
    '/user-orders': '用户订单',
    '/admins': '管理员账号',
    '/admin-api-keys': '管理员 API Key',
    '/platform-resources': '平台资源',
    '/platform-orders': '平台订单',
    '/email-config': '邮件配置',
    '/logs': '日志管理',
    '/payment-config': '支付配置',
    '/settings': '系统设置',
  }
  return map[route.path] || ''
})

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>

<style scoped>
.layout { display: flex; min-height: 100vh; background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16213e 100%); color: #fff; }
.sidebar { width: 240px; background: rgba(15,15,26,.95); backdrop-filter: blur(20px); border-right: 1px solid rgba(255,255,255,.08); display: flex; flex-direction: column; flex-shrink: 0; transition: width .3s; position: fixed; top: 0; left: 0; bottom: 0; z-index: 100; }
.sidebar.collapsed { width: 64px; }
.sidebar-header { height: 56px; display: flex; align-items: center; justify-content: space-between; padding: 0 16px; border-bottom: 1px solid rgba(255,255,255,.08); }
.logo { display: flex; align-items: center; gap: 8px; font-size: 16px; font-weight: 700; text-decoration: none; color: #fff; overflow: hidden; white-space: nowrap; }
.logo-icon { font-size: 20px; flex-shrink: 0; }
.logo-text { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
.collapse-btn { background: none; border: none; color: rgba(255,255,255,.5); cursor: pointer; padding: 4px; border-radius: 4px; display: flex; align-items: center; justify-content: center; transition: color .2s; }
.collapse-btn:hover { color: #fff; }
.sidebar-menu-wrap { flex: 1; padding: 8px 0; }
:deep(.el-menu) { border-right: none; }
:deep(.el-menu-item), :deep(.el-sub-menu__title) { height: 40px; line-height: 40px; margin: 2px 8px; border-radius: 8px; }
:deep(.el-menu-item:hover), :deep(.el-sub-menu__title:hover) { background: rgba(255,255,255,.06) !important; }
:deep(.el-menu-item.is-active) { background: rgba(102,126,234,.2) !important; color: #fff !important; }
:deep(.el-sub-menu .el-menu-item) { padding-left: 52px !important; height: 36px; line-height: 36px; font-size: 13px; }
.main-wrapper { flex: 1; transition: margin-left .3s; display: flex; flex-direction: column; min-height: 100vh; }
.topbar { height: 56px; background: rgba(15,15,26,.8); backdrop-filter: blur(20px); border-bottom: 1px solid rgba(255,255,255,.08); display: flex; align-items: center; justify-content: space-between; padding: 0 24px; position: sticky; top: 0; z-index: 50; }
.topbar-left { display: flex; align-items: center; }
.topbar-right { display: flex; align-items: center; }
.user-info { display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 4px 12px; border-radius: 20px; background: rgba(255,255,255,.08); transition: background .3s; }
.user-info:hover { background: rgba(255,255,255,.12); }
.user-avatar { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: #fff; font-weight: 600; font-size: 12px; }
.user-name { color: rgba(255,255,255,.85); font-size: 13px; }
:deep(.el-breadcrumb__inner) { color: rgba(255,255,255,.5) !important; }
:deep(.el-breadcrumb__inner.is-link) { color: rgba(255,255,255,.5) !important; }
:deep(.el-breadcrumb__inner.is-link:hover) { color: #fff !important; }
:deep(.el-breadcrumb__separator) { color: rgba(255,255,255,.3) !important; }
:deep(.el-breadcrumb__item:last-child .el-breadcrumb__inner) { color: rgba(255,255,255,.85) !important; }
:deep(.el-dropdown-menu) { background: rgba(26,26,46,.95); backdrop-filter: blur(20px); border: 1px solid rgba(255,255,255,.1); }
:deep(.el-dropdown-menu__item) { color: rgba(255,255,255,.8); }
:deep(.el-dropdown-menu__item:hover) { background: rgba(255,255,255,.1); color: #fff; }
.main-content { flex: 1; padding: 20px; }
@media (max-width: 768px) { .sidebar { width: 64px; } .main-wrapper { margin-left: 64px; } }
</style>
