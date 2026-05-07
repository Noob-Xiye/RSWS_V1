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
          <template #header>我的订单</template>
          <el-table :data="orders" v-loading="loading" stripe>
            <el-table-column prop="order_no" label="订单号" width="200" />
            <el-table-column prop="resource_title" label="资源" />
            <el-table-column prop="amount" label="金额" width="120">
              <template #default="{ row }">{{ row.amount }} USDT</template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="getStatusType(row.status)">{{ getStatusText(row.status) }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="created_at" label="创建时间" width="180">
              <template #default="{ row }">{{ row.created_at?.substring(0, 19).replace('T', ' ') }}</template>
            </el-table-column>
            <el-table-column label="操作" width="100">
              <template #default="{ row }">
                <el-button v-if="row.status === 'completed'" type="primary" size="small" link @click="handleDownload(row)">下载</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useUserStore } from '@/stores/user'

const router = useRouter()
const userStore = useUserStore()

const loading = ref(false)
const orders = ref<any[]>([])

function getStatusType(status: string) {
  const map: Record<string, string> = { pending: 'warning', paid: 'primary', completed: 'success', cancelled: 'info' }
  return map[status] || 'info'
}

function getStatusText(status: string) {
  const map: Record<string, string> = { pending: '待支付', paid: '已支付', completed: '已完成', cancelled: '已取消' }
  return map[status] || status
}

function handleLogout() {
  userStore.logout()
  router.push('/')
}

function handleDownload(_row: any) {
  ElMessage.success('下载功能开发中')
}

onMounted(async () => {
  if (!userStore.isLoggedIn) {
    router.push('/login')
    return
  }
  loading.value = true
  try {
    // TODO: 调用实际API
    orders.value = []
  } finally {
    loading.value = false
  }
})
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
  max-width: 1000px;
  margin: 0 auto;
  padding: 20px;
}
</style>