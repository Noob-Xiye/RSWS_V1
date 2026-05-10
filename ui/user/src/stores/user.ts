import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { register as apiRegister, getUserInfo, login as apiLogin } from '@/api/user'
import { setApiKey, getApiKey, removeApiKey, setApiSecret, getApiSecret, removeApiSecret } from '@/utils/storage'
import type { User } from '@/api/user'

export interface UserInfo {
  id: number
  email: string
  username: string
  nickname?: string
  avatar_url?: string | null
  balance?: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export const useUserStore = defineStore('user', () => {
  const userInfo = ref<UserInfo | null>(null)
  const apiKey = ref<string | null>(getApiKey())
  const apiSecret = ref<string | null>(getApiSecret())

  const isLoggedIn = computed(() => !!apiKey.value)
  const username = computed(() => userInfo.value?.nickname || userInfo.value?.username || '未登录')
  const balance = computed(() => userInfo.value?.balance || '0')

  async function login(loginStr: string, passwordOrCode: string, loginType: 'password' | 'code') {
    try {
      const res = await apiLogin({
        login: loginStr,
        password: loginType === 'password' ? passwordOrCode : undefined,
        verification_code: loginType === 'code' ? passwordOrCode : undefined,
        login_type: loginType,
      })
      if (res.success && res.data?.success && res.data.session_data?.api_key) {
        apiKey.value = res.data.session_data.api_key
        // Store api_secret if provided
        if (res.data.session_data.api_secret) {
          apiSecret.value = res.data.session_data.api_secret
          setApiSecret(res.data.session_data.api_secret)
        }
        setApiKey(res.data.session_data.api_key)
        // Set user info if provided
        if (res.data.user_info) {
          userInfo.value = {
            id: res.data.user_info.id ?? 0,
            email: res.data.user_info.email ?? '',
            username: res.data.user_info.username ?? '',
            nickname: res.data.user_info.nickname,
            avatar_url: res.data.user_info.avatar_url,
            is_active: res.data.user_info.is_active ?? false,
            created_at: '',
            updated_at: '',
          }
        }
        return { success: true }
      }
      return { success: false, message: res.data?.message || res.message || '登录失败' }
    } catch (err: any) {
      return { success: false, message: err?.message || '登录失败' }
    }
  }

  async function register(email: string, password: string, username: string) {
    try {
      const res = await apiRegister({ email, password, username })
      if (res.success && res.data?.success && res.data.user_info) {
        if (res.data.session_data?.api_key) {
          apiKey.value = res.data.session_data.api_key
          setApiKey(res.data.session_data.api_key)
          // Store api_secret if provided
          if (res.data.session_data.api_secret) {
            apiSecret.value = res.data.session_data.api_secret
            setApiSecret(res.data.session_data.api_secret)
          }
        }
        userInfo.value = {
          id: res.data.user_info.id ?? 0,
          email: res.data.user_info.email ?? '',
          username: res.data.user_info.username ?? '',
          nickname: res.data.user_info.nickname,
          avatar_url: res.data.user_info.avatar_url,
          is_active: res.data.user_info.is_active ?? false,
          created_at: '',
          updated_at: '',
        }
        return { success: true }
      }
      return { success: false, message: res.data?.message || res.message || '注册失败' }
    } catch (err: any) {
      return { success: false, message: err?.message || '注册失败' }
    }
  }

  async function fetchUserInfo() {
    if (!apiKey.value) return
    try {
      const res = await getUserInfo()
      if (res.success && res.data) {
        userInfo.value = {
          id: res.data.id ?? 0,
          email: res.data.email ?? '',
          username: res.data.username ?? '',
          nickname: res.data.nickname,
          avatar_url: res.data.avatar_url,
          is_active: res.data.is_active ?? false,
          created_at: res.data.created_at || '',
          updated_at: res.data.updated_at || '',
        }
      }
    } catch {
      // ignore
    }
  }

  function logout() {
    userInfo.value = null
    apiKey.value = null
    apiSecret.value = null
    removeApiKey()
  }

  // 初始化时尝试拉取用户信息
  if (apiKey.value) fetchUserInfo()

  return {
    userInfo,
    apiKey,
    isLoggedIn,
    username,
    balance,
    login,
    register,
    logout,
    fetchUserInfo,
  }
})