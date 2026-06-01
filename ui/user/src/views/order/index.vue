<template>
  <ModernLayout>
    <div class="orders-page">
      <div class="orders-container">
        <div class="page-header">
          <h1 class="page-title">
            <span class="gradient-text">我的订单</span>
          </h1>
          <div class="filter-bar">
            <button
              v-for="f in statusFilters"
              :key="f.value"
              class="filter-btn"
              :class="{ active: statusFilter === f.value }"
              @click="statusFilter = f.value; fetchOrders()"
            >{{ f.label }}</button>
          </div>
        </div>

        <div v-if="loading" class="loading-state">
          <el-icon class="is-loading" :size="32"><Loading /></el-icon>
        </div>

        <div v-else-if="filteredOrders.length === 0" class="empty-state">
          <el-icon :size="64"><FolderOpened /></el-icon>
          <p>暂无订单</p>
        </div>

        <div v-else class="order-list">
          <div v-for="order in filteredOrders" :key="order.id" class="order-card">
            <div class="order-header">
              <span class="order-no">#{{ order.id }}</span>
              <el-tag :type="getStatusType(order.status)" size="small" effect="dark">{{ getStatusText(order.status) }}</el-tag>
            </div>
            <div class="order-body">
              <div class="order-resource" @click="$router.push(`/resource/${order.resource_id}`)">
                <el-icon><Document /></el-icon>
                <span>{{ order.resource_title || `资源 #${order.resource_id}` }}</span>
              </div>
              <div class="order-details">
                <div class="detail-item">
                  <span class="detail-label">金额</span>
                  <span class="detail-value price">{{ (order.amount / 100).toFixed(2) }} USDT</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">支付方式</span>
                  <span class="detail-value">{{ getPaymentMethodText(order.payment_method) }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">创建时间</span>
                  <span class="detail-value">{{ formatDate(order.created_at) }}</span>
                </div>
              </div>
            </div>
            <div class="order-actions" v-if="order.status === 'pending' || order.status === 'completed'">
              <button v-if="order.status === 'pending'" class="btn-pay" @click="handlePay(order)">立即支付</button>
              <button v-if="order.status === 'pending'" class="btn-cancel" @click="handleCancel(order)">取消订单</button>
              <button v-if="order.status === 'completed'" class="btn-download" @click="handleDownload(order)">下载资源</button>
            </div>
          </div>
        </div>

        <div v-if="total > pageSize" class="pagination">
          <el-pagination
            v-model:current-page="page"
            :page-size="pageSize"
            :total="total"
            layout="prev, pager, next"
            :background="true"
            @current-change="fetchOrders"
          />
        </div>
      </div>
    </div>

    <!-- USDT 支付对话框 -->
    <el-dialog v-model="usdtDialogVisible" title="USDT 支付" width="500px" :close-on-click-modal="false">
      <div class="usdt-payment" v-if="currentOrder">
        <div class="payment-info-banner">
          <p>请向以下地址转账 <strong>{{ (currentOrder.amount / 100).toFixed(2) }} USDT</strong></p>
        </div>
        <div class="payment-detail-list">
          <div class="payment-detail-item">
            <span class="detail-label">网络</span>
            <el-tag size="small" effect="dark">{{ currentOrder.payment_method === 'usdt_trc20' ? 'TRC-20 (Tron)' : 'ERC-20 (Ethereum)' }}</el-tag>
          </div>
          <div class="payment-detail-item">
            <span class="detail-label">收款地址</span>
            <div class="address-box">
              <code>{{ usdtAddress }}</code>
              <el-button type="primary" size="small" @click="copyAddress">复制</el-button>
            </div>
          </div>
          <div class="payment-detail-item">
            <span class="detail-label">转账金额</span>
            <span class="detail-amount">{{ (currentOrder.amount / 100).toFixed(2) }} USDT</span>
          </div>
        </div>
        <div class="payment-notice">
          <h4>⚠️ 注意事项</h4>
          <ul>
            <li>请确保转账金额准确</li>
            <li>转账完成后请等待链上确认</li>
            <li>订单将在 {{ formatExpiredTime(currentOrder?.expired_at) }} 过期</li>
          </ul>
        </div>
        <div class="payment-polling" v-if="pollingStatus">
          <el-icon class="is-loading"><Loading /></el-icon>
          <span>正在等待支付确认...</span>
        </div>
      </div>
      <template #footer>
        <el-button @click="usdtDialogVisible = false">关闭</el-button>
        <el-button type="primary" @click="refreshOrderStatus" :loading="pollingStatus">刷新状态</el-button>
      </template>
    </el-dialog>
  </ModernLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { listOrders, cancelOrder, checkOrderStatus, getUsdtAddress, type Order } from '@/api/order'
import ModernLayout from '@/components/ModernLayout.vue'

const router = useRouter()
const userStore = useUserStore()

const loading = ref(false)
const orders = ref<Order[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = 20
const statusFilter = ref('')

const statusFilters = [
  { label: '全部', value: '' },
  { label: '待支付', value: 'pending' },
  { label: '已支付', value: 'paid' },
  { label: '已完成', value: 'completed' },
  { label: '已取消', value: 'cancelled' }
]

const usdtDialogVisible = ref(false)
const currentOrder = ref<Order | null>(null)
const usdtAddress = ref('')
const pollingStatus = ref(false)
let pollingTimer: ReturnType<typeof setInterval> | null = null

const filteredOrders = computed(() => {
  if (!statusFilter.value) return orders.value
  return orders.value.filter(o => o.status === statusFilter.value)
})

function getStatusType(status: string) {
  const map: Record<string, '' | 'success' | 'warning' | 'info' | 'danger'> = { pending: 'warning', paid: '', completed: 'success', cancelled: 'info', failed: 'danger', refunded: 'info' }
  return map[status] || 'info'
}

function getStatusText(status: string) {
  const map: Record<string, string> = { pending: '待支付', paid: '已支付', completed: '已完成', cancelled: '已取消', failed: '失败', refunded: '已退款' }
  return map[status] || status
}

function getPaymentMethodText(method?: string) {
  const map: Record<string, string> = { paypal: 'PayPal', usdt_trc20: 'USDT-TRC20', usdt_erc20: 'USDT-ERC20' }
  return method ? (map[method] || method) : '-'
}

function formatDate(dateStr: string) { return dateStr ? dateStr.substring(0, 19).replace('T', ' ') : '' }
function formatExpiredTime(dateStr: string | undefined) { return dateStr ? new Date(dateStr).toLocaleString('zh-CN') : '' }

async function fetchOrders() {
  if (!userStore.isLoggedIn) { router.push('/login'); return }
  loading.value = true
  try {
    const res = await listOrders({ page: page.value, page_size: pageSize })
    if (res.code === 0 && res.data) { orders.value = res.data.items; total.value = res.data.total }
  } catch { orders.value = [] }
  finally { loading.value = false }
}

async function handlePay(order: Order) {
  currentOrder.value = order
  if (order.payment_method === 'paypal') {
    try {
      const res = await initiatePayment(order.id)
      if (res.code === 0 && res.data?.approve_url) {
        window.location.href = res.data.approve_url
      } else {
        ElMessage.error(res.msg || '获取支付链接失败，请稍后重试')
      }
    } catch (e) {
      ElMessage.error('PayPal 支付暂不可用，请尝试 USDT 支付')
    }
    return
  }
  const network = order.payment_method === 'usdt_trc20' ? 'tron' : 'ethereum'
  try {
    const res = await getUsdtAddress(network)
    if (res.code === 0 && res.data) { usdtAddress.value = res.data.address; usdtDialogVisible.value = true; startPolling() }
    else ElMessage.error('获取收款地址失败')
  } catch { ElMessage.error('获取收款地址失败') }
}

async function handleCancel(order: Order) {
  try {
    await ElMessageBox.confirm('确定要取消该订单吗？', '提示', { type: 'warning' })
    const res = await cancelOrder(order.id)
    if (res.code === 0) { ElMessage.success('订单已取消'); fetchOrders() }
    else ElMessage.error(res.msg || '取消失败')
  } catch {}
}

function handleDownload(order: Order) { router.push(`/resource/${order.resource_id}`) }
function copyAddress() { navigator.clipboard.writeText(usdtAddress.value); ElMessage.success('已复制到剪贴板') }

function startPolling() {
  pollingStatus.value = true
  pollingTimer = setInterval(async () => {
    if (!currentOrder.value) return
    try {
      const res = await checkOrderStatus(currentOrder.value.id)
      if (res.code === 0 && res.data && (res.data.status === 'completed' || res.data.status === 'paid')) {
        stopPolling(); usdtDialogVisible.value = false; ElMessage.success('支付成功！'); fetchOrders()
      }
    } catch {}
  }, 5000)
}

function stopPolling() {
  pollingStatus.value = false
  if (pollingTimer) { clearInterval(pollingTimer); pollingTimer = null }
}

async function refreshOrderStatus() {
  if (!currentOrder.value) return
  try {
    const res = await checkOrderStatus(currentOrder.value.id)
    if (res.code === 0 && res.data) {
      if (res.data.status === 'completed' || res.data.status === 'paid') {
        stopPolling(); usdtDialogVisible.value = false; ElMessage.success('支付成功！'); fetchOrders()
      } else { ElMessage.info(`当前状态: ${getStatusText(res.data.status)}`) }
    }
  } catch { ElMessage.error('查询失败') }
}

onMounted(() => fetchOrders())
onUnmounted(() => stopPolling())
</script>

<style scoped>
.orders-page { padding: 32px 24px; }
.orders-container { max-width: 1000px; margin: 0 auto; }

.page-header { margin-bottom: 32px; }
.page-title { font-size: 28px; font-weight: 700; margin: 0 0 20px; }
.gradient-text {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.filter-bar { display: flex; gap: 8px; flex-wrap: wrap; }

.filter-btn {
  padding: 8px 18px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.3s;
}

.filter-btn:hover { background: rgba(255, 255, 255, 0.1); color: #fff; }

.filter-btn.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: transparent;
  color: #fff;
}

/* Loading */
.loading-state { text-align: center; padding: 60px; color: #667eea; }

/* 空状态 */
.empty-state { text-align: center; padding: 80px 20px; color: rgba(255, 255, 255, 0.5); }
.empty-state p { margin-top: 16px; font-size: 16px; }

/* 订单列表 */
.order-list { display: flex; flex-direction: column; gap: 16px; }

.order-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 24px;
  transition: border-color 0.3s;
}

.order-card:hover { border-color: rgba(255, 255, 255, 0.2); }

.order-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.order-no {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  font-family: monospace;
}

.order-body { margin-bottom: 16px; }

.order-resource {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: #667eea;
  cursor: pointer;
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 12px;
  transition: color 0.3s;
}

.order-resource:hover { color: #764ba2; }

.order-details { display: flex; gap: 24px; flex-wrap: wrap; }

.detail-item { display: flex; flex-direction: column; gap: 2px; }
.detail-label { font-size: 12px; color: rgba(255, 255, 255, 0.4); }
.detail-value { font-size: 14px; color: rgba(255, 255, 255, 0.8); }
.detail-value.price {
  font-weight: 700;
  background: linear-gradient(135deg, #f093fb, #f5576c);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.order-actions { display: flex; gap: 10px; }

.btn-pay, .btn-cancel, .btn-download {
  padding: 8px 20px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-pay {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

.btn-pay:hover { box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4); }

.btn-cancel {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.7);
}

.btn-cancel:hover { background: rgba(255, 255, 255, 0.1); color: #fff; }

.btn-download {
  background: linear-gradient(135deg, #67c23a 0%, #42b983 100%);
  color: #fff;
}

.btn-download:hover { box-shadow: 0 4px 12px rgba(103, 194, 58, 0.4); }

/* 分页 */
.pagination {
  display: flex;
  justify-content: center;
  margin-top: 40px;
}

.pagination :deep(.el-pagination.is-background .el-pager li) {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
}

.pagination :deep(.el-pagination.is-background .el-pager li:hover) { color: #fff; }

.pagination :deep(.el-pagination.is-background .el-pager li.is-active) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

/* USDT 支付对话框 */
.payment-info-banner {
  background: rgba(102, 126, 234, 0.15);
  border: 1px solid rgba(102, 126, 234, 0.3);
  border-radius: 12px;
  padding: 16px;
  text-align: center;
  margin-bottom: 20px;
  color: rgba(255, 255, 255, 0.9);
}

.payment-detail-list { display: flex; flex-direction: column; gap: 14px; margin-bottom: 20px; }
.payment-detail-item { display: flex; flex-direction: column; gap: 4px; }
.detail-label { font-size: 13px; color: rgba(255, 255, 255, 0.5); }

.detail-amount {
  font-size: 22px;
  font-weight: 700;
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.address-box { display: flex; align-items: center; gap: 10px; }
.address-box code {
  background: rgba(255, 255, 255, 0.1);
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 12px;
  word-break: break-all;
  color: rgba(255, 255, 255, 0.8);
}

.payment-notice {
  background: rgba(230, 162, 60, 0.1);
  border: 1px solid rgba(230, 162, 60, 0.3);
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 16px;
}

.payment-notice h4 { margin: 0 0 8px; color: #e6a23c; font-size: 14px; }
.payment-notice ul { margin: 0; padding-left: 20px; font-size: 12px; color: rgba(255, 255, 255, 0.6); }

.payment-polling { display: flex; align-items: center; justify-content: center; gap: 10px; margin-top: 16px; color: #667eea; }

@media (max-width: 768px) {
  .order-details { flex-direction: column; gap: 8px; }
  .order-actions { flex-direction: column; }
}
</style>
