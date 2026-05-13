import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { register as apiRegister, getUserInfo, login as apiLogin, type LoginResponse, type RegisterResponse, type UserInfo as ApiUserInfo } from '@/api/user'
import { setApiKey, getApiKey, removeApiKey, setUserId, getUserId, removeUserId } from '@/utils/storage'

export interface LocalUserInfo {
  id: number
  email: string
  username: string
  nickname: string
  avatar_url: string | null
  is_active: boolean
}

function mapUserInfo(u: Partial<ApiUserInfo>): LocalUserInfo {
  return {
    id: u.id ?? 0,
    email: u.email ?? '',
    username: u.username ?? '',
    nickname: u.nickname ?? '',
    avatar_url: u.avatar_url ?? null,
    is_active: u.is_active ?? false,
  }
}

export const useUserStore = defineStore('user', () => {
  const userInfo = ref<LocalUserInfo | null>(null)
  const apiKey = ref<string | null>(getApiKey())
  const userId = ref<string | null>(getUserId())

  const isLoggedIn = computed(() => !!apiKey.value)
  const username = computed(() => userInfo.value?.nickname || userInfo.value?.username || '未登录')

  async function login(loginStr: string, passwordOrCode: string, loginType: 'password' | 'code') {
    try {
      const res = await apiLogin({
        login_type: loginType,
        username: loginType === 'password' ? loginStr : undefined,
        email: loginType === 'code' ? loginStr : undefined,
        password: loginType === 'password' ? passwordOrCode : undefined,
        verification_code: loginType === 'code' ? passwordOrCode : undefined,
      })
      const loginData: LoginResponse | undefined = res.data
      if (res.code === 0 && loginData) {
        if (loginData.api_key && loginData.user?.id) {
          apiKey.value = loginData.api_key
          setApiKey(loginData.api_key)
          userId.value = String(loginData.user.id)
          setUserId(String(loginData.user.id))
        }
        if (loginData.user) {
          userInfo.value = mapUserInfo(loginData.user)
        }
        return { code: 0, msg: '登录成功' }
      }
      return { code: -1, msg: res.msg || '登录失败' }
    } catch (err: any) {
      return { code: -1, msg: err?.msg || err?.message || '登录失败' }
    }
  }

  async function register(email: string, password: string, username: string, nickname?: string) {
    try {
      const res = await apiRegister({ email, password, username, nickname: nickname || username })
      const regData: RegisterResponse | undefined = res.data
      if (res.code === 0 && regData?.user) {
        userInfo.value = {
          id: regData.user.id ?? 0,
          email: regData.user.email ?? '',
          username: regData.user.username ?? '',
          nickname: regData.user.nickname ?? '',
          avatar_url: null,
          is_active: true,
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
        userInfo.value = mapUserInfo(res.data)
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
    login,
    register,
    logout,
    fetchUserInfo,
  }
})
