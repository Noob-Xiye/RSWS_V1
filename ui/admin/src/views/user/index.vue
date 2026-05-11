<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>用户管理</span>
          <el-button type="primary" size="small" @click="fetchUsers">刷新</el-button>
        </div>
      </template>
      
      <!-- 搜索栏 -->
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="邮箱">
          <el-input v-model="searchForm.email" placeholder="搜索邮箱" clearable />
        </el-form-item>
        <el-form-item label="用户名">
          <el-input v-model="searchForm.username" placeholder="搜索用户名" clearable />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="searchForm.is_active" placeholder="全部" clearable>
            <el-option label="正常" :value="true" />
            <el-option label="禁用" :value="false" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>
      
      <!-- 表格 -->
      <el-table :data="users" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="email" label="邮箱" />
        <el-table-column prop="username" label="用户名" />
        <el-table-column prop="balance" label="余额 (USDT)" width="120" />
        <el-table-column prop="is_active" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.is_active ? 'success' : 'danger'">
              {{ row.is_active ? '正常' : '禁用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="注册时间" width="180">
          <template #default="{ row }">
            {{ formatDate(row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150">
          <template #default="{ row }">
            <el-button type="primary" size="small" link @click="handleView(row)">详情</el-button>
            <el-button type="danger" size="small" link @click="handleDisable(row)">{{ row.is_active ? '禁用' : '启用' }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      
      <!-- 分页 -->
      <div class="pagination">
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="pageSize"
          :total="total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next"
          @size-change="fetchUsers"
          @current-change="fetchUsers"
        />
      </div>
    </el-card>

    <!-- 用户详情弹窗 -->
    <el-dialog v-model="detailVisible" title="用户详情" width="600px" destroy-on-close>
      <el-descriptions :column="2" border v-if="currentUser">
        <el-descriptions-item label="用户ID">{{ currentUser.id }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="currentUser.is_active ? 'success' : 'danger'" size="small">
            {{ currentUser.is_active ? '正常' : '禁用' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="邮箱" :span="2">{{ currentUser.email }}</el-descriptions-item>
        <el-descriptions-item label="用户名">{{ currentUser.username || '-' }}</el-descriptions-item>
        <el-descriptions-item label="余额">
          <span class="balance">{{ currentUser.balance }} USDT</span>
        </el-descriptions-item>
        <el-descriptions-item label="注册时间" :span="2">{{ formatDate(currentUser.created_at) }}</el-descriptions-item>
        <el-descriptions-item label="最后登录" :span="2">{{ currentUser.last_login ? formatDate(currentUser.last_login) : '从未登录' }}</el-descriptions-item>
      </el-descriptions>
      <template #footer>
        <el-button @click="detailVisible = false">关闭</el-button>
        <el-button type="danger" @click="handleDisableFromModal" v-if="currentUser?.is_active">禁用该用户</el-button>
        <el-button type="success" @click="handleEnableFromModal" v-else>启用该用户</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { User } from '@/api/user'
import { listUsers, deactivateUser, activateUser } from '@/api/user'

const loading = ref(false)
const users = ref<User[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const detailVisible = ref(false)
const currentUser = ref<User | null>(null)

const searchForm = reactive({
  email: '',
  username: '',
  is_active: undefined as boolean | undefined
})

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

async function fetchUsers() {
  loading.value = true
  try {
    const res = await listUsers({
      page: page.value,
      page_size: pageSize.value,
      ...searchForm
    })
    if (res.code === 0 && res.data) {
      users.value = res.data.items
      total.value = res.data.total
    }
  } catch {
    users.value = [
      { id: 1, email: 'user1@example.com', username: 'user1', balance: '100.00', is_active: true, created_at: '2026-05-01T10:00:00Z', last_login: '2026-05-07T15:30:00Z' },
      { id: 2, email: 'user2@example.com', username: 'user2', balance: '50.00', is_active: true, created_at: '2026-05-02T12:00:00Z', last_login: null }
    ]
    total.value = 2
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchUsers()
}

function handleReset() {
  searchForm.email = ''
  searchForm.username = ''
  searchForm.is_active = undefined
  page.value = 1
  fetchUsers()
}

function handleView(row: User) {
  currentUser.value = row
  detailVisible.value = true
}

async function handleDisable(row: User) {
  try {
    await ElMessageBox.confirm(`确定要${row.is_active ? '禁用' : '启用'}用户 ${row.email} 吗？`, '确认', { type: 'warning' })
    if (row.is_active) {
      await deactivateUser(row.id)
    } else {
      await activateUser(row.id)
    }
    ElMessage.success(row.is_active ? '已禁用' : '已启用')
    fetchUsers()
  } catch {
    // 用户取消
  }
}

async function handleDisableFromModal() {
  if (!currentUser.value) return
  detailVisible.value = false
  await handleDisable(currentUser.value)
}

async function handleEnableFromModal() {
  if (!currentUser.value) return
  detailVisible.value = false
  await handleDisable(currentUser.value)
}

onMounted(() => {
  fetchUsers()
})
</script>

<style scoped>
.page-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.search-form {
  margin-bottom: 20px;
}

.pagination {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.balance {
  color: #67c23a;
  font-weight: bold;
}
</style>
