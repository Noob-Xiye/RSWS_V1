<template>
  <div class="home-page">
    <el-container>
      <el-header class="header">
        <div class="logo">RSWS</div>
        <el-menu mode="horizontal" :ellipsis="false" router>
          <el-menu-item index="/">首页</el-menu-item>
          <el-menu-item index="/orders" v-if="userStore.isLoggedIn">我的订单</el-menu-item>
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
            <el-button type="primary" size="small" @click="$router.push('/login')">登录</el-button>
            <el-button size="small" @click="$router.push('/register')">注册</el-button>
          </template>
        </div>
      </el-header>

      <el-main class="main">
        <div class="search-bar">
          <el-input v-model="keyword" placeholder="搜索资源" clearable @keyup.enter="handleSearch" style="max-width: 400px">
            <template #append>
              <el-button icon="Search" @click="handleSearch" />
            </template>
          </el-input>
          <el-select v-model="selectedCategory" placeholder="分类" clearable @change="handleSearch" style="width: 150px">
            <el-option label="全部" :value="0" />
            <el-option v-for="cat in categories" :key="cat.id" :label="cat.name" :value="cat.id" />
          </el-select>
        </div>

        <el-row :gutter="20" v-loading="loading">
          <el-col :span="6" v-for="item in resources" :key="item.id">
            <el-card class="resource-card" shadow="hover" @click="$router.push(`/resource/${item.id}`)">
              <div class="resource-cover">
                <el-image v-if="item.cover_image" :src="item.cover_image" fit="cover" />
                <el-icon v-else :size="60"><Document /></el-icon>
              </div>
              <div class="resource-info">
                <div class="resource-title">{{ item.title }}</div>
                <div class="resource-meta">
                  <span class="price">{{ item.price }} USDT</span>
                  <span class="downloads">{{ item.download_count }} 下载</span>
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>

        <div class="pagination">
          <el-pagination
            v-model:current-page="page"
            :page-size="pageSize"
            :total="total"
            layout="prev, pager, next"
            @current-change="fetchResources"
          />
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { listResources } from '@/api/resource'
import type { Resource } from '@/api/resource'
import { getCategoryList } from '@/api/category'

const router = useRouter()
const userStore = useUserStore()

const loading = ref(false)
const resources = ref<Resource[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = 12
const keyword = ref('')
const selectedCategory = ref<number>(0)
const categories = ref<{ id: number; name: string }[]>([])

async function fetchCategories() {
  try {
    const res = await getCategoryList()
    categories.value = res.filter((c: any) => c.is_active !== false)
  } catch {
    categories.value = []
  }
}

async function fetchResources() {
  loading.value = true
  try {
    const params: any = { page: page.value, page_size: pageSize }
    if (keyword.value) params.search = keyword.value
    if (selectedCategory.value) params.category_id = selectedCategory.value
    
    const res = await listResources(params)
    if (res.success && res.data) {
      resources.value = res.data.items
      total.value = res.data.total
    }
  } catch {
    resources.value = []
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchResources()
}

function handleLogout() {
  userStore.logout()
  router.push('/')
}

onMounted(() => {
  fetchCategories()
  fetchResources()
})
</script>

<style scoped>
.home-page {
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
  margin-right: 40px;
}

.user-area {
  margin-left: auto;
  display: flex;
  gap: 10px;
  align-items: center;
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

.search-bar {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.resource-card {
  cursor: pointer;
  margin-bottom: 20px;
}

.resource-cover {
  height: 150px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f0f0f0;
  border-radius: 4px;
}

.resource-info {
  padding: 10px 0;
}

.resource-title {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resource-meta {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  font-size: 12px;
}

.price {
  color: #f56c6c;
  font-weight: bold;
}

.downloads {
  color: #909399;
}

.pagination {
  display: flex;
  justify-content: center;
  margin-top: 30px;
}
</style>