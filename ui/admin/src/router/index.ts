import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/Login.vue'),
    meta: { requiresAuth: false }
  },
  {
    path: '/',
    redirect: '/dashboard'
  },
  // 数据概览
  {
    path: '/dashboard',
    name: 'Dashboard',
    component: () => import('@/views/Dashboard.vue'),
    meta: { requiresAuth: true, title: '数据概览' }
  },
  // 用户管理
  {
    path: '/users',
    name: 'UserAccounts',
    component: () => import('@/views/user/index.vue'),
    meta: { requiresAuth: true, title: '用户账号', group: '用户管理' }
  },
  {
    path: '/user-api-keys',
    name: 'UserApiKeys',
    component: () => import('@/views/user-api-key/index.vue'),
    meta: { requiresAuth: true, title: '用户 API Key', group: '用户管理' }
  },
  {
    path: '/user-resources',
    name: 'UserResources',
    component: () => import('@/views/user-resource/index.vue'),
    meta: { requiresAuth: true, title: '用户资源', group: '用户管理' }
  },
  {
    path: '/user-orders',
    name: 'UserOrders',
    component: () => import('@/views/user-order/index.vue'),
    meta: { requiresAuth: true, title: '用户订单', group: '用户管理' }
  },
  // 管理员管理
  {
    path: '/admins',
    name: 'AdminAccounts',
    component: () => import('@/views/admin/index.vue'),
    meta: { requiresAuth: true, title: '管理员账号', group: '管理员管理' }
  },
  {
    path: '/admin-api-keys',
    name: 'AdminApiKeys',
    component: () => import('@/views/admin-api-key/index.vue'),
    meta: { requiresAuth: true, title: '管理员 API Key', group: '管理员管理' }
  },
  {
    path: '/platform-resources',
    name: 'PlatformResources',
    component: () => import('@/views/resource/index.vue'),
    meta: { requiresAuth: true, title: '平台资源', group: '管理员管理' }
  },
  {
    path: '/platform-orders',
    name: 'PlatformOrders',
    component: () => import('@/views/order/index.vue'),
    meta: { requiresAuth: true, title: '平台订单', group: '管理员管理' }
  },
  // 系统设置
  {
    path: '/email-config',
    name: 'EmailConfig',
    component: () => import('@/views/email-config/index.vue'),
    meta: { requiresAuth: true, title: '邮件配置', group: '系统设置' }
  },
  {
    path: '/logs',
    name: 'LogQuery',
    component: () => import('@/views/log/index.vue'),
    meta: { requiresAuth: true, title: '日志管理', group: '系统设置' }
  },
  {
    path: '/payment-config',
    name: 'PaymentConfig',
    component: () => import('@/views/payment/index.vue'),
    meta: { requiresAuth: true, title: '支付配置', group: '系统设置' }
  },
  {
    path: '/settings',
    name: 'SystemSettings',
    component: () => import('@/views/settings/index.vue'),
    meta: { requiresAuth: true, title: '系统设置', group: '系统设置' }
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/dashboard'
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫
router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()
  
  if (to.meta.requiresAuth && !authStore.isLoggedIn) {
    next('/login')
  } else if (to.path === '/login' && authStore.isLoggedIn) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router
