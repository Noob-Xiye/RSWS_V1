<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>用户管理</span>
          <el-button type="primary" size="small">刷新</el-button>
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
            <el-button type="primary" size="small" link @click="handleView(row)">查看</el-button>
            <el-button type="danger" size="small" link @click="handleDisable(row)">禁用</el-button>
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
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { User } from '@/api/user'
import { listUsers, deactivateUser } from '@/api/user'

const loading = ref(false)
const users = ref<User[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)

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
    if (res.success && res.data) {
      users.value = res.data.items
      total.value = res.data.total
    }
  } catch {
    // 使用模拟数据
    users.value = [
      { id: 1, email: 'user1@example.com', username: 'user1', balance: '100.00', is_active: true, created_at: '2026-05-01T10:00:00Z' },
      { id: 2, email: 'user2@example.com', username: 'user2', balance: '50.00', is_active: true, created_at: '2026-05-02T12:00:00Z' }
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
  ElMessage.info(`查看用户: ${row.email}`)
}

async function handleDisable(row: User) {
  try {
    await ElMessageBox.confirm(`确定要禁用用户 ${row.email} 吗？`, '确认', { type: 'warning' })
    await deactivateUser(row.id)
    ElMessage.success('已禁用')
    fetchUsers()
  } catch {
    // 用户取消
  }
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
</style>