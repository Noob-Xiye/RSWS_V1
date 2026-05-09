<template>
  <div class="user-page">
    <el-container>
      <el-header class="header">
        <div class="logo" @click="$router.push('/')">RSWS</div>
        <el-menu mode="horizontal" router>
          <el-menu-item index="/">首页</el-menu-item>
          <el-menu-item index="/orders">我的订单</el-menu-item>
        </el-menu>
        <div class="user-area">
          <el-dropdown>
            <span class="user-link">
              <el-icon><User /></el-icon>
              {{ userStore.username }}
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="$router.push('/user')">用户中心</el-dropdown-item>
                <el-dropdown-item @click="handleLogout">退出</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-header>
      <el-main class="main">
        <el-row :gutter="20">
          <el-col :span="6">
            <el-card class="profile-card">
              <div class="avatar-section">
                <el-avatar :size="80" :src="userStore.userInfo?.avatar_url || undefined">
                  <el-icon :size="40"><User /></el-icon>
                </el-avatar>
                <h3>{{ userStore.userInfo?.nickname || userStore.userInfo?.username }}</h3>
                <p class="email">{{ userStore.userInfo?.email }}</p>
              </div>
              <el-menu :default-active="activeMenu" @select="handleMenuSelect">
                <el-menu-item index="profile">
                  <el-icon><User /></el-icon>
                  <span>个人资料</span>
                </el-menu-item>
                <el-menu-item index="password">
                  <el-icon><Lock /></el-icon>
                  <span>修改密码</span>
                </el-menu-item>
                <el-menu-item index="orders">
                  <el-icon><Document /></el-icon>
                  <span>我的订单</span>
                </el-menu-item>
              </el-menu>
            </el-card>
          </el-col>
          <el-col :span="18">
            <el-card v-if="activeMenu === 'profile'">
              <template #header>
                <span>个人资料</span>
              </template>
              <el-form :model="profileForm" label-width="100px" :rules="profileRules" ref="profileFormRef">
                <el-form-item label="用户名">
                  <el-input :value="userStore.userInfo?.username" disabled />
                </el-form-item>
                <el-form-item label="邮箱">
                  <el-input :value="userStore.userInfo?.email" disabled />
                </el-form-item>
                <el-form-item label="昵称" prop="nickname">
                  <el-input v-model="profileForm.nickname" placeholder="请输入昵称" />
                </el-form-item>
                <el-form-item label="头像">
                  <el-upload
                    class="avatar-uploader"
                    action="#"
                    :show-file-list="false"
                    :auto-upload="false"
                    @change="handleAvatarChange"
                  >
                    <el-avatar :size="100" :src="profileForm.avatar_url || undefined">
                      <el-icon :size="40"><Plus /></el-icon>
                    </el-avatar>
                  </el-upload>
                  <div class="avatar-tip">点击更换头像</div>
                </el-form-item>
                <el-form-item>
                  <el-button type="primary" :loading="savingProfile" @click="handleSaveProfile">保存修改</el-button>
                </el-form-item>
              </el-form>
            </el-card>

            <el-card v-else-if="activeMenu === 'password'">
              <template #header>
                <span>修改密码</span>
              </template>
              <el-form :model="passwordForm" label-width="100px" :rules="passwordRules" ref="passwordFormRef">
                <el-form-item label="当前密码" prop="old_password">
                  <el-input v-model="passwordForm.old_password" type="password" show-password placeholder="请输入当前密码" />
                </el-form-item>
                <el-form-item label="新密码" prop="new_password">
                  <el-input v-model="passwordForm.new_password" type="password" show-password placeholder="请输入新密码" />
                </el-form-item>
                <el-form-item label="确认密码" prop="confirm_password">
                  <el-input v-model="passwordForm.confirm_password" type="password" show-password placeholder="请再次输入新密码" />
                </el-form-item>
                <el-form-item>
                  <el-button type="primary" :loading="savingPassword" @click="handleChangePassword">修改密码</el-button>
                </el-form-item>
              </el-form>
            </el-card>

            <el-card v-else-if="activeMenu === 'orders'">
              <template #header>
                <div class="card-header">
                  <span>我的订单</span>
                  <el-button type="primary" link @click="$router.push('/orders')">查看全部</el-button>
                </div>
              </template>
              <el-table :data="recentOrders" v-loading="loadingOrders" stripe>
                <el-table-column prop="order_no" label="订单号" width="180" />
                <el-table-column prop="resource_title" label="资源" min-width="150" />
                <el-table-column prop="amount" label="金额" width="100">
                  <template #default="{ row }">{{ row.amount }} USDT</template>
                </el-table-column>
                <el-table-column prop="status" label="状态" width="100">
                  <template #default="{ row }">
                    <el-tag :type="getStatusType(row.status)">{{ getStatusText(row.status) }}</el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="created_at" label="创建时间" width="160">
                  <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
                </el-table-column>
              </el-table>
              <el-empty v-if="!loadingOrders && recentOrders.length === 0" description="暂无订单" />
            </el-card>
          </el-col>
        </el-row>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules, UploadFile } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { updateProfile, changePassword } from '@/api/user'
import { listOrders, type Order } from '@/api/order'

const router = useRouter()
const userStore = useUserStore()

const activeMenu = ref('profile')
const savingProfile = ref(false)
const savingPassword = ref(false)
const loadingOrders = ref(false)
const recentOrders = ref<Order[]>([])

const profileFormRef = ref<FormInstance>()
const passwordFormRef = ref<FormInstance>()

const profileForm = reactive({
  nickname: userStore.userInfo?.nickname || '',
  avatar_url: userStore.userInfo?.avatar_url || ''
})

const passwordForm = reactive({
  old_password: '',
  new_password: '',
  confirm_password: ''
})

const profileRules: FormRules = {
  nickname: [
    { required: true, message: '请输入昵称', trigger: 'blur' },
    { min: 2, max: 20, message: '昵称长度 2-20 个字符', trigger: 'blur' }
  ]
}

const passwordRules: FormRules = {
  old_password: [
    { required: true, message: '请输入当前密码', trigger: 'blur' }
  ],
  new_password: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { min: 6, max: 20, message: '密码长度 6-20 个字符', trigger: 'blur' }
  ],
  confirm_password: [
    { required: true, message: '请确认新密码', trigger: 'blur' },
    {
      validator: (_rule, value, callback) => {
        if (value !== passwordForm.new_password) {
          callback(new Error('两次输入的密码不一致'))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ]
}

function handleMenuSelect(key: string) {
  activeMenu.value = key
  if (key === 'orders') {
    fetchRecentOrders()
  }
}

function handleLogout() {
  userStore.logout()
  router.push('/')
}

function handleAvatarChange(file: UploadFile) {
  // TODO: 上传头像到服务器
  if (file.raw) {
    const url = URL.createObjectURL(file.raw)
    profileForm.avatar_url = url
    ElMessage.info('头像上传功能开发中')
  }
}

async function handleSaveProfile() {
  const valid = await profileFormRef.value?.validate().catch(() => false)
  if (!valid) return
  
  savingProfile.value = true
  try {
    const res = await updateProfile({
      nickname: profileForm.nickname,
      avatar_url: profileForm.avatar_url || undefined
    })
    if (res.success) {
      ElMessage.success('保存成功')
      await userStore.fetchUserInfo()
    } else {
      ElMessage.error(res.message || '保存失败')
    }
  } catch {
    ElMessage.error('保存失败')
  } finally {
    savingProfile.value = false
  }
}

async function handleChangePassword() {
  const valid = await passwordFormRef.value?.validate().catch(() => false)
  if (!valid) return
  
  savingPassword.value = true
  try {
    const res = await changePassword({
      old_password: passwordForm.old_password,
      new_password: passwordForm.new_password
    })
    if (res.success) {
      ElMessage.success('密码修改成功')
      passwordForm.old_password = ''
      passwordForm.new_password = ''
      passwordForm.confirm_password = ''
    } else {
      ElMessage.error(res.message || '修改失败')
    }
  } catch {
    ElMessage.error('修改失败')
  } finally {
    savingPassword.value = false
  }
}

async function fetchRecentOrders() {
  loadingOrders.value = true
  try {
    const res = await listOrders({ page: 1, limit: 5 })
    if (res.success && res.data) {
      recentOrders.value = res.data.items
    }
  } catch {
    recentOrders.value = []
  } finally {
    loadingOrders.value = false
  }
}

function getStatusType(status: string) {
  const map: Record<string, '' | 'success' | 'warning' | 'info' | 'danger'> = {
    pending: 'warning',
    paid: '',
    completed: 'success',
    cancelled: 'info',
    failed: 'danger'
  }
  return map[status] || 'info'
}

function getStatusText(status: string) {
  const map: Record<string, string> = {
    pending: '待支付',
    paid: '已支付',
    completed: '已完成',
    cancelled: '已取消',
    failed: '失败'
  }
  return map[status] || status
}

function formatDate(dateStr: string) {
  if (!dateStr) return ''
  return dateStr.substring(0, 19).replace('T', ' ')
}

onMounted(() => {
  if (!userStore.isLoggedIn) {
    router.push('/login')
    return
  }
  profileForm.nickname = userStore.userInfo?.nickname || ''
  profileForm.avatar_url = userStore.userInfo?.avatar_url || ''
})
</script>

<style scoped>
.user-page {
  min-height: 100vh;
  background: #f5f5f5;
}
.header {
  background: #fff;
  display: flex;
  align-items: center;
  padding: 0 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
.logo {
  font-size: 24px;
  font-weight: bold;
  color: #409eff;
  cursor: pointer;
  margin-right: 40px;
}
.user-area {
  margin-left: auto;
}
.user-link {
  display: flex;
  align-items: center;
  gap: 5px;
  cursor: pointer;
}
.main {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}
.profile-card {
  text-align: center;
}
.avatar-section {
  padding: 20px 0;
  border-bottom: 1px solid #ebeef5;
  margin-bottom: 10px;
}
.avatar-section h3 {
  margin: 10px 0 5px;
}
.avatar-section .email {
  color: #909399;
  font-size: 12px;
  margin: 0;
}
.profile-card .el-menu {
  border-right: none;
  text-align: left;
}
.avatar-uploader {
  cursor: pointer;
}
.avatar-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 8px;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>