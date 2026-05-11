# 前端重构计划

**日期**: 2026-05-05
**状态**: 待实施
**技术栈**: Vue 3 + Bun + TypeScript + Vite

---

## 一、技术栈选择

| 技术 | 版本 | 用途 |
|------|------|------|
| **Vue** | 3.4+ | 前端框架 |
| **Bun** | 1.0+ | 运行时 + 包管理器 + 打包器 |
| **Vite** | 5.0+ | 开发服务器 + 构建工具 |
| **TypeScript** | 5.0+ | 类型安全 |
| **Pinia** | 2.0+ | 状态管理 |
| **Vue Router** | 4.0+ | 路由 |
| **Element Plus** | 2.0+ | UI 组件库 |
| **Axios** | 1.0+ | HTTP 客户端 |

---

## 二、项目结构

```
frontend/
├── src/
│   ├── api/                 # API 请求
│   │   ├── auth.ts          # 认证 API
│   │   ├── resource.ts      # 资源 API
│   │   ├── order.ts         # 订单 API
│   │   ├── payment.ts       # 支付 API
│   │   └── index.ts         # API 统一导出
│   │
│   ├── assets/              # 静态资源
│   │   ├── images/
│   │   └── styles/
│   │       ├── global.scss
│   │       └── variables.scss
│   │
│   ├── components/          # 通用组件
│   │   ├── common/
│   │   │   ├── Header.vue
│   │   │   ├── Footer.vue
│   │   │   └── Loading.vue
│   │   ├── business/
│   │   │   ├── ResourceCard.vue
│   │   │   ├── OrderCard.vue
│   │   │   └── PaymentMethod.vue
│   │   └── ui/
│   │       └── Button.vue
│   │
│   ├── composables/         # 组合式函数 (hooks)
│   │   ├── useAuth.ts
│   │   ├── useResource.ts
│   │   ├── usePayment.ts
│   │   └── useNotification.ts
│   │
│   ├── layouts/             # 布局组件
│   │   ├── DefaultLayout.vue
│   │   ├── AdminLayout.vue
│   │   └── AuthLayout.vue
│   │
│   ├── pages/               # 页面组件
│   │   ├── home/
│   │   │   └── Index.vue
│   │   ├── auth/
│   │   │   ├── Login.vue
│   │   │   ├── Register.vue
│   │   │   └── ForgotPassword.vue
│   │   ├── resource/
│   │   │   ├── List.vue
│   │   │   ├── Detail.vue
│   │   │   └── Upload.vue
│   │   ├── order/
│   │   │   ├── List.vue
│   │   │   └── Detail.vue
│   │   ├── payment/
│   │   │   ├── Checkout.vue
│   │   │   └── Result.vue
│   │   ├── user/
│   │   │   ├── Profile.vue
│   │   │   └── Settings.vue
│   │   └── admin/
│   │       ├── Dashboard.vue
│   │       ├── ResourceManage.vue
│   │       ├── OrderManage.vue
│   │       └── ConfigManage.vue
│   │
│   ├── router/              # 路由配置
│   │   ├── index.ts
│   │   ├── routes.ts
│   │   └── guards.ts
│   │
│   ├── stores/              # Pinia 状态管理
│   │   ├── auth.ts
│   │   ├── resource.ts
│   │   ├── order.ts
│   │   └── config.ts
│   │
│   ├── types/               # TypeScript 类型定义
│   │   ├── api.d.ts
│   │   ├── resource.d.ts
│   │   ├── order.d.ts
│   │   └── user.d.ts
│   │
│   ├── utils/               # 工具函数
│   │   ├── request.ts       # HTTP 请求封装
│   │   ├── storage.ts       # 本地存储
│   │   ├── format.ts        # 格式化
│   │   └── validation.ts    # 表单验证
│   │
│   ├── App.vue              # 根组件
│   ├── main.ts              # 入口文件
│   └── env.d.ts             # 环境类型
│
├── public/                  # 公共静态资源
│   └── favicon.ico
│
├── .env                     # 环境变量
├── .env.development
├── .env.production
├── index.html               # HTML 模板
├── package.json
├── tsconfig.json
├── vite.config.ts
└── bunfig.toml              # Bun 配置
```

---

## 三、核心配置

### 1. package.json

```json
{
  "name": "rsws-frontend",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "bunx vite",
    "build": "bunx vite build",
    "preview": "bunx vite preview",
    "lint": "eslint src --ext .vue,.ts",
    "type-check": "vue-tsc --noEmit"
  },
  "dependencies": {
    "vue": "^3.4.0",
    "vue-router": "^4.3.0",
    "pinia": "^2.1.0",
    "element-plus": "^2.5.0",
    "axios": "^1.6.0",
    "@element-plus/icons-vue": "^2.3.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.3.0",
    "vue-tsc": "^1.8.0",
    "sass": "^1.69.0",
    "@types/node": "^20.10.0"
  }
}
```

### 2. vite.config.ts

```typescript
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
});
```

### 3. main.ts

```typescript
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import ElementPlus from 'element-plus';
import 'element-plus/dist/index.css';
import * as ElementPlusIcons from '@element-plus/icons-vue';

import App from './App.vue';
import router from './router';

const app = createApp(App);

// 注册 Element Plus 图标
for (const [key, component] of Object.entries(ElementPlusIcons)) {
  app.component(key, component);
}

app.use(createPinia());
app.use(router);
app.use(ElementPlus);

app.mount('#app');
```

---

## 四、核心模块设计

### 1. API 请求封装 (utils/request.ts)

```typescript
import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import { ElMessage } from 'element-plus';
import { useAuthStore } from '@/stores/auth';

const instance: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 30000,
});

// 请求拦截器：添加签名
instance.interceptors.request.use((config) => {
  const authStore = useAuthStore();
  if (authStore.apiKey && authStore.apiSecret) {
    const timestamp = Date.now();
    const nonce = Math.random().toString(36).substring(2);
    const signature = generateSignature({
      method: config.method?.toUpperCase() || 'GET',
      path: config.url || '/',
      timestamp,
      nonce,
      body: config.data,
      secret: authStore.apiSecret,
    });

    config.headers['X-Api-Key'] = authStore.apiKey;
    config.headers['X-Timestamp'] = timestamp;
    config.headers['X-Nonce'] = nonce;
    config.headers['X-Signature'] = signature;
  }
  return config;
});

// 响应拦截器
instance.interceptors.response.use(
  (response) => response.data,
  (error) => {
    if (error.response?.status === 401) {
      // 跳转登录
      const authStore = useAuthStore();
      authStore.logout();
    }
    ElMessage.error(error.response?.data?.message || '请求失败');
    return Promise.reject(error);
  }
);

export default instance;
```

### 2. 认证 Store (stores/auth.ts)

```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { login, logout } from '@/api/auth';
import type { User, LoginRequest } from '@/types';

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null);
  const apiKey = ref<string | null>(localStorage.getItem('api_key'));
  const apiSecret = ref<string | null>(localStorage.getItem('api_secret'));

  const isLoggedIn = computed(() => !!apiKey.value && !!user.value);

  async function signIn(request: LoginRequest) {
    const response = await login(request);
    user.value = response.user_info;
    apiKey.value = response.api_key;
    apiSecret.value = response.api_secret;

    localStorage.setItem('api_key', response.api_key);
    localStorage.setItem('api_secret', response.api_secret);
  }

  async function signOut() {
    await logout();
    user.value = null;
    apiKey.value = null;
    apiSecret.value = null;
    localStorage.removeItem('api_key');
    localStorage.removeItem('api_secret');
  }

  return {
    user,
    apiKey,
    apiSecret,
    isLoggedIn,
    signIn,
    signOut,
  };
});
```

### 3. 资源页面 (pages/resource/List.vue)

```vue
<template>
  <div class="resource-list">
    <!-- 搜索栏 -->
    <el-input
      v-model="searchQuery"
      placeholder="搜索资源..."
      @input="handleSearch"
    >
      <template #prefix>
        <el-icon><Search /></el-icon>
      </template>
    </el-input>

    <!-- 资源列表 -->
    <div class="grid" v-loading="loading">
      <ResourceCard
        v-for="resource in resources"
        :key="resource.id"
        :resource="resource"
        @click="goToDetail(resource.id)"
      />
    </div>

    <!-- 分页 -->
    <el-pagination
      v-model:current-page="page"
      :total="total"
      :page-size="pageSize"
      @change="fetchResources"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { Search } from '@element-plus/icons-vue';
import ResourceCard from '@/components/business/ResourceCard.vue';
import { getResources } from '@/api/resource';
import type { Resource } from '@/types';

const router = useRouter();

const resources = ref<Resource[]>([]);
const loading = ref(false);
const searchQuery = ref('');
const page = ref(1);
const pageSize = ref(20);
const total = ref(0);

async function fetchResources() {
  loading.value = true;
  try {
    const response = await getResources({
      page: page.value,
      page_size: pageSize.value,
      query: searchQuery.value,
    });
    resources.value = response.items;
    total.value = response.total;
  } finally {
    loading.value = false;
  }
}

function goToDetail(id: number) {
  router.push(`/resource/${id}`);
}

let searchTimer: number;
function handleSearch() {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    page.value = 1;
    fetchResources();
  }, 300);
}

onMounted(fetchResources);
</script>

<style scoped lang="scss">
.resource-list {
  padding: 20px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  margin-top: 20px;
}
</style>
```

---

## 五、与后端 API 对接

### API 签名机制 (Cregis 方案)

前端请求需在 Query 参数中携带签名：

```
?api_key=ak_xxxxxxxxxxxx
&timestamp=1714848000000
&nonce=abc123
&sign=md5_hex_signature
```

### 签名生成 (Cregis MD5)

```typescript
import CryptoJS from 'crypto-js';

/**
 * Cregis 签名算法
 * 1. 排除 sign 参数，按 key ASCII 升序排序
 * 2. 拼接: api_secret + key1 + value1 + key2 + value2 + ...
 * 3. MD5 计算并转小写 hex
 */
function generateSignature(
  params: Record<string, string>,
  apiSecret: string
): string {
  // 1. 排除 sign 参数，排序
  const keys = Object.keys(params)
    .filter(key => key !== 'sign')
    .sort();
  
  // 2. 拼接 key + value
  const paramStr = keys.map(key => key + params[key]).join('');
  
  // 3. 拼接 api_secret 并计算 MD5
  const signStr = apiSecret + paramStr;
  return CryptoJS.MD5(signStr).toString();
}

// 使用示例
function makeAuthenticatedRequest(url: string, apiKey: string, apiSecret: string) {
  const timestamp = Date.now().toString();
  const nonce = Math.random().toString(36).substring(2, 15);
  
  const params: Record<string, string> = {
    api_key: apiKey,
    timestamp,
    nonce,
  };
  
  // 添加签名
  params.sign = generateSignature(params, apiSecret);
  
  // 构建带签名的 URL
  const queryString = new URLSearchParams(params).toString();
  return fetch(`${url}?${queryString}`);
}
```

---

## 六、开发计划

| 阶段 | 任务 | 时间 |
|------|------|------|
| **Phase 1** | 项目搭建 + 基础配置 | 0.5 天 |
| **Phase 2** | API 层 + 认证模块 | 1 天 |
| **Phase 3** | 用户端页面 (首页、资源、订单) | 2 天 |
| **Phase 4** | 支付模块 (PayPal + USDT) | 1 天 |
| **Phase 5** | 管理后台 | 1.5 天 |
| **Phase 6** | 测试 + 优化 | 1 天 |

**总计: 约 7 天**

---

## 七、Bun 优势

| 特性 | Bun | npm/pnpm |
|------|-----|----------|
| **安装速度** | 🚀 20x 更快 | 基准 |
| **运行时** | 内置 | 需 Node.js |
| **打包器** | 内置 | 需 Webpack/Vite |
| **测试运行器** | 内置 | 需 Jest |
| **TypeScript** | 原生支持 | 需编译 |

### 安装 Bun

```bash
# Windows (PowerShell)
powershell -c "irm bun.sh/install.ps1 | iex"

# 或使用 scoop
scoop install bun
```

### 使用 Bun

```bash
# 安装依赖
bun install

# 启动开发服务器
bun dev

# 构建
bun run build
```

---

## 八、下一步

1. 安装 Bun 运行时
2. 创建项目目录
3. 初始化 package.json
4. 配置 Vite + TypeScript
5. 开始开发

---

**准备好开始了吗？**
