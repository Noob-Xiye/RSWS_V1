import React from 'react';
import { Card, Statistic, Row, Col, Button } from 'antd';
import { WalletOutlined, PlusOutlined, MinusOutlined } from '@ant-design/icons';
import styled from 'styled-components';

const WalletCard = styled(Card)`
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.1), rgba(0, 153, 204, 0.1));
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 16px;
  
  .ant-card-body {
    padding: 24px;
  }
  
  .ant-statistic-title {
    color: rgba(255, 255, 255, 0.7) !important;
    font-size: 14px;
  }
  
  .ant-statistic-content {
    color: #00d4ff !important;
    font-weight: 600;
  }
`;

const ActionButtons = styled.div`
  display: flex;
  gap: 12px;
  margin-top: 16px;
  justify-content: center;
`;

interface WalletDisplayProps {
  balance: number;
  currency: string;
  frozenAmount?: number;
  onDeposit?: () => void;
  onWithdraw?: () => void;
}

const WalletDisplay: React.FC<WalletDisplayProps> = ({
  balance,
  currency,
  frozenAmount = 0,
  onDeposit,
  onWithdraw
}) => {
  const formatAmount = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currency,
      minimumFractionDigits: 2,
    }).format(amount);
  };
  
  return (
    <WalletCard
      title={
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <WalletOutlined style={{ color: '#00d4ff' }} />
          <span style={{ color: 'white' }}>我的钱包</span>
        </div>
      }
    >
      <Row gutter={16}>
        <Col span={12}>
          <Statistic
            title="可用余额"
            value={balance - frozenAmount}
            formatter={(value) => formatAmount(Number(value))}
          />
        </Col>
        <Col span={12}>
          <Statistic
            title="冻结金额"
            value={frozenAmount}
            formatter={(value) => formatAmount(Number(value))}
          />
        </Col>
      </Row>
      
      <ActionButtons>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={onDeposit}
        >
          充值
        </Button>
        <Button
          type="default"
          icon={<MinusOutlined />}
          onClick={onWithdraw}
        >
          提现
        </Button>
      </ActionButtons>
    </WalletCard>
  );
};

export default WalletDisplay;