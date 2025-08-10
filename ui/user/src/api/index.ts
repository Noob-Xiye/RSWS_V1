import axios from 'axios';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json'
  }
});

// 请求拦截器
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// 响应拦截器
apiClient.interceptors.response.use(
  (response) => response.data,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      window.location.href = '/auth/login';
    }
    return Promise.reject(error);
  }
);

// 认证相关API
export const authAPI = {
  login: (email: string, password: string) => 
    apiClient.post('/api/auth/login', { email, password }),
  register: (userData: any) => 
    apiClient.post('/api/auth/register', userData),
  logout: () => 
    apiClient.post('/api/auth/logout'),
  refreshToken: () => 
    apiClient.post('/api/auth/refresh'),
  forgotPassword: (email: string) => 
    apiClient.post('/api/auth/forgot-password', { email }),
  resetPassword: (token: string, password: string) => 
    apiClient.post('/api/auth/reset-password', { token, password })
};

// 用户相关API
// 更新用户API路径以匹配后端路由
export const userAPI = {
  getProfile: () => 
    apiClient.get('/api/user/profile'),
  updateProfile: (data: any) => 
    apiClient.put('/api/user/profile', data),
  getWallet: () => 
    apiClient.get('/api/user/wallet'),
  getOrders: (params?: any) => 
    apiClient.get('/api/user/orders', { params }),
  getOrderDetail: (orderId: string) => 
    apiClient.get(`/api/orders/${orderId}`),
  getTransactions: (params?: any) => 
    apiClient.get('/api/user/transactions', { params })
};

// 资源相关API
export const resourceAPI = {
  getResources: (params?: any) => 
    apiClient.get('/api/resources', { params }),
  getResourceDetail: (id: string) => 
    apiClient.get(`/api/resources/${id}`),
  searchResources: (query: string, params?: any) => 
    apiClient.get('/api/resources/search', { params: { q: query, ...params } }),
  uploadResource: (data: FormData) => 
    apiClient.post('/api/resources', data, {
      headers: { 'Content-Type': 'multipart/form-data' }
    }),
  updateResource: (id: string, data: any) => 
    apiClient.put(`/api/resources/${id}`, data),
  deleteResource: (id: string) => 
    apiClient.delete(`/api/resources/${id}`),
  downloadResource: (id: string) => 
    apiClient.get(`/api/resources/${id}/download`, { responseType: 'blob' })
};

// 支付相关API
export const paymentAPI = {
  createOrder: (resourceId: string, paymentMethod: string) => 
    apiClient.post('/api/payment/create-order', { resourceId, paymentMethod }),
  getPaymentMethods: () => 
    apiClient.get('/api/payment/methods'),
  processPayment: (orderId: string, paymentData: any) => 
    apiClient.post(`/api/payment/process/${orderId}`, paymentData),
  getPaymentStatus: (orderId: string) => 
    apiClient.get(`/api/payment/status/${orderId}`),
  generateQRCode: (orderId: string) => 
    apiClient.get(`/api/payment/qrcode/${orderId}`)
};

// 配置相关API
export const configAPI = {
  getPublicConfig: () => 
    apiClient.get('/api/config/public'),
  getCategories: () => 
    apiClient.get('/api/config/categories'),
  getTags: () => 
    apiClient.get('/api/config/tags')
};