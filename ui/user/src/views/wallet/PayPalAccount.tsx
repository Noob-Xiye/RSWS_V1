import React, { useState, useEffect } from 'react';
import { 
  Card, 
  Button, 
  Table, 
  Modal, 
  Form, 
  Input, 
  message, 
  Tag, 
  Space,
  Popconfirm,
  Alert,
  Statistic,
  Row,
  Col
} from 'antd';
import { 
  PlusOutlined, 
  DeleteOutlined, 
  EditOutlined, 
  PayCircleOutlined,
  SyncOutlined,
  DollarOutlined
} from '@ant-design/icons';
import { paypalAPI } from '../../api/paypal';
import type { ColumnsType } from 'antd/es/table';

interface PayPalAccount {
  id: string;
  email: string;
  status: 'active' | 'inactive' | 'pending';
  balance: number;
  isDefault: boolean;
  createdAt: string;
  lastSyncAt?: string;
}

const PayPalAccountPage: React.FC = () => {
  const [accounts, setAccounts] = useState<PayPalAccount[]>([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [editingAccount, setEditingAccount] = useState<PayPalAccount | null>(null);
  const [totalBalance, setTotalBalance] = useState(0);
  const [form] = Form.useForm();

  useEffect(() => {
    fetchAccounts();
  }, []);

  const fetchAccounts = async () => {
    setLoading(true);
    try {
      const response = await paypalAPI.getAccounts();
      setAccounts(response.data.accounts);
      setTotalBalance(response.data.totalBalance);
    } catch (error: any) {
      message.error('获取PayPal账户失败');
    } finally {
      setLoading(false);
    }
  };

  const handleAdd = () => {
    setEditingAccount(null);
    setModalVisible(true);
    form.resetFields();
  };

  const handleEdit = (account: PayPalAccount) => {
    setEditingAccount(account);
    setModalVisible(true);
    form.setFieldsValue({
      email: account.email,
    });
  };

  const handleDelete = async (id: string) => {
    try {
      await paypalAPI.deleteAccount(id);
      message.success('删除成功');
      fetchAccounts();
    } catch (error: any) {
      message.error('删除失败');
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await paypalAPI.setDefaultAccount(id);
      message.success('设置默认账户成功');
      fetchAccounts();
    } catch (error: any) {
      message.error('设置失败');
    }
  };

  const handleSyncBalance = async (id: string) => {
    try {
      await paypalAPI.syncBalance(id);
      message.success('同步余额成功');
      fetchAccounts();
    } catch (error: any) {
      message.error('同步失败');
    }
  };

  const onFinish = async (values: { email: string }) => {
    try {
      if (editingAccount) {
        await paypalAPI.updateAccount(editingAccount.id, values);
        message.success('更新成功');
      } else {
        await paypalAPI.addAccount(values);
        message.success('添加成功');
      }
      setModalVisible(false);
      fetchAccounts();
    } catch (error: any) {
      message.error(error.response?.data?.message || '操作失败');
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'green';
      case 'inactive': return 'red';
      case 'pending': return 'orange';
      default: return 'default';
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'active': return '活跃';
      case 'inactive': return '未激活';
      case 'pending': return '待验证';
      default: return '未知';
    }
  };

  const columns: ColumnsType<PayPalAccount> = [
    {
      title: 'PayPal邮箱',
      dataIndex: 'email',
      key: 'email',
      render: (email: string, record: PayPalAccount) => (
        <Space>
          {email}
          {record.isDefault && <Tag color="blue">默认</Tag>}
        </Space>
      ),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={getStatusColor(status)}>
          {getStatusText(status)}
        </Tag>
      ),
    },
    {
      title: '余额',
      dataIndex: 'balance',
      key: 'balance',
      render: (balance: number) => (
        <Statistic 
          value={balance} 
          precision={2} 
          prefix="$" 
          valueStyle={{ fontSize: '14px' }}
        />
      ),
    },
    {
      title: '最后同步',
      dataIndex: 'lastSyncAt',
      key: 'lastSyncAt',
      render: (lastSyncAt: string) => 
        lastSyncAt ? new Date(lastSyncAt).toLocaleString() : '未同步',
    },
    {
      title: '操作',
      key: 'action',
      render: (_, record: PayPalAccount) => (
        <Space>
          <Button
            type="text"
            icon={<SyncOutlined />}
            onClick={() => handleSyncBalance(record.id)}
            title="同步余额"
          />
          {!record.isDefault && (
            <Button
              type="text"
              onClick={() => handleSetDefault(record.id)}
              title="设为默认"
            >
              设为默认
            </Button>
          )}
          <Button
            type="text"
            icon={<EditOutlined />}
            onClick={() => handleEdit(record)}
            title="编辑"
          />
          <Popconfirm
            title="确定要删除这个PayPal账户吗？"
            onConfirm={() => handleDelete(record.id)}
            okText="确定"
            cancelText="取消"
          >
            <Button
              type="text"
              danger
              icon={<DeleteOutlined />}
              title="删除"
            />
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div className="paypal-account-page">
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col span={24}>
          <Alert
            message="PayPal账户管理"
            description="您可以添加多个PayPal账户，用于接收付款和提现。请确保邮箱地址正确且已验证。"
            type="info"
            showIcon
          />
        </Col>
      </Row>

      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col span={8}>
          <Card>
            <Statistic
              title="PayPal总余额"
              value={totalBalance}
              precision={2}
              prefix={<DollarOutlined />}
              suffix="USD"
            />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic
              title="账户数量"
              value={accounts.length}
              prefix={<PayCircleOutlined />}
            />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic
              title="活跃账户"
              value={accounts.filter(acc => acc.status === 'active').length}
              prefix={<PayCircleOutlined />}
            />
          </Card>
        </Col>
      </Row>

      <Card
        title="PayPal账户列表"
        extra={
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={handleAdd}
          >
            添加账户
          </Button>
        }
      >
        <Table
          columns={columns}
          dataSource={accounts}
          rowKey="id"
          loading={loading}
          pagination={{ pageSize: 10 }}
        />
      </Card>

      <Modal
        title={editingAccount ? '编辑PayPal账户' : '添加PayPal账户'}
        open={modalVisible}
        onCancel={() => {
          setModalVisible(false);
          setEditingAccount(null);
          form.resetFields();
        }}
        footer={null}
        width={500}
      >
        <Form form={form} onFinish={onFinish} layout="vertical">
          <Form.Item
            name="email"
            label="PayPal邮箱地址"
            rules={[
              { required: true, message: '请输入PayPal邮箱地址' },
              { type: 'email', message: '请输入有效的邮箱地址' }
            ]}
          >
            <Input 
              placeholder="请输入PayPal邮箱地址" 
              size="large"
            />
          </Form.Item>
          
          <Alert
            message="注意事项"
            description="请确保输入的邮箱地址是您的PayPal账户邮箱，且该账户已完成验证。"
            type="warning"
            showIcon
            style={{ marginBottom: 16 }}
          />
          
          <Form.Item>
            <Space style={{ width: '100%', justifyContent: 'flex-end' }}>
              <Button onClick={() => setModalVisible(false)}>
                取消
              </Button>
              <Button type="primary" htmlType="submit">
                {editingAccount ? '更新' : '添加'}
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default PayPalAccountPage;