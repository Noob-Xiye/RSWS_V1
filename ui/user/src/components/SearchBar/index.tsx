import React, { useState } from 'react';
import { Input, Select, Button, Space } from 'antd';
import { SearchOutlined, FilterOutlined } from '@ant-design/icons';
import styled from 'styled-components';
import { debounce } from '../../utils';

const { Option } = Select;

const SearchContainer = styled.div`
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 24px;
  margin-bottom: 24px;
`;

const SearchRow = styled.div`
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 16px;
  
  @media (max-width: 768px) {
    flex-direction: column;
    align-items: stretch;
  }
`;

const FilterRow = styled.div`
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
  
  @media (max-width: 768px) {
    justify-content: center;
  }
`;

const StyledInput = styled(Input)`
  flex: 1;
  min-width: 300px;
  
  @media (max-width: 768px) {
    min-width: auto;
  }
`;

interface SearchBarProps {
  onSearch?: (query: string, filters: SearchFilters) => void;
  loading?: boolean;
}

interface SearchFilters {
  category?: string;
  priceRange?: [number, number];
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

const SearchBar: React.FC<SearchBarProps> = ({ onSearch, loading }) => {
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<SearchFilters>({});
  const [showFilters, setShowFilters] = useState(false);
  
  const debouncedSearch = debounce((searchQuery: string, searchFilters: SearchFilters) => {
    onSearch?.(searchQuery, searchFilters);
  }, 300);
  
  const handleSearch = () => {
    onSearch?.(query, filters);
  };
  
  const handleQueryChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newQuery = e.target.value;
    setQuery(newQuery);
    debouncedSearch(newQuery, filters);
  };
  
  const handleFilterChange = (key: keyof SearchFilters, value: any) => {
    const newFilters = { ...filters, [key]: value };
    setFilters(newFilters);
    debouncedSearch(query, newFilters);
  };
  
  return (
    <SearchContainer>
      <SearchRow>
        <StyledInput
          placeholder="搜索资源、作者或标签..."
          value={query}
          onChange={handleQueryChange}
          prefix={<SearchOutlined style={{ color: '#00d4ff' }} />}
          size="large"
        />
        <Button
          type="primary"
          icon={<SearchOutlined />}
          size="large"
          loading={loading}
          onClick={handleSearch}
        >
          搜索
        </Button>
        <Button
          icon={<FilterOutlined />}
          size="large"
          onClick={() => setShowFilters(!showFilters)}
        >
          筛选
        </Button>
      </SearchRow>
      
      {showFilters && (
        <FilterRow>
          <Select
            placeholder="选择分类"
            style={{ width: 120 }}
            allowClear
            value={filters.category}
            onChange={(value) => handleFilterChange('category', value)}
          >
            <Option value="software">软件</Option>
            <Option value="design">设计</Option>
            <Option value="document">文档</Option>
            <Option value="media">媒体</Option>
            <Option value="other">其他</Option>
          </Select>
          
          <Select
            placeholder="排序方式"
            style={{ width: 120 }}
            value={filters.sortBy}
            onChange={(value) => handleFilterChange('sortBy', value)}
          >
            <Option value="createdAt">最新</Option>
            <Option value="downloadCount">下载量</Option>
            <Option value="rating">评分</Option>
            <Option value="price">价格</Option>
          </Select>
          
          <Select
            placeholder="排序顺序"
            style={{ width: 100 }}
            value={filters.sortOrder}
            onChange={(value) => handleFilterChange('sortOrder', value)}
          >
            <Option value="desc">降序</Option>
            <Option value="asc">升序</Option>
          </Select>
        </FilterRow>
      )}
    </SearchContainer>
  );
};

export default SearchBar;