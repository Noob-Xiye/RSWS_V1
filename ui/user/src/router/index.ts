import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { useUserStore } from '@/stores/user'

// 需要登录的路由
const protectedRoutes = ['/user', '/orders']

// 登录后不能访问的路由 (如 login, register)
const guestRoutes = ['/login', '/register']

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/home/index.vue'),
    meta: { title: '首页' }
  },
  {
    path: '/resource/:id',
    name: 'ResourceDetail',
    component: () => import('@/views/resource/detail.vue'),
    meta: { title: '资源详情' }
  },
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/user/Login.vue'),
    meta: { title: '登录' }
  },
  {
    path: '/register',
    name: 'Register',
    component: () => import('@/views/user/Register.vue'),
    meta: { title: '注册' }
  },
  {
    path: '/user',
    name: 'UserCenter',
    component: () => import('@/views/user/index.vue'),
    meta: { title: '用户中心' }
  },
  {
    path: '/orders',
    name: 'Orders',
    component: () => import('@/views/order/index.vue'),
    meta: { title: '我的订单' }
  },
  {
    path: '/payment/success',
    name: 'PaymentSuccess',
    component: () => import('@/views/payment/success.vue'),
    meta: { title: '支付成功' }
  },
  {
    path: '/payment/cancel',
    name: 'PaymentCancel',
    component: () => import('@/views/payment/cancel.vue'),
    meta: { title: '支付取消' }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

router.beforeEach((to, _from, next) => {
  const userStore = useUserStore()
  const isLoggedIn = userStore.isLoggedIn
  
  // 需要登录的路由但用户未登录
  if (protectedRoutes.some(path => to.path.startsWith(path)) && !isLoggedIn) {
    next({ path: '/login', query: { redirect: to.fullPath } })
    return
  }
  
  // 已登录用户访问登录/注册页，跳转到用户中心
  if (guestRoutes.includes(to.path) && isLoggedIn) {
    next('/user')
    return
  }
  
  // 更新页面标题
  document.title = `${to.meta.title || ''} - RSWS`
  next()
})

export default router