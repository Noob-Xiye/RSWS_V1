// 新增PayPal相关API
import { apiClient } from './index';

export const paypalAPI = {
  // 获取PayPal账户列表
  getAccounts: () => apiClient.get('/api/user/paypal/accounts'),
  
  // 添加PayPal账户
  addAccount: (data: { email: string }) => 
    apiClient.post('/api/user/paypal/accounts', data),
  
  // 删除PayPal账户
  deleteAccount: (id: string) => 
    apiClient.delete(`/api/user/paypal/accounts/${id}`),
  
  // 获取PayPal余额
  getBalance: (accountId: string) => 
    apiClient.get(`/api/user/paypal/accounts/${accountId}/balance`),
  
  // 跨平台转换
  crossPlatformExchange: (data: {
    fromPlatform: 'paypal' | 'usdt';
    toPlatform: 'paypal' | 'usdt';
    amount: number;
  }) => apiClient.post('/api/user/cross-platform/exchange', data),
  
  // 获取汇率
  getExchangeRate: (from: string, to: string) => 
    apiClient.get(`/api/user/cross-platform/rate?from=${from}&to=${to}`),
};