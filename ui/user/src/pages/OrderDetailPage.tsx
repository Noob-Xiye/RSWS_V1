import React, { useState, useEffect } from 'react';
import { Card, Button, Descriptions, Tag, Spin, Result, Timeline, Divider } from 'antd';
import { CheckCircleOutlined, ClockCircleOutlined, CloseCircleOutlined, DownloadOutlined } from '@ant-design/icons';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { useParams, useNavigate } from 'react-router-dom';
import { OrderService, ResourceService } from '../utils/api';
import { Order } from '../types';

const OrderDetailContainer = styled.div`
  min-height: 100vh;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
  padding: 20px;
  display: flex;
  align-items: flex-start;
  justify-content: center;
`;

const OrderCard = styled(Card)`
  width: 100%;
  max-width: 800px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;

  .ant-card-body {
    padding: 40px;
  }

  .ant-descriptions-item-label {
    color: rgba(255, 255, 255, 0.7);
  }

  .ant-descriptions-item-content {
    color: white;
  }

  .order-status {
    margin-bottom: 24px;
    text-align: center;
    padding: 20px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    
    .status-icon {
      font-size: 48px;
      margin-bottom: 16px;
    }
    
    .status-title {
      font-size: 24px;
      font-weight: bold;
      margin-bottom: 8px;
    }
    
    .status-desc {
      color: rgba(255, 255, 255, 0.7);
    }
  }

  .order-timeline {
    margin-top: 24px;
    padding: 20px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    
    .ant-timeline-item-content {
      color: rgba(255, 255, 255, 0.8);
    }
  }
`;

const Title = styled.h1`
  text-align: center;
  color: white;
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 30px;
  background: linear-gradient(135deg, #00d4ff, #ffffff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
`;

const OrderDetailPage: React.FC = () => {
  const { orderId } = useParams<{ orderId: string }>();
  const navigate = useNavigate();
  const [order, setOrder] = useState<Order | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (orderId) {
      loadOrderData();
    }
  }, [orderId]);

  const loadOrderData = async () => {
    try {
      const response = await OrderService.getOrderById(orderId!);
      if (response.success) {
        setOrder(response.data);
      }
    } catch (error) {
      console.error('Failed to load order data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleDownload = async () => {
    if (!order) return;
    
    try {
      const response = await ResourceService.downloadResource(order.resourceId);
      if (response.success) {
        const link = document.createElement('a');
        link.href = response.data.downloadUrl;
        link.download = response.data.filename;
        link.click();
      }
    } catch (error) {
      console.error('Download failed:', error);
    }
  };

  const handleCancelOrder = async () => {
    if (!order || order.status !== 'pending') return;
    
    try {
      const response = await OrderService.cancelOrder(order.id);
      if (response.success) {
        // 重新加载订单数据
        loadOrderData();
      }
    } catch (error) {
      console.error('Cancel order failed:', error);
    }
  };

  const handlePayOrder = () => {
    if (!order || order.status !== 'pending') return;
    navigate(`/payment/${order.resourceId}`);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircleOutlined className="status-icon" style={{ color: '#52c41a' }} />;
      case 'pending':
        return <ClockCircleOutlined className="status-icon" style={{ color: '#faad14' }} />;
      case 'failed':
      case 'cancelled':
        return <CloseCircleOutlined className="status-icon" style={{ color: '#f5222d' }} />;
      default:
        return <ClockCircleOutlined className="status-icon" style={{ color: '#1890ff' }} />;
    }
  };

  const getStatusTitle = (status: string) => {
    switch (status) {
      case 'completed': return '订单已完成';
      case 'pending': return '等待支付';
      case 'failed': return '支付失败';
      case 'cancelled': return '订单已取消';
      case 'refunded': return '已退款';
      default: return '处理中';
    }
  };

  const getStatusDescription = (status: string) => {
    switch (status) {
      case 'completed': return '您已成功购买该资源，可以下载使用了';
      case 'pending': return '请尽快完成支付，订单将在30分钟后自动取消';
      case 'failed': return '支付过程中出现问题，请重新尝试';
      case 'cancelled': return '订单已取消，您可以重新下单';
      case 'refunded': return '订单金额已退回您的账户';
      default: return '订单正在处理中';
    }
  };

  if (loading) {
    return (
      <OrderDetailContainer>
        <Spin size="large" />
      </OrderDetailContainer>
    );
  }

  if (!order) {
    return (
      <OrderDetailContainer>
        <Result
          status="error"
          title={<span style={{ color: 'white' }}>订单不存在</span>}
          subTitle={<span style={{ color: 'rgba(255, 255, 255, 0.7)' }}>无法找到该订单信息</span>}
          extra={<Button type="primary" onClick={() => navigate('/user-center')}>返回个人中心</Button>}
        />
      </OrderDetailContainer>
    );
  }

  return (
    <OrderDetailContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        style={{ width: '100%', maxWidth: '800px' }}
      >
        <OrderCard>
          <Title>订单详情</Title>
          
          <div className="order-status">
            {getStatusIcon(order.status)}
            <div className="status-title">{getStatusTitle(order.status)}</div>
            <div className="status-desc">{getStatusDescription(order.status)}</div>
          </div>

          <Descriptions bordered column={1} size="middle">
            <Descriptions.Item label="订单编号">
              <span style={{ fontFamily: 'monospace' }}>{order.id}</span>
            </Descriptions.Item>
            <Descriptions.Item label="资源名称">{order.resource.title}</Descriptions.Item>
            <Descriptions.Item label="订单金额">
              <span style={{ color: '#00d4ff', fontWeight: 'bold' }}>¥{order.amount}</span>
            </Descriptions.Item>
            <Descriptions.Item label="支付方式">{order.paymentMethod || '未支付'}</Descriptions.Item>
            <Descriptions.Item label="订单状态">
              <Tag color={
                order.status === 'completed' ? 'green' :
                order.status === 'pending' ? 'orange' :
                'red'
              }>
                {getStatusTitle(order.status)}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="创建时间">
              {new Date(order.createdAt).toLocaleString()}
            </Descriptions.Item>
            {order.completedAt && (
              <Descriptions.Item label="完成时间">
                {new Date(order.completedAt).toLocaleString()}
              </Descriptions.Item>
            )}
          </Descriptions>

          <Divider style={{ borderColor: 'rgba(255, 255, 255, 0.1)' }} />

          <div style={{ display: 'flex', justifyContent: 'center', gap: '16px', marginTop: '24px' }}>
            {order.status === 'pending' && (
              <>
                <Button type="primary" size="large" onClick={handlePayOrder}>
                  立即支付
                </Button>
                <Button danger size="large" onClick={handleCancelOrder}>
                  取消订单
                </Button>
              </>
            )}
            
            {order.status === 'completed' && (
              <Button 
                type="primary" 
                size="large" 
                icon={<DownloadOutlined />}
                onClick={handleDownload}
              >
                下载资源
              </Button>
            )}
            
            <Button size="large" onClick={() => navigate('/user-center')}>
              返回个人中心
            </Button>
          </div>

          <div className="order-timeline">
            <h3 style={{ color: 'white', marginBottom: '16px' }}>订单流程</h3>
            <Timeline>
              <Timeline.Item color="blue">
                订单创建 ({new Date(order.createdAt).toLocaleString()})
              </Timeline.Item>
              
              {order.status !== 'pending' && (
                <Timeline.Item 
                  color={order.status === 'completed' ? 'green' : 'red'}
                >
                  {order.status === 'completed' ? '支付成功' : '订单取消/失败'}
                  {order.completedAt && ` (${new Date(order.completedAt).toLocaleString()})`}
                </Timeline.Item>
              )}
              
              {order.status === 'completed' && (
                <Timeline.Item color="green">
                  订单完成，资源可下载
                </Timeline.Item>
              )}
            </Timeline>
          </div>
        </OrderCard>
      </motion.div>
    </OrderDetailContainer>
  );
};

export default OrderDetailPage;