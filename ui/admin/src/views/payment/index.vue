<template>
  <div class="page-container">
    <el-card>
      <template #header>
        <span>支付配置</span>
      </template>
      
      <el-tabs v-model="activeTab">
        <el-tab-pane label="USDT 地址" name="usdt">
          <el-form label-width="120px">
            <el-form-item label="TRC20 地址">
              <el-input v-model="usdtForm.trc20" placeholder="输入 TRC20 收款地址" style="max-width: 500px" />
            </el-form-item>
            <el-form-item label="ERC20 地址">
              <el-input v-model="usdtForm.erc20" placeholder="输入 ERC20 收款地址" style="max-width: 500px" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" :loading="usdtLoading" @click="handleSaveUsdt">保存</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
        
        <el-tab-pane label="PayPal" name="paypal">
          <el-form label-width="120px">
            <el-form-item label="Client ID">
              <el-input v-model="paypalForm.client_id" placeholder="PayPal Client ID" style="max-width: 500px" />
            </el-form-item>
            <el-form-item label="Secret">
              <el-input v-model="paypalForm.secret" type="password" placeholder="PayPal Secret" show-password style="max-width: 500px" />
            </el-form-item>
            <el-form-item label="模式">
              <el-radio-group v-model="paypalForm.mode">
                <el-radio value="sandbox">沙盒</el-radio>
                <el-radio value="live">正式</el-radio>
              </el-radio-group>
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="handleSavePaypal">保存</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
        
        <el-tab-pane label="API Key 管理" name="apikey">
          <div class="apikey-header">
            <el-button type="primary" size="small" @click="showCreateApiKey">新建 API Key</el-button>
          </div>
          <el-table :data="apiKeys" stripe>
            <el-table-column prop="name" label="名称" />
            <el-table-column prop="api_key" label="Key">
              <template #default="{ row }">
                <code class="apikey-code">{{ row.api_key?.substring(0, 20) }}...</code>
              </template>
            </el-table-column>
            <el-table-column prop="is_active" label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.is_active ? 'success' : 'danger'">{{ row.is_active ? '正常' : '禁用' }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="created_at" label="创建时间" width="180">
              <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
            </el-table-column>
            <el-table-column label="操作" width="100">
              <template #default="{ row }">
                <el-button type="danger" size="small" link @click="handleDeleteApiKey(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </el-card>
    
    <!-- 创建 API Key 对话框 -->
    <el-dialog v-model="createApiKeyVisible" title="新建 API Key" width="400px">
      <el-form :model="newApiKeyForm" label-width="80px">
        <el-form-item label="名称">
          <el-input v-model="newApiKeyForm.name" placeholder="API Key 名称" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createApiKeyVisible = false">取消</el-button>
        <el-button type="primary" @click="handleCreateApiKey">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { AdminApiKeyResponse } from '@/api/admin'
import { listApiKeys, createApiKey, deleteApiKey, listUsdtWallets, updateUsdtWallet } from '@/api/admin'

const activeTab = ref('usdt')
const usdtLoading = ref(false)

const usdtForm = reactive({ trc20: '', erc20: '' })
const paypalForm = reactive({ client_id: '', secret: '', mode: 'sandbox' as 'sandbox' | 'live' })

const apiKeys = ref<AdminApiKeyResponse[]>([])
const createApiKeyVisible = ref(false)
const newApiKeyForm = reactive({ name: '' })

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN')
}

// ========== USDT 钱包 ==========
async function fetchUsdtWallets() {
  try {
    const res = await listUsdtWallets()
    if (res.code === 0 && res.data) {
      // 按 network 填充表单
      for (const wallet of res.data) {
        if (wallet.network === 'TRC20') usdtForm.trc20 = wallet.address
        if (wallet.network === 'ERC20') usdtForm.erc20 = wallet.address
      }
    }
  } catch {
    // ignore
  }
}

async function handleSaveUsdt() {
  if (!usdtForm.trc20 && !usdtForm.erc20) {
    ElMessage.warning('请至少填写一个钱包地址')
    return
  }
  usdtLoading.value = true
  try {
    if (usdtForm.trc20) await updateUsdtWallet('TRC20', usdtForm.trc20)
    if (usdtForm.erc20) await updateUsdtWallet('ERC20', usdtForm.erc20)
    ElMessage.success('USDT 钱包地址已保存')
  } catch {
    ElMessage.error('保存失败，请重试')
  } finally {
    usdtLoading.value = false
  }
}

// ========== PayPal（后端暂无 CRUD 端点，暂时 mock）==========
function handleSavePaypal() {
  ElMessage.info('PayPal 配置管理功能开发中')
}

// ========== API Key ==========
async function fetchApiKeys() {
  try {
    const res = await listApiKeys()
    if (res.code === 0 && res.data) {
      apiKeys.value = res.data
    }
  } catch {
    apiKeys.value = []
  }
}

function showCreateApiKey() {
  newApiKeyForm.name = ''
  createApiKeyVisible.value = true
}

async function handleCreateApiKey() {
  if (!newApiKeyForm.name) {
    ElMessage.warning('请输入名称')
    return
  }
  try {
    await createApiKey(newApiKeyForm)
    ElMessage.success('创建成功')
    createApiKeyVisible.value = false
    fetchApiKeys()
  } catch {
    ElMessage.error('创建失败')
  }
}

async function handleDeleteApiKey(row: AdminApiKeyResponse) {
  try {
    await ElMessageBox.confirm('确定删除此 API Key？', '确认', { type: 'warning' })
    await deleteApiKey(row.id)
    ElMessage.success('已删除')
    fetchApiKeys()
  } catch {}
}

onMounted(() => {
  fetchUsdtWallets()
  fetchApiKeys()
})
</script>

<style scoped>
.page-container { padding: 20px; }
.apikey-header { margin-bottom: 20px; }
.apikey-code { background: #f5f5f5; padding: 2px 6px; border-radius: 4px; }
</style>
