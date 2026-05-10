<template>
  <ModernLayout :showFooter="false">
    <div class="detail-page">
      <el-skeleton :loading="loading" animated>
        <template #default>
          <div class="detail-container" v-if="resource">
            <div class="detail-grid">
              <!-- 左侧：资源信息 -->
              <div class="detail-main">
                <div class="detail-card">
                  <!-- 封面图 -->
                  <div class="cover-section" v-if="resource.cover_image">
                    <el-image :src="resource.cover_image" fit="cover" class="cover-image" />
                  </div>

                  <!-- 标题 -->
                  <div class="title-section">
                    <h1 class="resource-title">{{ resource.title }}</h1>
                    <div class="title-meta">
                      <el-tag class="category-tag" effect="dark" round>{{ resource.category_id || '未分类' }}</el-tag>
                      <span class="meta-item">
                        <el-icon><Download /></el-icon>
                        {{ resource.download_count || 0 }} 次下载
                      </span>
                      <span class="meta-item">
                        <el-icon><Calendar /></el-icon>
                        {{ formatDate(resource.created_at) }}
                      </span>
                    </div>
                  </div>

                  <!-- 描述 -->
                  <div class="desc-section" v-if="resource.description">
                    <h3>简介</h3>
                    <p>{{ resource.description }}</p>
                  </div>

                  <!-- 详细内容 -->
                  <div class="content-section" v-if="resource.detail_description">
                    <h3>详细内容</h3>
                    <div class="rich-content" v-html="resource.detail_description"></div>
                  </div>
                </div>
              </div>

              <!-- 右侧：购买卡片 -->
              <div class="detail-sidebar">
                <div class="purchase-card">
                  <div class="price-section">
                    <span class="price-label">价格</span>
                    <div class="price-value">
                      <span class="price-amount">{{ formatPrice(resource.price) }}</span>
                      <span class="price-unit">USDT</span>
                    </div>
                  </div>

                  <div class="info-list">
                    <div class="info-item">
                      <el-icon><Download /></el-icon>
                      <span>{{ resource.download_count || 0 }} 次下载</span>
                    </div>
                    <div class="info-item">
                      <el-icon><Calendar /></el-icon>
                      <span>{{ formatDate(resource.created_at) }}</span>
                    </div>
                  </div>

                  <div class="divider"></div>

                  <!-- 已购买 -->
                  <template v-if="isPurchased">
                    <div class="purchased-badge">
                      <el-icon><CircleCheck /></el-icon>
                      <span>您已购买此资源</span>
                    </div>
                    <button class="btn-download" :disabled="downloading" @click="handleDownload">
                      <el-icon v-if="!downloading"><Download /></el-icon>
                      <el-icon v-else class="is-loading"><Loading /></el-icon>
                      {{ downloading ? '准备中...' : '下载资源' }}
                    </button>
                  </template>

                  <!-- 未购买 -->
                  <template v-else>
                    <div class="payment-section">
                      <span class="payment-label">支付方式</span>
                      <div class="payment-options">
                        <label
                          v-for="opt in paymentOptions"
                          :key="opt.value"
                          class="payment-option"
                          :class="{ active: paymentMethod === opt.value }"
                        >
                          <input type="radio" :value="opt.value" v-model="paymentMethod" />
                          <span>{{ opt.label }}</span>
                        </label>
                      </div>
                    </div>
                    <button class="btn-purchase" :disabled="purchasing" @click="handlePurchase">
                      <el-icon v-if="!purchasing"><ShoppingCart /></el-icon>
                      <el-icon v-else class="is-loading"><Loading /></el-icon>
                      {{ purchasing ? '创建订单中...' : '立即购买' }}
                    </button>
                  </template>
                </div>
              </div>
            </div>
          </div>
        </template>
      </el-skeleton>

      <!-- USDT 支付对话框 -->
      <el-dialog v-model="usdtDialogVisible" title="USDT 支付" width="500px" :close-on-click-modal="false" class="payment-dialog">
        <div class="usdt-payment" v-if="resource">
          <div class="payment-info-banner">
            <p>请向以下地址转账 <strong>{{ formatPrice(resource.price) }} USDT</strong></p>
          </div>
          <div class="payment-detail-list">
            <div class="payment-detail-item">
              <span class="detail-label">网络</span>
              <span class="detail-value">
                <el-tag size="small" effect="dark">
                  {{ paymentMethod === 'usdt_trc20' ? 'TRC-20 (Tron)' : 'ERC-20 (Ethereum)' }}
                </el-tag>
              </span>
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
              <span class="detail-amount">{{ formatPrice(resource.price) }} USDT</span>
            </div>
            <div class="payment-detail-item">
              <span class="detail-label">订单号</span>
              <span>{{ currentOrderNo }}</span>
            </div>
          </div>
          <div class="payment-notice">
            <h4>⚠️ 注意事项</h4>
            <ul>
              <li>请确保转账金额准确</li>
              <li>转账完成后请等待链上确认</li>
              <li>支付完成后可前往"我的订单"查看状态</li>
            </ul>
          </div>
          <div class="payment-polling" v-if="pollingStatus">
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
  </ModernLayout>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { getResource, checkPurchase, getDownloadInfo, type ResourceDetail } from '@/api/resource'
import { createOrder, checkOrderStatus, getUsdtAddress } from '@/api/order'
import ModernLayout from '@/components/ModernLayout.vue'

const route = useRoute()
const router = useRouter()
const userStore = useUserStore()

const loading = ref(true)
const resource = ref<ResourceDetail | null>(null)
const isPurchased = ref(false)
const downloading = ref(false)
const purchasing = ref(false)
const paymentMethod = ref<'usdt_trc20' | 'usdt_erc20' | 'paypal'>('usdt_trc20')
const paymentOptions = [
  { label: 'USDT-TRC20', value: 'usdt_trc20' as const },
  { label: 'USDT-ERC20', value: 'usdt_erc20' as const },
  { label: 'PayPal', value: 'paypal' as const }
]

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
  if (!userStore.isLoggedIn) { isPurchased.value = false; return }
  try {
    const res = await checkPurchase(resourceId)
    if (res.success && res.data) isPurchased.value = res.data.purchased || false
  } catch { isPurchased.value = false }
}

function formatDate(dateStr?: string) { return dateStr ? dateStr.substring(0, 10) : '' }
function formatPrice(price: number) { return (price / 100).toFixed(2) }

async function handlePurchase() {
  if (!userStore.isLoggedIn) { ElMessage.warning('请先登录'); router.push('/login'); return }
  if (!resource.value) return
  purchasing.value = true
  try {
    const res = await createOrder({ resource_id: resource.value.id, payment_method: paymentMethod.value })
    if (res.success && res.data) {
      currentOrderId.value = res.data.id
      currentOrderNo.value = `ORD-${res.data.id}`
      if (paymentMethod.value === 'paypal') {
        if (res.data.approve_url) { window.location.href = res.data.approve_url }
        else if (res.data.paypal_order_id) { window.location.href = `https://www.sandbox.paypal.com/checkoutnow?token=${res.data.paypal_order_id}` }
        else { ElMessage.info(res.data.message || 'PayPal 订单已创建'); router.push('/orders') }
      } else {
        const network = paymentMethod.value === 'usdt_trc20' ? 'tron' : 'ethereum'
        const addrRes = await getUsdtAddress(network)
        if (addrRes.success && addrRes.data) { usdtAddress.value = addrRes.data.address; usdtDialogVisible.value = true; startPolling() }
        else { ElMessage.error('获取收款地址失败') }
      }
    } else { ElMessage.error(res.message || '创建订单失败') }
  } catch (err: any) { ElMessage.error(err?.message || '购买失败') }
  finally { purchasing.value = false }
}

async function handleDownload() {
  if (!resource.value) return
  await checkPurchasedStatus(resource.value.id)
  if (!isPurchased.value) { ElMessage.warning('请先购买此资源'); return }
  downloading.value = true
  try {
    const res = await getDownloadInfo(resource.value.id)
    if (res.success && res.data) {
      const link = document.createElement('a'); link.href = res.data.file_url; link.download = res.data.file_name || 'resource'
      document.body.appendChild(link); link.click(); document.body.removeChild(link)
    } else if (resource.value.file_url) { window.open(resource.value.file_url, '_blank') }
    else { ElMessage.error('下载链接不可用，请联系客服') }
  } catch { ElMessage.error('下载失败') }
  finally { downloading.value = false }
}

function copyAddress() { navigator.clipboard.writeText(usdtAddress.value); ElMessage.success('已复制到剪贴板') }
function goToOrders() { stopPolling(); usdtDialogVisible.value = false; router.push('/orders') }

function startPolling() {
  pollingStatus.value = true
  pollingTimer = setInterval(async () => {
    if (!currentOrderId.value) return
    try {
      const res = await checkOrderStatus(currentOrderId.value)
      if (res.success && res.data && (res.data.status === 'completed' || res.data.status === 'paid')) {
        stopPolling(); usdtDialogVisible.value = false; ElMessage.success('支付成功！'); isPurchased.value = true
      }
    } catch {}
  }, 5000)
}

function stopPolling() {
  pollingStatus.value = false
  if (pollingTimer) { clearInterval(pollingTimer); pollingTimer = null }
}

onMounted(() => fetchResource())
onUnmounted(() => stopPolling())
</script>

<style scoped>
.detail-page {
  padding: 32px 24px;
}

.detail-container {
  max-width: 1400px;
  margin: 0 auto;
}

.detail-grid {
  display: grid;
  grid-template-columns: 1fr 360px;
  gap: 24px;
}

/* 主内容卡片 */
.detail-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  overflow: hidden;
}

.cover-section {
  max-height: 400px;
  overflow: hidden;
}

.cover-image {
  width: 100%;
  border-radius: 0;
}

.title-section {
  padding: 28px 32px 20px;
}

.resource-title {
  font-size: 28px;
  font-weight: 700;
  margin-bottom: 16px;
}

.title-meta {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.category-tag {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 14px;
}

.desc-section, .content-section {
  padding: 0 32px 24px;
}

.desc-section h3, .content-section h3 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 12px;
  color: rgba(255, 255, 255, 0.9);
}

.desc-section p {
  color: rgba(255, 255, 255, 0.7);
  line-height: 1.8;
}

.rich-content {
  color: rgba(255, 255, 255, 0.7);
  line-height: 1.8;
}

/* 购买侧栏 */
.purchase-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 28px;
  position: sticky;
  top: 96px;
}

.price-section {
  margin-bottom: 20px;
}

.price-label {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.5);
  display: block;
  margin-bottom: 8px;
}

.price-value {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.price-amount {
  font-size: 36px;
  font-weight: 800;
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.price-unit {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.5);
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 14px;
}

.divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.1);
  margin: 20px 0;
}

/* 已购买状态 */
.purchased-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #67c23a;
  font-size: 15px;
  margin-bottom: 16px;
}

.btn-download, .btn-purchase {
  width: 100%;
  padding: 14px 20px;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: all 0.3s;
}

.btn-download {
  background: linear-gradient(135deg, #67c23a 0%, #42b983 100%);
  color: #fff;
}

.btn-download:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(103, 194, 58, 0.4);
}

.btn-purchase {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

.btn-purchase:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
}

.btn-download:disabled, .btn-purchase:disabled {
  opacity: 0.7;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

/* 支付方式 */
.payment-section {
  margin-bottom: 20px;
}

.payment-label {
  display: block;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.5);
  margin-bottom: 10px;
}

.payment-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.payment-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
  transition: all 0.3s;
}

.payment-option input { display: none; }

.payment-option:hover {
  border-color: rgba(255, 255, 255, 0.2);
}

.payment-option.active {
  border-color: #667eea;
  background: rgba(102, 126, 234, 0.15);
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

.payment-detail-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 20px;
}

.payment-detail-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-label {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
}

.detail-amount {
  font-size: 22px;
  font-weight: 700;
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.address-box {
  display: flex;
  align-items: center;
  gap: 10px;
}

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

.payment-notice h4 {
  margin: 0 0 8px;
  color: #e6a23c;
  font-size: 14px;
}

.payment-notice ul {
  margin: 0;
  padding-left: 20px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
}

.payment-polling {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  margin-top: 16px;
  color: #667eea;
}

/* 响应式 */
@media (max-width: 900px) {
  .detail-grid {
    grid-template-columns: 1fr;
  }

  .purchase-card {
    position: static;
  }
}
</style>
