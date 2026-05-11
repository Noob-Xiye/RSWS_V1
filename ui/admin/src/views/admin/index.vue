<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>管理员管理</span>
          <el-button type="primary" size="small" @click="showCreate">新建管理员</el-button>
        </div>
      </template>

      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="角色">
          <el-select v-model="searchForm.role" placeholder="全部" clearable>
            <el-option label="超级管理员" value="super_admin" />
            <el-option label="管理员" value="admin" />
            <el-option label="运营" value="operator" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="searchForm.is_active" placeholder="全部" clearable>
            <el-option label="启用" :value="true" />
            <el-option label="停用" :value="false" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>

      <el-table :data="admins" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="username" label="用户名" width="150" />
        <el-table-column prop="email" label="邮箱" min-width="200" />
        <el-table-column prop="role" label="角色" width="120">
          <template #default="{ row }">
            <el-tag :type="getRoleType(row.role)">{{ getRoleText(row.role) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="is_active" label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.is_active ? 'success' : 'danger'">{{ row.is_active ? '启用' : '停用' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="last_login" label="最后登录" width="180">
          <template #default="{ row }">{{ row.last_login ? formatDate(row.last_login) : '从未登录' }}</template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="180">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="200">
          <template #default="{ row }">
            <el-button v-if="row.is_active" type="warning" size="small" link @click="handleDeactivate(row)">停用</el-button>
            <el-button v-else type="success" size="small" link @click="handleActivate(row)">启用</el-button>
            <el-button type="primary" size="small" link @click="handleResetPwd(row)">重置密码</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination">
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="pageSize"
          :total="total"
          :page-sizes="[10, 20, 50]"
          layout="total, sizes, prev, pager, next"
          @size-change="fetchAdmins"
          @current-change="fetchAdmins"
        />
      </div>
    </el-card>

    <!-- 创建管理员对话框 -->
    <el-dialog v-model="createVisible" title="新建管理员" width="450px">
      <el-form :model="createForm" :rules="createRules" ref="createFormRef" label-width="90px">
        <el-form-item label="用户名" prop="username">
          <el-input v-model="createForm.username" placeholder="管理员用户名" />
        </el-form-item>
        <el-form-item label="邮箱" prop="email">
          <el-input v-model="createForm.email" placeholder="管理员邮箱" />
        </el-form-item>
        <el-form-item label="密码" prop="password">
          <el-input v-model="createForm.password" type="password" show-password placeholder="初始密码" />
        </el-form-item>
        <el-form-item label="角色" prop="role">
          <el-select v-model="createForm.role" placeholder="选择角色">
            <el-option label="超级管理员" value="super_admin" />
            <el-option label="管理员" value="admin" />
            <el-option label="运营" value="operator" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="handleCreate">创建</el-button>
      </template>
    </el-dialog>

    <!-- 重置密码对话框 -->
    <el-dialog v-model="resetPwdVisible" title="重置密码" width="400px">
      <el-form :model="resetPwdForm" :rules="resetPwdRules" ref="resetPwdFormRef" label-width="80px">
        <el-form-item label="新密码" prop="password">
          <el-input v-model="resetPwdForm.password" type="password" show-password placeholder="输入新密码" />
        </el-form-item>
        <el-form-item label="确认密码" prop="confirmPassword">
          <el-input v-model="resetPwdForm.confirmPassword" type="password" show-password placeholder="再次输入新密码" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="resetPwdVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="handleResetPwdSubmit">确认重置</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import type { AdminInfo } from '@/api/admin'
import { listAdmins, createAdmin, deactivateAdmin, activateAdmin, resetAdminPassword } from '@/api/admin'

const loading = ref(false)
const submitting = ref(false)
const admins = ref<AdminInfo[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)

const searchForm = reactive({
  role: '',
  is_active: undefined as boolean | undefined
})

// 创建表单
const createVisible = ref(false)
const createFormRef = ref<FormInstance>()
const createForm = reactive({
  username: '',
  email: '',
  password: '',
  role: 'operator'
})
const createRules: FormRules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码长度不能少于6位', trigger: 'blur' }
  ],
  role: [{ required: true, message: '请选择角色', trigger: 'change' }]
}

// 重置密码表单
const resetPwdVisible = ref(false)
const resetPwdFormRef = ref<FormInstance>()
const resetPwdForm = reactive({
  adminId: 0,
  password: '',
  confirmPassword: ''
})
const resetPwdRules: FormRules = {
  password: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { min: 6, message: '密码长度不能少于6位', trigger: 'blur' }
  ],
  confirmPassword: [
    { required: true, message: '请再次输入密码', trigger: 'blur' },
    {
      validator: (_rule, value, callback) => {
        if (value !== resetPwdForm.password) {
          callback(new Error('两次输入的密码不一致'))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ]
}

function getRoleType(role: string) {
  const map: Record<string, string> = { super_admin: 'danger', admin: 'warning', operator: 'info' }
  return map[role] || 'info'
}

function getRoleText(role: string) {
  const map: Record<string, string> = { super_admin: '超级管理员', admin: '管理员', operator: '运营' }
  return map[role] || role
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

async function fetchAdmins() {
  loading.value = true
  try {
    const params: Record<string, unknown> = { page: page.value, page_size: pageSize.value }
    if (searchForm.role) params.role = searchForm.role
    if (searchForm.is_active !== undefined) params.is_active = searchForm.is_active
    const res = await listAdmins(params)
    if (res.code === 0 && res.data) {
      const data = res.data as { items: AdminInfo[]; total: number; page: number; page_size: number }
      admins.value = data.items
      total.value = data.total
    }
  } catch {
    admins.value = []
    total.value = 0
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchAdmins()
}

function handleReset() {
  searchForm.role = ''
  searchForm.is_active = undefined
  handleSearch()
}

function showCreate() {
  createForm.username = ''
  createForm.email = ''
  createForm.password = ''
  createForm.role = 'operator'
  createVisible.value = true
}

async function handleCreate() {
  if (!createFormRef.value) return
  await createFormRef.value.validate(async (valid) => {
    if (!valid) return
    submitting.value = true
    try {
      await createAdmin(createForm)
      ElMessage.success('创建成功')
      createVisible.value = false
      fetchAdmins()
    } catch {
      ElMessage.error('创建失败')
    } finally {
      submitting.value = false
    }
  })
}

async function handleDeactivate(row: AdminInfo) {
  try {
    await ElMessageBox.confirm(`确定停用管理员 "${row.username}" 吗？`, '确认', { type: 'warning' })
    await deactivateAdmin(row.id)
    ElMessage.success('已停用')
    fetchAdmins()
  } catch {}
}

async function handleActivate(row: AdminInfo) {
  try {
    await activateAdmin(row.id)
    ElMessage.success('已启用')
    fetchAdmins()
  } catch {}
}

function handleResetPwd(row: AdminInfo) {
  resetPwdForm.adminId = row.id
  resetPwdForm.password = ''
  resetPwdForm.confirmPassword = ''
  resetPwdVisible.value = true
}

async function handleResetPwdSubmit() {
  if (!resetPwdFormRef.value) return
  await resetPwdFormRef.value.validate(async (valid) => {
    if (!valid) return
    submitting.value = true
    try {
      await resetAdminPassword(resetPwdForm.adminId, resetPwdForm.password)
      ElMessage.success('密码已重置')
      resetPwdVisible.value = false
    } catch {
      ElMessage.error('重置失败')
    } finally {
      submitting.value = false
    }
  })
}

onMounted(() => fetchAdmins())
</script>

<style scoped>
.page-container { padding: 20px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
.search-form { margin-bottom: 20px; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
</style>