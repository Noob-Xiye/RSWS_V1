import { useState, useEffect } from 'react';
import { Resource, PaginatedResponse } from '../types';
import { ResourceService } from '../utils/api';

interface UseResourcesParams {
  page?: number;
  limit?: number;
  category?: string;
  search?: string;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  minPrice?: number;
  maxPrice?: number;
}

export const useResources = (params: UseResourcesParams = {}) => {
  const [resources, setResources] = useState<Resource[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [pagination, setPagination] = useState({
    current: 1,
    pageSize: 12,
    total: 0
  });

  const fetchResources = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await ResourceService.getResources({
        page: pagination.current,
        limit: pagination.pageSize,
        ...params
      });
      
      if (response.success) {
        setResources(response.data.items);
        setPagination(prev => ({
          ...prev,
          total: response.data.total
        }));
      } else {
        setError(response.message || '获取资源失败');
      }
    } catch (err) {
      setError('网络错误，请稍后重试');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchResources();
  }, [params.page, params.limit, params.category, params.search, params.sortBy, params.sortOrder, params.minPrice, params.maxPrice]);

  const refetch = () => {
    fetchResources();
  };

  const changePage = (page: number, pageSize?: number) => {
    setPagination(prev => ({
      ...prev,
      current: page,
      pageSize: pageSize || prev.pageSize
    }));
  };

  return {
    resources,
    loading,
    error,
    pagination,
    refetch,
    changePage
  };
};