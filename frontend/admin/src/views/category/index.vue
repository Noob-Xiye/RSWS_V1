<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>分类管理</span>
          <el-button type="primary" @click="openDialog()">
            <el-icon><Plus /></el-icon> 新建分类
          </el-button>
        </div>
      </template>

      <el-table
        ref="tableRef"
        :data="treeData"
        v-loading="loading"
        stripe
        row-key="id"
        :tree-props="{ children: 'children', hasChildren: 'hasChildren' }"
        :row-class-name="rowClassName"
        default-expand-all
      >
        <el-table-column prop="name" label="分类名称" min-width="200" />
        <el-table-column prop="description" label="描述" min-width="200" show-overflow-tooltip>
          <template #default="{ row }">{{ row.description || '-' }}</template>
        </el-table-column>
        <el-table-column prop="sort_order" label="排序" width="80" align="center" />
        <el-table-column prop="resource_count" label="资源数" width="90" align="center" />
        <el-table-column prop="is_active" label="状态" width="90" align="center">
          <template #default="{ row }">
            <el-switch
              :model-value="row.is_active"
              @change="(val: boolean) => handleToggleStatus(row, val)"
              :loading="row._switching"
            />
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="170">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" link @click="openDialog(row)">
              编辑
            </el-button>
            <el-button
              v-if="!row.resource_count"
              type="danger" size="small" link
              @click="handleDelete(row)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 新建/编辑对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingCategory ? '编辑分类' : '新建分类'"
      width="480px"
      :close-on-click-modal="false"
    >
      <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-width="80px"
        @submit.prevent
      >
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" placeholder="请输入分类名称" maxlength="100" show-word-limit />
        </el-form-item>
        <el-form-item label="父分类" prop="parent_id">
          <el-select v-model="form.parent_id" placeholder="无（顶级分类）" clearable style="width: 100%">
            <el-option
              v-for="cat in parentOptions"
              :key="cat.id"
              :label="cat.name"
              :value="cat.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="描述" prop="description">
          <el-input
            v-model="form.description"
            type="textarea"
            placeholder="请输入分类描述（选填）"
            :rows="3"
            maxlength="500"
            show-word-limit
          />
        </el-form-item>
        <el-form-item label="排序" prop="sort_order">
          <el-input-number v-model="form.sort_order" :min="0" :max="9999" />
          <span class="form-tip">数值越小越靠前</span>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, nextTick } from 'vue'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import Sortable from 'sortablejs'
import type { Category } from '@/api/category'
import { adminListCategories, createCategory, updateCategory, deleteCategory, batchUpdateSort } from '@/api/category'

interface TreeRow extends Category {
  children?: TreeRow[]
  _switching?: boolean
}

const loading = ref(false)
const submitting = ref(false)
const categories = ref<TreeRow[]>([])
let sortableInstance: Sortable | null = null
const tableRef = ref<InstanceType<typeof import('element-plus')['ElTable']>>()

// 构建树形数据
const treeData = computed(() => {
  const map = new Map<number, TreeRow>()
  const roots: TreeRow[] = []

  // 创建所有节点
  categories.value.forEach(cat => {
    map.set(cat.id, { ...cat, children: [] })
  })

  // 构建树
  categories.value.forEach(cat => {
    const node = map.get(cat.id)!
    if (cat.parent_id && map.has(cat.parent_id)) {
      map.get(cat.parent_id)!.children!.push(node)
    } else {
      roots.push(node)
    }
  })

  return roots
})

// 父分类选项（排除自身及其后代）
const parentOptions = computed(() => {
  if (!editingCategory.value) return categories.value
  const id = editingCategory.value.id
  // 排除自身和后代
  const descendantIds = new Set<number>()
  const collectDescendants = (parentId: number) => {
    descendantIds.add(parentId)
    categories.value.forEach(c => {
      if (c.parent_id === parentId) collectDescendants(c.id)
    })
  }
  collectDescendants(id)
  return categories.value.filter(c => !descendantIds.has(c.id))
})

// 对话框
const dialogVisible = ref(false)
const editingCategory = ref<Category | null>(null)
const formRef = ref<FormInstance>()

const form = reactive({
  name: '',
  description: '',
  parent_id: null as number | null,
  sort_order: 0,
})

const rules: FormRules = {
  name: [
    { required: true, message: '请输入分类名称', trigger: 'blur' },
    { max: 100, message: '名称不能超过100个字符', trigger: 'blur' },
  ],
}

function rowClassName({ row }: { row: TreeRow }) {
  return row.is_active ? '' : 'inactive-row'
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}


function initSortable() {
  nextTick(() => {
    const el = tableRef.value?.$el?.querySelector('.el-table__body-wrapper tbody')
    if (!el || sortableInstance) return

    sortableInstance = Sortable.create(el, {
      animation: 150,
      handle: '.el-table__row', // 整行可拖
      ghostClass: 'sortable-ghost',
      onEnd: async (evt) => {
        const { oldIndex, newIndex } = evt
        if (oldIndex === newIndex) return

        // 只处理顶级分类拖拽（简化）
        const items = treeData.value
        const moved = items.splice(oldIndex!, 1)[0]
        items.splice(newIndex!, 0, moved)

        // 更新排序
        const orders = items.map((c, i) => ({ id: c.id, sort_order: i }))
        try {
          const res = await batchUpdateSort(orders)
          if (res.code !== 0) {
            ElMessage.error(res.msg || '排序失败')
            fetchCategories() // 回滚
          }
        } catch {
          ElMessage.error('排序失败')
          fetchCategories()
        }
      },
    })
  })
}

async function fetchCategories() {
  loading.value = true
  try {
    const res = await adminListCategories()
    if (res.code === 0 && res.data) {
      categories.value = res.data.categories
    }
  } catch {
    ElMessage.error('获取分类列表失败')
  } finally {
    loading.value = false
    initSortable()
  }
}

function openDialog(category?: Category) {
  editingCategory.value = category || null
  if (category) {
    form.name = category.name
    form.description = category.description || ''
    form.parent_id = category.parent_id
    form.sort_order = category.sort_order
  } else {
    form.name = ''
    form.description = ''
    form.parent_id = null
    form.sort_order = Math.max(...categories.value.map(c => c.sort_order), 0) + 1
  }
  dialogVisible.value = true
}

async function handleSubmit() {
  if (!formRef.value) return
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    if (editingCategory.value) {
      const res = await updateCategory(editingCategory.value.id, {
        name: form.name.trim(),
        description: form.description.trim() || undefined,
        parent_id: form.parent_id,
        sort_order: form.sort_order,
      })
      if (res.code === 0) {
        ElMessage.success('更新成功')
        dialogVisible.value = false
        fetchCategories()
      } else {
        ElMessage.error(res.msg || '更新失败')
      }
    } else {
      const res = await createCategory({
        name: form.name.trim(),
        description: form.description.trim() || undefined,
        parent_id: form.parent_id,
        sort_order: form.sort_order,
      })
      if (res.code === 0) {
        ElMessage.success('创建成功')
        dialogVisible.value = false
        fetchCategories()
      } else {
        ElMessage.error(res.msg || '创建失败')
      }
    }
  } catch {
    ElMessage.error('操作失败')
  } finally {
    submitting.value = false
  }
}

async function handleToggleStatus(category: TreeRow, val: boolean) {
  category._switching = true
  try {
    const res = await updateCategory(category.id, { is_active: val })
    if (res.code === 0) {
      ElMessage.success(val ? '已启用' : '已停用')
      fetchCategories()
    } else {
      ElMessage.error(res.msg || '操作失败')
    }
  } catch {
    ElMessage.error('操作失败')
  } finally {
    category._switching = false
  }
}

async function handleDelete(category: Category) {
  try {
    await ElMessageBox.confirm(
      `确定删除分类「${category.name}」吗？子分类将变为顶级分类。此操作不可恢复。`,
      '删除确认',
      { type: 'warning', confirmButtonText: '确定删除', cancelButtonText: '取消' }
    )
  } catch {
    return
  }

  try {
    const res = await deleteCategory(category.id)
    if (res.code === 0) {
      ElMessage.success('删除成功')
      fetchCategories()
    } else {
      ElMessage.error(res.msg || '删除失败')
    }
  } catch {
    ElMessage.error('删除失败')
  }
}

onMounted(() => fetchCategories())
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

.form-tip {
  margin-left: 12px;
  color: #999;
  font-size: 12px;
}

:deep(.inactive-row) {
  opacity: 0.5;
}

:deep(.sortable-ghost) {
  opacity: 0.4;
  background: #f0f9ff;
}
</style>
