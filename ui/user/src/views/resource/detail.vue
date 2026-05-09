<template>
  <div class="detail-page">
    <el-container>
      <el-header class="header">
        <div class="logo" @click="$router.push('/')">RSWS</div>
        <el-menu mode="horizontal" router>
          <el-menu-item index="/">首页</el-menu-item>
          <el-menu-item index="/orders">我的订单</el-menu-item>
        </el-menu>
        <div class="user-area">
          <template v-if="userStore.isLoggedIn">
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
          </template>
          <template v-else>
            <el-button type="primary" @click="$router.push('/login')">登录</el-button>
          </template>
        </div>
      </el-header>
      <el-main class="main">
        <el-skeleton :loading="loading" animated>
          <template #default>
            <el-row :gutter="20" v-if="resource">
              <el-col :span="16">
                <el-card>
                  <template #header>
                    <div class="resource-header">
                      <h2>{{ resource.title }}</h2>
                      <el-tag>{{ resource.category_id || '未分类' }}</el-tag>
                    </div>
                  </template>
                  <div class="cover-image" v-if="resource.cover_image">
                    <el-image :src="resource.cover_image" fit="cover" />
                  </div>
                  <div class="description">
                    <h4>简介</h4>
                    <p>{{ resource.description }}</p>
                  </div>
                  <div class="content" v-if="resource.content">
                    <h4>详细内容</h4>
                    <div v-html="resource.content"></div>
                  </div>
                </el-card>
              </el-col>
              <el-col :span="8">
                <el-card class="purchase-card">
                  <div class="price-section">
                    <span class="label">价格</span>
                    <span class="price">{{ formatPrice(resource.price) }} USDT</span>
                  </div>
                  <div class="info-section">
                    <div class="info-item">
                      <el-icon><Download /></el-icon>
                      <span>{{ resource.download_count || 0 }} 次下载</span>
                    </div>
                    <div class="info-item">
                      <el-icon><Calendar /></el-icon>
                      <span>{{ formatDate(resource.created_at) }}</span>
                    </div>
                  </div>
                  <el-divider />
                  
                  <!-- 已购买：显示下载按钮 -->
                  <template v-if="isPurchased">
                    <el-alert type="success" :closable="false" style="margin-bottom: 16px">
                      <template #title>您已购买此资源</template>
                    </el-alert>
                    <el-button type="primary" size="large" :loading="downloading" @click="handleDownload">
                      <el-icon><Download /></el-icon>
                      下载资源
                    </el-button>
                  </template>
                  
                  <!-- 未购买：显示购买按钮 -->
                  <template v-else>
                    <div class="payment-method">
                      <span class="label">支付方式</span>
                      <el-radio-group v-model="paymentMethod">
                        <el-radio-button value="usdt_trc20">USDT-TRC20</el-radio-button>
                        <el-radio-button value="usdt_erc20">USDT-ERC20</el-radio-button>
                        <el-radio-button value="paypal">PayPal</el-radio-button>
                      </el-radio-group>
                    </div>
                    <el-button type="primary" size="large" :loading="purchasing" @click="handlePurchase">
                      立即购买
                    </el-button>
                  </template>
                </el-card>
              </el-col>
            </el-row>
          </template>
        </el-skeleton>
      </el-main>
    </el-container>

    <!-- USDT 支付对话框 -->
    <el-dialog v-model="usdtDialogVisible" title="USDT 支付" width="500px" :close-on-click-modal="false">
      <div class="usdt-payment">
        <el-alert type="info" :closable="false" style="margin-bottom: 20px">
          <template #title>请向以下地址转账 {{ resource?.price }} USDT</template>
        </el-alert>
        <el-form label-width="100px">
          <el-form-item label="网络">
            <el-tag>{{ paymentMethod === 'usdt_trc20' ? 'TRC-20 (Tron)' : 'ERC-20 (Ethereum)' }}</el-tag>
          </el-form-item>
          <el-form-item label="收款地址">
            <div class="address-box">
              <code>{{ usdtAddress }}</code>
              <el-button type="primary" size="small" @click="copyAddress">复制</el-button>
            </div>
          </el-form-item>
          <el-form-item label="转账金额">
            <span class="amount">{{ resource?.price }} USDT</span>
          </el-form-item>
          <el-form-item label="订单号">
            <span>{{ currentOrderNo }}</span>
          </el-form-item>
        </el-form>
        <el-alert type="warning" :closable="false">
          <template #title>注意事项</template>
          <ul style="margin: 0; padding-left: 20px; font-size: 12px">
            <li>请确保转账金额准确</li>
            <li>转账完成后请等待链上确认</li>
            <li>支付完成后可前往"我的订单"查看状态</li>
          </ul>
        </el-alert>
        <div class="payment-status" v-if="pollingStatus">
          <el-icon class="is-loading"><Loading /></el-icon>
          <span>正在等待支付确认...</span>
        </div>
      </div>
      <template #footer>
        <el-button @click="usdtDialogVisible = false">关闭</el-button>
        <el-button type="primary" @click="goToOrders">查看订单</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { getResource, checkPurchase, getDownloadInfo } from '@/api/resource'
import { createOrder, checkOrderStatus, getUsdtAddress } from '@/api/order'

const route = useRoute()
const router = useRouter()
const userStore = useUserStore()

const loading = ref(true)
const resource = ref<ResourceDetail | null>(null)
const isPurchased = ref(false)
const downloading = ref(false)
const purchasing = ref(false)
const paymentMethod = ref<'usdt_trc20' | 'usdt_erc20' | 'paypal'>('usdt_trc20')

const usdtDialogVisible = ref(false)
const usdtAddress = ref('')
const currentOrderNo = ref('')
const currentOrderId = ref<number | null>(null)
const pollingStatus = ref(false)
let pollingTimer: ReturnType<typeof setInterval> | null = null

async function fetchResource() {
  const id = Number(route.params.id)
  if (!id) return
  
  loading.value = true
  try {
    const res = await getResource(id)
    if (res.success && res.data) {
      resource.value = res.data
      // 检查是否已购买
      await checkPurchasedStatus(id)
    } else {
      ElMessage.error('资源不存在')
      router.push('/')
    }
  } catch {
    ElMessage.error('加载失败')
  } finally {
    loading.value = false
  }
}

async function checkPurchasedStatus(resourceId: number) {
  if (!userStore.isLoggedIn) {
    isPurchased.value = false
    return
  }
  try {
    const res = await checkPurchase(resourceId)
    if (res.success && res.data) {
      isPurchased.value = res.data.purchased || false
    }
  } catch {
    isPurchased.value = false
  }
}

function formatDate(dateStr?: string) {
  if (!dateStr) return ''
  return dateStr.substring(0, 10)
}

function formatPrice(price: number) {
  // 后端 price 是 i64（分），转换为元
  return (price / 100).toFixed(2)
}

function handleLogout() {
  userStore.logout()
  router.push('/')
}

async function handlePurchase() {
  if (!userStore.isLoggedIn) {
    ElMessage.warning('请先登录')
    router.push('/login')
    return
  }
  
  if (!resource.value) return
  
  purchasing.value = true
  try {
    const res = await createOrder({
      resource_id: resource.value.id,
      payment_method: paymentMethod.value
    })
    
    if (res.success && res.data) {
      currentOrderId.value = res.data.id
      currentOrderNo.value = `ORD-${res.data.id}`
      
      if (paymentMethod.value === 'paypal') {
        // PayPal 支付：后端返回 approve_url 直接跳转
        if (res.data.approve_url) {
          window.location.href = res.data.approve_url
        } else if (res.data.paypal_order_id) {
          // 备用：手动构造 PayPal URL
          const paypalUrl = `https://www.sandbox.paypal.com/checkoutnow?token=${res.data.paypal_order_id}`
          window.location.href = paypalUrl
        } else {
          ElMessage.info(res.data.message || 'PayPal 订单已创建，请使用 USDT 支付')
          router.push('/orders')
        }
      } else {
        // USDT 支付
        const network = paymentMethod.value === 'usdt_trc20' ? 'tron' : 'ethereum'
        const addrRes = await getUsdtAddress(network)
        if (addrRes.success && addrRes.data) {
          usdtAddress.value = addrRes.data.address
          usdtDialogVisible.value = true
          startPolling()
        } else {
          ElMessage.error('获取收款地址失败')
        }
      }
    } else {
      ElMessage.error(res.message || '创建订单失败')
    }
  } catch (err: any) {
    ElMessage.error(err?.message || '购买失败')
  } finally {
    purchasing.value = false
  }
}

async function handleDownload() {
  if (!resource.value) return
  
  // 再次检查购买状态
  await checkPurchasedStatus(resource.value.id)
  if (!isPurchased.value) {
    ElMessage.warning('请先购买此资源')
    return
  }
  
  downloading.value = true
  try {
    const res = await getDownloadInfo(resource.value.id)
    if (res.success && res.data) {
      // 触发下载
      const link = document.createElement('a')
      link.href = res.data.file_url
      link.download = res.data.file_name || 'resource'
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
    } else {
      // 如果没有专属下载接口，尝试直接打开 file_url
      if (resource.value.file_url) {
        window.open(resource.value.file_url, '_blank')
      } else {
        ElMessage.error('下载链接不可用，请联系客服')
      }
    }
  } catch {
    ElMessage.error('下载失败')
  } finally {
    downloading.value = false
  }
}

function copyAddress() {
  navigator.clipboard.writeText(usdtAddress.value)
  ElMessage.success('已复制到剪贴板')
}

function goToOrders() {
  stopPolling()
  usdtDialogVisible.value = false
  router.push('/orders')
}

function startPolling() {
  pollingStatus.value = true
  pollingTimer = setInterval(async () => {
    if (!currentOrderId.value) return
    try {
      const res = await checkOrderStatus(currentOrderId.value)
      if (res.success && res.data) {
        if (res.data.status === 'completed' || res.data.status === 'paid') {
          stopPolling()
          usdtDialogVisible.value = false
          ElMessage.success('支付成功！')
          isPurchased.value = true
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

onMounted(() => fetchResource())
onUnmounted(() => stopPolling())
</script>

<style scoped>
.detail-page {
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
.resource-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.resource-header h2 {
  margin: 0;
}
.cover-image {
  margin-bottom: 20px;
}
.cover-image .el-image {
  width: 100%;
  max-height: 400px;
  border-radius: 8px;
}
.description h4, .content h4 {
  margin: 20px 0 10px;
  color: #303133;
}
.description p {
  color: #606266;
  line-height: 1.8;
}
.purchase-card {
  position: sticky;
  top: 20px;
}
.price-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
.price-section .label {
  font-size: 14px;
  color: #909399;
}
.price-section .price {
  font-size: 28px;
  font-weight: bold;
  color: #f56c6c;
}
.info-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.info-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #909399;
  font-size: 14px;
}
.payment-method {
  margin-bottom: 16px;
}
.payment-method .label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  color: #606266;
}
.purchase-card .el-button {
  width: 100%;
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