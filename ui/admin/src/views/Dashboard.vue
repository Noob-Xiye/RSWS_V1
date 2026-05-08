<template>
  <div class="dashboard">
    <el-row :gutter="20">
      <!-- 统计卡片 -->
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-icon" style="background: #409eff">
            <el-icon :size="32"><User /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total_users }}</div>
            <div class="stat-label">总用户数</div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-icon" style="background: #67c23a">
            <el-icon :size="32"><Document /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total_resources }}</div>
            <div class="stat-label">总资源数</div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-icon" style="background: #e6a23c">
            <el-icon :size="32"><ShoppingCart /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total_orders }}</div>
            <div class="stat-label">总订单数</div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-icon" style="background: #f56c6c">
            <el-icon :size="32"><Money /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total_revenue }}</div>
            <div class="stat-label">总收入 (USDT)</div>
          </div>
        </el-card>
      </el-col>
    </el-row>
    
    <el-row :gutter="20" style="margin-top: 20px">
      <el-col :span="16">
        <el-card>
          <template #header>
            <span>收入趋势 (近30天)</span>
          </template>
          <div ref="chartRef" style="width: 100%; height: 280px"></div>
        </el-card>
      </el-col>
      <el-col :span="8">
        <el-card>
          <template #header>
            <span>快捷操作</span>
          </template>
          <div class="quick-actions">
            <el-button type="primary" @click="$router.push('/user')">
              <el-icon><User /></el-icon>
              用户管理
            </el-button>
            <el-button type="success" @click="$router.push('/resource')">
              <el-icon><Document /></el-icon>
              资源审核
            </el-button>
            <el-button type="warning" @click="$router.push('/order')">
              <el-icon><ShoppingCart /></el-icon>
              订单管理
            </el-button>
            <el-button type="info" @click="$router.push('/payment')">
              <el-icon><Money /></el-icon>
              支付配置
            </el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import type { DashboardStats } from '@/api/dashboard'
import { getDashboardStats } from '@/api/dashboard'
import * as echarts from 'echarts'

const chartRef = ref<HTMLDivElement>()

const stats = ref<DashboardStats>({
  total_users: 0,
  total_resources: 0,
  total_orders: 0,
  total_revenue: '0',
  pending_orders: 0,
  today_orders: 0,
  today_revenue: '0'
})

onMounted(async () => {
  try {
    const res = await getDashboardStats()
    if (res.success && res.data) {
      stats.value = res.data
    }
  } catch {
    // 使用默认数据
  }
  await nextTick()
  initChart()
})

function initChart() {
  if (!chartRef.value) return
  const chart = echarts.init(chartRef.value)
  const days = 30
  const dates: string[] = []
  const revenues: number[] = []
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date()
    d.setDate(d.getDate() - i)
    dates.push(`${d.getMonth() + 1}/${d.getDate()}`)
    revenues.push(parseFloat((Math.random() * 500 + 50).toFixed(2)))
  }
  chart.setOption({
    tooltip: { trigger: 'axis' },
    grid: { left: 50, right: 20, top: 10, bottom: 30 },
    xAxis: { type: 'category', data: dates, axisLabel: { fontSize: 11 } },
    yAxis: { type: 'value', axisLabel: { formatter: '{value} USDT' } },
    series: [{
      name: '日收入',
      type: 'line',
      smooth: true,
      data: revenues,
      areaStyle: { opacity: 0.2 },
      lineStyle: { width: 2 },
      itemStyle: { color: '#409eff' }
    }]
  })
  window.addEventListener('resize', () => chart.resize())
}
</script>

<style scoped>
.dashboard {
  padding: 20px;
}

.stat-card {
  display: flex;
  align-items: center;
  padding: 10px;
}

.stat-card :deep(.el-card__body) {
  display: flex;
  align-items: center;
  width: 100%;
}

.stat-icon {
  width: 60px;
  height: 60px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  margin-right: 15px;
}

.stat-content {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: bold;
  color: #303133;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 5px;
}

.quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.quick-actions .el-button {
  flex: 1;
  min-width: 120px;
}
</style>