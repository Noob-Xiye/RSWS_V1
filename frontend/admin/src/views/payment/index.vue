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

        <el-tab-pane label="支付方式" name="methods">
          <div style="max-width: 700px">
            <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:16px">
              <h3 style="margin:0">支付方式管理</h3>
              <el-button type="primary" @click="openMethodDialog()">新增方式</el-button>
            </div>
            <el-table :data="methodList" border stripe size="small" empty-text="暂无支付方式">
              <el-table-column prop="method_type" label="类型" width="120" />
              <el-table-column prop="method_name" label="名称" width="140" />
              <el-table-column label="状态" width="90" align="center">
                <template #default="{ row }">
                  <el-tag :type="row.is_enabled ? 'success' : 'info'" size="small">
                    {{ row.is_enabled ? '启用' : '禁用' }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column label="操作" width="160" align="center">
                <template #default="{ row }">
                  <el-button size="small" @click="openMethodDialog(row)">编辑</el-button>
                  <el-button size="small" type="danger" @click="handleDeleteMethod(row)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 新增/编辑支付方式弹窗 -->
    <el-dialog
      :title="methodForm.id ? '编辑支付方式' : '新增支付方式'"
      v-model="methodDialogVisible"
      width="500px"
    >
      <el-form :model="methodForm" :rules="methodRules" ref="methodFormRef" label-width="100px">
        <el-form-item label="类型标识" prop="method_type">
          <el-input v-model="methodForm.method_type" :disabled="!!methodForm.id" placeholder="如: paypal, usdt, wechat" />
        </el-form-item>
        <el-form-item label="显示名称" prop="method_name">
          <el-input v-model="methodForm.method_name" placeholder="如: PayPal、USDT" />
        </el-form-item>
        <el-form-item label="是否启用">
          <el-switch v-model="methodForm.is_enabled" />
        </el-form-item>
        <el-form-item label="配置 (JSON)">
          <el-input
            v-model="methodForm.config"
            type="textarea"
            :rows="4"
            placeholder='可选，如: {"client_id": "xxx"}'
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="methodDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSaveMethod">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox, type FormInstance } from 'element-plus'
import { listUsdtWallets, updateUsdtWallet } from '@/api/admin'
import {
  listPaymentMethods,
  createPaymentMethod,
  deletePaymentMethod,
} from '@/api/payment'

const activeTab = ref('usdt')
const usdtLoading = ref(false)
const methodDialogVisible = ref(false)
const methodList = ref<any[]>([])
const methodFormRef = ref<FormInstance>()

const usdtForm = reactive({ trc20: '', erc20: '' })
const methodForm = reactive({
  id: 0,
  method_type: '',
  method_name: '',
  is_enabled: true,
  config: '{}',
})

const methodRules = {
  method_type: [{ required: true, message: '请填写类型标识', trigger: 'blur' }],
  method_name: [{ required: true, message: '请填写显示名称', trigger: 'blur' }],
}

// ========== USDT 钱包 ==========
async function fetchUsdtWallets() {
  try {
    const res = await listUsdtWallets()
    if (res.code === 0 && res.data) {
      for (const w of res.data) {
        if (w.network === 'TRC20') usdtForm.trc20 = w.address
        if (w.network === 'ERC20') usdtForm.erc20 = w.address
      }
    }
  } catch { /* ignore */ }
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

// ========== 支付方式管理 ==========
async function fetchMethods() {
  try {
    const res = await listPaymentMethods()
    if (res.code === 0 && res.data) methodList.value = res.data
  } catch (e: any) {
    ElMessage.error(e.message || '获取支付方式失败')
  }
}

function openMethodDialog(row?: any) {
  if (row) {
    methodForm.id = row.id
    methodForm.method_type = row.method_type
    methodForm.method_name = row.method_name
    methodForm.is_enabled = row.is_enabled
    methodForm.config = JSON.stringify(row.config || {}, null, 2)
  } else {
    methodForm.id = 0
    methodForm.method_type = ''
    methodForm.method_name = ''
    methodForm.is_enabled = true
    methodForm.config = '{}'
  }
  methodDialogVisible.value = true
}

async function handleSaveMethod() {
  const ok = await (methodFormRef.value?.validate().catch(() => false))
  if (!ok) return
  let parsedConfig = {}
  try { parsedConfig = JSON.parse(methodForm.config) } catch {
    ElMessage.error('config 必须是合法 JSON'); return
  }
  try {
    await createPaymentMethod({
      method_type: methodForm.method_type,
      method_name: methodForm.method_name,
      is_enabled: methodForm.is_enabled,
      config: parsedConfig,
    })
    ElMessage.success('保存成功')
    methodDialogVisible.value = false
    await fetchMethods()
  } catch (e: any) {
    ElMessage.error(e.message || '保存失败')
  }
}

async function handleDeleteMethod(row: any) {
  try {
    await ElMessageBox.confirm(`确认禁用「${row.method_name}」？`, '提示', { type: 'warning' })
    await deletePaymentMethod(row.id)
    ElMessage.success('已禁用')
    await fetchMethods()
  } catch (e: any) {
    if (e !== 'cancel' && e?.message) ElMessage.error(e.message)
  }
}

onMounted(() => {
  fetchUsdtWallets()
  fetchMethods()
})
</script>

<style scoped>
.page-container { padding: 20px; }
</style>
