<template>
  <ModernLayout>
    <div class="user-page">
      <div class="user-container">
        <div class="user-grid">
          <!-- 左侧：用户卡片 -->
          <div class="sidebar">
            <div class="profile-card">
              <div class="avatar-section">
                <el-avatar :size="80" :src="userStore.userInfo?.avatar_url || undefined" class="user-avatar">
                  {{ userStore.userInfo?.username?.charAt(0)?.toUpperCase() }}
                </el-avatar>
                <h3 class="user-nickname">{{ userStore.userInfo?.nickname || userStore.userInfo?.username }}</h3>
                <p class="user-email">{{ userStore.userInfo?.email }}</p>
              </div>

              <nav class="sidebar-menu">
                <button class="menu-item" :class="{ active: activeMenu === 'profile' }" @click="activeMenu = 'profile'">
                  <el-icon><User /></el-icon>
                  <span>个人资料</span>
                </button>
                <button class="menu-item" :class="{ active: activeMenu === 'password' }" @click="activeMenu = 'password'">
                  <el-icon><Lock /></el-icon>
                  <span>修改密码</span>
                </button>
                <button class="menu-item" :class="{ active: activeMenu === 'orders' }" @click="activeMenu = 'orders'">
                  <el-icon><Document /></el-icon>
                  <span>我的订单</span>
                </button>
              </nav>
            </div>
          </div>

          <!-- 右侧：内容区 -->
          <div class="content">
            <!-- 个人资料 -->
            <div class="content-card" v-if="activeMenu === 'profile'">
              <h2 class="card-title">个人资料</h2>
              <el-form :model="profileForm" label-width="100px" :rules="profileRules" ref="profileFormRef">
                <el-form-item label="用户名">
                  <el-input :value="userStore.userInfo?.username" disabled class="form-input" />
                </el-form-item>
                <el-form-item label="邮箱">
                  <el-input :value="userStore.userInfo?.email" disabled class="form-input" />
                </el-form-item>
                <el-form-item label="昵称" prop="nickname">
                  <el-input v-model="profileForm.nickname" placeholder="请输入昵称" class="form-input" />
                </el-form-item>
                <el-form-item label="头像">
                  <el-upload action="#" :show-file-list="false" :auto-upload="false" @change="handleAvatarChange">
                    <el-avatar :size="100" :src="profileForm.avatar_url || undefined" class="avatar-upload">
                      <el-icon :size="40"><Plus /></el-icon>
                    </el-avatar>
                  </el-upload>
                  <span class="avatar-tip">点击更换头像</span>
                </el-form-item>
                <el-form-item>
                  <button type="button" class="btn-save" :disabled="savingProfile" @click="handleSaveProfile">
                    {{ savingProfile ? '保存中...' : '保存修改' }}
                  </button>
                </el-form-item>
              </el-form>
            </div>

            <!-- 修改密码 -->
            <div class="content-card" v-else-if="activeMenu === 'password'">
              <h2 class="card-title">修改密码</h2>
              <el-form :model="passwordForm" label-width="100px" :rules="passwordRules" ref="passwordFormRef">
                <el-form-item label="当前密码" prop="old_password">
                  <el-input v-model="passwordForm.old_password" type="password" show-password placeholder="请输入当前密码" class="form-input" />
                </el-form-item>
                <el-form-item label="新密码" prop="new_password">
                  <el-input v-model="passwordForm.new_password" type="password" show-password placeholder="请输入新密码" class="form-input" />
                </el-form-item>
                <el-form-item label="确认密码" prop="confirm_password">
                  <el-input v-model="passwordForm.confirm_password" type="password" show-password placeholder="请再次输入新密码" class="form-input" />
                </el-form-item>
                <el-form-item>
                  <button type="button" class="btn-save" :disabled="savingPassword" @click="handleChangePassword">
                    {{ savingPassword ? '修改中...' : '修改密码' }}
                  </button>
                </el-form-item>
              </el-form>
            </div>

            <!-- 我的订单 -->
            <div class="content-card" v-else-if="activeMenu === 'orders'">
              <div class="card-title-row">
                <h2 class="card-title">我的订单</h2>
                <router-link to="/orders" class="view-all-link">查看全部 →</router-link>
              </div>
              <div v-if="loadingOrders" class="loading-state">
                <el-icon class="is-loading" :size="24"><Loading /></el-icon>
              </div>
              <div v-else-if="recentOrders.length === 0" class="empty-state">
                <el-icon :size="48"><FolderOpened /></el-icon>
                <p>暂无订单</p>
              </div>
              <div v-else class="order-list">
                <div v-for="order in recentOrders" :key="order.id" class="order-item">
                  <div class="order-info">
                    <span class="order-no">#{{ order.id }}</span>
                    <span class="order-resource">{{ order.resource_title }}</span>
                  </div>
                  <div class="order-right">
                    <span class="order-amount">{{ (order.amount / 100).toFixed(2) }} USDT</span>
                    <el-tag :type="getStatusType(order.status)" size="small" effect="dark">{{ getStatusText(order.status) }}</el-tag>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </ModernLayout>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules, UploadFile } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { updateProfile, changePassword } from '@/api/user'
import { listOrders, type Order } from '@/api/order'
import ModernLayout from '@/components/ModernLayout.vue'

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

const passwordForm = reactive({ old_password: '', new_password: '', confirm_password: '' })

const profileRules: FormRules = {
  nickname: [
    { required: true, message: '请输入昵称', trigger: 'blur' },
    { min: 2, max: 20, message: '昵称长度 2-20 个字符', trigger: 'blur' }
  ]
}

const passwordRules: FormRules = {
  old_password: [{ required: true, message: '请输入当前密码', trigger: 'blur' }],
  new_password: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { min: 6, max: 20, message: '密码长度 6-20 个字符', trigger: 'blur' }
  ],
  confirm_password: [
    { required: true, message: '请确认新密码', trigger: 'blur' },
    { validator: (_rule, value, callback) => { value !== passwordForm.new_password ? callback(new Error('两次输入的密码不一致')) : callback() }, trigger: 'blur' }
  ]
}

function handleAvatarChange(file: UploadFile) {
  if (file.raw) {
    profileForm.avatar_url = URL.createObjectURL(file.raw)
    ElMessage.info('头像上传功能开发中')
  }
}

async function handleSaveProfile() {
  const valid = await profileFormRef.value?.validate().catch(() => false)
  if (!valid) return
  savingProfile.value = true
  try {
    const res = await updateProfile({ nickname: profileForm.nickname, avatar_url: profileForm.avatar_url || undefined })
    if (res.code === 0) { ElMessage.success('保存成功'); await userStore.fetchUserInfo() }
    else ElMessage.error(res.msg || '保存失败')
  } catch { ElMessage.error('保存失败') }
  finally { savingProfile.value = false }
}

async function handleChangePassword() {
  const valid = await passwordFormRef.value?.validate().catch(() => false)
  if (!valid) return
  savingPassword.value = true
  try {
    const res = await changePassword({ old_password: passwordForm.old_password, new_password: passwordForm.new_password })
    if (res.code === 0) { ElMessage.success('密码修改成功'); passwordForm.old_password = ''; passwordForm.new_password = ''; passwordForm.confirm_password = '' }
    else ElMessage.error(res.msg || '修改失败')
  } catch { ElMessage.error('修改失败') }
  finally { savingPassword.value = false }
}

async function fetchRecentOrders() {
  loadingOrders.value = true
  try {
    const res = await listOrders({ page: 1, page_size: 5 })
    if (res.code === 0 && res.data) recentOrders.value = res.data.items
  } catch { recentOrders.value = [] }
  finally { loadingOrders.value = false }
}

function getStatusType(status: string) {
  const map: Record<string, '' | 'success' | 'warning' | 'info' | 'danger'> = { pending: 'warning', paid: '', completed: 'success', cancelled: 'info', failed: 'danger' }
  return map[status] || 'info'
}

function getStatusText(status: string) {
  const map: Record<string, string> = { pending: '待支付', paid: '已支付', completed: '已完成', cancelled: '已取消', failed: '失败' }
  return map[status] || status
}

onMounted(() => {
  if (!userStore.isLoggedIn) { router.push('/login'); return }
  profileForm.nickname = userStore.userInfo?.nickname || ''
  profileForm.avatar_url = userStore.userInfo?.avatar_url || ''
})
</script>

<style scoped>
.user-page { padding: 32px 24px; }
.user-container { max-width: 1200px; margin: 0 auto; }
.user-grid { display: grid; grid-template-columns: 280px 1fr; gap: 24px; }

/* 侧栏 */
.profile-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  overflow: hidden;
}

.avatar-section {
  text-align: center;
  padding: 32px 20px 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.user-avatar {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
  font-size: 32px;
  font-weight: 600;
}

.user-nickname {
  margin: 12px 0 4px;
  font-size: 18px;
  font-weight: 600;
}

.user-email {
  color: rgba(255, 255, 255, 0.5);
  font-size: 13px;
  margin: 0;
}

.sidebar-menu {
  padding: 12px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 12px 16px;
  border: none;
  background: transparent;
  border-radius: 10px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}

.menu-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fff;
}

.menu-item.active {
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.2) 0%, rgba(118, 75, 162, 0.2) 100%);
  color: #fff;
}

/* 内容区 */
.content-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 32px;
}

.card-title {
  font-size: 22px;
  font-weight: 700;
  margin: 0 0 28px;
}

.card-title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.card-title-row .card-title { margin-bottom: 0; }

.view-all-link {
  color: #667eea;
  text-decoration: none;
  font-size: 14px;
}

.view-all-link:hover { color: #764ba2; }

/* 表单 */
.form-input :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  box-shadow: none;
}

.form-input :deep(.el-input__inner) { color: #fff; }
.form-input :deep(.el-input__inner::placeholder) { color: rgba(255, 255, 255, 0.4); }

.form-input :deep(.el-input.is-disabled .el-input__wrapper) {
  background: rgba(255, 255, 255, 0.03);
  opacity: 0.7;
}

.avatar-upload {
  background: rgba(255, 255, 255, 0.1);
  cursor: pointer;
  transition: all 0.3s;
}

.avatar-upload:hover { background: rgba(255, 255, 255, 0.15); }

.avatar-tip {
  display: block;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
  margin-top: 8px;
}

.btn-save {
  padding: 12px 32px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  border-radius: 10px;
  color: #fff;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-save:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
}

.btn-save:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

/* 订单列表 */
.loading-state { text-align: center; padding: 40px; color: #667eea; }

.empty-state {
  text-align: center;
  padding: 40px;
  color: rgba(255, 255, 255, 0.5);
}

.empty-state p { margin-top: 12px; }

.order-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.order-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
}

.order-info { display: flex; flex-direction: column; gap: 4px; }
.order-no { font-size: 13px; color: rgba(255, 255, 255, 0.5); }
.order-resource { font-size: 15px; }

.order-right { display: flex; align-items: center; gap: 12px; }
.order-amount { font-weight: 600; color: #f093fb; }

/* 响应式 */
@media (max-width: 768px) {
  .user-grid { grid-template-columns: 1fr; }
  .sidebar-menu { display: flex; flex-direction: row; gap: 8px; overflow-x: auto; }
}
</style>
