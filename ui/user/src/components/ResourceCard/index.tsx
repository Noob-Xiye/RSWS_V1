import React from 'react';
import { Card, Tag, Avatar, Rate, Button } from 'antd';
import { DownloadOutlined, EyeOutlined, HeartOutlined } from '@ant-design/icons';
import styled from 'styled-components';
import { Resource } from '../../types';
import { formatPrice, formatDate } from '../../utils';

const { Meta } = Card;

const StyledCard = styled(Card)`
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.3s ease;
  
  &:hover {
    transform: translateY(-4px);
    border-color: #00d4ff;
    box-shadow: 0 8px 32px rgba(0, 212, 255, 0.2);
  }
  
  .ant-card-cover {
    height: 200px;
    overflow: hidden;
    
    img {
      width: 100%;
      height: 100%;
      object-fit: cover;
      transition: transform 0.3s ease;
    }
    
    &:hover img {
      transform: scale(1.05);
    }
  }
  
  .ant-card-body {
    padding: 16px;
  }
  
  .ant-card-meta-title {
    color: white !important;
    font-size: 16px;
    font-weight: 600;
  }
  
  .ant-card-meta-description {
    color: rgba(255, 255, 255, 0.7) !important;
  }
`;

const CardHeader = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
`;

const PriceTag = styled.div`
  background: linear-gradient(135deg, #00d4ff, #0099cc);
  color: white;
  padding: 4px 12px;
  border-radius: 20px;
  font-weight: 600;
  font-size: 14px;
`;

const StatsRow = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
`;

const StatItem = styled.div`
  display: flex;
  align-items: center;
  gap: 4px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
  
  .anticon {
    color: #00d4ff;
  }
`;

const TagsContainer = styled.div`
  margin: 8px 0;
  
  .ant-tag {
    background: rgba(0, 212, 255, 0.1);
    border: 1px solid rgba(0, 212, 255, 0.3);
    color: #00d4ff;
    border-radius: 12px;
    margin-bottom: 4px;
  }
`;

interface ResourceCardProps {
  resource: Resource;
  onView?: (resource: Resource) => void;
  onDownload?: (resource: Resource) => void;
  onFavorite?: (resource: Resource) => void;
}

const ResourceCard: React.FC<ResourceCardProps> = ({
  resource,
  onView,
  onDownload,
  onFavorite
}) => {
  const handleView = () => {
    onView?.(resource);
  };
  
  const handleDownload = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDownload?.(resource);
  };
  
  const handleFavorite = (e: React.MouseEvent) => {
    e.stopPropagation();
    onFavorite?.(resource);
  };
  
  return (
    <StyledCard
      hoverable
      cover={
        <img
          alt={resource.title}
          src={resource.thumbnailUrl || '/placeholder-image.jpg'}
          onClick={handleView}
        />
      }
      actions={[
        <Button
          key="view"
          type="text"
          icon={<EyeOutlined />}
          onClick={handleView}
        >
          查看
        </Button>,
        <Button
          key="download"
          type="text"
          icon={<DownloadOutlined />}
          onClick={handleDownload}
        >
          下载
        </Button>,
        <Button
          key="favorite"
          type="text"
          icon={<HeartOutlined />}
          onClick={handleFavorite}
        >
          收藏
        </Button>
      ]}
    >
      <CardHeader>
        <PriceTag>{formatPrice(resource.price, resource.currency)}</PriceTag>
        <Rate disabled defaultValue={resource.rating} style={{ fontSize: 12 }} />
      </CardHeader>
      
      <Meta
        title={resource.title}
        description={resource.description}
        avatar={
          <Avatar src={resource.author.avatar} size="small">
            {resource.author.username[0]}
          </Avatar>
        }
      />
      
      <TagsContainer>
        <Tag>{resource.category}</Tag>
        {resource.tags.slice(0, 2).map(tag => (
          <Tag key={tag}>{tag}</Tag>
        ))}
        {resource.tags.length > 2 && (
          <Tag>+{resource.tags.length - 2}</Tag>
        )}
      </TagsContainer>
      
      <StatsRow>
        <StatItem>
          <DownloadOutlined />
          {resource.downloadCount}
        </StatItem>
        <StatItem>
          <EyeOutlined />
          {formatDate(resource.createdAt)}
        </StatItem>
      </StatsRow>
    </StyledCard>
  );
};

export default ResourceCard;