import React, { useState, useEffect } from 'react';
import { 
  Row, 
  Col, 
  Card, 
  Input, 
  Select, 
  Button, 
  Tag, 
  Rate, 
  Pagination,
  Spin,
  Empty,
  Typography,
  Space,
  Breadcrumb
} from 'antd';
import { 
  SearchOutlined, 
  DownloadOutlined, 
  EyeOutlined,
  ShoppingCartOutlined,
  FilterOutlined,
  CodeOutlined,
  ShieldCheckOutlined,
  HomeOutlined
} from '@ant-design/icons';
import styled from 'styled-components';
import { motion } from 'framer-motion';
import { Link } from 'react-router-dom';
import { Resource, ResourceCategory } from '../types';
import { ResourceService } from '../utils/api';

const { Search } = Input;
const { Option } = Select;
const { Title, Text, Paragraph } = Typography;

const PageContainer = styled.div`
  padding: 70px 2rem 2rem;
  min-height: 100vh;
  max-width: 1400px;
  margin: 0 auto;
`;

const FilterSection = styled(motion.div)`
  background: rgba(26, 26, 46, 0.6);
  backdrop-filter: blur(10px);
  border: 1px solid ${props => props.theme.colors.border};
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 2rem;
`;

const ResourceCard = styled(motion(Card))`
  background: rgba(26, 26, 46, 0.6) !important;
  border: 1px solid ${props => props.theme.colors.border} !important;
  border-radius: 12px !important;
  backdrop-filter: blur(10px);
  height: 100%;
  
  .ant-card-body {
    padding: 1.5rem !important;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  
  &:hover {
    border-color: ${props => props.theme.colors.primary} !important;
    box-shadow: ${props => props.theme.shadows.glow} !important;
    transform: translateY(-5px);
  }
`;

const ResourceImage = styled.div<{ $bgImage?: string }>`
  width: 100%;
  height: 200px;
  background: ${props => props.$bgImage ? `url(${props.$bgImage})` : 'linear-gradient(45deg, #1a1a2e, #16213e)'};
  background-size: cover;
  background-position: center;
  border-radius: 8px;
  margin-bottom: 1rem;
  position: relative;
  overflow: hidden;
  
  &::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(45deg, rgba(0, 212, 255, 0.1), rgba(0, 153, 204, 0.1));
  }
`;

const ResourceMeta = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
`;

const ResourceTitle = styled(Title)`
  &.ant-typography {
    color: ${props => props.theme.colors.text.primary} !important;
    font-size: 1.2rem !important;
    margin-bottom: 0.5rem !important;
    line-height: 1.4 !important;
  }
`;

const ResourceDescription = styled(Paragraph)`
  &.ant-typography {
    color: ${props => props.theme.colors.text.secondary} !important;
    font-size: 0.9rem !important;
    margin-bottom: 1rem !important;
    flex: 1;
  }
`;

const ResourceFooter = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
  padding-top: 1rem;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
`;

const PriceTag = styled.div`
  font-size: 1.2rem;
  font-weight: bold;
  color: ${props => props.theme.colors.primary};
`;

const ActionButtons = styled.div`
  display: flex;
  gap: 0.5rem;
`;

const ResourcesPage: React.FC = () => {
  const [resources, setResources] = useState<Resource[]>([]);
  const [categories, setCategories] = useState<ResourceCategory[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchText, setSearchText] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [sortBy, setSortBy] = useState('createdAt');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize] = useState(12);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    fetchResources();
  }, [searchText, selectedCategory, sortBy, sortOrder, currentPage]);

  const fetchResources = async () => {
    try {
      setLoading(true);
      const response = await ResourceService.getResources({
        page: currentPage,
        pageSize,
        category: selectedCategory,
        search: searchText,
        sortBy,
        sortOrder
      });
      
      if (response.success) {
        setResources(response.data.items);
        setTotal(response.data.total);
      }
    } catch (error) {
      console.error('Failed to fetch resources:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = (value: string) => {
    setSearchText(value);
    setCurrentPage(1);
  };

  const handleCategoryChange = (value: string) => {
    setSelectedCategory(value);
    setCurrentPage(1);
  };

  const handleSortChange = (value: string) => {
    const [field, order] = value.split('-');
    setSortBy(field);
    setSortOrder(order as 'asc' | 'desc');
    setCurrentPage(1);
  };

  const formatPrice = (price: number) => {
    return price === 0 ? '免费' : `¥${price}`;
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <PageContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
      >
        <Breadcrumb
          style={{ marginBottom: '2rem' }}
          items={[
            {
              href: '/',
              title: <HomeOutlined />,
            },
            {
              title: '资源中心',
            },
          ]}
        />
        
        <Title level={2} style={{ color: '#ffffff', marginBottom: '2rem' }}>
          <CodeOutlined style={{ marginRight: '0.5rem', color: '#00d4ff' }} />
          编程与安全资源
        </Title>
        
        <FilterSection
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.1 }}
        >
          <Row gutter={[16, 16]} align="middle">
            <Col xs={24} sm={12} md={8}>
              <Search
                placeholder="搜索资源..."
                allowClear
                enterButton={<SearchOutlined />}
                size="large"
                onSearch={handleSearch}
                style={{ width: '100%' }}
              />
            </Col>
            
            <Col xs={24} sm={6} md={4}>
              <Select
                placeholder="选择分类"
                allowClear
                size="large"
                style={{ width: '100%' }}
                onChange={handleCategoryChange}
              >
                <Option value="programming">编程开发</Option>
                <Option value="security">网络安全</Option>
                <Option value="tools">开发工具</Option>
                <Option value="frameworks">框架库</Option>
              </Select>
            </Col>
            
            <Col xs={24} sm={6} md={4}>
              <Select
                placeholder="排序方式"
                size="large"
                style={{ width: '100%' }}
                defaultValue="createdAt-desc"
                onChange={handleSortChange}
              >
                <Option value="createdAt-desc">最新发布</Option>
                <Option value="downloadCount-desc">下载最多</Option>
                <Option value="rating-desc">评分最高</Option>
                <Option value="price-asc">价格最低</Option>
              </Select>
            </Col>
          </Row>
        </FilterSection>
        
        {loading ? (
          <div style={{ textAlign: 'center', padding: '3rem' }}>
            <Spin size="large" />
          </div>
        ) : resources.length === 0 ? (
          <Empty
            description="暂无资源"
            style={{ padding: '3rem' }}
          />
        ) : (
          <>
            <Row gutter={[24, 24]}>
              {resources.map((resource, index) => (
                <Col xs={24} sm={12} md={8} lg={6} key={resource.id}>
                  <ResourceCard
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.6, delay: index * 0.1 }}
                    whileHover={{ y: -5 }}
                  >
                    <ResourceImage $bgImage={resource.thumbnailUrl} />
                    
                    <ResourceMeta>
                      <ResourceTitle level={5} ellipsis={{ rows: 2 }}>
                        {resource.title}
                      </ResourceTitle>
                      
                      <ResourceDescription ellipsis={{ rows: 3 }}>
                        {resource.description}
                      </ResourceDescription>
                      
                      <Space wrap style={{ marginBottom: '1rem' }}>
                        {resource.tags.slice(0, 3).map(tag => (
                          <Tag key={tag} color="blue">{tag}</Tag>
                        ))}
                      </Space>
                      
                      <div style={{ marginBottom: '1rem' }}>
                        <Rate disabled defaultValue={resource.rating} size="small" />
                        <Text style={{ color: '#b0b0b0', marginLeft: '0.5rem', fontSize: '0.8rem' }}>
                          {resource.downloadCount} 下载 · {formatFileSize(resource.fileSize)}
                        </Text>
                      </div>
                      
                      <ResourceFooter>
                        <PriceTag>{formatPrice(resource.price)}</PriceTag>
                        
                        <ActionButtons>
                          <Link to={`/resource/${resource.id}`}>
                            <Button 
                              type="primary" 
                              size="small"
                              icon={<EyeOutlined />}
                            >
                              查看
                            </Button>
                          </Link>
                          
                          {resource.price > 0 ? (
                            <Button 
                              size="small"
                              icon={<ShoppingCartOutlined />}
                              style={{
                                borderColor: '#00d4ff',
                                color: '#00d4ff'
                              }}
                            >
                              购买
                            </Button>
                          ) : (
                            <Button 
                              size="small"
                              icon={<DownloadOutlined />}
                              style={{
                                borderColor: '#52c41a',
                                color: '#52c41a'
                              }}
                            >
                              下载
                            </Button>
                          )}
                        </ActionButtons>
                      </ResourceFooter>
                    </ResourceMeta>
                  </ResourceCard>
                </Col>
              ))}
            </Row>
            
            <div style={{ textAlign: 'center', marginTop: '3rem' }}>
              <Pagination
                current={currentPage}
                pageSize={pageSize}
                total={total}
                onChange={setCurrentPage}
                showSizeChanger={false}
                showQuickJumper
                showTotal={(total, range) => 
                  `第 ${range[0]}-${range[1]} 条，共 ${total} 条`
                }
              />
            </div>
          </>
        )}
      </motion.div>
    </PageContainer>
  );
};

export default ResourcesPage;