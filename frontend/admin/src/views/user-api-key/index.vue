<template>
  <div class="page-container">

    <el-card>
      <template #header>
        <div class="card-header">
          <span>用户 API Key</span>
          <div class="header-right">
            <el-select v-model="selectedUserId" placeholder="选择用户" clearable filterable style="width: 200px" @change="fetchUserKeys">
              <el-option
                v-for="u in users"
                :key="u.id"
                :label="`${u.username} (ID: ${u.id})`"
                :value="u.id"
              />
            </el-select>
            <el-button type="primary" @click="showCreateDialog = true" :disabled="!selectedUserId">
              <el-icon><Plus /></el-icon> 创建 Key
            </el-button>
          </div>
        </div>
      </template>

      <el-empty v-if="!selectedUserId" description="请先选择用户" />
      <el-table v-else :data="keys" v-loading="loading" stripe>
        <el-table-column prop="name" label="名称" min-width="140" />
        <el-table-column label="API Key" min-width="300">
          <template #default="{ row }">
            <code class="api-key">{{ row._masked ? maskKey(row.api_key) : row.api_key }}</code>
            <el-button size="small" link @click="row._masked = !row._masked">
              <el-icon><View v-if="row._masked" /><Hide v-else /></el-icon>
            </el-button>
            <el-button size="small" link type="primary" @click="copyKey(row.api_key)">
              <el-icon><CopyDocument /></el-icon>
            </el-button>
          </template>
        </el-table-column>
        <el-table-column prop="permissions" label="权限" min-width="140">
          <template #default="{ row }">
            <el-tag v-for="p in row.permissions" :key="p" size="small" class="mr-1">{{ p }}</el-tag>
            <span v-if="!row.permissions?.length" class="text-muted">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="rate_limit" label="限速" width="100" align="center">
          <template #default="{ row }">{{ row.rate_limit ?? '-' }}</template>
        </el-table-column>
        <el-table-column prop="is_active" label="状态" width="80" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="row.is_active ? 'success' : 'info'">
              {{ row.is_active ? '启用' : '停用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="expires_at" label="过期时间" width="160">
          <template #default="{ row }">
            <span v-if="row.expires_at">{{ formatDate(row.expires_at) }}</span>
            <el-tag v-else type="success" size="small">永不过期</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="140" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" link @click="handleToggle(row)">
              {{ row.is_active ? '停用' : '启用' }}
            </el-button>
            <el-button type="danger" size="small" link @click="handleDelete(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 创建对话框 -->
    <el-dialog v-model="showCreateDialog" title="创建用户 API Key" width="460px">
      <el-form :model="createForm" label-width="90px">
        <el-form-item label="名称">
          <el-input v-model="createForm.name" placeholder="Key 名称" />
        </el-form-item>
        <el-form-item label="权限">
          <el-select v-model="createForm.permissions" multiple style="width:100%">
            <el-option label="全部" value="all" />
          </el-select>
        </el-form-item>
        <el-form-item label="限速">
          <el-input-number v-model="createForm.rate_limit" :min="10" :max="1000" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" :loading="creating" @click="handleCreate">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Plus, View, Hide, CopyDocument } from '@element-plus/icons-vue'
import { listUsers, listUserApiKeys, createUserApiKey, deleteUserApiKey, toggleUserApiKey, type UserApiKey } from '@/api/user'

const loading = ref(false)
const creating = ref(false)
const selectedUserId = ref<number | null>(null)
const showCreateDialog = ref(false)
const users = ref<any[]>([])
const keys = ref<UserApiKey[]>([])

const createForm = reactive({ name: '', permissions: [] as string[], rate_limit: 100 })

function maskKey(k: string) {
  if (k.length < 12) return '*'.repeat(k.length)
  return k.slice(0, 6) + '*'.repeat(k.length - 12) + k.slice(-6)
}

function formatDate(d: string) {
  return new Date(d).toLocaleString('zh-CN')
}

async function fetchUsers() {
  try {
    const res = await listUsers()
    if (res.code === 0) {
      users.value = res.data?.users || res.data?.items || []
    }
  } catch { /* ignore */ }
}

async function fetchUserKeys() {
  if (!selectedUserId.value) { keys.value = []; return }
  loading.value = true
  try {
    const res = await listUserApiKeys(selectedUserId.value)
    if (res.code === 0) {
      keys.value = (res.data?.items || []).map(k => ({ ...k, _masked: true }))
    } else {
      ElMessage.error(res.msg || '获取 API Key 列表失败')
      keys.value = []
    }
  } catch {
    ElMessage.error('获取 API Key 列表失败')
    keys.value = []
  } finally {
    loading.value = false
  }
}

function copyKey(key: string) {
  navigator.clipboard.writeText(key).then(() => ElMessage.success('已复制')).catch(() => ElMessage.error('复制失败'))
}

async function handleCreate() {
  if (!createForm.name.trim()) { ElMessage.warning('请输入名称'); return }
  creating.value = true
  try {
    const res = await createUserApiKey(selectedUserId.value!, {
      name: createForm.name,
      permissions: createForm.permissions.length ? createForm.permissions : ['read'],
      rate_limit: createForm.rate_limit,
    })
    if (res.code === 0) {
      ElMessage.success('API Key 创建成功')
      showCreateDialog.value = false
      createForm.name = ''
      createForm.permissions = []
      createForm.rate_limit = 100
      await fetchUserKeys()
    } else {
      ElMessage.error(res.msg || '创建失败')
    }
  } catch {
    ElMessage.error('创建失败')
  } finally {
    creating.value = false
  }
}

async function handleDelete(row: UserApiKey) {
  try {
    await ElMessageBox.confirm(`确定要删除 API Key「${row.name}」吗？此操作不可恢复。`, '确认删除', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    })
    const res = await deleteUserApiKey(selectedUserId.value!, row.id)
    if (res.code === 0) {
      ElMessage.success('删除成功')
      await fetchUserKeys()
    } else {
      ElMessage.error(res.msg || '删除失败')
    }
  } catch { /* cancelled */ }
}

async function handleToggle(row: UserApiKey) {
  try {
    const res = await toggleUserApiKey(selectedUserId.value!, row.id)
    if (res.code === 0) {
      row.is_active = res.data?.is_active ?? !row.is_active
      ElMessage.success(row.is_active ? '已启用' : '已停用')
    } else {
      ElMessage.error(res.msg || '切换失败')
    }
  } catch {
    ElMessage.error('切换失败')
  }
}

onMounted(() => fetchUsers())
</script>

<style scoped>
.page-container { padding: 20px; }
.mb-16 { margin-bottom: 16px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
.header-right { display: flex; gap: 8px; align-items: center; }
.api-key { font-family: monospace; font-size: 12px; background: #f5f5f5; padding: 2px 6px; border-radius: 4px; }
.mr-1 { margin-right: 4px; }
.text-muted { color: #999; }
</style>
