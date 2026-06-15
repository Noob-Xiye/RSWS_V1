<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <el-tabs v-model="activeTab">
          <el-tab-pane label="日志查询" name="query" />
          <el-tab-pane label="日志配置" name="config" />
        </el-tabs>
      </template>

      <!-- Tab 1: 日志查询 -->
      <div v-if="activeTab === 'query'">
        <el-form :inline="true" :model="searchForm" class="search-form">
          <el-form-item label="级别">
            <el-select v-model="searchForm.level" placeholder="全部" clearable>
              <el-option label="INFO" value="info" />
              <el-option label="WARN" value="warn" />
              <el-option label="ERROR" value="error" />
              <el-option label="DEBUG" value="debug" />
              <el-option label="FATAL" value="fatal" />
            </el-select>
          </el-form-item>
          <el-form-item label="模块">
            <el-input v-model="searchForm.module" placeholder="模块" clearable />
          </el-form-item>
          <el-form-item label="时间范围">
            <el-date-picker
              v-model="dateRange"
              type="datetimerange"
              range-separator="至"
              start-placeholder="开始时间"
              end-placeholder="结束时间"
              value-format="YYYY-MM-DDTHH:mm:ss"
            />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="handleSearch">搜索</el-button>
            <el-button @click="handleReset">重置</el-button>
          </el-form-item>
        </el-form>

        <el-table :data="logs" v-loading="loading" stripe max-height="600">
          <el-table-column prop="created_at" label="时间" width="180">
            <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
          </el-table-column>
          <el-table-column prop="log_level" label="级别" width="80">
            <template #default="{ row }">
              <el-tag :type="getLevelType(row.log_level)" size="small">{{ row.log_level.toUpperCase() }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="module" label="模块" width="120" />
          <el-table-column prop="message" label="消息" min-width="300" show-overflow-tooltip />
          <el-table-column label="操作" width="80">
            <template #default="{ row }">
              <el-button type="primary" size="small" link @click="showDetail(row)">详情</el-button>
            </template>
          </el-table-column>
        </el-table>

        <div class="pagination">
          <el-pagination
            v-model:current-page="page"
            v-model:page-size="pageSize"
            :total="total"
            :page-sizes="[20, 50, 100, 200]"
            layout="total, sizes, prev, pager, next"
            @size-change="fetchLogs"
            @current-change="fetchLogs"
          />
        </div>
      </div>

      <!-- Tab 2: 日志配置 -->
      <div v-else>
        <el-table :data="configs" v-loading="configLoading" stripe>
          <el-table-column prop="config_key" label="配置项" width="200" />
          <el-table-column prop="config_value" label="值" min-width="200">
            <template #default="{ row }">
              <el-tag v-if="row.config_key.includes('enable') || row.config_key.includes('active')" :type="row.config_value === 'true' ? 'success' : 'danger'">
                {{ row.config_value }}
              </el-tag>
              <span v-else>{{ row.config_value }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="description" label="说明" min-width="200" />
          <el-table-column prop="is_active" label="启用" width="80">
            <template #default="{ row }">
              <el-switch v-model="row.is_active" @change="handleToggleConfig(row)" />
            </template>
          </el-table-column>
          <el-table-column label="操作" width="120">
            <template #default="{ row }">
              <el-button size="small" @click="handleEditConfig(row)">编辑</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-card>

    <!-- 日志详情对话框 -->
    <el-dialog v-model="detailVisible" title="日志详情" width="600px">
      <el-descriptions :column="1" border>
        <el-descriptions-item label="时间">{{ currentLog?.created_at }}</el-descriptions-item>
        <el-descriptions-item label="级别">{{ currentLog?.log_level }}</el-descriptions-item>
        <el-descriptions-item label="模块">{{ currentLog?.module }}</el-descriptions-item>
        <el-descriptions-item label="消息">{{ currentLog?.message }}</el-descriptions-item>
        <el-descriptions-item label="上下文">
          <pre class="log-detail">{{ currentLog?.context ? JSON.stringify(currentLog.context, null, 2) : '无' }}</pre>
        </el-descriptions-item>
        <el-descriptions-item v-if="currentLog?.ip_address" label="IP 地址">{{ currentLog.ip_address }}</el-descriptions-item>
        <el-descriptions-item v-if="currentLog?.request_id" label="请求 ID">{{ currentLog.request_id }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>

    <!-- 配置编辑对话框 -->
    <el-dialog v-model="configDialogVisible" title="编辑日志配置" width="500px">
      <el-form :model="configForm" label-width="100px">
        <el-form-item label="配置项">{{ configForm.config_key }}</el-form-item>
        <el-form-item label="值">
          <el-input v-model="configForm.config_value" />
        </el-form-item>
        <el-form-item label="类型">
          <el-input v-model="configForm.config_type" disabled />
        </el-form-item>
        <el-form-item label="说明">
          <el-input v-model="configForm.description" type="textarea" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="configDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSaveConfig">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import type { SystemLog, LogConfig } from '@/api/log'
import { querySystemLogs, listLogConfigs, updateLogConfig } from '@/api/log'
import { ElMessage } from 'element-plus'

const activeTab = ref('query')
const loading = ref(false)
const configLoading = ref(false)
const logs = ref<SystemLog[]>([])
const configs = ref<LogConfig[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(50)

const searchForm = reactive({ level: '', module: '' })
const dateRange = ref<[string, string] | null>(null)

const detailVisible = ref(false)
const currentLog = ref<SystemLog | null>(null)

const configDialogVisible = ref(false)
const configForm = reactive({
  config_key: '',
  config_value: '',
  config_type: '',
  description: '',
})

function getLevelType(level: string) {
  const map: Record<string, string> = { debug: 'info', info: 'info', warn: 'warning', error: 'danger', fatal: 'danger' }
  return map[level.toLowerCase()] || 'info'
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

async function fetchLogs() {
  loading.value = true
  try {
    const params: Record<string, unknown> = { page: page.value, page_size: pageSize.value, ...searchForm }
    if (dateRange.value) {
      params.start_time = dateRange.value[0]
      params.end_time = dateRange.value[1]
    }
    const res = await querySystemLogs(params)
    if (res.code === 0 && res.data) {
      logs.value = res.data.items
      total.value = res.data.total
    }
  } catch {
    logs.value = []
    total.value = 0
  } finally {
    loading.value = false
  }
}

async function fetchConfigs() {
  configLoading.value = true
  try {
    const res = await listLogConfigs()
    if (res.code === 0 && res.data) {
      configs.value = res.data
    }
  } catch {
    configs.value = []
  } finally {
    configLoading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchLogs()
}

function handleReset() {
  searchForm.level = ''
  searchForm.module = ''
  dateRange.value = null
  fetchLogs()
}

function showDetail(row: SystemLog) {
  currentLog.value = row
  detailVisible.value = true
}

async function handleToggleConfig(row: LogConfig) {
  try {
    await updateLogConfig(row.config_key, { config_value: String(row.is_active) })
    ElMessage.success('更新成功')
  } catch {
    row.is_active = !row.is_active
    ElMessage.error('更新失败')
  }
}

function handleEditConfig(row: LogConfig) {
  configForm.config_key = row.config_key
  configForm.config_value = row.config_value
  configForm.config_type = row.config_type || ''
  configForm.description = row.description || ''
  configDialogVisible.value = true
}

async function handleSaveConfig() {
  try {
    await updateLogConfig(configForm.config_key, {
      config_value: configForm.config_value,
      config_type: configForm.config_type || undefined,
      description: configForm.description || undefined,
    })
    ElMessage.success('保存成功')
    configDialogVisible.value = false
    fetchConfigs()
  } catch {
    ElMessage.error('保存失败')
  }
}

onMounted(() => {
  fetchLogs()
  fetchConfigs()
})
</script>

<style scoped>
.page-container { padding: 20px; }
.search-form { margin-bottom: 20px; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
.log-detail { background: #f5f5f5; padding: 10px; border-radius: 4px; max-height: 300px; overflow: auto; white-space: pre-wrap; word-break: break-all; }
</style>
