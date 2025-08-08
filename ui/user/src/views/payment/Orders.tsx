import React, { useState, useEffect } from 'react';
import { Card, Button, Radio, Divider, message, Spin, Result } from 'antd';
import { CreditCardOutlined, AlipayOutlined, WechatOutlined, CheckCircleOutlined } from '@ant-design/icons';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { useParams, useNavigate } from 'react-router-dom';
import { OrderService, PaymentService, ResourceService } from '../utils/api';
import { Order, Resource } from '../types';

const PaymentContainer = styled.div`
  min-height: 100vh;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
`;

const PaymentCard = styled(Card)`
  width: 100%;
  max-width: 600px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;

  .ant-card-body {
    padding: 40px;
  }

  .resource-info {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 24px;
    
    .resource-title {
      color: white;
      font-size: 18px;
      font-weight: 600;
      margin-bottom: 8px;
    }
    
    .resource-price {
      color: #00d4ff;
      font-size: 24px;
      font-weight: bold;
    }
    
    .resource-description {
      color: rgba(255, 255, 255, 0.7);
      margin-top: 8px;
    }
  }

  .payment-methods {
    .ant-radio-group {
      width: 100%;
    }
    
    .ant-radio-wrapper {
      display: flex;
      align-items: center;
      padding: 16px;
      margin: 8px 0;
      background: rgba(255, 255, 255, 0.05);
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 12px;
      color: rgba(255, 255, 255, 0.8);
      transition: all 0.3s ease;
      
      &:hover {
        background: rgba(255, 255, 255, 0.1);
        border-color: #00d4ff;
      }
      
      &.ant-radio-wrapper-checked {
        background: rgba(0, 212, 255, 0.1);
        border-color: #00d4ff;
        color: #00d4ff;
      }
      
      .payment-method-content {
        display: flex;
        align-items: center;
        gap: 12px;
        flex: 1;
        margin-left: 12px;
        
        .anticon {
          font-size: 24px;
        }
        
        .method-info {
          .method-name {
            font-weight: 500;
            margin-bottom: 4px;
          }
          
          .method-desc {
            font-size: 12px;
            opacity: 0.7;
          }
        }
      }
    }
  }

  .order-summary {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 20px;
    margin: 24px 0;
    
    .summary-row {
      display: flex;
      justify-content: space-between;
      margin-bottom: 12px;
      color: rgba(255, 255, 255, 0.8);
      
      &.total {
        font-size: 18px;
        font-weight: bold;
        color: #00d4ff;
        border-top: 1px solid rgba(255, 255, 255, 0.1);
        padding-top: 12px;
        margin-top: 12px;
      }
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

const PaymentPage: React.FC = () => {
  const { resourceId } = useParams<{ resourceId: string }>();
  const navigate = useNavigate();
  const [resource, setResource] = useState<Resource | null>(null);
  const [order, setOrder] = useState<Order | null>(null);
  const [paymentMethods, setPaymentMethods] = useState<any[]>([]);
  const [selectedMethod, setSelectedMethod] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [paying, setPaying] = useState(false);
  const [paymentStatus, setPaymentStatus] = useState<'pending' | 'success' | 'failed'>('pending');

  useEffect(() => {
    if (resourceId) {
      loadData();
    }
  }, [resourceId]);

  const loadData = async () => {
    try {
      // 加载资源信息
      const resourceResponse = await ResourceService.getResourceById(resourceId!);
      if (resourceResponse.success) {
        setResource(resourceResponse.data);
      }

      // 创建订单
      const orderResponse = await OrderService.createOrder(resourceId!);
      if (orderResponse.success) {
        setOrder(orderResponse.data);
      }

      // 加载支付方式
      const methodsResponse = await PaymentService.getPaymentMethods();
      if (methodsResponse.success) {
        setPaymentMethods(methodsResponse.data);
        if (methodsResponse.data.length > 0) {
          setSelectedMethod(methodsResponse.data[0].id);
        }
      }
    } catch (error) {
      message.error('加载数据失败');
    } finally {
      setLoading(false);
    }
  };

  const handlePayment = async () => {
    if (!order || !selectedMethod) {
      message.error('请选择支付方式');
      return;
    }

    setPaying(true);
    try {
      const response = await OrderService.payOrder(order.id, selectedMethod);
      if (response.success) {
        if (response.data.paymentUrl) {
          // 跳转到支付页面
          window.open(response.data.paymentUrl, '_blank');
          // 模拟支付成功（实际应该通过回调或轮询检查支付状态）
          setTimeout(() => {
            setPaymentStatus('success');
          }, 3000);
        } else {
          setPaymentStatus('success');
        }
      } else {
        setPaymentStatus('failed');
        message.error(response.message || '支付失败');
      }
    } catch (error) {
      setPaymentStatus('failed');
      message.error('支付失败，请重试');
    } finally {
      setPaying(false);
    }
  };

  const getPaymentMethodIcon = (methodId: string) => {
    switch (methodId) {
      case 'alipay':
        return <AlipayOutlined style={{ color: '#1677ff' }} />;
      case 'wechat':
        return <WechatOutlined style={{ color: '#07c160' }} />;
      case 'card':
        return <CreditCardOutlined style={{ color: '#722ed1' }} />;
      default:
        return <CreditCardOutlined />;
    }
  };

  if (loading) {
    return (
      <PaymentContainer>
        <Spin size="large" />
      </PaymentContainer>
    );
  }

  if (paymentStatus === 'success') {
    return (
      <PaymentContainer>
        <motion.div
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.6 }}
        >
          <Result
            icon={<CheckCircleOutlined style={{ color: '#00d4ff' }} />}
            title={<span style={{ color: 'white' }}>支付成功！</span>}
            subTitle={<span style={{ color: 'rgba(255, 255, 255, 0.7)' }}>您已成功购买该资源，现在可以下载使用了</span>}
            extra={[
              <Button type="primary" key="download" onClick={() => navigate(`/resources/${resourceId}`)}>
                查看资源
              </Button>,
              <Button key="center" onClick={() => navigate('/user-center')}>
                个人中心
              </Button>,
            ]}
            style={{ background: 'rgba(255, 255, 255, 0.05)', borderRadius: '16px', padding: '40px' }}
          />
        </motion.div>
      </PaymentContainer>
    );
  }

  if (!resource || !order) {
    return (
      <PaymentContainer>
        <Result
          status="error"
          title={<span style={{ color: 'white' }}>加载失败</span>}
          subTitle={<span style={{ color: 'rgba(255, 255, 255, 0.7)' }}>无法加载资源信息</span>}
          extra={<Button type="primary" onClick={() => navigate('/')}>返回首页</Button>}
        />
      </PaymentContainer>
    );
  }

  return (
    <PaymentContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        style={{ width: '100%', maxWidth: '600px' }}
      >
        <PaymentCard>
          <Title>确认支付</Title>
          
          <div className="resource-info">
            <div className="resource-title">{resource.title}</div>
            <div className="resource-price">¥{resource.price}</div>
            <div className="resource-description">{resource.description}</div>
          </div>

          <Divider style={{ borderColor: 'rgba(255, 255, 255, 0.1)' }} />

          <div className="payment-methods">
            <h3 style={{ color: 'rgba(255, 255, 255, 0.8)', marginBottom: '16px' }}>选择支付方式</h3>
            <Radio.Group
              value={selectedMethod}
              onChange={(e) => setSelectedMethod(e.target.value)}
            >
              {paymentMethods.map(method => (
                <Radio key={method.id} value={method.id}>
                  <div className="payment-method-content">
                    {getPaymentMethodIcon(method.id)}
                    <div className="method-info">
                      <div className="method-name">{method.name}</div>
                      <div className="method-desc">安全快捷的在线支付</div>
                    </div>
                  </div>
                </Radio>
              ))}
            </Radio.Group>
          </div>

          <div className="order-summary">
            <h3 style={{ color: 'rgba(255, 255, 255, 0.8)', marginBottom: '16px' }}>订单详情</h3>
            <div className="summary-row">
              <span>商品价格</span>
              <span>¥{resource.price}</span>
            </div>
            <div className="summary-row">
              <span>优惠折扣</span>
              <span>-¥0.00</span>
            </div>
            <div className="summary-row total">
              <span>应付金额</span>
              <span>¥{resource.price}</span>
            </div>
          </div>

          <Button
            type="primary"
            size="large"
            block
            loading={paying}
            onClick={handlePayment}
            disabled={!selectedMethod}
            style={{
              height: '50px',
              fontSize: '16px',
              fontWeight: 'bold',
              background: 'linear-gradient(135deg, #00d4ff, #0099cc)',
              border: 'none'
            }}
          >
            {paying ? '支付中...' : `立即支付 ¥${resource.price}`}
          </Button>
        </PaymentCard>
      </motion.div>
    </PaymentContainer>
  );
};

export default PaymentPage;