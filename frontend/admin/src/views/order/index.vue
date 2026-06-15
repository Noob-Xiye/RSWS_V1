<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>订单管理</span>
          <el-button type="primary" size="small" @click="fetchOrders">刷新</el-button>
        </div>
      </template>

      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="状态">
          <el-select v-model="searchForm.status" placeholder="全部" clearable>
            <el-option label="待支付" value="pending" />
            <el-option label="已支付" value="paid" />
            <el-option label="已完成" value="completed" />
            <el-option label="已取消" value="cancelled" />
            <el-option label="已退款" value="refunded" />
          </el-select>
        </el-form-item>
        <el-form-item label="支付方式">
          <el-select v-model="searchForm.payment_method" placeholder="全部" clearable>
            <el-option label="PayPal" value="paypal" />
            <el-option label="USDT-TRC20" value="usdt_trc20" />
            <el-option label="USDT-ERC20" value="usdt_erc20" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>

      <el-table :data="orders" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="70" />
        <el-table-column prop="user_name" label="买家" width="130">
          <template #default="{ row }">{{ row.user_name || row.user_email || '-' }}</template>
        </el-table-column>
        <el-table-column prop="resource_title" label="资源" min-width="180">
          <template #default="{ row }">{{ row.resource_title || '-' }}</template>
        </el-table-column>
        <el-table-column prop="amount" label="金额 (USDT)" width="120">
          <template #default="{ row }">{{ formatAmount(row.amount) }}</template>
        </el-table-column>
        <el-table-column prop="payment_method" label="支付方式" width="120">
          <template #default="{ row }">{{ getPaymentMethod(row.payment_method) }}</template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">{{ getStatusText(row.status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="180">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="160" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" link @click="handleView(row)">详情</el-button>
            <el-button v-if="row.status === 'paid'" type="success" size="small" link @click="handleComplete(row)">完成</el-button>
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
          @size-change="fetchOrders"
          @current-change="fetchOrders"
        />
      </div>
    </el-card>

    <!-- 订单详情弹窗 -->
    <el-dialog v-model="detailVisible" title="订单详情" width="600px" destroy-on-close>
      <template v-if="currentOrder">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="订单 ID">{{ currentOrder.id }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusType(currentOrder.status)" size="small">{{ getStatusText(currentOrder.status) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="支付方式">{{ getPaymentMethod(currentOrder.payment_method) }}</el-descriptions-item>
          <el-descriptions-item label="订单金额">
            <span class="amount">{{ formatAmount(currentOrder.amount) }} USDT</span>
          </el-descriptions-item>
          <el-descriptions-item label="买家">{{ currentOrder.user_name || '-' }}</el-descriptions-item>
          <el-descriptions-item label="买家邮箱">{{ currentOrder.user_email || '-' }}</el-descriptions-item>
          <el-descriptions-item label="资源名称" :span="2">{{ currentOrder.resource_title || '-' }}</el-descriptions-item>
          <el-descriptions-item label="创建时间">{{ formatDate(currentOrder.created_at) }}</el-descriptions-item>
          <el-descriptions-item label="过期时间">{{ currentOrder.expired_at ? formatDate(currentOrder.expired_at) : '-' }}</el-descriptions-item>
          <el-descriptions-item label="更新时间" :span="2">{{ formatDate(currentOrder.updated_at) }}</el-descriptions-item>
        </el-descriptions>
      </template>
      <template #footer>
        <el-button @click="detailVisible = false">关闭</el-button>
        <el-button v-if="currentOrder?.status === 'paid'" type="success" @click="handleCompleteFromModal">确认完成</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { AdminOrder, AdminOrderListParams } from '@/api/order'
import { adminListOrders, completeOrder } from '@/api/order'

const loading = ref(false)
const orders = ref<AdminOrder[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const detailVisible = ref(false)
const currentOrder = ref<AdminOrder | null>(null)

const searchForm = reactive({
  status: '' as string,
  payment_method: '' as string,
})

function getStatusType(status: string) {
  const map: Record<string, string> = { pending: 'warning', paid: 'primary', completed: 'success', cancelled: 'info', refunded: 'danger' }
  return map[status] || 'info'
}

function getStatusText(status: string) {
  const map: Record<string, string> = { pending: '待支付', paid: '已支付', completed: '已完成', cancelled: '已取消', refunded: '已退款' }
  return map[status] || status
}

function getPaymentMethod(method: string | null) {
  if (!method) return '-'
  const map: Record<string, string> = { paypal: 'PayPal', usdt_trc20: 'USDT-TRC20', usdt_erc20: 'USDT-ERC20' }
  return map[method] || method
}

function formatAmount(amount: number | string) {
  return (Number(amount) / 100).toFixed(2)
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

async function fetchOrders() {
  loading.value = true
  try {
    const params: AdminOrderListParams = { page: page.value, page_size: pageSize.value }
    if (searchForm.status) params.status = searchForm.status
    if (searchForm.payment_method) params.payment_method = searchForm.payment_method
    const res = await adminListOrders(params)
    if (res.code === 0 && res.data) {
      orders.value = res.data.items
      total.value = res.data.total
    }
  } catch (e) {
    console.error('Failed to fetch orders:', e)
    ElMessage.error('获取订单列表失败')
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchOrders()
}

function handleReset() {
  searchForm.status = ''
  searchForm.payment_method = ''
  handleSearch()
}

function handleView(row: AdminOrder) {
  currentOrder.value = row
  detailVisible.value = true
}

async function handleComplete(row: AdminOrder) {
  try {
    await ElMessageBox.confirm(`确认完成订单 #${row.id}？`, '确认', { type: 'warning' })
    await completeOrder(row.id)
    ElMessage.success('订单已完成')
    fetchOrders()
  } catch {
    // 用户取消
  }
}

async function handleCompleteFromModal() {
  if (!currentOrder.value) return
  detailVisible.value = false
  await handleComplete(currentOrder.value)
}

onMounted(() => fetchOrders())
</script>

<style scoped>
.page-container { padding: 20px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
.search-form { margin-bottom: 20px; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
.amount { color: #67c23a; font-weight: bold; font-size: 16px; }
</style>
