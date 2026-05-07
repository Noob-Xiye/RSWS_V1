<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>资源管理</span>
          <el-button type="primary" size="small" @click="fetchResources">刷新</el-button>
        </div>
      </template>
      
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="标题">
          <el-input v-model="searchForm.title" placeholder="搜索标题" clearable />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="searchForm.status" placeholder="全部" clearable>
            <el-option label="待审核" value="pending" />
            <el-option label="已通过" value="approved" />
            <el-option label="已拒绝" value="rejected" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>
      
      <el-table :data="resources" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="title" label="标题" min-width="200" />
        <el-table-column prop="price" label="价格 (USDT)" width="120" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">{{ getStatusText(row.status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="creator_name" label="创作者" width="120" />
        <el-table-column prop="download_count" label="下载量" width="80" />
        <el-table-column prop="created_at" label="创建时间" width="180">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="180">
          <template #default="{ row }">
            <el-button v-if="row.status === 'pending'" type="success" size="small" link @click="handleApprove(row)">通过</el-button>
            <el-button v-if="row.status === 'pending'" type="danger" size="small" link @click="handleReject(row)">拒绝</el-button>
            <el-button type="primary" size="small" link @click="handleView(row)">查看</el-button>
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
          @size-change="fetchResources"
          @current-change="fetchResources"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { Resource } from '@/api/resource'
import { listResources, updateResourceStatus } from '@/api/resource'

const loading = ref(false)
const resources = ref<Resource[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)

const searchForm = reactive({
  title: '',
  status: ''
})

function getStatusType(status: string) {
  const map: Record<string, string> = { pending: 'warning', approved: 'success', rejected: 'danger', draft: 'info' }
  return map[status] || 'info'
}

function getStatusText(status: string) {
  const map: Record<string, string> = { pending: '待审核', approved: '已通过', rejected: '已拒绝', draft: '草稿' }
  return map[status] || status
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

async function fetchResources() {
  loading.value = true
  try {
    const res = await listResources({ page: page.value, page_size: pageSize.value, ...searchForm })
    if (res.success && res.data) {
      resources.value = res.data.items
      total.value = res.data.total
    }
  } catch {
    // mock
    resources.value = []
    total.value = 0
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchResources()
}

function handleReset() {
  searchForm.title = ''
  searchForm.status = ''
  fetchResources()
}

function handleView(row: Resource) {
  ElMessage.info(`查看资源: ${row.title}`)
}

async function handleApprove(row: Resource) {
  try {
    await ElMessageBox.confirm(`确定通过资源 "${row.title}" 吗？`, '确认', { type: 'success' })
    await updateResourceStatus(row.id, 'approved')
    ElMessage.success('已通过')
    fetchResources()
  } catch {}
}

async function handleReject(row: Resource) {
  try {
    const { value } = await ElMessageBox.prompt('请输入拒绝原因', '拒绝资源', { inputPattern: /.+/, inputErrorMessage: '请输入拒绝原因' })
    await updateResourceStatus(row.id, 'rejected', value)
    ElMessage.success('已拒绝')
    fetchResources()
  } catch {}
}

onMounted(() => fetchResources())
</script>

<style scoped>
.page-container { padding: 20px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
.search-form { margin-bottom: 20px; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
</style>