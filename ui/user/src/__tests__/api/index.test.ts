import { authAPI, userAPI, resourceAPI } from '../../api';
import { apiClient } from '../../api';

// Mock axios
jest.mock('axios');

describe('API Tests', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });
  
  describe('authAPI', () => {
    it('should call login endpoint with correct parameters', async () => {
      const mockResponse = { token: 'test-token', user: { id: '1', email: 'test@example.com' } };
      (apiClient.post as jest.Mock).mockResolvedValue(mockResponse);
      
      const result = await authAPI.login('test@example.com', 'password');
      
      expect(apiClient.post).toHaveBeenCalledWith('/api/auth/login', {
        email: 'test@example.com',
        password: 'password'
      });
      expect(result).toEqual(mockResponse);
    });
    
    it('should call register endpoint with user data', async () => {
      const userData = {
        email: 'test@example.com',
        username: 'testuser',
        password: 'password'
      };
      const mockResponse = { success: true };
      (apiClient.post as jest.Mock).mockResolvedValue(mockResponse);
      
      const result = await authAPI.register(userData);
      
      expect(apiClient.post).toHaveBeenCalledWith('/api/auth/register', userData);
      expect(result).toEqual(mockResponse);
    });
  });
  
  describe('resourceAPI', () => {
    it('should fetch resources with parameters', async () => {
      const params = { page: 1, limit: 10, category: 'software' };
      const mockResponse = { items: [], total: 0 };
      (apiClient.get as jest.Mock).mockResolvedValue(mockResponse);
      
      const result = await resourceAPI.getResources(params);
      
      expect(apiClient.get).toHaveBeenCalledWith('/api/resources', { params });
      expect(result).toEqual(mockResponse);
    });
    
    it('should search resources with query', async () => {
      const query = 'test';
      const params = { category: 'software' };
      const mockResponse = { items: [], total: 0 };
      (apiClient.get as jest.Mock).mockResolvedValue(mockResponse);
      
      const result = await resourceAPI.searchResources(query, params);
      
      expect(apiClient.get).toHaveBeenCalledWith('/api/resources/search', {
        params: { q: query, ...params }
      });
      expect(result).toEqual(mockResponse);
    });
  });
});