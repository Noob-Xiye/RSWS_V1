<template>
  <ModernLayout>
    <!-- Hero 区域 -->
    <section class="hero">
      <div class="hero-content">
        <h1 class="hero-title">
          <span class="gradient-text">优质资源</span>
          <span>触手可及</span>
        </h1>
        <p class="hero-subtitle">发现、购买、下载高质量数字资源</p>

        <div class="search-container">
          <div class="search-box">
            <el-input
              v-model="keyword"
              placeholder="搜索你想要的资源..."
              size="large"
              clearable
              @keyup.enter="handleSearch"
              class="search-input"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
            <el-button type="primary" size="large" class="search-btn" @click="handleSearch">
              搜索
            </el-button>
          </div>

          <div class="category-tags">
            <span
              class="category-tag"
              :class="{ active: selectedCategory === 0 }"
              @click="selectCategory(0)"
            >全部</span>
            <span
              v-for="cat in categories"
              :key="cat.id"
              class="category-tag"
              :class="{ active: selectedCategory === cat.id }"
              @click="selectCategory(cat.id)"
            >{{ cat.name }}</span>
          </div>
        </div>
      </div>

      <div class="hero-bg">
        <div class="bg-blob blob-1"></div>
        <div class="bg-blob blob-2"></div>
        <div class="bg-blob blob-3"></div>
      </div>
    </section>

    <!-- 资源列表 -->
    <section class="resources-section">
      <div class="section-container">
        <div class="section-header">
          <h2 class="section-title">
            <span class="gradient-text">热门资源</span>
          </h2>
          <span class="resource-count">共 {{ total }} 个资源</span>
        </div>

        <div v-loading="loading" class="resources-grid">
          <div
            v-for="item in resources"
            :key="item.id"
            class="resource-card"
            @click="$router.push(`/resource/${item.id}`)"
          >
            <div class="card-cover">
              <el-image v-if="item.cover_image" :src="item.cover_image" fit="cover" class="cover-image" />
              <div v-else class="cover-placeholder">
                <el-icon :size="48"><Document /></el-icon>
              </div>
              <div class="card-overlay">
                <span class="view-btn">查看详情</span>
              </div>
            </div>
            <div class="card-content">
              <h3 class="card-title">{{ item.title }}</h3>
              <p class="card-desc">{{ item.description || '暂无描述' }}</p>
              <div class="card-footer">
                <div class="card-price">
                  <span class="price-value">{{ (item.price / 100).toFixed(2) }}</span>
                  <span class="price-unit">USDT</span>
                </div>
                <div class="card-stats">
                  <el-icon><Download /></el-icon>
                  <span>{{ item.download_count }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="!loading && resources.length === 0" class="empty-state">
          <el-icon :size="64"><FolderOpened /></el-icon>
          <p>暂无资源</p>
        </div>

        <div v-if="total > pageSize" class="pagination">
          <el-pagination
            v-model:current-page="page"
            :page-size="pageSize"
            :total="total"
            layout="prev, pager, next"
            :background="true"
            @current-change="fetchResources"
          />
        </div>
      </div>
    </section>
  </ModernLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { listResources } from '@/api/resource'
import type { Resource } from '@/api/resource'
import { getCategoryList } from '@/api/category'
import ModernLayout from '@/components/ModernLayout.vue'

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

function selectCategory(catId: number) {
  selectedCategory.value = catId
  page.value = 1
  fetchResources()
}

onMounted(() => {
  fetchCategories()
  fetchResources()
})
</script>

<style scoped>
/* Hero 区域 */
.hero {
  position: relative;
  padding: 80px 24px 60px;
  text-align: center;
  overflow: hidden;
}

.hero-content {
  position: relative;
  z-index: 1;
  max-width: 800px;
  margin: 0 auto;
}

.hero-title {
  font-size: 56px;
  font-weight: 800;
  line-height: 1.2;
  margin-bottom: 16px;
}

.hero-title span {
  display: block;
}

.gradient-text {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.hero-subtitle {
  font-size: 20px;
  color: rgba(255, 255, 255, 0.7);
  margin-bottom: 48px;
}

/* 搜索栏 */
.search-container {
  max-width: 600px;
  margin: 0 auto;
}

.search-box {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}

.search-input {
  flex: 1;
}

.search-input :deep(.el-input__wrapper) {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 12px;
  box-shadow: none;
}

.search-input :deep(.el-input__inner) {
  color: #fff;
}

.search-input :deep(.el-input__inner::placeholder) {
  color: rgba(255, 255, 255, 0.5);
}

.search-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  border-radius: 12px;
  padding: 0 32px;
}

.search-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
}

.category-tags {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 12px;
}

.category-tag {
  padding: 8px 20px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 20px;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.3s;
}

.category-tag:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.category-tag.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: transparent;
  color: #fff;
}

/* 背景装饰 */
.hero-bg {
  position: absolute;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
}

.bg-blob {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.5;
}

.blob-1 {
  width: 400px;
  height: 400px;
  background: #667eea;
  top: -100px;
  right: -100px;
  animation: float 8s ease-in-out infinite;
}

.blob-2 {
  width: 300px;
  height: 300px;
  background: #764ba2;
  bottom: 0;
  left: -50px;
  animation: float 10s ease-in-out infinite reverse;
}

.blob-3 {
  width: 200px;
  height: 200px;
  background: #f093fb;
  top: 50%;
  left: 50%;
  animation: float 12s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translate(0, 0); }
  50% { transform: translate(30px, -30px); }
}

/* 资源列表 */
.resources-section {
  padding: 20px 24px 80px;
}

.section-container {
  max-width: 1400px;
  margin: 0 auto;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 32px;
}

.section-title {
  font-size: 28px;
  font-weight: 700;
}

.resource-count {
  color: rgba(255, 255, 255, 0.5);
  font-size: 14px;
}

.resources-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
}

/* 资源卡片 */
.resource-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.resource-card:hover {
  transform: translateY(-8px);
  border-color: rgba(102, 126, 234, 0.5);
  box-shadow: 0 20px 40px rgba(102, 126, 234, 0.2);
}

.card-cover {
  position: relative;
  height: 180px;
  overflow: hidden;
}

.cover-image {
  width: 100%;
  height: 100%;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.2) 0%, rgba(118, 75, 162, 0.2) 100%);
  color: rgba(255, 255, 255, 0.5);
}

.card-overlay {
  position: absolute;
  inset: 0;
  background: rgba(15, 15, 26, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.3s;
}

.resource-card:hover .card-overlay {
  opacity: 1;
}

.view-btn {
  padding: 12px 28px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
}

.card-content {
  padding: 20px;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-desc {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 16px;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-price {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.price-value {
  font-size: 20px;
  font-weight: 700;
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.price-unit {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
}

.card-stats {
  display: flex;
  align-items: center;
  gap: 4px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 13px;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 80px 20px;
  color: rgba(255, 255, 255, 0.5);
}

.empty-state p {
  margin-top: 16px;
  font-size: 16px;
}

/* 分页 */
.pagination {
  display: flex;
  justify-content: center;
  margin-top: 48px;
}

.pagination :deep(.el-pagination.is-background .el-pager li) {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
}

.pagination :deep(.el-pagination.is-background .el-pager li:hover) {
  color: #fff;
}

.pagination :deep(.el-pagination.is-background .el-pager li.is-active) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

.pagination :deep(.btn-prev),
.pagination :deep(.btn-next) {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
}

/* 响应式 */
@media (max-width: 768px) {
  .hero-title {
    font-size: 36px;
  }

  .hero-subtitle {
    font-size: 16px;
  }

  .search-box {
    flex-direction: column;
  }

  .search-btn {
    width: 100%;
  }

  .resources-grid {
    grid-template-columns: 1fr;
  }
}
</style>
