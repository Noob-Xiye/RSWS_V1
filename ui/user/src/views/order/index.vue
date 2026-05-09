<template>
  <div class="orders-page">
    <el-container>
      <el-header class="header">
        <div class="logo" @click="$router.push('/')">RSWS</div>
        <el-menu mode="horizontal" router>
          <el-menu-item index="/">首页</el-menu-item>
          <el-menu-item index="/orders">我的订单</el-menu-item>
        </el-menu>
        <div class="user-area">
          <el-dropdown>
            <span class="user-link">
              <el-icon><User /></el-icon>
              {{ userStore.username }}
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="$router.push('/user')">用户中心</el-dropdown-item>
                <el-dropdown-item @click="handleLogout">退出</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-header>
      <el-main class="main">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>我的订单</span>
              <el-select v-model="statusFilter" placeholder="状态筛选" clearable size="small" @change="fetchOrders" style="width: 120px">
                <el-option label="全部" value="" />
                <el-option label="待支付" value="pending" />
                <el-option label="已支付" value="paid" />
                <el-option label="已完成" value="completed" />
                <el-option label="已取消" value="cancelled" />
              </el-select>
            </div>
          </template>
          <el-empty v-if="!loading && orders.length === 0" description="暂无订单" />
          <el-table :data="filteredOrders" v-loading="loading" stripe v-else>
            <el-table-column prop="order_no" label="订单号" width="200" />
            <el-table-column prop="resource_title" label="资源" min-width="150">
              <template #default="{ row }">
                <span class="resource-title" @click="$router.push(`/resource/${row.resource_id}`)">{{ row.resource_title || `资源 #${row.resource_id}` }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="amount" label="金额" width="120">
              <template #default="{ row }">{{ (row.amount / 100).toFixed(2) }} USDT</template>
            </el-table-column>
            <el-table-column prop="payment_method" label="支付方式" width="120">
              <template #default="{ row }">{{ getPaymentMethodText(row.payment_method) }}</template>
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
                <template v-if="row.status === 'pending'">
                  <el-button type="primary" size="small" link @click="handlePay(row)">支付</el-button>
                  <el-button type="danger" size="small" link @click="handleCancel(row)">取消</el-button>
                </template>
                <template v-else-if="row.status === 'completed'">
                  <el-button type="primary" size="small" link @click="handleDownload(row)">下载</el-button>
                </template>
                <template v-else-if="row.status === 'paid'">
                  <el-button type="info" size="small" link disabled>处理中</el-button>
                </template>
              </template>
            </el-table-column>
          </el-table>
          <div class="pagination" v-if="total > pageSize">
            <el-pagination
              v-model:current-page="page"
              :page-size="pageSize"
              :total="total"
              layout="prev, pager, next"
              @current-change="fetchOrders"
            />
          </div>
        </el-card>
      </el-main>
    </el-container>

    <!-- USDT 支付对话框 -->
    <el-dialog v-model="usdtDialogVisible" title="USDT 支付" width="500px" :close-on-click-modal="false">
      <div class="usdt-payment" v-if="currentOrder">
        <el-alert type="info" :closable="false" style="margin-bottom: 20px">
          <template #title>请向以下地址转账 {{ currentOrder ? (currentOrder.amount / 100).toFixed(2) : 0 }} USDT</template>
        </el-alert>
        <el-form label-width="100px">
          <el-form-item label="网络">
            <el-tag>{{ currentOrder.payment_method === 'usdt_trc20' ? 'TRC-20 (Tron)' : 'ERC-20 (Ethereum)' }}</el-tag>
          </el-form-item>
          <el-form-item label="收款地址">
            <div class="address-box">
              <code>{{ usdtAddress }}</code>
              <el-button type="primary" size="small" @click="copyAddress">复制</el-button>
            </div>
          </el-form-item>
          <el-form-item label="转账金额">
            <span class="amount">{{ currentOrder ? (currentOrder.amount / 100).toFixed(2) : 0 }} USDT</span>
          </el-form-item>
        </el-form>
        <el-alert type="warning" :closable="false">
          <template #title>注意事项</template>
          <ul style="margin: 0; padding-left: 20px; font-size: 12px">
            <li>请确保转账金额准确</li>
            <li>转账完成后请等待链上确认</li>
            <li>订单将在 {{ formatExpiredTime(currentOrder?.expired_at) }} 过期</li>
          </ul>
        </el-alert>
        <div class="payment-status" v-if="pollingStatus">
          <el-icon class="is-loading"><Loading /></el-icon>
          <span>正在等待支付确认...</span>
        </div>
      </div>
      <template #footer>
        <el-button @click="usdtDialogVisible = false">关闭</el-button>
        <el-button type="primary" @click="refreshOrderStatus" :loading="pollingStatus">刷新状态</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { listOrders, cancelOrder, checkOrderStatus, getUsdtAddress, type Order } from '@/api/order'

const router = useRouter()
const userStore = useUserStore()

const loading = ref(false)
const orders = ref<Order[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = 20
const statusFilter = ref('')

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
  const map: Record<string, '' | 'success' | 'warning' | 'info' | 'danger'> = {
    pending: 'warning',
    paid: '',
    completed: 'success',
    cancelled: 'info',
    failed: 'danger',
    refunded: 'info'
  }
  return map[status] || 'info'
}

function getStatusText(status: string) {
  const map: Record<string, string> = {
    pending: '待支付',
    paid: '已支付',
    completed: '已完成',
    cancelled: '已取消',
    failed: '失败',
    refunded: '已退款'
  }
  return map[status] || status
}

function getPaymentMethodText(method: string) {
  const map: Record<string, string> = {
    paypal: 'PayPal',
    usdt_trc20: 'USDT-TRC20',
    usdt_erc20: 'USDT-ERC20'
  }
  return map[method] || method
}

function formatDate(dateStr: string) {
  if (!dateStr) return ''
  return dateStr.substring(0, 19).replace('T', ' ')
}

function formatExpiredTime(dateStr: string | undefined) {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

async function fetchOrders() {
  if (!userStore.isLoggedIn) {
    router.push('/login')
    return
  }
  loading.value = true
  try {
    const res = await listOrders({ page: page.value, limit: pageSize })
    if (res.success && res.data) {
      orders.value = res.data.items
      total.value = res.data.total
    }
  } catch {
    orders.value = []
  } finally {
    loading.value = false
  }
}

function handleLogout() {
  userStore.logout()
  router.push('/')
}

async function handlePay(order: Order) {
  currentOrder.value = order
  
  if (order.payment_method === 'paypal') {
    // PayPal 支付 - 跳转到 PayPal
    ElMessage.info('PayPal 支付功能开发中')
    return
  }
  
  // USDT 支付
  const network = order.payment_method === 'usdt_trc20' ? 'tron' : 'ethereum'
  try {
    const res = await getUsdtAddress(network)
    if (res.success && res.data) {
      usdtAddress.value = res.data.address
      usdtDialogVisible.value = true
      startPolling()
    } else {
      ElMessage.error('获取收款地址失败')
    }
  } catch {
    ElMessage.error('获取收款地址失败')
  }
}

async function handleCancel(order: Order) {
  try {
    await ElMessageBox.confirm('确定要取消该订单吗？', '提示', { type: 'warning' })
    const res = await cancelOrder(order.id)
    if (res.success) {
      ElMessage.success('订单已取消')
      fetchOrders()
    } else {
      ElMessage.error(res.message || '取消失败')
    }
  } catch {
    // 用户取消
  }
}

async function handleDownload(order: Order) {
  // 跳转到资源详情页下载
  router.push(`/resource/${order.resource_id}`)
}

function copyAddress() {
  navigator.clipboard.writeText(usdtAddress.value)
  ElMessage.success('已复制到剪贴板')
}

function startPolling() {
  pollingStatus.value = true
  pollingTimer = setInterval(async () => {
    if (!currentOrder.value) return
    try {
      const res = await checkOrderStatus(currentOrder.value.id)
      if (res.success && res.data) {
        if (res.data.status === 'completed' || res.data.status === 'paid') {
          stopPolling()
          usdtDialogVisible.value = false
          ElMessage.success('支付成功！')
          fetchOrders()
        }
      }
    } catch {
      // ignore
    }
  }, 5000)
}

function stopPolling() {
  pollingStatus.value = false
  if (pollingTimer) {
    clearInterval(pollingTimer)
    pollingTimer = null
  }
}

async function refreshOrderStatus() {
  if (!currentOrder.value) return
  try {
    const res = await checkOrderStatus(currentOrder.value.id)
    if (res.success && res.data) {
      if (res.data.status === 'completed' || res.data.status === 'paid') {
        stopPolling()
        usdtDialogVisible.value = false
        ElMessage.success('支付成功！')
        fetchOrders()
      } else {
        ElMessage.info(`当前状态: ${getStatusText(res.data.status)}`)
      }
    }
  } catch {
    ElMessage.error('查询失败')
  }
}

onMounted(() => fetchOrders())
onUnmounted(() => stopPolling())
</script>

<style scoped>
.orders-page {
  min-height: 100vh;
  background: #f5f5f5;
}
.header {
  background: #fff;
  display: flex;
  align-items: center;
  padding: 0 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
.logo {
  font-size: 24px;
  font-weight: bold;
  color: #409eff;
  cursor: pointer;
  margin-right: 40px;
}
.user-area {
  margin-left: auto;
}
.user-link {
  display: flex;
  align-items: center;
  gap: 5px;
  cursor: pointer;
}
.main {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.resource-title {
  color: #409eff;
  cursor: pointer;
}
.resource-title:hover {
  text-decoration: underline;
}
.pagination {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}
.usdt-payment .address-box {
  display: flex;
  align-items: center;
  gap: 10px;
}
.usdt-payment .address-box code {
  background: #f5f5f5;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 12px;
  word-break: break-all;
}
.usdt-payment .amount {
  font-size: 20px;
  font-weight: bold;
  color: #f56c6c;
}
.usdt-payment .payment-status {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  margin-top: 20px;
  color: #409eff;
}
</style>