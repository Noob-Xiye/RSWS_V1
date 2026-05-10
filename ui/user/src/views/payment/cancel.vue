<template>
  <div class="payment-page">
    <div class="bg-decoration">
      <div class="bg-blob blob-1"></div>
      <div class="bg-blob blob-2"></div>
    </div>

    <div class="payment-card">
      <div class="icon-wrapper cancel">
        <el-icon :size="48"><CircleCloseFilled /></el-icon>
      </div>
      <h2 class="title">支付已取消</h2>
      <p class="subtitle">您已取消支付，订单将在 15 分钟后自动关闭</p>

      <div v-if="orderId" class="order-id-display">
        <span class="label">订单号</span>
        <span class="value">{{ orderId }}</span>
      </div>

      <div class="actions">
        <button class="btn-primary" @click="retryPayment">重新支付</button>
        <button class="btn-secondary" @click="goToOrders">查看我的订单</button>
      </div>

      <p v-if="countdown > 0" class="auto-redirect">将在 {{ countdown }} 秒后返回订单列表...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { CircleCloseFilled } from '@element-plus/icons-vue'
import { getOrder } from '@/api/order'
import type { Order } from '@/api/order'

const router = useRouter()
const route = useRoute()
const orderInfo = ref<Order | null>(null)
const countdown = ref(8)
let timer: ReturnType<typeof setInterval>

const orderId = route.query.order_id as string

onMounted(async () => {
  if (orderId) {
    try {
      const res = await getOrder(parseInt(orderId))
      if (res.success && res.data) orderInfo.value = res.data
    } catch {}
  }
  timer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) { clearInterval(timer); router.replace('/orders') }
  }, 1000)
})

onUnmounted(() => clearInterval(timer))

async function retryPayment() {
  if (!orderInfo.value) { router.push('/orders'); return }
  if (orderInfo.value.status === 'pending') router.push(`/resource/${orderInfo.value.resource_id}`)
  else router.push('/orders')
}

function goToOrders() { router.push('/orders') }
</script>

<style scoped>
.payment-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16213e 100%);
  padding: 20px;
  position: relative;
  overflow: hidden;
}

.bg-decoration { position: absolute; inset: 0; pointer-events: none; }
.bg-blob { position: absolute; border-radius: 50%; filter: blur(100px); opacity: 0.4; }
.blob-1 { width: 400px; height: 400px; background: #e6a23c; top: -150px; right: -100px; }
.blob-2 { width: 300px; height: 300px; background: #f56c6c; bottom: -100px; left: -50px; }

.payment-card {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 24px;
  padding: 48px 40px;
  text-align: center;
  max-width: 480px;
  width: 100%;
  position: relative;
  z-index: 1;
}

.icon-wrapper {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  border-radius: 50%;
  margin-bottom: 20px;
}

.icon-wrapper.cancel {
  background: rgba(230, 162, 60, 0.15);
  color: #e6a23c;
}

.title { margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #fff; }
.subtitle { margin: 0 0 28px; color: rgba(255, 255, 255, 0.6); font-size: 15px; }

.order-id-display {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 28px;
}

.label { color: rgba(255, 255, 255, 0.5); display: block; margin-bottom: 4px; font-size: 14px; }
.value { font-weight: 600; font-size: 16px; color: rgba(255, 255, 255, 0.9); }

.actions { display: flex; gap: 12px; margin-bottom: 20px; }

.btn-primary, .btn-secondary {
  flex: 1;
  padding: 14px 20px;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
  border: none;
}

.btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

.btn-primary:hover { box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4); }

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.8);
}

.btn-secondary:hover { background: rgba(255, 255, 255, 0.1); color: #fff; }

.auto-redirect { color: rgba(255, 255, 255, 0.4); font-size: 13px; margin: 0; }
</style>
