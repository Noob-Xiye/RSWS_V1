import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { adminLogin, type LoginResponse } from '@/api/admin'
import { setApiKey, getApiKey, removeApiKey, setAdminId, getAdminId, removeAdminId } from '@/utils/storage'
import type { AdminInfo } from '@/api/admin'

export const useAuthStore = defineStore('auth', () => {
  const adminInfo = ref<AdminInfo | null>(null)
  const apiKey = ref<string | null>(getApiKey())
  const adminId = ref<string | null>(getAdminId())

  const isLoggedIn = computed(() => !!apiKey.value)
  const adminName = computed(() => adminInfo.value?.email || '未登录')

  async function login(email: string, password: string) {
    try {
      const res = await adminLogin(email, password)
      // 后端 ApiResponse 格式: { code: 0, msg: "success", data: LoginResponse }
      // LoginResponse: { admin: AdminInfo, api_key: string, expires_at: string }
      const loginData = res.data
      if (res.code === 0 && loginData) {
        apiKey.value = loginData.api_key
        setApiKey(loginData.api_key)
        adminId.value = String(loginData.admin.id)
        setAdminId(String(loginData.admin.id))
        adminInfo.value = loginData.admin
        return { code: 0, msg: '登录成功' }
      }
      return { code: -1, msg: res.msg || '登录失败' }
    } catch (error: unknown) {
      const err = error as { msg?: string; message?: string; response?: { data?: { msg?: string } } }
      return { code: -1, msg: err?.msg || err?.message || err?.response?.data?.msg || '网络错误' }
    }
  }

  function logout() {
    adminInfo.value = null
    apiKey.value = null
    adminId.value = null
    removeApiKey()
    removeAdminId()
  }

  // 初始化时：如果已有 apiKey + adminId，尝试获取管理员信息
  // 但由于登录响应已包含 adminInfo，不需要额外请求

  return {
    adminInfo,
    apiKey,
    adminId,
    isLoggedIn,
    adminName,
    login,
    logout,
  }
})
