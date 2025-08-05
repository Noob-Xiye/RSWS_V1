import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { 
  Row, 
  Col, 
  Card, 
  Button, 
  Tag, 
  Rate, 
  Divider,
  Typography,
  Space,
  Breadcrumb,
  Image,
  Tabs,
  List,
  Avatar,
  Spin,
  message,
  Modal
} from 'antd';
import { 
  DownloadOutlined, 
  ShoppingCartOutlined,
  ShareAltOutlined,
  HeartOutlined,
  EyeOutlined,
  UserOutlined,
  CalendarOutlined,
  FileTextOutlined,
  SafetyOutlined,
  ToolOutlined,
  HomeOutlined
} from '@ant-design/icons';
import styled from 'styled-components';
import { motion } from 'framer-motion';
import { Resource } from '../types';
import { ResourceService } from '../utils/api';

const { Title, Text, Paragraph } = Typography;
const { TabPane } = Tabs;

const PageContainer = styled.div`
  padding: 70px 2rem 2rem;
  min-height: 100vh;
  max-width: 1400px;
  margin: 0 auto;
`;

const ResourceHeader = styled(motion.div)`
  background: rgba(26, 26, 46, 0.6);
  backdrop-filter: blur(10px);
  border: 1px solid ${props => props.theme.colors.border};
  border-radius: 16px;
  padding: 2rem;
  margin-bottom: 2rem;
`;

const ResourceImage = styled.div<{ $bgImage?: string }>`
  width: 100%;
  height: 300px;
  background: ${props => props.$bgImage ? `url(${props.$bgImage})` : 'linear-gradient(45deg, #1a1a2e, #16213e)'};
  background-size: cover;
  background-position: center;
  border-radius: 12px;
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

const PriceSection = styled.div`
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid ${props => props.theme.colors.primary};
  border-radius: 12px;
  padding: 1.5rem;
  text-align: center;
`;

const PriceText = styled.div`
  font-size: 2rem;
  font-weight: bold;
  color: ${props => props.theme.colors.primary};
  margin-bottom: 1rem;
`;

const ActionButton = styled(Button)`
  width: 100%;
  height: 50px;
  font-size: 1.1rem;
  margin-bottom: 1rem;
  
  &.primary {
    background: linear-gradient(45deg, ${props => props.theme.colors.primary}, ${props => props.theme.colors.secondary});
    border: none;
    
    &:hover {
      box-shadow: ${props => props.theme.shadows.glow};
    }
  }
`;

const ContentCard = styled(Card)`
  background: rgba(26, 26, 46, 0.6) !important;
  border: 1px solid ${props => props.theme.colors.border} !important;
  border-radius: 12px !important;
  backdrop-filter: blur(10px);
  margin-bottom: 2rem;
  
  .ant-card-head {
    border-bottom: 1px solid ${props => props.theme.colors.border} !important;
    
    .ant-card-head-title {
      color: ${props => props.theme.colors.text.primary} !important;
    }
  }
`;

const SpecTable = styled.div`
  .spec-row {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    
    &:last-child {
      border-bottom: none;
    }
    
    .spec-label {
      color: ${props => props.theme.colors.text.secondary};
      font-weight: 500;
    }
    
    .spec-value {
      color: ${props => props.theme.colors.text.primary};
    }
  }
`;

const ResourceDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [resource, setResource] = useState<Resource | null>(null);
  const [loading, setLoading] = useState(true);
  const [purchasing, setPurchasing] = useState(false);
  const [downloading, setDownloading] = useState(false);

  useEffect(() => {
    if (id) {
      fetchResource(id);
    }
  }, [id]);

  const fetchResource = async (resourceId: string) => {
    try {
      setLoading(true);
      const response = await ResourceService.getResource(resourceId);
      if (response.success) {
        setResource(response.data);
      }
    } catch (error) {
      console.error('Failed to fetch resource:', error);
      message.error('获取资源详情失败');
    } finally {
      setLoading(false);
    }
  };

  const handlePurchase = async () => {
    if (!resource) return;
    
    try {
      setPurchasing(true);
      const response = await ResourceService.purchaseResource(resource.id, 'alipay');
      if (response.success) {
        message.success('购买成功！');
        // 跳转到支付页面或显示支付二维码
      }
    } catch (error) {
      console.error('Purchase failed:', error);
      message.error('购买失败，请重试');
    } finally {
      setPurchasing(false);
    }
  };

  const handleDownload = async () => {
    if (!resource) return;
    
    try {
      setDownloading(true);
      const blob = await ResourceService.downloadResource(resource.id);
      
      // 创建下载链接
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = resource.fileName;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);
      
      message.success('下载开始！');
    } catch (error) {
      console.error('Download failed:', error);
      message.error('下载失败，请重试');
    } finally {
      setDownloading(false);
    }
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

  if (loading) {
    return (
      <PageContainer>
        <div style={{ textAlign: 'center', padding: '5rem' }}>
          <Spin size="large" />
        </div>
      </PageContainer>
    );
  }

  if (!resource) {
    return (
      <PageContainer>
        <div style={{ textAlign: 'center', padding: '5rem' }}>
          <Title level={3} style={{ color: '#ffffff' }}>资源不存在</Title>
          <Link to="/resources">
            <Button type="primary">返回资源列表</Button>
          </Link>
        </div>
      </PageContainer>
    );
  }

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
              href: '/resources',
              title: '资源中心',
            },
            {
              title: resource.title,
            },
          ]}
        />
        
        <ResourceHeader
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.1 }}
        >
          <Row gutter={[32, 32]}>
            <Col xs={24} md={16}>
              <ResourceImage $bgImage={resource.thumbnailUrl} />
            </Col>
            
            <Col xs={24} md={8}>
              <Title level={2} style={{ color: '#ffffff', marginBottom: '1rem' }}>
                {resource.title}
              </Title>
              
              <Space wrap style={{ marginBottom: '1rem' }}>
                {resource.tags.map(tag => (
                  <Tag key={tag} color="blue">{tag}</Tag>
                ))}
              </Space>
              
              <div style={{ marginBottom: '1rem' }}>
                <Rate disabled defaultValue={resource.rating} />
                <Text style={{ color: '#b0b0b0', marginLeft: '0.5rem' }}>
                  ({resource.downloadCount} 下载)
                </Text>
              </div>
              
              <Paragraph style={{ color: '#b0b0b0', marginBottom: '2rem' }}>
                {resource.description}
              </Paragraph>
              
              <PriceSection>
                <PriceText>{formatPrice(resource.price)}</PriceText>
                
                {resource.price > 0 ? (
                  <ActionButton 
                    type="primary" 
                    className="primary"
                    icon={<ShoppingCartOutlined />}
                    loading={purchasing}
                    onClick={handlePurchase}
                  >
                    立即购买
                  </ActionButton>
                ) : (
                  <ActionButton 
                    type="primary" 
                    className="primary"
                    icon={<DownloadOutlined />}
                    loading={downloading}
                    onClick={handleDownload}
                  >
                    免费下载
                  </ActionButton>
                )}
                
                <Space style={{ width: '100%', justifyContent: 'space-between' }}>
                  <Button icon={<HeartOutlined />} size="small">
                    收藏
                  </Button>
                  <Button icon={<ShareAltOutlined />} size="small">
                    分享
                  </Button>
                </Space>
              </PriceSection>
            </Col>
          </Row>
        </ResourceHeader>
        
        <Row gutter={[24, 24]}>
          <Col xs={24} lg={16}>
            <ContentCard>
              <Tabs defaultActiveKey="description">
                <TabPane tab="详细描述" key="description" icon={<FileTextOutlined />}>
                  <div style={{ color: '#ffffff', lineHeight: 1.8 }}>
                    {resource.detailDescription || resource.description}
                  </div>
                </TabPane>
                
                {resource.usageGuide && (
                  <TabPane tab="使用指南" key="usage" icon={<ToolOutlined />}>
                    <div style={{ color: '#ffffff', lineHeight: 1.8 }}>
                      {resource.usageGuide}
                    </div>
                  </TabPane>
                )}
                
                {resource.precautions && (
                  <TabPane tab="注意事项" key="precautions" icon={<SafetyOutlined />}>
                    <div style={{ color: '#ffffff', lineHeight: 1.8 }}>
                      {resource.precautions}
                    </div>
                  </TabPane>
                )}
              </Tabs>
            </ContentCard>
          </Col>
          
          <Col xs={24} lg={8}>
            <ContentCard title="资源信息">
              <SpecTable>
                <div className="spec-row">
                  <span className="spec-label">文件大小</span>
                  <span className="spec-value">{formatFileSize(resource.fileSize)}</span>
                </div>
                <div className="spec-row">
                  <span className="spec-label">文件类型</span>
                  <span className="spec-value">{resource.contentType}</span>
                </div>
                <div className="spec-row">
                  <span className="spec-label">发布时间</span>
                  <span className="spec-value">
                    {new Date(resource.createdAt).toLocaleDateString()}
                  </span>
                </div>
                <div className="spec-row">
                  <span className="spec-label">更新时间</span>
                  <span className="spec-value">
                    {new Date(resource.updatedAt).toLocaleDateString()}
                  </span>
                </div>
                <div className="spec-row">
                  <span className="spec-label">下载次数</span>
                  <span className="spec-value">{resource.downloadCount}</span>
                </div>
              </SpecTable>
            </ContentCard>
            
            <ContentCard title="作者信息">
              <Space>
                <Avatar size={64} icon={<UserOutlined />} src={resource.author.avatar} />
                <div>
                  <Title level={5} style={{ color: '#ffffff', margin: 0 }}>
                    {resource.author.username}
                  </Title>
                  <Text style={{ color: '#b0b0b0' }}>
                    {resource.author.email}
                  </Text>
                </div>
              </Space>
            </ContentCard>
          </Col>
        </Row>
      </motion.div>
    </PageContainer>
  );
};

export default ResourceDetailPage;