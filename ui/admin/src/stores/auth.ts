import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { adminLogin } from '@/api/admin'
import { setToken, getToken, removeToken, setApiKey, getApiKey, removeApiKey } from '@/utils/storage'
import type { AdminInfo } from '@/api/admin'

export const useAuthStore = defineStore('auth', () => {
  const adminInfo = ref<AdminInfo | null>(null)
  const token = ref<string | null>(getToken())
  const apiKey = ref<string | null>(getApiKey())

  const isLoggedIn = computed(() => !!apiKey.value)
  const adminName = computed(() => adminInfo.value?.email || '未登录')

  async function login(email: string, password: string) {
    try {
      const res = await adminLogin(email, password)
      if (res.success && res.data) {
        const { admin, token: sessionToken } = res.data
        token.value = sessionToken
        apiKey.value = sessionToken  // 用 session token 作为 API Key
        adminInfo.value = admin
        setToken(sessionToken)
        setApiKey(sessionToken)
        return { success: true }
      }
      return { success: false, message: res.message || '登录失败' }
    } catch (error: unknown) {
      const err = error as { response?: { data?: { message?: string } } }
      return { success: false, message: err.response?.data?.message || '网络错误' }
    }
  }

  function logout() {
    adminInfo.value = null
    token.value = null
    apiKey.value = null
    removeToken()
    removeApiKey()
  }

  // 初始化时获取管理员信息（已从登录响应获取 adminInfo，跳过 fetchAdminInfo）
  if (apiKey.value) {
    // 已通过登录获取 adminInfo，不需要重复请求
  }

  return {
    adminInfo,
    token,
    apiKey,
    isLoggedIn,
    adminName,
    login,
    logout,
  }
})