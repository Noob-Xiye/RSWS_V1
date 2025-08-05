import axios, { AxiosInstance, AxiosResponse } from 'axios';
import { message } from 'antd';
import {
  User,
  Resource,
  ResourceCategory,
  Order,
  ApiResponse,
  PaginatedResponse
} from '../types';

class ApiClient {
  private instance: AxiosInstance;

  constructor() {
    this.instance = axios.create({
      baseURL: process.env.REACT_APP_API_URL || 'http://localhost:8080/api',
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // 请求拦截器
    this.instance.interceptors.request.use(
      (config) => {
        const token = localStorage.getItem('token');
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
      },
      (error) => {
        return Promise.reject(error);
      }
    );

    // 响应拦截器
    this.instance.interceptors.response.use(
      (response: AxiosResponse) => {
        return response.data;
      },
      (error) => {
        if (error.response?.status === 401) {
          localStorage.removeItem('token');
          localStorage.removeItem('user');
          window.location.href = '/auth';
          message.error('登录已过期，请重新登录');
        } else if (error.response?.status >= 500) {
          message.error('服务器错误，请稍后重试');
        } else if (error.response?.data?.message) {
          message.error(error.response.data.message);
        } else {
          message.error('网络错误，请检查网络连接');
        }
        return Promise.reject(error);
      }
    );
  }

  public get<T = any>(url: string, params?: any): Promise<T> {
    return this.instance.get(url, { params });
  }

  public post<T = any>(url: string, data?: any): Promise<T> {
    return this.instance.post(url, data);
  }

  public put<T = any>(url: string, data?: any): Promise<T> {
    return this.instance.put(url, data);
  }

  public delete<T = any>(url: string): Promise<T> {
    return this.instance.delete(url);
  }

  public upload<T = any>(url: string, formData: FormData): Promise<T> {
    return this.instance.post(url, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
  }
}

const apiClient = new ApiClient();

// 认证服务
export class AuthService {
  static async login(email: string, password: string): Promise<ApiResponse<{ token: string; user: User }>> {
    return apiClient.post('/auth/login', { email, password });
  }

  static async register(userData: {
    username: string;
    email: string;
    password: string;
  }): Promise<ApiResponse<User>> {
    return apiClient.post('/auth/register', userData);
  }

  static async logout(): Promise<ApiResponse<null>> {
    return apiClient.post('/auth/logout');
  }

  static async refreshToken(): Promise<ApiResponse<{ token: string }>> {
    return apiClient.post('/auth/refresh');
  }

  static async forgotPassword(email: string): Promise<ApiResponse<null>> {
    return apiClient.post('/auth/forgot-password', { email });
  }

  static async resetPassword(token: string, password: string): Promise<ApiResponse<null>> {
    return apiClient.post('/auth/reset-password', { token, password });
  }
}

// 用户服务
export class UserService {
  static async getCurrentUser(): Promise<ApiResponse<User>> {
    return apiClient.get('/user/profile');
  }

  static async updateProfile(userData: Partial<User>): Promise<ApiResponse<User>> {
    return apiClient.put('/user/profile', userData);
  }

  static async changePassword(oldPassword: string, newPassword: string): Promise<ApiResponse<null>> {
    return apiClient.put('/user/password', { oldPassword, newPassword });
  }

  static async uploadAvatar(file: File): Promise<ApiResponse<{ avatarUrl: string }>> {
    const formData = new FormData();
    formData.append('avatar', file);
    return apiClient.upload('/user/avatar', formData);
  }

  static async getMyOrders(params?: {
    page?: number;
    limit?: number;
    status?: string;
  }): Promise<ApiResponse<PaginatedResponse<Order>>> {
    return apiClient.get('/user/orders', params);
  }

  static async getMyPurchases(params?: {
    page?: number;
    limit?: number;
  }): Promise<ApiResponse<PaginatedResponse<Resource>>> {
    return apiClient.get('/user/purchases', params);
  }

  // 在UserService中添加获取交易记录的方法
  static async getTransactions(params?: {
  page?: number;
  limit?: number;
  status?: string;
  startDate?: string;
  endDate?: string;
  paymentMethod?: string;
  }): Promise<ApiResponse<PaginatedResponse<Transaction>>> {
  return apiClient.get('/user/transactions', params);
  }
  
  // 在OrderService中添加获取订单列表的方法
  static async getUserOrders(params?: {
  page?: number;
  limit?: number;
  status?: string;
  startDate?: string;
  endDate?: string;
  }): Promise<ApiResponse<PaginatedResponse<Order>>> {
  return apiClient.get('/orders', params);
  }
}

// 资源服务
export class ResourceService {
  static async getResources(params?: {
    page?: number;
    limit?: number;
    category?: string;
    search?: string;
    sortBy?: string;
    sortOrder?: 'asc' | 'desc';
    minPrice?: number;
    maxPrice?: number;
  }): Promise<ApiResponse<PaginatedResponse<Resource>>> {
    return apiClient.get('/resources', params);
  }

  static async getResourceById(id: string): Promise<ApiResponse<Resource>> {
    return apiClient.get(`/resources/${id}`);
  }

  static async getMyResources(params?: {
    page?: number;
    limit?: number;
    status?: string;
  }): Promise<ApiResponse<PaginatedResponse<Resource>>> {
    return apiClient.get('/user/resources', params);
  }

  static async uploadResource(resourceData: {
    title: string;
    description: string;
    category: string;
    price: number;
    tags: string[];
    detailDescription?: string;
    specifications?: Record<string, any>;
    usageGuide?: string;
    precautions?: string;
    displayImages?: string[];
  }): Promise<ApiResponse<Resource>> {
    return apiClient.post('/resources', resourceData);
  }

  static async updateResource(id: string, resourceData: Partial<Resource>): Promise<ApiResponse<Resource>> {
    return apiClient.put(`/resources/${id}`, resourceData);
  }

  static async deleteResource(id: string): Promise<ApiResponse<null>> {
    return apiClient.delete(`/resources/${id}`);
  }

  static async uploadResourceFile(resourceId: string, file: File): Promise<ApiResponse<{ fileUrl: string }>> {
    const formData = new FormData();
    formData.append('file', file);
    return apiClient.upload(`/resources/${resourceId}/file`, formData);
  }

  static async downloadResource(resourceId: string): Promise<ApiResponse<{ downloadUrl: string; filename: string }>> {
    return apiClient.post(`/resources/${resourceId}/download`);
  }

  static async getCategories(): Promise<ApiResponse<ResourceCategory[]>> {
    return apiClient.get('/resources/categories');
  }

  static async searchResources(query: string, params?: {
    page?: number;
    limit?: number;
    category?: string;
  }): Promise<ApiResponse<PaginatedResponse<Resource>>> {
    return apiClient.get('/resources/search', { q: query, ...params });
  }
}

// 订单服务
export class OrderService {
  static async createOrder(resourceId: string): Promise<ApiResponse<Order>> {
    return apiClient.post('/orders', { resourceId });
  }

  static async getOrderById(id: string): Promise<ApiResponse<Order>> {
    return apiClient.get(`/orders/${id}`);
  }

  static async payOrder(orderId: string, paymentMethod: string): Promise<ApiResponse<{ paymentUrl?: string }>> {
    return apiClient.post(`/orders/${orderId}/pay`, { paymentMethod });
  }

  static async cancelOrder(orderId: string): Promise<ApiResponse<null>> {
    return apiClient.post(`/orders/${orderId}/cancel`);
  }

  static async getOrderStatus(orderId: string): Promise<ApiResponse<{ status: string }>> {
    return apiClient.get(`/orders/${orderId}/status`);
  }
}

// 支付服务
export class PaymentService {
  static async getPaymentMethods(): Promise<ApiResponse<{ id: string; name: string; icon: string }[]>> {
    return apiClient.get('/payment/methods');
  }

  static async createPayment(orderId: string, method: string): Promise<ApiResponse<{ paymentUrl: string }>> {
    return apiClient.post('/payment/create', { orderId, method });
  }

  static async verifyPayment(paymentId: string): Promise<ApiResponse<{ status: string }>> {
    return apiClient.get(`/payment/${paymentId}/verify`);
  }
}

export default apiClient;