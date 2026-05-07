<template>
  <div class="detail-page">
    <el-container>
      <el-header class="header">
        <div class="logo" @click="$router.push('/')">RSWS</div>
        <div class="user-area">
          <el-button v-if="!userStore.isLoggedIn" type="primary" size="small" @click="$router.push('/login')">登录</el-button>
        </div>
      </el-header>
      <el-main class="main" v-loading="loading">
        <el-row :gutter="40">
          <el-col :span="12">
            <div class="resource-cover">
              <el-image v-if="resource?.cover_image" :src="resource.cover_image" fit="contain" />
              <el-icon v-else :size="100"><Document /></el-icon>
            </div>
          </el-col>
          <el-col :span="12">
            <h1 class="title">{{ resource?.title }}</h1>
            <div class="meta">
              <span class="creator">{{ resource?.creator_name }}</span>
              <span class="date">{{ resource?.created_at?.substring(0, 10) }}</span>
              <span class="downloads">{{ resource?.download_count }} 次下载</span>
            </div>
            <div class="price">{{ resource?.price }} USDT</div>
            <el-divider />
            <div class="description">{{ resource?.description }}</div>
            <el-divider />
            <div class="actions">
              <el-button type="primary" size="large" @click="handlePurchase" :loading="purchasing">
                立即购买
              </el-button>
              <el-button v-if="purchased" type="success" size="large" @click="handleDownload">
                下载
              </el-button>
            </div>
          </el-col>
        </el-row>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { getResource } from '@/api/resource'

const route = useRoute()
const router = useRouter()
const userStore = useUserStore()

const loading = ref(false)
const purchasing = ref(false)
const purchased = ref(false)
const resource = ref<Awaited<ReturnType<typeof getResource>>['data'] | null>(null)

async function fetchResource() {
  loading.value = true
  try {
    const res = await getResource(Number(route.params.id))
    if (res.success && res.data) resource.value = res.data
  } catch {
    ElMessage.error('资源不存在')
    router.push('/')
  } finally {
    loading.value = false
  }
}

async function handlePurchase() {
  if (!userStore.isLoggedIn) {
    ElMessage.warning('请先登录')
    router.push('/login')
    return
  }
  purchasing.value = true
  try {
    ElMessage.success('购买功能开发中')
  } finally {
    purchasing.value = false
  }
}

function handleDownload() {
  ElMessage.success('下载功能开发中')
}

onMounted(() => fetchResource())
</script>

<style scoped>
.detail-page {
  min-height: 100vh;
  background: #fff;
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
}
.user-area {
  margin-left: auto;
}
.main {
  max-width: 1200px;
  margin: 0 auto;
  padding: 40px;
}
.resource-cover {
  height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f5;
  border-radius: 8px;
}
.title {
  font-size: 24px;
  margin-bottom: 10px;
}
.meta {
  color: #909399;
  font-size: 14px;
  display: flex;
  gap: 20px;
}
.price {
  font-size: 28px;
  color: #f56c6c;
  font-weight: bold;
  margin-top: 20px;
}
.description {
  line-height: 1.8;
  color: #666;
}
.actions {
  margin-top: 30px;
}
</style>