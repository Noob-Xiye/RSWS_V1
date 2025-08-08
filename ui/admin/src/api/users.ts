import { apiClient } from './index';

export const userManagementAPI = {
  // 获取用户列表
  getUsers: (params: {
    page?: number;
    pageSize?: number;
    search?: string;
    status?: string;
    level?: number;
    startDate?: string;
    endDate?: string;
  }) => apiClient.get('/api/admin/users', { params }),
  
  // 获取用户统计
  getUserStats: () => apiClient.get('/api/admin/users/stats'),
  
  // 获取用户详情
  getUserDetail: (id: string) => apiClient.get(`/api/admin/users/${id}`),
  
  // 更新用户信息
  updateUser: (id: string, data: {
    username?: string;
    email?: string;
    level?: number;
    commissionRate?: number;
  }) => apiClient.put(`/api/admin/users/${id}`, data),
  
  // 更新用户状态
  updateUserStatus: (id: string, status: string) => 
    apiClient.post(`/api/admin/users/${id}/status`, { status }),
  
  // 删除用户
  deleteUser: (id: string) => apiClient.delete(`/api/admin/users/${id}`),
  
  // 重置用户密码
  resetUserPassword: (id: string) => 
    apiClient.post(`/api/admin/users/${id}/reset-password`),
  
  // 获取用户登录日志
  getUserLoginLogs: (id: string, params?: {
    page?: number;
    pageSize?: number;
  }) => apiClient.get(`/api/admin/users/${id}/login-logs`, { params }),
  
  // 获取用户操作日志
  getUserOperationLogs: (id: string, params?: {
    page?: number;
    pageSize?: number;
  }) => apiClient.get(`/api/admin/users/${id}/operation-logs`, { params }),
  
  // 批量操作用户
  batchUpdateUsers: (data: {
    userIds: string[];
    action: 'activate' | 'deactivate' | 'ban' | 'delete';
  }) => apiClient.post('/api/admin/users/batch', data),
  
  // 导出用户数据
  exportUsers: (params: {
    format: 'csv' | 'excel';
    filters?: any;
  }) => apiClient.get('/api/admin/users/export', { 
    params,
    responseType: 'blob'
  }),
};