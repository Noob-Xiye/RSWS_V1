import React, { useState, useEffect } from 'react';
import { Card, Table, Tag, Button, DatePicker, Select, Form, Spin, Empty } from 'antd';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { useNavigate } from 'react-router-dom';
import { UserService } from '../utils/api';

const { RangePicker } = DatePicker;

const TransactionsContainer = styled.div`
  min-height: 100vh;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
  padding: 20px;
`;

const ContentCard = styled(Card)`
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  margin-bottom: 24px;

  .ant-card-body {
    padding: 24px;
  }

  .ant-table {
    background: transparent;
    
    .ant-table-thead > tr > th {
      background: rgba(255, 255, 255, 0.05);
      border-bottom: 1px solid rgba(255, 255, 255, 0.1);
      color: rgba(255, 255, 255, 0.8);
    }
    
    .ant-table-tbody > tr > td {
      border-bottom: 1px solid rgba(255, 255, 255, 0.05);
      color: rgba(255, 255, 255, 0.7);
    }
    
    .ant-table-tbody > tr:hover > td {
      background: rgba(255, 255, 255, 0.05);
    }
  }
`;

const FilterForm = styled(Form)`
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 24px;
  
  .ant-form-item-label > label {
    color: rgba(255, 255, 255, 0.8);
  }
  
  .ant-select-selector {
    background: rgba(255, 255, 255, 0.1) !important;
    border-color: rgba(255, 255, 255, 0.2) !important;
  }
  
  .ant-select-selection-item {
    color: white;
  }
  
  .ant-picker {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
    
    input {
      color: white;
    }
    
    .ant-picker-suffix {
      color: rgba(255, 255, 255, 0.7);
    }
  }
`;

const Title = styled.h1`
  color: white;
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 24px;
  background: linear-gradient(135deg, #00d4ff, #ffffff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
`;

interface Transaction {
  id: string;
  orderId: string;
  amount: number;
  currency: string;
  paymentMethod: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled';
  createdAt: string;
  completedAt?: string;
}

const TransactionsPage: React.FC = () => {
  const navigate = useNavigate();
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loading, setLoading] = useState(true);
  const [filters, setFilters] = useState({
    status: undefined,
    dateRange: undefined,
    paymentMethod: undefined,
  });
  const [pagination, setPagination] = useState({
    current: 1,
    pageSize: 10,
    total: 0,
  });

  useEffect(() => {
    loadTransactions();
  }, [pagination.current, filters]);

  const loadTransactions = async () => {
    setLoading(true);
    try {
      // 这里需要后端提供获取交易记录的API
      const response = await UserService.getTransactions({
        page: pagination.current,
        limit: pagination.pageSize,
        status: filters.status,
        startDate: filters.dateRange?.[0]?.toISOString(),
        endDate: filters.dateRange?.[1]?.toISOString(),
        paymentMethod: filters.paymentMethod,
      });

      if (response.success) {
        setTransactions(response.data.items);
        setPagination({
          ...pagination,
          total: response.data.total,
        });
      }
    } catch (error) {
      console.error('Failed to load transactions:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleTableChange = (pagination: any) => {
    setPagination({
      ...pagination,
      current: pagination.current,
    });
  };

  const handleFilterChange = (values: any) => {
    setFilters(values);
    setPagination({
      ...pagination,
      current: 1,
    });
  };

  const columns = [
    {
      title: '交易ID',
      dataIndex: 'id',
      key: 'id',
      render: (text: string) => (
        <span style={{ color: '#00d4ff', fontFamily: 'monospace' }}>{text}</span>
      ),
    },
    {
      title: '订单ID',
      dataIndex: 'orderId',
      key: 'orderId',
      render: (text: string) => (
        <Button 
          type="link" 
          onClick={() => navigate(`/orders/${text}`)}
          style={{ padding: 0 }}
        >
          {text}
        </Button>
      ),
    },
    {
      title: '金额',
      dataIndex: 'amount',
      key: 'amount',
      render: (amount: number, record: Transaction) => (
        <span style={{ color: '#00d4ff', fontWeight: 'bold' }}>
          {record.currency} {amount}
        </span>
      ),
    },
    {
      title: '支付方式',
      dataIndex: 'paymentMethod',
      key: 'paymentMethod',
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const statusMap = {
          'completed': { color: 'green', text: '已完成' },
          'pending': { color: 'orange', text: '处理中' },
          'failed': { color: 'red', text: '失败' },
          'cancelled': { color: 'gray', text: '已取消' },
        };
        const statusInfo = statusMap[status as keyof typeof statusMap] || { color: 'default', text: status };
        return <Tag color={statusInfo.color}>{statusInfo.text}</Tag>;
      },
    },
    {
      title: '创建时间',
      dataIndex: 'createdAt',
      key: 'createdAt',
      render: (text: string) => new Date(text).toLocaleString(),
    },
    {
      title: '完成时间',
      dataIndex: 'completedAt',
      key: 'completedAt',
      render: (text: string) => text ? new Date(text).toLocaleString() : '-',
    },
  ];

  return (
    <TransactionsContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
      >
        <Title>交易记录</Title>

        <FilterForm
          layout="inline"
          onFinish={handleFilterChange}
        >
          <Form.Item name="status" label="状态">
            <Select
              placeholder="选择状态"
              allowClear
              style={{ width: 120 }}
              options={[
                { value: 'completed', label: '已完成' },
                { value: 'pending', label: '处理中' },
                { value: 'failed', label: '失败' },
                { value: 'cancelled', label: '已取消' },
              ]}
            />
          </Form.Item>

          <Form.Item name="paymentMethod" label="支付方式">
            <Select
              placeholder="选择支付方式"
              allowClear
              style={{ width: 120 }}
              options={[
                { value: 'alipay', label: '支付宝' },
                { value: 'wechat', label: '微信支付' },
                { value: 'card', label: '信用卡' },
                { value: 'usdt', label: 'USDT' },
              ]}
            />
          </Form.Item>

          <Form.Item name="dateRange" label="日期范围">
            <RangePicker />
          </Form.Item>

          <Form.Item>
            <Button type="primary" htmlType="submit">
              筛选
            </Button>
          </Form.Item>
        </FilterForm>

        <ContentCard>
          <Table
            columns={columns}
            dataSource={transactions}
            rowKey="id"
            pagination={{
              ...pagination,
              showSizeChanger: true,
              showTotal: (total) => `共 ${total} 条记录`,
            }}
            onChange={handleTableChange}
            loading={loading}
            locale={{
              emptyText: <Empty description="暂无交易记录" image={Empty.PRESENTED_IMAGE_SIMPLE} />,
            }}
          />
        </ContentCard>
      </motion.div>
    </TransactionsContainer>
  );
};

export default TransactionsPage;