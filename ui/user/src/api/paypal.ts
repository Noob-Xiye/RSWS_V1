import { apiClient } from './index';

export const paypalAPI = {
  // 获取PayPal账户列表
  getAccounts: () => apiClient.get('/api/user/paypal/accounts'),
  
  // 添加PayPal账户
  addAccount: (data: { email: string; description?: string }) => 
    apiClient.post('/api/user/paypal/accounts', data),
  
  // 更新PayPal账户
  updateAccount: (id: string, data: { email?: string; description?: string }) => 
    apiClient.put(`/api/user/paypal/accounts/${id}`, data),
  
  // 删除PayPal账户
  deleteAccount: (id: string) => 
    apiClient.delete(`/api/user/paypal/accounts/${id}`),
  
  // 设置默认账户
  setDefaultAccount: (id: string) => 
    apiClient.post(`/api/user/paypal/accounts/${id}/set-default`),
  
  // 同步余额
  syncBalance: (id: string) => 
    apiClient.post(`/api/user/paypal/accounts/${id}/sync-balance`),
  
  // 获取PayPal余额
  getBalance: (accountId: string) => 
    apiClient.get(`/api/user/paypal/accounts/${accountId}/balance`),
  
  // 跨平台转换
  crossPlatformExchange: (data: {
    fromPlatform: 'paypal' | 'usdt';
    toPlatform: 'paypal' | 'usdt';
    amount: number;
    fromAccountId?: string;
    toAccountId?: string;
  }) => apiClient.post('/api/user/cross-platform/exchange', data),
  
  // 获取汇率
  getExchangeRate: (from: string, to: string) => 
    apiClient.get(`/api/user/cross-platform/rate?from=${from}&to=${to}`),
  
  // 获取转换历史
  getExchangeHistory: (params: {
    page?: number;
    pageSize?: number;
    startDate?: string;
    endDate?: string;
  }) => apiClient.get('/api/user/cross-platform/history', { params }),
};