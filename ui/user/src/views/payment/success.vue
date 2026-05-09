<template>
  <div class="payment-page">
    <el-card class="payment-card">
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
          <span class="value">{{ orderInfo.amount ? (orderInfo.amount / 100).toFixed(2) : '0' }} USDT</span>
        </div>
      </div>

      <div class="actions">
        <el-button type="primary" size="large" @click="goToResource">前往下载资源</el-button>
        <el-button size="large" @click="goToOrders">查看我的订单</el-button>
      </div>

      <p v-if="countdown > 0" class="auto-redirect">将在 {{ countdown }} 秒后自动跳转...</p>
    </el-card>
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
      if (res.success && res.data) orderInfo.value = res.data
    } catch { /* ignore */ }
  }
  timer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      clearInterval(timer)
      if (orderInfo.value?.resource_id) {
        router.replace(`/resource/${orderInfo.value.resource_id}`)
      } else {
        router.replace('/orders')
      }
    }
  }, 1000)
})

onUnmounted(() => clearInterval(timer))

function goToResource() {
  if (orderInfo.value?.resource_id) {
    router.push(`/resource/${orderInfo.value.resource_id}`)
  } else {
    router.push('/orders')
  }
}

function goToOrders() {
  router.push('/orders')
}
</script>

<style scoped>
.payment-page {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
}
.payment-card {
  width: 480px;
  text-align: center;
}
.icon-wrapper {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  border-radius: 50%;
  margin-bottom: 16px;
}
.icon-wrapper.success { background: #e8f8f5; color: #11998e; }
.title { margin: 0 0 8px; font-size: 24px; }
.subtitle { margin: 0 0 24px; color: #666; }
.order-info {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
  text-align: left;
}
.info-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #eee;
}
.info-row:last-child { border-bottom: none; }
.label { color: #888; }
.value { font-weight: 600; }
.actions { display: flex; gap: 12px; justify-content: center; margin-bottom: 16px; }
.actions .el-button { flex: 1; }
.auto-redirect { color: #aaa; font-size: 13px; margin: 0; }
</style>