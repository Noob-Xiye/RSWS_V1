<template>
  <div class="user-center">
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
                <el-dropdown-item @click="handleLogout">退出</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-header>
      <el-main class="main">
        <el-card>
          <el-descriptions title="用户信息" :column="2" border>
            <el-descriptions-item label="用户名">{{ userStore.userInfo?.username }}</el-descriptions-item>
            <el-descriptions-item label="邮箱">{{ userStore.userInfo?.email }}</el-descriptions-item>
            <el-descriptions-item label="余额">{{ userStore.userInfo?.balance }} USDT</el-descriptions-item>
          </el-descriptions>
        </el-card>
        <el-card style="margin-top: 20px">
          <template #header>快捷操作</template>
          <el-button type="primary" @click="$router.push('/orders')">我的订单</el-button>
          <el-button @click="$router.push('/')">浏览资源</el-button>
        </el-card>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useUserStore } from '@/stores/user'

const router = useRouter()
const userStore = useUserStore()

function handleLogout() {
  userStore.logout()
  router.push('/')
}
</script>

<style scoped>
.user-center {
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