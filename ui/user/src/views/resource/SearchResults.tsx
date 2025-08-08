import React, { useState, useEffect } from 'react';
import { Card, Input, Select, Pagination, Empty, Spin, Tag, Button } from 'antd';
import { SearchOutlined, EyeOutlined, ShoppingCartOutlined, FilterOutlined } from '@ant-design/icons';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { ResourceService } from '../utils/api';
import { Resource, ResourceCategory } from '../types';

const SearchContainer = styled.div`
  min-height: 100vh;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
  padding: 20px;
`;

const SearchHeader = styled.div`
  max-width: 1200px;
  margin: 0 auto 30px;
  
  .search-bar {
    display: flex;
    gap: 16px;
    margin-bottom: 20px;
    
    .ant-input-search {
      flex: 1;
      
      .ant-input {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        color: white;
        
        &::placeholder {
          color: rgba(255, 255, 255, 0.5);
        }
      }
      
      .ant-btn {
        background: linear-gradient(135deg, #00d4ff, #0099cc);
        border: none;
      }
    }
    
    .ant-select {
      min-width: 150px;
      
      .ant-select-selector {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        color: white;
      }
    }
  }
  
  .search-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: rgba(255, 255, 255, 0.7);
    
    .result-count {
      font-size: 16px;
      
      .highlight {
        color: #00d4ff;
        font-weight: bold;
      }
    }
    
    .sort-options {
      display: flex;
      align-items: center;
      gap: 12px;
    }
  }
`;

const ResultsGrid = styled.div`
  max-width: 1200px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 24px;
  margin-bottom: 40px;
`;

const ResourceCard = styled(Card)`
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  overflow: hidden;
  transition: all 0.3s ease;
  
  &:hover {
    transform: translateY(-8px);
    box-shadow: 0 20px 40px rgba(0, 212, 255, 0.2);
    border-color: rgba(0, 212, 255, 0.3);
  }
  
  .ant-card-body {
    padding: 20px;
  }
  
  .resource-image {
    width: 100%;
    height: 180px;
    background: linear-gradient(135deg, #1a1a2e, #16213e);
    border-radius: 12px;
    margin-bottom: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.3);
    font-size: 48px;
  }
  
  .resource-title {
    color: white;
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 8px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  
  .resource-description {
    color: rgba(255, 255, 255, 0.7);
    font-size: 14px;
    margin-bottom: 12px;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  
  .resource-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    
    .price {
      color: #00d4ff;
      font-size: 20px;
      font-weight: bold;
    }
    
    .category {
      background: rgba(0, 212, 255, 0.2);
      color: #00d4ff;
      border: none;
      border-radius: 12px;
    }
  }
  
  .resource-tags {
    margin-bottom: 16px;
    
    .ant-tag {
      background: rgba(255, 255, 255, 0.1);
      border: 1px solid rgba(255, 255, 255, 0.2);
      color: rgba(255, 255, 255, 0.8);
      border-radius: 8px;
      margin-bottom: 4px;
    }
  }
  
  .resource-actions {
    display: flex;
    gap: 8px;
    
    .ant-btn {
      flex: 1;
      border-radius: 8px;
      
      &.view-btn {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        color: rgba(255, 255, 255, 0.8);
        
        &:hover {
          background: rgba(255, 255, 255, 0.2);
          border-color: #00d4ff;
          color: #00d4ff;
        }
      }
      
      &.buy-btn {
        background: linear-gradient(135deg, #00d4ff, #0099cc);
        border: none;
        color: white;
        
        &:hover {
          background: linear-gradient(135deg, #0099cc, #007399);
          transform: translateY(-2px);
        }
      }
    }
  }
`;

const PaginationContainer = styled.div`
  display: flex;
  justify-content: center;
  margin-top: 40px;
  
  .ant-pagination {
    .ant-pagination-item {
      background: rgba(255, 255, 255, 0.1);
      border: 1px solid rgba(255, 255, 255, 0.2);
      
      a {
        color: rgba(255, 255, 255, 0.8);
      }
      
      &:hover {
        border-color: #00d4ff;
        
        a {
          color: #00d4ff;
        }
      }
      
      &.ant-pagination-item-active {
        background: #00d4ff;
        border-color: #00d4ff;
        
        a {
          color: white;
        }
      }
    }
  }
`;

const SearchResultsPage: React.FC = () => {
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const [resources, setResources] = useState<Resource[]>([]);
  const [categories, setCategories] = useState<ResourceCategory[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize] = useState(12);
  
  const query = searchParams.get('q') || '';
  const category = searchParams.get('category') || '';
  const sortBy = searchParams.get('sortBy') || 'createdAt';
  const sortOrder = searchParams.get('sortOrder') || 'desc';

  useEffect(() => {
    loadCategories();
  }, []);

  useEffect(() => {
    searchResources();
  }, [query, category, sortBy, sortOrder, currentPage]);

  const loadCategories = async () => {
    try {
      const response = await ResourceService.getCategories();
      if (response.success) {
        setCategories(response.data);
      }
    } catch (error) {
      console.error('加载分类失败:', error);
    }
  };

  const searchResources = async () => {
    setLoading(true);
    try {
      const response = await ResourceService.searchResources(query, {
        page: currentPage,
        limit: pageSize,
        category: category || undefined
      });
      
      if (response.success) {
        setResources(response.data.items);
        setTotal(response.data.total);
      }
    } catch (error) {
      console.error('搜索失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = (value: string) => {
    const newParams = new URLSearchParams(searchParams);
    if (value) {
      newParams.set('q', value);
    } else {
      newParams.delete('q');
    }
    setSearchParams(newParams);
    setCurrentPage(1);
  };

  const handleCategoryChange = (value: string) => {
    const newParams = new URLSearchParams(searchParams);
    if (value) {
      newParams.set('category', value);
    } else {
      newParams.delete('category');
    }
    setSearchParams(newParams);
    setCurrentPage(1);
  };

  const handleSortChange = (value: string) => {
    const newParams = new URLSearchParams(searchParams);
    const [field, order] = value.split('-');
    newParams.set('sortBy', field);
    newParams.set('sortOrder', order);
    setSearchParams(newParams);
  };

  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };

  const handleViewResource = (resourceId: string) => {
    navigate(`/resources/${resourceId}`);
  };

  const handleBuyResource = (resourceId: string) => {
    navigate(`/payment/${resourceId}`);
  };

  return (
    <SearchContainer>
      <SearchHeader>
        <div className="search-bar">
          <Input.Search
            placeholder="搜索资源..."
            defaultValue={query}
            onSearch={handleSearch}
            size="large"
            enterButton={<SearchOutlined />}
          />
          
          <Select
            placeholder="选择分类"
            value={category || undefined}
            onChange={handleCategoryChange}
            size="large"
            allowClear
          >
            {categories.map(cat => (
              <Select.Option key={cat.id} value={cat.id}>
                {cat.name}
              </Select.Option>
            ))}
          </Select>
        </div>
        
        <div className="search-info">
          <div className="result-count">
            找到 <span className="highlight">{total}</span> 个相关资源
            {query && (
              <span> 关于 "<span className="highlight">{query}</span>"</span>
            )}
          </div>
          
          <div className="sort-options">
            <FilterOutlined style={{ color: 'rgba(255, 255, 255, 0.5)' }} />
            <Select
              value={`${sortBy}-${sortOrder}`}
              onChange={handleSortChange}
              size="small"
              style={{ width: 120 }}
            >
              <Select.Option value="createdAt-desc">最新发布</Select.Option>
              <Select.Option value="price-asc">价格从低到高</Select.Option>
              <Select.Option value="price-desc">价格从高到低</Select.Option>
              <Select.Option value="downloads-desc">下载量最多</Select.Option>
            </Select>
          </div>
        </div>
      </SearchHeader>

      {loading ? (
        <div style={{ display: 'flex', justifyContent: 'center', padding: '100px 0' }}>
          <Spin size="large" />
        </div>
      ) : resources.length === 0 ? (
        <div style={{ display: 'flex', justifyContent: 'center', padding: '100px 0' }}>
          <Empty
            description={
              <span style={{ color: 'rgba(255, 255, 255, 0.5)' }}>
                没有找到相关资源
              </span>
            }
          />
        </div>
      ) : (
        <>
          <ResultsGrid>
            {resources.map((resource, index) => (
              <motion.div
                key={resource.id}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
              >
                <ResourceCard>
                  <div className="resource-image">
                    {resource.displayImages && resource.displayImages.length > 0 ? (
                      <img
                        src={resource.displayImages[0]}
                        alt={resource.title}
                        style={{ width: '100%', height: '100%', objectFit: 'cover', borderRadius: '12px' }}
                      />
                    ) : (
                      <EyeOutlined />
                    )}
                  </div>
                  
                  <div className="resource-title">{resource.title}</div>
                  <div className="resource-description">{resource.description}</div>
                  
                  <div className="resource-meta">
                    <div className="price">¥{resource.price}</div>
                    <Tag className="category">{resource.category}</Tag>
                  </div>
                  
                  {resource.tags && resource.tags.length > 0 && (
                    <div className="resource-tags">
                      {resource.tags.slice(0, 3).map(tag => (
                        <Tag key={tag}>{tag}</Tag>
                      ))}
                      {resource.tags.length > 3 && (
                        <Tag>+{resource.tags.length - 3}</Tag>
                      )}
                    </div>
                  )}
                  
                  <div className="resource-actions">
                    <Button
                      className="view-btn"
                      icon={<EyeOutlined />}
                      onClick={() => handleViewResource(resource.id)}
                    >
                      查看
                    </Button>
                    <Button
                      className="buy-btn"
                      icon={<ShoppingCartOutlined />}
                      onClick={() => handleBuyResource(resource.id)}
                    >
                      购买
                    </Button>
                  </div>
                </ResourceCard>
              </motion.div>
            ))}
          </ResultsGrid>
          
          <PaginationContainer>
            <Pagination
              current={currentPage}
              total={total}
              pageSize={pageSize}
              onChange={handlePageChange}
              showSizeChanger={false}
              showQuickJumper
              showTotal={(total, range) => 
                `第 ${range[0]}-${range[1]} 条，共 ${total} 条`
              }
            />
          </PaginationContainer>
        </>
      )}
    </SearchContainer>
  );
};

export default SearchResultsPage;