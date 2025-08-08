// 用户相关类型定义
export interface User {
  id: string;
  email: string;
  username: string;
  avatar?: string;
  createdAt: string;
  updatedAt: string;
  wallet?: Wallet;
}

export interface Wallet {
  id: string;
  userId: string;
  balance: number;
  currency: string;
  frozenAmount: number;
}

// 资源相关类型定义
export interface Resource {
  id: string;
  title: string;
  description: string;
  category: string;
  tags: string[];
  price: number;
  currency: string;
  fileUrl: string;
  thumbnailUrl?: string;
  downloadCount: number;
  rating: number;
  authorId: string;
  author: User;
  createdAt: string;
  updatedAt: string;
}

// 订单相关类型定义
export interface Order {
  id: string;
  userId: string;
  resourceId: string;
  resource: Resource;
  amount: number;
  currency: string;
  status: 'pending' | 'paid' | 'cancelled' | 'refunded';
  paymentMethod: string;
  paymentId?: string;
  createdAt: string;
  updatedAt: string;
}

// 支付相关类型定义
export interface PaymentMethod {
  id: string;
  name: string;
  type: 'crypto' | 'fiat';
  enabled: boolean;
  config: Record<string, any>;
}

// API响应类型定义
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  error?: string;
}

// 分页类型定义
export interface PaginationParams {
  page: number;
  limit: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}