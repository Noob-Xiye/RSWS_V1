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
  Statistic,
  Row,
  Col
} from 'antd';
import { 
  PlusOutlined, 
  DeleteOutlined, 
  EditOutlined, 
  PayCircleOutlined,
  DollarOutlined
} from '@ant-design/icons';
import { paypalAPI } from '../../api';

interface PayPalAccount {
  id: string;
  email: string;
  status: 'active' | 'inactive' | 'pending';
  balance: number;
  isDefault: boolean;
  createdAt: string;
  lastSyncAt: string;
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
    } catch (error) {
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
    form.setFieldsValue(account);
  };

  const handleDelete = async (id: string) => {
    try {
      await paypalAPI.deleteAccount(id);
      message.success('删除成功');
      fetchAccounts();
    } catch (error) {
      message.error('删除失败');
    }
  };

  const handleSubmit = async (values: any) => {
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
    } catch (error) {
      message.error(editingAccount ? '更新失败' : '添加失败');
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await paypalAPI.setDefaultAccount(id);
      message.success('设置默认账户成功');
      fetchAccounts();
    } catch (error) {
      message.error('设置失败');
    }
  };

  const handleSyncBalance = async (id: string) => {
    try {
      await paypalAPI.syncBalance(id);
      message.success('同步余额成功');
      fetchAccounts();
    } catch (error) {
      message.error('同步失败');
    }
  };

  const columns = [
    {
      title: 'PayPal邮箱',
      dataIndex: 'email',
      key: 'email',
      render: (email: string, record: PayPalAccount) => (
        <Space>
          <PayCircleOutlined style={{ color: '#0070ba' }} />
          {email}
          {record.isDefault && <Tag color="gold">默认</Tag>}
        </Space>
      ),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const statusConfig = {
          active: { color: 'green', text: '活跃' },
          inactive: { color: 'red', text: '未激活' },
          pending: { color: 'orange', text: '待验证' },
        };
        const config = statusConfig[status as keyof typeof statusConfig];
        return <Tag color={config.color}>{config.text}</Tag>;
      },
    },
    {
      title: '余额',
      dataIndex: 'balance',
      key: 'balance',
      render: (balance: number) => (
        <Statistic
          value={balance}
          precision={2}
          prefix={<DollarOutlined />}
          valueStyle={{ fontSize: '14px' }}
        />
      ),
    },
    {
      title: '最后同步',
      dataIndex: 'lastSyncAt',
      key: 'lastSyncAt',
      render: (date: string) => new Date(date).toLocaleString(),
    },
    {
      title: '操作',
      key: 'action',
      render: (_, record: PayPalAccount) => (
        <Space>
          <Button
            type="link"
            size="small"
            onClick={() => handleSyncBalance(record.id)}
          >
            同步余额
          </Button>
          {!record.isDefault && (
            <Button
              type="link"
              size="small"
              onClick={() => handleSetDefault(record.id)}
            >
              设为默认
            </Button>
          )}
          <Button
            type="link"
            size="small"
            icon={<EditOutlined />}
            onClick={() => handleEdit(record)}
          >
            编辑
          </Button>
          <Popconfirm
            title="确定要删除这个PayPal账户吗？"
            onConfirm={() => handleDelete(record.id)}
            okText="确定"
            cancelText="取消"
          >
            <Button
              type="link"
              size="small"
              danger
              icon={<DeleteOutlined />}
            >
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div className="paypal-account-page">
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col span={8}>
          <Card>
            <Statistic
              title="PayPal总余额"
              value={totalBalance}
              precision={2}
              prefix={<DollarOutlined />}
              valueStyle={{ color: '#0070ba' }}
            />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic
              title="账户数量"
              value={accounts.length}
              suffix="个"
            />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic
              title="活跃账户"
              value={accounts.filter(acc => acc.status === 'active').length}
              suffix="个"
              valueStyle={{ color: '#52c41a' }}
            />
          </Card>
        </Col>
      </Row>

      <Card
        title="PayPal账户管理"
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
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={handleSubmit}
        >
          <Form.Item
            name="email"
            label="PayPal邮箱"
            rules={[
              { required: true, message: '请输入PayPal邮箱' },
              { type: 'email', message: '请输入有效的邮箱地址' }
            ]}
          >
            <Input placeholder="请输入PayPal账户邮箱" />
          </Form.Item>
          
          <Form.Item
            name="description"
            label="备注"
          >
            <Input.TextArea 
              placeholder="可选：添加账户备注信息" 
              rows={3}
            />
          </Form.Item>

          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                {editingAccount ? '更新' : '添加'}
              </Button>
              <Button onClick={() => setModalVisible(false)}>
                取消
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default PayPalAccountPage;