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
            <div class="stat-label">总用户数 <span class="sub">(近30天 +{{ stats.new_users_30d }})</span></div>
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
            <div class="stat-label">总资源数 <span class="sub">(活跃 {{ stats.active_resources }})</span></div>
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
            <div class="stat-label">总订单数 <span class="sub">(完成 {{ stats.completed_orders }})</span></div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-icon" style="background: #f56c6c">
            <el-icon :size="32"><Money /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ formatRevenue(stats.total_revenue) }}</div>
            <div class="stat-label">总收入 (USDT) <span class="sub">(近30天 {{ formatRevenue(stats.revenue_30d) }})</span></div>
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
import type { DashboardStats, DailyOrderCount } from '@/api/dashboard'
import { getDashboardStats } from '@/api/dashboard'
import * as echarts from 'echarts'

const chartRef = ref<HTMLDivElement>()

const stats = ref<DashboardStats>({
  total_users: 0,
  new_users_30d: 0,
  total_orders: 0,
  completed_orders: 0,
  total_revenue: 0,
  revenue_30d: 0,
  total_resources: 0,
  active_resources: 0,
  new_resources_30d: 0,
  orders_trend: []
})

// 收入单位：分 -> 元
function formatRevenue(cents: number): string {
  return (cents / 100).toFixed(2)
}

onMounted(async () => {
  try {
    const res = await getDashboardStats()
    if (res.code === 0 && res.data) {
      stats.value = res.data
    }
  } catch {
    // 使用默认数据
  }
  await nextTick()
  initChart(stats.value.orders_trend)
})

function initChart(trend: DailyOrderCount[]) {
  if (!chartRef.value) return
  const chart = echarts.init(chartRef.value)
  
  // 使用真实数据或生成占位数据
  const dates: string[] = []
  const counts: number[] = []
  
  if (trend.length > 0) {
    trend.forEach(item => {
      // date 格式 YYYY-MM-DD -> M/D
      const [y, m, d] = item.date.split('-')
      dates.push(`${parseInt(m)}/${parseInt(d)}`)
      counts.push(item.count)
    })
  } else {
    // 无数据时显示占位
    for (let i = 29; i >= 0; i--) {
      const d = new Date()
      d.setDate(d.getDate() - i)
      dates.push(`${d.getMonth() + 1}/${d.getDate()}`)
      counts.push(0)
    }
  }
  
  chart.setOption({
    tooltip: { trigger: 'axis' },
    grid: { left: 50, right: 20, top: 10, bottom: 30 },
    xAxis: { type: 'category', data: dates, axisLabel: { fontSize: 11 } },
    yAxis: { type: 'value', axisLabel: { formatter: '{value} 单' } },
    series: [{
      name: '日订单数',
      type: 'line',
      smooth: true,
      data: counts,
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

.stat-label .sub {
  font-size: 12px;
  color: #c0c4cc;
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