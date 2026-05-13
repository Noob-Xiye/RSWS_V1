<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <span>日志查询</span>
      </template>
      
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
    </el-card>
    
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
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import type { SystemLog } from '@/api/log'
import { querySystemLogs } from '@/api/log'

const loading = ref(false)
const logs = ref<SystemLog[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(50)

const searchForm = reactive({ level: '', module: '' })
const dateRange = ref<[string, string] | null>(null)

const detailVisible = ref(false)
const currentLog = ref<SystemLog | null>(null)

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

onMounted(() => fetchLogs())
</script>

<style scoped>
.page-container { padding: 20px; }
.search-form { margin-bottom: 20px; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
.log-detail { background: #f5f5f5; padding: 10px; border-radius: 4px; max-height: 300px; overflow: auto; white-space: pre-wrap; word-break: break-all; }
</style>