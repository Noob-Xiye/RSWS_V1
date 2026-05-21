<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>资源管理</span>
          <div>
            <el-button type="primary" size="small" @click="handleCreate">
              <el-icon><Plus /></el-icon> 创建资源
            </el-button>
            <el-button size="small" @click="fetchResources">
              <el-icon><Refresh /></el-icon> 刷新
            </el-button>
          </div>
        </div>
      </template>

      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="关键词">
          <el-input v-model="searchForm.search" placeholder="搜索标题/描述" clearable style="width: 200px" />
        </el-form-item>
        <el-form-item label="分类">
          <el-select v-model="searchForm.category_id" placeholder="全部分类" clearable style="width: 180px">
            <el-option
              v-for="cat in categoryOptions"
              :key="cat.id"
              :label="cat.name"
              :value="cat.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="searchForm.active" placeholder="全部" clearable style="width: 120px">
            <el-option label="上架" :value="true" />
            <el-option label="下架" :value="false" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>

      <el-table
        :data="filteredResources"
        v-loading="loading"
        stripe
        row-key="id"
        :row-class-name="({ row }: { row: Resource }) => row.is_active ? '' : 'inactive-row'"
      >
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="title" label="标题" min-width="200" show-overflow-tooltip />
        <el-table-column label="分类" width="120">
          <template #default="{ row }">
            <el-tag size="small">{{ getCategoryName(row.category_id) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="price" label="价格 (USDT)" width="120" align="right">
          <template #default="{ row }">{{ (Number(row.price) / 100).toFixed(2) }}</template>
        </el-table-column>
        <el-table-column label="状态" width="80" align="center">
          <template #default="{ row }">
            <el-tag :type="row.is_active ? 'success' : 'info'" size="small">
              {{ row.is_active ? '上架' : '下架' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="download_count" label="下载量" width="90" align="center" />
        <el-table-column label="支持系统" width="150">
          <template #default="{ row }">
            <el-tag v-for="os in row.supported_os || []" :key="os" size="small" style="margin-right: 2px">
              {{ os === 'macos' ? 'macOS' : os === 'ios' ? 'iOS' : os.charAt(0).toUpperCase() + os.slice(1) }}
            </el-tag>
            <span v-if="!row.supported_os || row.supported_os.length === 0" class="text-muted">未设置</span>
          </template>
        </el-table-column>
        <el-table-column prop="user_id" label="发布者ID" width="100" />
        <el-table-column prop="created_at" label="创建时间" width="170">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="220" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" link @click="handleView(row)">详情</el-button>
            <el-button type="primary" size="small" link @click="handleEdit(row)">编辑</el-button>
            <el-button
              :type="row.is_active ? 'warning' : 'success'"
              size="small" link
              @click="handleToggle(row)"
            >
              {{ row.is_active ? '下架' : '上架' }}
            </el-button>
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

    <!-- 资源详情对话框 -->
    <el-dialog v-model="detailVisible" title="资源详情" width="680px">
      <el-descriptions v-if="currentResource" :column="2" border>
        <el-descriptions-item label="ID" :span="1">{{ currentResource.id }}</el-descriptions-item>
        <el-descriptions-item label="发布者ID" :span="1">{{ currentResource.user_id }}</el-descriptions-item>
        <el-descriptions-item label="标题" :span="2">{{ currentResource.title }}</el-descriptions-item>
        <el-descriptions-item label="分类" :span="1">
          <el-tag>{{ getCategoryName(currentResource.category_id) }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="状态" :span="1">
          <el-tag :type="currentResource.is_active ? 'success' : 'info'">
            {{ currentResource.is_active ? '上架' : '下架' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="价格" :span="1">{{ (Number(currentResource.price) / 100).toFixed(2) }} USDT</el-descriptions-item>
        <el-descriptions-item label="下载量" :span="1">{{ currentResource.download_count }}</el-descriptions-item>
        <el-descriptions-item label="描述" :span="2">{{ currentResource.description || '无' }}</el-descriptions-item>
        <el-descriptions-item label="详细介绍" :span="2">{{ currentResource.detail_description || '无' }}</el-descriptions-item>
        <el-descriptions-item label="使用指南" :span="2">{{ currentResource.usage_guide || '无' }}</el-descriptions-item>
        <el-descriptions-item label="注意事项" :span="2">{{ currentResource.precautions || '无' }}</el-descriptions-item>
        <el-descriptions-item label="支持系统" :span="2">
          <el-tag v-for="os in currentResource.supported_os || []" :key="os" size="small" style="margin-right: 4px">
            {{ os === 'macos' ? 'macOS' : os === 'ios' ? 'iOS' : os.charAt(0).toUpperCase() + os.slice(1) }}
          </el-tag>
          <span v-if="!currentResource.supported_os || currentResource.supported_os.length === 0" class="text-muted">未设置</span>
        </el-descriptions-item>
        <el-descriptions-item label="创建时间" :span="1">{{ formatDate(currentResource.created_at) }}</el-descriptions-item>
        <el-descriptions-item label="更新时间" :span="1">{{ formatDate(currentResource.updated_at) }}</el-descriptions-item>
      </el-descriptions>
      <template #footer>
        <el-button @click="detailVisible = false">关闭</el-button>
        <el-button
          v-if="currentResource"
          :type="currentResource.is_active ? 'warning' : 'success'"
          @click="handleToggle(currentResource!); detailVisible = false"
        >
          {{ currentResource.is_active ? '下架此资源' : '上架此资源' }}
        </el-button>
        <el-button type="danger" @click="handleDelete(currentResource!); detailVisible = false">
          删除此资源
        </el-button>
      </template>
    </el-dialog>

    <!-- 创建/编辑资源对话框 -->
    <el-dialog v-model="formVisible" :title="isEditing ? '编辑资源' : '创建资源'" width="800px">
      <el-form :model="form" label-width="100px">
        <el-form-item label="标题" required>
          <el-input v-model="form.title" placeholder="请输入资源标题" />
        </el-form-item>
        <el-form-item label="分类">
          <el-select v-model="form.category_id" placeholder="请选择分类" clearable>
            <el-option v-for="cat in categoryOptions" :key="cat.id" :label="cat.name" :value="cat.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="价格 (USDT)" required>
          <el-input-number v-model="form.price" :min="0" :precision="2" />
        </el-form-item>
        <el-form-item label="支持系统">
          <el-checkbox-group v-model="form.supported_os">
            <el-checkbox label="windows">Windows</el-checkbox>
            <el-checkbox label="macos">macOS</el-checkbox>
            <el-checkbox label="linux">Linux</el-checkbox>
            <el-checkbox label="ios">iOS</el-checkbox>
            <el-checkbox label="android">Android</el-checkbox>
          </el-checkbox-group>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="form.description" type="textarea" :rows="3" />
        </el-form-item>
        <el-form-item label="详细介绍">
          <el-input v-model="form.detail_description" type="textarea" :rows="5" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="formVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { Resource } from '@/api/resource'
import { listResources, deleteResource, toggleResourceActive, getCategoryOptions } from '@/api/resource'
import type { Category } from '@/api/category'

const loading = ref(false)
const resources = ref<Resource[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const categoryOptions = ref<Category[]>([])

const searchForm = reactive({
  search: '',
  category_id: undefined as number | undefined,
  active: undefined as boolean | undefined,
})

const detailVisible = ref(false)
const formVisible = ref(false)
const isEditing = ref(false)
const currentResource = ref<Resource | null>(null)
const form = reactive({
  id: undefined as number | undefined,
  title: '',
  description: '',
  price: 0,
  category_id: undefined as number | undefined,
  supported_os: [] as string[],
  detail_description: '',
  usage_guide: '',
  precautions: '',
})

/** 前端本地根据 is_active 过滤（后端列表不传 active 参数时返回全部） */
const filteredResources = computed(() => {
  if (searchForm.active === undefined) return resources.value
  return resources.value.filter(r => r.is_active === searchForm.active)
})

function getCategoryName(categoryId: number | null): string {
  if (!categoryId) return '未分类'
  const cat = categoryOptions.value.find(c => c.id === categoryId)
  return cat?.name || `#${categoryId}`
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

async function fetchResources() {
  loading.value = true
  try {
    const params: Record<string, any> = {
      page: page.value,
      page_size: pageSize.value,
    }
    if (searchForm.search) params.search = searchForm.search
    if (searchForm.category_id) params.category_id = searchForm.category_id

    const res = await listResources(params)
    if (res.code === 0 && res.data) {
      resources.value = res.data.items
      total.value = res.data.total
    }
  } catch {
    ElMessage.error('获取资源列表失败')
  } finally {
    loading.value = false
  }
}

async function fetchCategories() {
  categoryOptions.value = await getCategoryOptions()
}

function handleSearch() {
  page.value = 1
  fetchResources()
}

function handleReset() {
  searchForm.search = ''
  searchForm.category_id = undefined
  searchForm.active = undefined
  page.value = 1
  fetchResources()
}

function handleView(row: Resource) {
  currentResource.value = row
  detailVisible.value = true
}

function handleCreate() {
  isEditing.value = false
  Object.assign(form, {
    id: undefined,
    title: '',
    description: '',
    price: 0,
    category_id: undefined,
    supported_os: [],
    detail_description: '',
    usage_guide: '',
    precautions: '',
  })
  formVisible.value = true
}

function handleEdit(row: Resource) {
  isEditing.value = true
  Object.assign(form, {
    id: row.id,
    title: row.title,
    description: row.description || '',
    price: row.price / 100, // 分转元
    category_id: row.category_id,
    supported_os: row.supported_os || [],
    detail_description: row.detail_description || '',
    usage_guide: row.usage_guide || '',
    precautions: row.precautions || '',
  })
  formVisible.value = true
}

async function handleSubmit() {
  if (!form.title) {
    ElMessage.warning('请输入资源标题')
    return
  }
  try {
    const payload = {
      ...form,
      price: Math.round(form.price * 100), // 元转分
    }
    if (isEditing.value && form.id) {
      // await updateResource(form.id, payload)
      ElMessage.success('更新成功（后端待实现）')
    } else {
      // await createResource(payload)
      ElMessage.success('创建成功（后端待实现）')
    }
    formVisible.value = false
    fetchResources()
  } catch (err) {
    ElMessage.error('操作失败')
  }
}

async function handleToggle(row: Resource) {
  const action = row.is_active ? '下架' : '上架'
  try {
    await ElMessageBox.confirm(`确定${action}资源「${row.title}」吗？`, `${action}确认`, {
      type: 'warning',
    })
    const res = await toggleResourceActive(row.id, !row.is_active)
    if (res.code === 0) {
      ElMessage.success(`${action}成功`)
      fetchResources()
    } else {
      ElMessage.error(res.msg || `${action}失败`)
    }
  } catch {}
}

async function handleDelete(row: Resource) {
  try {
    await ElMessageBox.confirm(
      `确定删除资源「${row.title}」吗？此操作不可恢复。`,
      '删除确认',
      { type: 'danger', confirmButtonText: '确定删除', cancelButtonText: '取消' }
    )
    const res = await deleteResource(row.id)
    if (res.code === 0) {
      ElMessage.success('删除成功')
      fetchResources()
    } else {
      ElMessage.error(res.msg || '删除失败')
    }
  } catch {}
}

onMounted(() => {
  fetchCategories()
  fetchResources()
})
</script>

<style scoped>
.page-container { padding: 20px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
.search-form { margin-bottom: 20px; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }

:deep(.inactive-row) {
  opacity: 0.5;
}
</style>
