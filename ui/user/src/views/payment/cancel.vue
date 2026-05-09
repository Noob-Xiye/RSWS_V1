<template>
  <div class="payment-page">
    <el-card class="payment-card">
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
        <el-button type="primary" size="large" @click="retryPayment">重新支付</el-button>
        <el-button size="large" @click="goToOrders">查看我的订单</el-button>
      </div>

      <p v-if="countdown > 0" class="auto-redirect">将在 {{ countdown }} 秒后返回订单列表...</p>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { CircleCloseFilled } from '@element-plus/icons-vue'
import { getOrder, checkOrderStatus } from '@/api/order'
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
    } catch { /* ignore */ }
  }
  timer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      clearInterval(timer)
      router.replace('/orders')
    }
  }, 1000)
})

onUnmounted(() => clearInterval(timer))

async function retryPayment() {
  if (!orderInfo.value) { router.push('/orders'); return }
  const status = orderInfo.value.status
  if (status === 'pending') {
    // 仍可跳转到资源详情页重新发起支付
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
  background: linear-gradient(135deg, #eb3349 0%, #f45c43 100%);
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
.icon-wrapper.cancel { background: #fde8e8; color: #eb3349; }
.title { margin: 0 0 8px; font-size: 24px; }
.subtitle { margin: 0 0 24px; color: #666; }
.order-id-display {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
}
.label { color: #888; display: block; margin-bottom: 4px; }
.value { font-weight: 600; font-size: 16px; }
.actions { display: flex; gap: 12px; justify-content: center; margin-bottom: 16px; }
.actions .el-button { flex: 1; }
.auto-redirect { color: #aaa; font-size: 13px; margin: 0; }
</style>