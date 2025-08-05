export interface User {
  id: string;
  username: string;
  email: string;
  avatar?: string;
  role: 'user' | 'admin';
  createdAt: string;
}

export interface Resource {
  id: string;
  title: string;
  description: string;
  detailDescription?: string;
  price: number;
  category: ResourceCategory;
  tags: string[];
  fileName: string;
  fileSize: number;
  contentType: string;
  downloadCount: number;
  rating: number;
  author: User;
  createdAt: string;
  updatedAt: string;
  specifications?: Record<string, any>;
  usageGuide?: string;
  precautions?: string;
  displayImages?: string[];
  previewUrl?: string;
  thumbnailUrl?: string;
}

export interface ResourceCategory {
  id: string;
  name: string;
  description: string;
  icon: string;
  parentId?: string;
}

// 添加Transaction接口
export interface Transaction {
  id: string;
  orderId: string;
  userId: string;
  amount: number;
  currency: string;
  paymentMethod: string;
  providerTransactionId?: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled';
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
}

// 更新Order接口
export interface Order {
  id: string;
  userId: string;
  resourceId: string;
  resource: Resource;
  amount: number;
  status: 'pending' | 'paid' | 'completed' | 'cancelled' | 'refunded' | 'failed';
  paymentMethod: string;
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
  expiredAt?: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message?: string;
  code?: number;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}