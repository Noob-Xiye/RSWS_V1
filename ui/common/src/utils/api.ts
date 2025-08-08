import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

// API客户端配置
interface ApiClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
}

// 创建API客户端
export const createApiClient = (config: ApiClientConfig): AxiosInstance => {
  const client = axios.create({
    baseURL: config.baseURL,
    timeout: config.timeout || 10000,
    headers: {
      'Content-Type': 'application/json',
      ...config.headers,
    },
  });
  
  // 请求拦截器
  client.interceptors.request.use(
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
  client.interceptors.response.use(
    (response: AxiosResponse) => response.data,
    (error) => {
      if (error.response?.status === 401) {
        localStorage.removeItem('token');
        window.location.href = '/auth/login';
      }
      return Promise.reject(error);
    }
  );
  
  return client;
};

// API响应包装器
export const apiWrapper = {
  get: <T = any>(client: AxiosInstance, url: string, config?: AxiosRequestConfig) => 
    client.get<T>(url, config),
  post: <T = any>(client: AxiosInstance, url: string, data?: any, config?: AxiosRequestConfig) => 
    client.post<T>(url, data, config),
  put: <T = any>(client: AxiosInstance, url: string, data?: any, config?: AxiosRequestConfig) => 
    client.put<T>(url, data, config),
  delete: <T = any>(client: AxiosInstance, url: string, config?: AxiosRequestConfig) => 
    client.delete<T>(url, config),
};