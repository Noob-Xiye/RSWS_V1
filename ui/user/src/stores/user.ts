import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { register as apiRegister, getUserInfo, login as apiLogin, type LoginResponse, type RegisterResponse } from '@/api/user'
import { setApiKey, getApiKey, removeApiKey, setUserId, getUserId, removeUserId } from '@/utils/storage'

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
  const userId = ref<string | null>(getUserId())

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
      // 后端 ApiResponse 格式: { code: 0, msg: "success", data: LoginResponse }
      const loginData = res.data
      if (res.code === 0 && loginData) {
        if (loginData.api_key && loginData.user?.id) {
          apiKey.value = loginData.api_key
          setApiKey(loginData.api_key)
          userId.value = String(loginData.user.id)
          setUserId(String(loginData.user.id))
        }
        if (loginData.user) {
          userInfo.value = {
            id: loginData.user.id ?? 0,
            email: loginData.user.email ?? '',
            username: loginData.user.username ?? '',
            nickname: loginData.user.nickname,
            avatar_url: loginData.user.avatar_url,
            is_active: loginData.user.is_active ?? false,
            created_at: '',
            updated_at: '',
          }
        }
        return { code: 0, msg: '登录成功' }
      }
      return { code: -1, msg: res.msg || '登录失败' }
    } catch (err: any) {
      return { code: -1, msg: err?.msg || err?.message || '登录失败' }
    }
  }

  async function register(email: string, password: string, username: string) {
    try {
      const res = await apiRegister({ email, password, username })
      // 后端 ApiResponse 格式: { code: 0, msg: "success", data: RegisterResponse }
      const regData = res.data
      if (res.code === 0 && regData) {
        // 注册不返回 api_key，前端只存 userInfo
        if (regData.user) {
          userInfo.value = {
            id: regData.user.id ?? 0,
            email: regData.user.email ?? '',
            username: regData.user.username ?? '',
            nickname: regData.user.nickname,
            avatar_url: regData.user.avatar_url,
            is_active: regData.user.is_active ?? false,
            created_at: '',
            updated_at: '',
          }
        }
        return { code: 0, msg: '注册成功' }
      }
      return { code: -1, msg: res.msg || '注册失败' }
    } catch (err: any) {
      return { code: -1, msg: err?.msg || err?.message || '注册失败' }
    }
  }

  async function fetchUserInfo() {
    if (!apiKey.value) return
    try {
      const res = await getUserInfo()
      if (res.code === 0 && res.data) {
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
    userId.value = null
    removeApiKey()
    removeUserId()
  }

  // 初始化时尝试拉取用户信息
  if (apiKey.value) fetchUserInfo()

  return {
    userInfo,
    apiKey,
    userId,
    isLoggedIn,
    username,
    balance,
    login,
    register,
    logout,
    fetchUserInfo,
  }
})
