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
        <el-form-item label="订单号">
          <el-input v-model="searchForm.order_no" placeholder="搜索订单号" clearable />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="searchForm.status" placeholder="全部" clearable>
            <el-option label="待支付" value="pending" />
            <el-option label="已支付" value="paid" />
            <el-option label="已完成" value="completed" />
            <el-option label="已取消" value="cancelled" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>
      
      <el-table :data="orders" v-loading="loading" stripe>
        <el-table-column prop="order_no" label="订单号" width="200" />
        <el-table-column prop="user_name" label="买家" width="120" />
        <el-table-column prop="resource_title" label="资源" min-width="200" />
        <el-table-column prop="amount" label="金额 (USDT)" width="120" />
        <el-table-column prop="payment_method" label="支付方式" width="100">
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
        <el-table-column label="操作" width="150">
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
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import type { Order, OrderListParams } from '@/api/order'
import { listOrders } from '@/api/order'

const loading = ref(false)
const orders = ref<Order[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)

const searchForm = reactive({
  order_no: '',
  status: '' as '' | 'pending' | 'paid' | 'completed' | 'cancelled'
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

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

async function fetchOrders() {
  loading.value = true
  try {
    const params: OrderListParams = { page: page.value, page_size: pageSize.value }
    if (searchForm.order_no) params.order_no = searchForm.order_no
    if (searchForm.status) params.status = searchForm.status
    const res = await listOrders(params)
    if (res.success && res.data) {
      orders.value = res.data.items
      total.value = res.data.total
    }
  } catch {
    orders.value = []
    total.value = 0
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchOrders()
}

function handleReset() {
  searchForm.order_no = ''
  searchForm.status = ''
  fetchOrders()
}

function handleView(row: Order) {
  ElMessage.info(`订单详情: ${row.order_no}`)
}

function handleComplete(_row: Order) {
  ElMessage.success('订单已完成')
}

onMounted(() => fetchOrders())
</script>

<style scoped>
.page-container { padding: 20px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
.search-form { margin-bottom: 20px; }
.pagination { margin-top: 20px; display: flex; justify-content: flex-end; }
</style>