import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { adminLogin, type LoginResponse } from '@/api/admin'
import { setApiKey, getApiKey, removeApiKey, setAdminId, getAdminId, removeAdminId, setAdminInfo, getAdminInfo, removeAdminInfo } from '@/utils/storage'
import type { AdminInfo } from '@/api/admin'

export const useAuthStore = defineStore('auth', () => {
  // 从 localStorage 恢复状态
  const savedInfo = getAdminInfo()
  const adminInfo = ref<AdminInfo | null>(savedInfo as AdminInfo | null)
  const apiKey = ref<string | null>(getApiKey())
  const adminId = ref<string | null>(getAdminId())

  const isLoggedIn = computed(() => !!apiKey.value)
  const adminName = computed(() => adminInfo.value?.email || (apiKey.value ? 'Admin' : '未登录'))

  async function login(email: string, password: string) {
    try {
      const res = await adminLogin(email, password)
      const loginData = res.data
      if (res.code === 0 && loginData) {
        apiKey.value = loginData.api_key
        setApiKey(loginData.api_key)
        adminId.value = String(loginData.admin.id)
        setAdminId(String(loginData.admin.id))
        adminInfo.value = loginData.admin
        setAdminInfo(loginData.admin as unknown as Record<string, unknown>)
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
    removeAdminInfo()
  }

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