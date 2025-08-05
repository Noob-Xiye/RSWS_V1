import axios from 'axios';
import { API_BASE_URL } from '../config';

const api = axios.create({
  baseURL: API_BASE_URL,
});

// 请求拦截器，添加认证头
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('adminToken');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// 响应拦截器，处理认证错误
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response && error.response.status === 401) {
      localStorage.removeItem('adminToken');
      localStorage.removeItem('adminInfo');
      window.location.href = '/admin/login';
    }
    return Promise.reject(error);
  }
);

// 管理员登录
export const login = async (email: string, password: string) => {
  const response = await api.post('/admin/auth/login', { email, password });
  return response.data;
};

// 获取管理员信息
export const getAdminInfo = async () => {
  const response = await api.get('/admin/admins/me');
  return response.data;
};

// 创建管理员
export const createAdmin = async (adminData: any) => {
  const response = await api.post('/admin/admins', adminData);
  return response.data;
};

// 获取管理员列表
export const getAdmins = async (page = 1, pageSize = 10, role?: string) => {
  const params = { page, page_size: pageSize, role };
  const response = await api.get('/admin/admins', { params });
  return response.data;
};

// 更新管理员信息
export const updateAdmin = async (id: number, adminData: any) => {
  const response = await api.put(`/admin/admins/${id}`, adminData);
  return response.data;
};