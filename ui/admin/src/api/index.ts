import { createApiClient, apiWrapper } from '@rsws/common';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';

const apiClient = createApiClient({
  baseURL: API_BASE_URL,
  timeout: 10000,
});

// 管理员认证API
// 更新管理员API路径以匹配后端路由
export const adminAPI = {
  login: (username: string, password: string) => 
    apiWrapper.post(apiClient, '/api/admin/auth/login', { username, password }),
  logout: () => 
    apiWrapper.post(apiClient, '/api/admin/auth/logout'),
  verifyToken: () => 
    apiWrapper.get(apiClient, '/api/admin/auth/verify'),
  refreshToken: () => 
    apiWrapper.post(apiClient, '/api/admin/auth/refresh'),
};

// 用户管理API路径更新
export const userManagementAPI = {
  getUsers: (params?: any) => 
    apiWrapper.get(apiClient, '/api/admin/users', { params }),
  getUserDetail: (userId: string) => 
    apiWrapper.get(apiClient, `/api/admin/users/${userId}`),
  updateUser: (userId: string, data: any) => 
    apiWrapper.put(apiClient, `/api/admin/users/${userId}`, data),
  deleteUser: (userId: string) => 
    apiWrapper.delete(apiClient, `/api/admin/users/${userId}`),
  banUser: (userId: string, reason: string) => 
    apiWrapper.post(apiClient, `/api/admin/users/${userId}/ban`, { reason }),
  unbanUser: (userId: string) => 
    apiWrapper.post(apiClient, `/api/admin/users/${userId}/unban`),
};

// 资源管理API
export const resourceManagementAPI = {
  getResources: (params?: any) => 
    apiWrapper.get(apiClient, '/api/admin/resources', { params }),
  getResourceDetail: (resourceId: string) => 
    apiWrapper.get(apiClient, `/api/admin/resources/${resourceId}`),
  approveResource: (resourceId: string) => 
    apiWrapper.post(apiClient, `/api/admin/resources/${resourceId}/approve`),
  rejectResource: (resourceId: string, reason: string) => 
    apiWrapper.post(apiClient, `/api/admin/resources/${resourceId}/reject`, { reason }),
  deleteResource: (resourceId: string) => 
    apiWrapper.delete(apiClient, `/api/admin/resources/${resourceId}`),
};

// 订单管理API
export const orderManagementAPI = {
  getOrders: (params?: any) => 
    apiWrapper.get(apiClient, '/api/admin/orders', { params }),
  getOrderDetail: (orderId: string) => 
    apiWrapper.get(apiClient, `/api/admin/orders/${orderId}`),
  updateOrderStatus: (orderId: string, status: string) => 
    apiWrapper.put(apiClient, `/api/admin/orders/${orderId}/status`, { status }),
  refundOrder: (orderId: string, reason: string) => 
    apiWrapper.post(apiClient, `/api/admin/orders/${orderId}/refund`, { reason }),
};

// 系统配置API
export const configAPI = {
  getConfig: () => 
    apiWrapper.get(apiClient, '/api/admin/config'),
  updateConfig: (config: any) => 
    apiWrapper.put(apiClient, '/api/admin/config', config),
  getPaymentMethods: () => 
    apiWrapper.get(apiClient, '/api/admin/config/payment-methods'),
  updatePaymentMethod: (methodId: string, config: any) => 
    apiWrapper.put(apiClient, `/api/admin/config/payment-methods/${methodId}`, config),
};

// 统计数据API
export const statisticsAPI = {
  getDashboardStats: () => 
    apiWrapper.get(apiClient, '/api/admin/statistics/dashboard'),
  getUserStats: (period: string) => 
    apiWrapper.get(apiClient, '/api/admin/statistics/users', { params: { period } }),
  getRevenueStats: (period: string) => 
    apiWrapper.get(apiClient, '/api/admin/statistics/revenue', { params: { period } }),
  getResourceStats: (period: string) => 
    apiWrapper.get(apiClient, '/api/admin/statistics/resources', { params: { period } }),
};