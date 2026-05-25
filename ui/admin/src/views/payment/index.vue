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
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { listUsdtWallets, updateUsdtWallet } from '@/api/admin'

const activeTab = ref('usdt')
const usdtLoading = ref(false)

const usdtForm = reactive({ trc20: '', erc20: '' })
const paypalForm = reactive({ client_id: '', secret: '', mode: 'sandbox' as 'sandbox' | 'live' })

// ========== USDT 钱包 ==========
async function fetchUsdtWallets() {
  try {
    const res = await listUsdtWallets()
    if (res.code === 0 && res.data) {
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

onMounted(() => {
  fetchUsdtWallets()
})
</script>

<style scoped>
.page-container { padding: 20px; }
</style>