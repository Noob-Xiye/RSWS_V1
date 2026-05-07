import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { login as apiLogin, register as apiRegister, getUserInfo } from '@/api/user'
import { setApiKey, getApiKey, removeApiKey } from '@/utils/storage'
import type { User } from '@/api/user'

export const useUserStore = defineStore('user', () => {
  const userInfo = ref<User | null>(null)
  const apiKey = ref<string | null>(getApiKey())

  const isLoggedIn = computed(() => !!apiKey.value)
  const username = computed(() => userInfo.value?.username || '未登录')

  async function login(email: string, password: string) {
    const res = await apiLogin(email, password)
    if (res.success && res.data) {
      apiKey.value = res.data.api_key
      userInfo.value = res.data.user
      setApiKey(res.data.api_key)
      return { success: true }
    }
    return { success: false, message: res.message }
  }

  async function register(email: string, password: string, username?: string) {
    const res = await apiRegister(email, password, username)
    if (res.success && res.data) {
      apiKey.value = res.data.api_key
      userInfo.value = res.data.user
      setApiKey(res.data.api_key)
      return { success: true }
    }
    return { success: false, message: res.message }
  }

  async function fetchUserInfo() {
    if (!apiKey.value) return
    const res = await getUserInfo()
    if (res.success && res.data) userInfo.value = res.data
  }

  function logout() {
    userInfo.value = null
    apiKey.value = null
    removeApiKey()
  }

  if (apiKey.value) fetchUserInfo()

  return { userInfo, apiKey, isLoggedIn, username, login, register, logout, fetchUserInfo }
})