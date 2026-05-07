import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { adminLogin, getAdminInfo } from '@/api/admin'
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
        token.value = res.data.token
        apiKey.value = res.data.api_key
        setToken(res.data.token)
        setApiKey(res.data.api_key)
        
        // 获取管理员信息
        await fetchAdminInfo()
        return { success: true }
      }
      return { success: false, message: res.message || '登录失败' }
    } catch (error: unknown) {
      const err = error as { response?: { data?: { message?: string } } }
      return { success: false, message: err.response?.data?.message || '网络错误' }
    }
  }

  async function fetchAdminInfo() {
    if (!apiKey.value) return
    try {
      const res = await getAdminInfo()
      if (res.success && res.data) {
        adminInfo.value = res.data
      }
    } catch {
      // ignore
    }
  }

  function logout() {
    adminInfo.value = null
    token.value = null
    apiKey.value = null
    removeToken()
    removeApiKey()
  }

  // 初始化时获取管理员信息
  if (apiKey.value) {
    fetchAdminInfo()
  }

  return {
    adminInfo,
    token,
    apiKey,
    isLoggedIn,
    adminName,
    login,
    logout,
    fetchAdminInfo
  }
})