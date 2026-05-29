<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>管理员 API Key</span>
          <el-button type="primary" @click="openCreateDialog">
            <el-icon><Plus /></el-icon> 创建 Key
          </el-button>
        </div>
      </template>

      <el-table :data="keys" v-loading="loading" stripe>
        <el-table-column prop="name" label="名称" min-width="140" />
        <el-table-column label="API Key" min-width="320">
          <template #default="{ row }">
            <div class="api-key-cell">
              <code class="api-key">{{ row._masked ? maskKey(row.api_key) : row.api_key }}</code>
              <el-button
                size="small"
                link
                @click="row._masked = !row._masked"
                :title="row._masked ? '显示' : '隐藏'"
              >
                <el-icon><View v-if="row._masked" /><Hide v-else /></el-icon>
              </el-button>
              <el-button size="small" link type="primary" @click="copyKey(row.api_key)" title="复制">
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="permissions" label="权限" min-width="160">
          <template #default="{ row }">
            <el-tag v-for="p in row.permissions" :key="p" size="small" class="mr-1">{{ p }}</el-tag>
            <span v-if="!row.permissions?.length" class="text-gray">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="rate_limit" label="限速 (req/min)" width="130" align="center">
          <template #default="{ row }">{{ row.rate_limit ?? '-' }}</template>
        </el-table-column>
        <el-table-column prop="expires_at" label="过期时间" width="170">
          <template #default="{ row }">
            <span v-if="row.expires_at">{{ formatDate(row.expires_at) }}</span>
            <el-tag v-else type="success" size="small">永不过期</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="is_active" label="状态" width="90" align="center">
          <template #default="{ row }">
            <el-switch
              :model-value="row.is_active"
              @change="(val: boolean) => handleToggle(row, val)"
              :loading="row._toggling"
            />
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="170">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button type="danger" size="small" link @click="handleDelete(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 创建对话框 -->
    <el-dialog v-model="dialogVisible" title="创建 API Key" width="480px" :close-on-click-modal="false">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="90px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" placeholder="例如：生产环境 Key" maxlength="100" show-word-limit />
        </el-form-item>
        <el-form-item label="权限" prop="permissions">
          <el-select v-model="form.permissions" multiple placeholder="选择权限（留空表示全部）" style="width: 100%">
            <el-option label="全部权限" value="all" />
            <el-option label="用户管理" value="user:read" />
            <el-option label="资源管理" value="resource:read" />
            <el-option label="订单管理" value="order:read" />
            <el-option label="管理员管理" value="admin:manage" />
            <el-option label="配置管理" value="config:manage" />
            <el-option label="日志查看" value="log:read" />
          </el-select>
        </el-form-item>
        <el-form-item label="限速" prop="rate_limit">
          <el-input-number v-model="form.rate_limit" :min="10" :max="10000" :step="10" placeholder="请求数/分钟" />
          <span class="form-tip">0 表示使用默认值 100</span>
        </el-form-item>
        <el-form-item label="有效期" prop="expires_in_days">
          <el-input-number v-model="form.expires_in_days" :min="1" :max="3650" placeholder="天数，留空永不过期" />
          <span class="form-tip">留空或 0 表示永不过期</span>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="handleCreate">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { Plus, View, Hide, CopyDocument } from '@element-plus/icons-vue'
import { listApiKeys, createApiKey, deleteApiKey, toggleApiKey } from '@/api/admin'
import type { AdminApiKeyResponse } from '@/api/admin'

interface KeyRow extends AdminApiKeyResponse {
  _masked?: boolean
  _toggling?: boolean
}

const loading = ref(false)
const submitting = ref(false)
const keys = ref<KeyRow[]>([])
const dialogVisible = ref(false)
const formRef = ref<FormInstance>()

const form = reactive({
  name: '',
  permissions: [] as string[],
  rate_limit: 100,
  expires_in_days: 30,
})

const rules: FormRules = {
  name: [
    { required: true, message: '请输入 Key 名称', trigger: 'blur' },
    { max: 100, message: '名称不能超过 100 个字符', trigger: 'blur' },
  ],
}

function maskKey(key: string) {
  if (key.length < 12) return '*'.repeat(key.length)
  return key.slice(0, 6) + '*'.repeat(key.length - 12) + key.slice(-6)
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

async function fetchKeys() {
  loading.value = true
  try {
    const res = await listApiKeys()
    if (res.code === 0) {
      keys.value = (res.data || []).map(k => ({ ...k, _masked: true }))
    } else {
      ElMessage.error(res.msg || '获取失败')
    }
  } catch {
    ElMessage.error('获取 API Key 列表失败')
  } finally {
    loading.value = false
  }
}

function openCreateDialog() {
  form.name = ''
  form.permissions = []
  form.rate_limit = 100
  form.expires_in_days = 30
  dialogVisible.value = true
}

async function handleCreate() {
  if (!formRef.value) return
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    const payload: { name: string; permissions?: string[]; rate_limit?: number; expires_in_days?: number } = {
      name: form.name.trim(),
    }
    if (form.permissions.length > 0) {
      payload.permissions = form.permissions
    }
    if (form.rate_limit > 0) {
      payload.rate_limit = form.rate_limit
    }
    if (form.expires_in_days > 0) {
      payload.expires_in_days = form.expires_in_days
    }

    const res = await createApiKey(payload)
    if (res.code === 0) {
      ElMessage.success('创建成功')
      dialogVisible.value = false
      fetchKeys()
    } else {
      ElMessage.error(res.msg || '创建失败')
    }
  } catch {
    ElMessage.error('创建失败')
  } finally {
    submitting.value = false
  }
}

async function handleToggle(row: KeyRow, val: boolean) {
  row._toggling = true
  try {
    const res = await toggleApiKey(row.id, val)
    if (res.code === 0) {
      ElMessage.success(val ? '已启用' : '已停用')
      row.is_active = val
    } else {
      ElMessage.error(res.msg || '操作失败')
    }
  } catch {
    ElMessage.error('操作失败')
  } finally {
    row._toggling = false
  }
}

async function handleDelete(row: KeyRow) {
  try {
    await ElMessageBox.confirm(
      `确定删除 API Key「${row.name}」吗？删除后不可恢复。`,
      '删除确认',
      { type: 'warning', confirmButtonText: '确定删除', cancelButtonText: '取消' }
    )
  } catch {
    return
  }

  try {
    const res = await deleteApiKey(row.id)
    if (res.code === 0) {
      ElMessage.success('已删除')
      keys.value = keys.value.filter(k => k.id !== row.id)
    } else {
      ElMessage.error(res.msg || '删除失败')
    }
  } catch {
    ElMessage.error('删除失败')
  }
}

function copyKey(key: string) {
  navigator.clipboard.writeText(key).then(() => {
    ElMessage.success('已复制到剪贴板')
  }).catch(() => {
    ElMessage.error('复制失败，请手动复制')
  })
}

onMounted(() => fetchKeys())
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

.api-key-cell {
  display: flex;
  align-items: center;
  gap: 4px;
}

.api-key {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #666;
  background: #f5f5f5;
  padding: 2px 6px;
  border-radius: 4px;
  word-break: break-all;
}

.form-tip {
  margin-left: 12px;
  color: #999;
  font-size: 12px;
}

.mr-1 {
  margin-right: 4px;
}

.text-gray {
  color: #999;
}
</style>
