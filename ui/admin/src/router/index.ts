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
  {
    path: '/dashboard',
    name: 'Dashboard',
    component: () => import('@/views/Dashboard.vue'),
    meta: { requiresAuth: true, title: '数据概览' }
  },
  {
    path: '/user',
    name: 'UserManagement',
    component: () => import('@/views/user/index.vue'),
    meta: { requiresAuth: true, title: '用户管理' }
  },
  {
    path: '/resource',
    name: 'ResourceManagement',
    component: () => import('@/views/resource/index.vue'),
    meta: { requiresAuth: true, title: '资源管理' }
  },
  {
    path: '/order',
    name: 'OrderManagement',
    component: () => import('@/views/order/index.vue'),
    meta: { requiresAuth: true, title: '订单管理' }
  },
  {
    path: '/payment',
    name: 'PaymentConfig',
    component: () => import('@/views/payment/index.vue'),
    meta: { requiresAuth: true, title: '支付配置' }
  },
  {
    path: '/log',
    name: 'LogQuery',
    component: () => import('@/views/log/index.vue'),
    meta: { requiresAuth: true, title: '日志查询' }
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