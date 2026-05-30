<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>用户订单查询</span>
          <div class="header-right">
            <el-select v-model="filterUserId" placeholder="筛选用户（留空查全部）" clearable filterable style="width: 220px" @change="fetchOrders">
              <el-option
                v-for="u in users"
                :key="u.id"
                :label="`${u.username || u.nickname || u.email} (ID: ${u.id})`"
                :value="u.id"
              />
            </el-select>
            <el-select v-model="filterStatus" placeholder="订单状态" clearable style="width: 140px" @change="fetchOrders">
              <el-option label="全部" :value="''" />
              <el-option label="待支付" value="pending" />
              <el-option label="已支付" value="paid" />
              <el-option label="已完成" value="completed" />
              <el-option label="已取消" value="cancelled" />
              <el-option label="已退款" value="refunded" />
            </el-select>
            <el-button @click="fetchOrders">刷新</el-button>
          </div>
        </div>
      </template>

      <el-table :data="orders" v-loading="loading" stripe>
        <el-table-column prop="id" label="订单号" width="180">
          <template #default="{ row }">
            <span class="order-id">{{ row.id }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="user_id" label="用户 ID" width="120" />
        <el-table-column prop="resource_id" label="资源 ID" width="100" />
        <el-table-column prop="resource_title" label="资源名称" min-width="180" show-overflow-tooltip />
        <el-table-column prop="amount" label="金额" width="100" align="right">
          <template #default="{ row }">${{ row.amount }}</template>
        </el-table-column>
        <el-table-column prop="payment_method" label="支付方式" width="110">
          <template #default="{ row }">
            <el-tag size="small" :type="methodTag(row.payment_method)">
              {{ methodLabel(row.payment_method) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="90" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="statusTag(row.status)">{{ statusLabel(row.status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="下单时间" width="160">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
        <el-table-column prop="paid_at" label="支付时间" width="160">
          <template #default="{ row }">{{ row.paid_at ? formatDate(row.paid_at) : '-' }}</template>
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
          @change="fetchOrders"
        />
      </div>
    </el-card>

    <!-- 订单详情弹窗 -->
    <el-dialog v-model="detailVisible" title="订单详情" width="560px">
      <el-descriptions v-if="detail" :column="2" border size="small">
        <el-descriptions-item label="订单号">{{ detail.id }}</el-descriptions-item>
        <el-descriptions-item label="用户 ID">{{ detail.user_id }}</el-descriptions-item>
        <el-descriptions-item label="资源 ID">{{ detail.resource_id }}</el-descriptions-item>
        <el-descriptions-item label="金额">${{ detail.amount }}</el-descriptions-item>
        <el-descriptions-item label="支付方式">{{ methodLabel(detail.payment_method) }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag size="small" :type="statusTag(detail.status)">{{ statusLabel(detail.status) }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="创建时间">{{ formatDate(detail.created_at) }}</el-descriptions-item>
        <el-descriptions-item label="支付时间">{{ detail.paid_at ? formatDate(detail.paid_at) : '-' }}</el-descriptions-item>
        <el-descriptions-item label="交易哈希" :span="2">{{ detail.transaction_hash || '-' }}</el-descriptions-item>
        <el-descriptions-item v-if="detail.payment_details" label="支付详情" :span="2">
          <pre class="detail-json">{{ JSON.stringify(detail.payment_details, null, 2) }}</pre>
        </el-descriptions-item>
      </el-descriptions>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { listUsers } from '@/api/user'
import { adminListOrders as listOrders } from '@/api/order'

const loading = ref(false)
const orders = ref<any[]>([])
const users = ref<any[]>([])
const filterUserId = ref<number | null>(null)
const filterStatus = ref('')
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const detailVisible = ref(false)
const detail = ref<any>(null)

async function fetchUsers() {
  try {
    const res = await listUsers()
    if (res.code === 0) {
      users.value = res.data?.users || res.data?.items || []
    }
  } catch { /* ignore */ }
}

async function fetchOrders() {
  loading.value = true
  try {
    const params: Record<string, any> = { page: page.value, page_size: pageSize.value }
    if (filterUserId.value) params.user_id = filterUserId.value
    if (filterStatus.value) params.status = filterStatus.value
    const res = await listOrders(params)
    if (res.code === 0) {
      const d = res.data
      orders.value = d.orders || d.items || []
      total.value = d.total || 0
    } else {
      ElMessage.error(res.msg || '获取失败')
    }
  } catch {
    ElMessage.error('获取订单列表失败')
  } finally {
    loading.value = false
  }
}

function viewDetail(row: any) {
  detail.value = row
  detailVisible.value = true
}

function formatDate(d: string) {
  return d ? new Date(d).toLocaleString('zh-CN') : '-'
}

function statusTag(s: string) {
  const map: Record<string, any> = {
    pending: 'warning', paid: 'success', completed: 'success',
    cancelled: 'info', refunded: 'danger',
  }
  return map[s] || ''
}
function statusLabel(s: string) {
  const map: Record<string, string> = {
    pending: '待支付', paid: '已支付', completed: '已完成',
    cancelled: '已取消', refunded: '已退款',
  }
  return map[s] || s
}
function methodLabel(m: string) {
  const map: Record<string, string> = {
    paypal: 'PayPal', usdt_trc20: 'USDT (TRC20)', usdt_erc20: 'USDT (ERC20)',
  }
  return map[m] || m || '-'
}
function methodTag(m: string) {
  const map: Record<string, any> = { paypal: '', usdt_trc20: 'warning', usdt_erc20: 'warning' }
  return map[m] || ''
}

onMounted(() => { fetchUsers(); fetchOrders() })
</script>

<style scoped>
.page-container { padding: 20px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
.header-right { display: flex; gap: 8px; align-items: center; }
.order-id { font-family: monospace; font-size: 12px; }
.pagination { margin-top: 16px; display: flex; justify-content: flex-end; }
.detail-json { font-size: 11px; color: #666; margin: 0; white-space: pre-wrap; }
</style>
