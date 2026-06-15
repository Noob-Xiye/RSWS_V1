<template>
  <div class="payment-page">
    <div class="bg-decoration">
      <div class="bg-blob blob-1"></div>
      <div class="bg-blob blob-2"></div>
    </div>

    <div class="payment-card">
      <div class="icon-wrapper success">
        <el-icon :size="48"><CircleCheckFilled /></el-icon>
      </div>
      <h2 class="title">支付成功</h2>
      <p class="subtitle">您的订单已支付成功，资源已解锁</p>

      <div v-if="orderInfo" class="order-info">
        <div class="info-row">
          <span class="label">订单号</span>
          <span class="value">{{ orderInfo.id }}</span>
        </div>
        <div class="info-row">
          <span class="label">资源</span>
          <span class="value">{{ orderInfo.resource_title }}</span>
        </div>
        <div class="info-row">
          <span class="label">金额</span>
          <span class="value price">{{ orderInfo.amount ? (orderInfo.amount / 100).toFixed(2) : '0' }} USDT</span>
        </div>
      </div>

      <div class="actions">
        <button class="btn-primary" @click="goToResource">前往下载资源</button>
        <button class="btn-secondary" @click="goToOrders">查看我的订单</button>
      </div>

      <p v-if="countdown > 0" class="auto-redirect">将在 {{ countdown }} 秒后自动跳转...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { CircleCheckFilled } from '@element-plus/icons-vue'
import { getOrder } from '@/api/order'
import type { Order } from '@/api/order'

const router = useRouter()
const route = useRoute()
const orderInfo = ref<Order | null>(null)
const countdown = ref(5)
let timer: ReturnType<typeof setInterval>

const orderId = route.query.order_id as string

onMounted(async () => {
  if (orderId) {
    try {
      const res = await getOrder(parseInt(orderId))
      if (res.code === 0 && res.data) orderInfo.value = res.data
    } catch {}
  }
  timer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      clearInterval(timer)
      if (orderInfo.value?.resource_id) router.replace(`/resource/${orderInfo.value.resource_id}`)
      else router.replace('/orders')
    }
  }, 1000)
})

onUnmounted(() => clearInterval(timer))

function goToResource() {
  if (orderInfo.value?.resource_id) router.push(`/resource/${orderInfo.value.resource_id}`)
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
.blob-1 { width: 400px; height: 400px; background: #67c23a; top: -150px; right: -100px; }
.blob-2 { width: 300px; height: 300px; background: #42b983; bottom: -100px; left: -50px; }

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

.icon-wrapper.success {
  background: rgba(103, 194, 58, 0.15);
  color: #67c23a;
}

.title { margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #fff; }
.subtitle { margin: 0 0 28px; color: rgba(255, 255, 255, 0.6); font-size: 15px; }

.order-info {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 28px;
  text-align: left;
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.info-row:last-child { border-bottom: none; }
.label { color: rgba(255, 255, 255, 0.5); font-size: 14px; }
.value { font-weight: 600; font-size: 14px; color: rgba(255, 255, 255, 0.9); }
.value.price {
  background: linear-gradient(135deg, #f093fb, #f5576c);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

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
  background: linear-gradient(135deg, #67c23a 0%, #42b983 100%);
  color: #fff;
}

.btn-primary:hover { box-shadow: 0 8px 20px rgba(103, 194, 58, 0.4); }

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.8);
}

.btn-secondary:hover { background: rgba(255, 255, 255, 0.1); color: #fff; }

.auto-redirect { color: rgba(255, 255, 255, 0.4); font-size: 13px; margin: 0; }
</style>
