<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>用户资源管理</span>
          <div class="header-right">
            <el-select v-model="filterUserId" placeholder="筛选用户" clearable filterable style="width: 200px" @change="fetchResources">
              <el-option
                v-for="u in users"
                :key="u.id"
                :label="`${u.username || u.nickname || u.email} (ID: ${u.id})`"
                :value="u.id"
              />
            </el-select>
            <el-select v-model="filterCategory" placeholder="分类" clearable style="width: 160px" @change="fetchResources">
              <el-option v-for="c in categories" :key="c.id" :label="c.name" :value="c.id" />
            </el-select>
            <el-input v-model="keyword" placeholder="搜索标题/描述" clearable style="width: 180px" @change="fetchResources" />
            <el-button @click="fetchResources">搜索</el-button>
          </div>
        </div>
      </template>

      <el-table :data="resources" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="title" label="标题" min-width="180" show-overflow-tooltip />
        <el-table-column prop="owner_type" label="所有者类型" width="110">
          <template #default="{ row }">
            <el-tag size="small" :type="row.owner_type === 'user' ? '' : 'danger'">
              {{ row.owner_type === 'user' ? '用户' : '平台' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="provider_id" label="用户 ID" width="120" />
        <el-table-column prop="category_id" label="分类" width="120">
          <template #default="{ row }">
            {{ getCategoryName(row.category_id) }}
          </template>
        </el-table-column>
        <el-table-column prop="price" label="价格" width="90" align="right">
          <template #default="{ row }">${{ row.price }}</template>
        </el-table-column>
        <el-table-column prop="download_count" label="下载次数" width="100" align="center" />
        <el-table-column prop="is_active" label="状态" width="80" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="row.is_active ? 'success' : 'info'">
              {{ row.is_active ? '上架' : '下架' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="160">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" link @click="viewDetail(row)">详情</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination" v-if="total > 0">
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="pageSize"
          :total="total"
          :page-sizes="[10, 20, 50]"
          layout="total, sizes, prev, pager, next"
          @change="fetchResources"
        />
      </div>
    </el-card>

    <!-- 资源详情弹窗 -->
    <el-dialog v-model="detailVisible" title="资源详情" width="600px">
      <el-descriptions v-if="detail" :column="2" border size="small">
        <el-descriptions-item label="ID">{{ detail.id }}</el-descriptions-item>
        <el-descriptions-item label="分类">{{ getCategoryName(detail.category_id) }}</el-descriptions-item>
        <el-descriptions-item label="标题" :span="2">{{ detail.title }}</el-descriptions-item>
        <el-descriptions-item label="描述" :span="2">{{ detail.description || '-' }}</el-descriptions-item>
        <el-descriptions-item label="价格">${{ detail.price }}</el-descriptions-item>
        <el-descriptions-item label="下载次数">{{ detail.download_count }}</el-descriptions-item>
        <el-descriptions-item label="所有者">{{ detail.owner_type }} #{{ detail.provider_id }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag size="small" :type="detail.is_active ? 'success' : 'info'">
            {{ detail.is_active ? '上架' : '下架' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="文件地址" :span="2">
          <a :href="detail.file_url" target="_blank" v-if="detail.file_url">{{ detail.file_url }}</a>
          <span v-else>-</span>
        </el-descriptions-item>
        <el-descriptions-item label="创建时间">{{ formatDate(detail.created_at) }}</el-descriptions-item>
        <el-descriptions-item label="更新时间">{{ formatDate(detail.updated_at) }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { listUsers } from '@/api/user'
import { listResources } from '@/api/resource'
import { adminListCategories } from '@/api/category'

const loading = ref(false)
const resources = ref<any[]>([])
const users = ref<any[]>([])
const categories = ref<any[]>([])
const filterUserId = ref<number | null>(null)
const filterCategory = ref<number | null>(null)
const keyword = ref('')
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const detailVisible = ref(false)
const detail = ref<any>(null)

async function fetchUsers() {
  try {
    const res = await listUsers()
    if (res.code === 0) users.value = res.data?.users || res.data?.items || []
  } catch { /* ignore */ }
}

async function fetchCategories() {
  try {
    const res = await adminListCategories()
    if (res.code === 0) categories.value = res.data?.categories || []
  } catch { /* ignore */ }
}

async function fetchResources() {
  loading.value = true
  try {
    const params: Record<string, any> = { page: page.value, page_size: pageSize.value }
    if (filterUserId.value) params.user_id = filterUserId.value
    if (filterCategory.value) params.category_id = filterCategory.value
    if (keyword.value) params.keyword = keyword.value
    const res = await listResources(params)
    if (res.code === 0) {
      const d = res.data
      resources.value = d.resources || d.items || []
      total.value = d.total || 0
    } else {
      ElMessage.error(res.msg || '获取失败')
    }
  } catch {
    ElMessage.error('获取资源列表失败')
  } finally {
    loading.value = false
  }
}

function getCategoryName(id: number) {
  return categories.value.find(c => c.id === id)?.name || id
}

function formatDate(d: string) {
  return d ? new Date(d).toLocaleString('zh-CN') : '-'
}

function viewDetail(row: any) {
  detail.value = row
  detailVisible.value = true
}

onMounted(() => { fetchUsers(); fetchCategories(); fetchResources() })
</script>

<style scoped>
.page-container { padding: 20px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
.header-right { display: flex; gap: 8px; align-items: center; }
.pagination { margin-top: 16px; display: flex; justify-content: flex-end; }
</style>
