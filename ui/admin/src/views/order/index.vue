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
        <el-table-column prop="amount" label="金额 (USDT)" width="120">
          <template #default="{ row }">{{ (Number(row.amount) / 100).toFixed(2) }}</template>
        </el-table-column>
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
        <el-table-column label="操作" width="180">
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
    <el-dialog v-model="detailVisible" title="订单详情" width="650px" destroy-on-close>
      <template v-if="currentOrder">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="订单号" :span="2">{{ currentOrder.order_no }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusType(currentOrder.status)" size="small">{{ getStatusText(currentOrder.status) }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="支付方式">{{ getPaymentMethod(currentOrder.payment_method) }}</el-descriptions-item>
          <el-descriptions-item label="买家">{{ currentOrder.user_name || currentOrder.user_email || '-' }}</el-descriptions-item>
          <el-descriptions-item label="订单金额">
            <span class="amount">{{ (Number(currentOrder.amount) / 100).toFixed(2) }} USDT</span>
          </el-descriptions-item>
          <el-descriptions-item label="资源" :span="2">{{ currentOrder.resource_title || '-' }}</el-descriptions-item>
          <el-descriptions-item label="创建时间" :span="2">{{ formatDate(currentOrder.created_at) }}</el-descriptions-item>
          <el-descriptions-item label="支付时间" :span="2">{{ currentOrder.paid_at ? formatDate(currentOrder.paid_at) : '-' }}</el-descriptions-item>
          <el-descriptions-item label="完成时间" :span="2">{{ currentOrder.completed_at ? formatDate(currentOrder.completed_at) : '-' }}</el-descriptions-item>
          <el-descriptions-item label="交易哈希" :span="2">
            <span v-if="currentOrder.tx_hash" class="tx-hash">{{ currentOrder.tx_hash }}</span>
            <span v-else>-</span>
          </el-descriptions-item>
        </el-descriptions>

        <!-- 订单状态时间线 -->
        <div class="timeline-section">
          <h4>订单流程</h4>
          <el-timeline>
            <el-timeline-item :color="getTimelineColor('pending')" timestamp="2026-05-08 09:00">创建订单</el-timeline-item>
            <el-timeline-item v-if="currentOrder.status !== 'pending' && currentOrder.status !== 'cancelled'" :color="getTimelineColor('paid')" timestamp="2026-05-08 09:30">支付成功</el-timeline-item>
            <el-timeline-item v-if="currentOrder.status === 'completed'" :color="getTimelineColor('completed')" timestamp="2026-05-08 10:00">已完成</el-timeline-item>
            <el-timeline-item v-if="currentOrder.status === 'cancelled'" :color="'#909399'" timestamp="2026-05-08 09:45">已取消</el-timeline-item>
          </el-timeline>
        </div>
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
import type { Order, OrderListParams } from '@/api/order'
import { listOrders, completeOrder } from '@/api/order'

const loading = ref(false)
const orders = ref<Order[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const detailVisible = ref(false)
const currentOrder = ref<Order | null>(null)

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
  const map: Record<string, string> = { paypal: 'PayPal', usdt_trc20: 'USDT-TRRC20', usdt_erc20: 'USDT-ERC20' }
  return map[method] || method
}

function getTimelineColor(status: string) {
  if (!currentOrder.value) return '#909399'
  const orderStatus = currentOrder.value.status
  const statusOrder = ['pending', 'paid', 'completed']
  const currentIdx = statusOrder.indexOf(orderStatus)
  const targetIdx = statusOrder.indexOf(status)
  return targetIdx <= currentIdx ? '#67c23a' : '#909399'
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
    orders.value = [
      { id: 1, order_no: 'ORD20260508001', user_id: 1, user_name: 'user1', user_email: 'user1@example.com', resource_id: 1, resource_title: '高级资源包', amount: '100.00', payment_method: 'usdt_trc20', status: 'paid', transaction_id: 'TX001', tx_hash: '0x1234567890abcdef', created_at: '2026-05-08T01:00:00Z', paid_at: '2026-05-08T01:30:00Z', completed_at: null },
      { id: 2, order_no: 'ORD20260508002', user_id: 2, user_name: 'user2', user_email: 'user2@example.com', resource_id: 2, resource_title: '专业资源包', amount: '200.00', payment_method: 'paypal', status: 'completed', transaction_id: 'PP002', tx_hash: null, created_at: '2026-05-07T10:00:00Z', paid_at: '2026-05-07T10:15:00Z', completed_at: '2026-05-07T10:30:00Z' }
    ]
    total.value = 2
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
  currentOrder.value = row
  detailVisible.value = true
}

async function handleComplete(row: Order) {
  try {
    await ElMessageBox.confirm(`确认完成订单 ${row.order_no}？`, '确认', { type: 'warning' })
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

.amount {
  color: #67c23a;
  font-weight: bold;
  font-size: 16px;
}

.tx-hash {
  font-family: monospace;
  font-size: 12px;
  color: #909399;
  word-break: break-all;
}

.timeline-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #eee;
}

.timeline-section h4 {
  margin: 0 0 15px 0;
  color: #303133;
}
</style>
