import React, { useState, useEffect } from 'react';
import {
  Card,
  Table,
  Button,
  Input,
  Select,
  Space,
  Tag,
  Modal,
  Form,
  message,
  Popconfirm,
  Avatar,
  Tooltip,
  DatePicker,
  Row,
  Col,
  Statistic
} from 'antd';
import {
  SearchOutlined,
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  UserOutlined,
  LockOutlined,
  UnlockOutlined,
  EyeOutlined
} from '@ant-design/icons';
import { userManagementAPI } from '../../api';
import type { ColumnsType } from 'antd/es/table';

const { Search } = Input;
const { Option } = Select;
const { RangePicker } = DatePicker;

interface User {
  id: string;
  username: string;
  email: string;
  avatar?: string;
  status: 'active' | 'inactive' | 'banned';
  level: number;
  commissionRate: number;
  totalEarnings: number;
  resourceCount: number;
  createdAt: string;
  lastLoginAt: string;
}

interface UserStats {
  totalUsers: number;
  activeUsers: number;
  newUsersToday: number;
  bannedUsers: number;
}

const UserList: React.FC = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [stats, setStats] = useState<UserStats>({
    totalUsers: 0,
    activeUsers: 0,
    newUsersToday: 0,
    bannedUsers: 0
  });
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [searchText, setSearchText] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('');
  const [pagination, setPagination] = useState({
    current: 1,
    pageSize: 10,
    total: 0
  });
  const [form] = Form.useForm();

  useEffect(() => {
    fetchUsers();
    fetchStats();
  }, [pagination.current, pagination.pageSize, searchText, statusFilter]);

  const fetchUsers = async () => {
    setLoading(true);
    try {
      const response = await userManagementAPI.getUsers({
        page: pagination.current,
        pageSize: pagination.pageSize,
        search: searchText,
        status: statusFilter
      });
      setUsers(response.data.users);
      setPagination(prev => ({
        ...prev,
        total: response.data.total
      }));
    } catch (error) {
      message.error('获取用户列表失败');
    } finally {
      setLoading(false);
    }
  };

  const fetchStats = async () => {
    try {
      const response = await userManagementAPI.getUserStats();
      setStats(response.data);
    } catch (error) {
      console.error('获取用户统计失败:', error);
    }
  };

  const handleStatusChange = async (userId: string, status: string) => {
    try {
      await userManagementAPI.updateUserStatus(userId, status);
      message.success('用户状态更新成功');
      fetchUsers();
      fetchStats();
    } catch (error) {
      message.error('状态更新失败');
    }
  };

  const handleEdit = (user: User) => {
    setEditingUser(user);
    setModalVisible(true);
    form.setFieldsValue({
      username: user.username,
      email: user.email,
      level: user.level,
      commissionRate: user.commissionRate
    });
  };

  const handleDelete = async (userId: string) => {
    try {
      await userManagementAPI.deleteUser(userId);
      message.success('用户删除成功');
      fetchUsers();
      fetchStats();
    } catch (error) {
      message.error('删除失败');
    }
  };

  const handleSubmit = async (values: any) => {
    try {
      if (editingUser) {
        await userManagementAPI.updateUser(editingUser.id, values);
        message.success('用户信息更新成功');
      }
      setModalVisible(false);
      fetchUsers();
    } catch (error) {
      message.error('操作失败');
    }
  };

  const columns: ColumnsType<User> = [
    {
      title: '用户',
      key: 'user',
      render: (_, record) => (
        <Space>
          <Avatar 
            src={record.avatar} 
            icon={<UserOutlined />}
            size="small"
          />
          <div>
            <div>{record.username}</div>
            <div style={{ fontSize: '12px', color: '#666' }}>
              {record.email}
            </div>
          </div>
        </Space>
      ),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const statusConfig = {
          active: { color: 'green', text: '正常' },
          inactive: { color: 'orange', text: '未激活' },
          banned: { color: 'red', text: '已封禁' },
        };
        const config = statusConfig[status as keyof typeof statusConfig];
        return <Tag color={config.color}>{config.text}</Tag>;
      },
    },
    {
      title: '等级',
      dataIndex: 'level',
      key: 'level',
      render: (level: number) => <Tag color="blue">LV.{level}</Tag>,
    },
    {
      title: '佣金比例',
      dataIndex: 'commissionRate',
      key: 'commissionRate',
      render: (rate: number) => `${(rate * 100).toFixed(1)}%`,
    },
    {
      title: '总收益',
      dataIndex: 'totalEarnings',
      key: 'totalEarnings',
      render: (earnings: number) => `$${earnings.toFixed(2)}`,
    },
    {
      title: '资源数',
      dataIndex: 'resourceCount',
      key: 'resourceCount',
    },
    {
      title: '注册时间',
      dataIndex: 'createdAt',
      key: 'createdAt',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: '操作',
      key: 'action',
      render: (_, record) => (
        <Space>
          <Tooltip title="查看详情">
            <Button
              type="text"
              size="small"
              icon={<EyeOutlined />}
              onClick={() => {/* 查看用户详情 */}}
            />
          </Tooltip>
          <Tooltip title="编辑">
            <Button
              type="text"
              size="small"
              icon={<EditOutlined />}
              onClick={() => handleEdit(record)}
            />
          </Tooltip>
          {record.status === 'active' ? (
            <Tooltip title="封禁用户">
              <Popconfirm
                title="确定要封禁这个用户吗？"
                onConfirm={() => handleStatusChange(record.id, 'banned')}
              >
                <Button
                  type="text"
                  size="small"
                  danger
                  icon={<LockOutlined />}
                />
              </Popconfirm>
            </Tooltip>
          ) : (
            <Tooltip title="解封用户">
              <Button
                type="text"
                size="small"
                icon={<UnlockOutlined />}
                onClick={() => handleStatusChange(record.id, 'active')}
              />
            </Tooltip>
          )}
          <Tooltip title="删除">
            <Popconfirm
              title="确定要删除这个用户吗？此操作不可恢复！"
              onConfirm={() => handleDelete(record.id)}
            >
              <Button
                type="text"
                size="small"
                danger
                icon={<DeleteOutlined />}
              />
            </Popconfirm>
          </Tooltip>
        </Space>
      ),
    },
  ];

  return (
    <div className="user-list-page">
      {/* 统计卡片 */}
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col span={6}>
          <Card>
            <Statistic
              title="总用户数"
              value={stats.totalUsers}
              prefix={<UserOutlined />}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="活跃用户"
              value={stats.activeUsers}
              valueStyle={{ color: '#52c41a' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="今日新增"
              value={stats.newUsersToday}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="封禁用户"
              value={stats.bannedUsers}
              valueStyle={{ color: '#ff4d4f' }}
            />
          </Card>
        </Col>
      </Row>

      {/* 用户列表 */}
      <Card
        title="用户管理"
        extra={
          <Space>
            <Search
              placeholder="搜索用户名或邮箱"
              allowClear
              style={{ width: 200 }}
              onSearch={setSearchText}
            />
            <Select
              placeholder="状态筛选"
              allowClear
              style={{ width: 120 }}
              onChange={setStatusFilter}
            >
              <Option value="active">正常</Option>
              <Option value="inactive">未激活</Option>
              <Option value="banned">已封禁</Option>
            </Select>
          </Space>
        }
      >
        <Table
          columns={columns}
          dataSource={users}
          rowKey="id"
          loading={loading}
          pagination={{
            ...pagination,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => 
              `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
            onChange: (page, pageSize) => {
              setPagination(prev => ({
                ...prev,
                current: page,
                pageSize: pageSize || 10
              }));
            }
          }}
        />
      </Card>

      {/* 编辑用户模态框 */}
      <Modal
        title="编辑用户信息"
        open={modalVisible}
        onCancel={() => {
          setModalVisible(false);
          setEditingUser(null);
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
            name="username"
            label="用户名"
            rules={[{ required: true, message: '请输入用户名' }]}
          >
            <Input />
          </Form.Item>
          
          <Form.Item
            name="email"
            label="邮箱"
            rules={[
              { required: true, message: '请输入邮箱' },
              { type: 'email', message: '请输入有效的邮箱地址' }
            ]}
          >
            <Input />
          </Form.Item>
          
          <Form.Item
            name="level"
            label="用户等级"
            rules={[{ required: true, message: '请选择用户等级' }]}
          >
            <Select>
              <Option value={1}>LV.1 新手</Option>
              <Option value={2}>LV.2 进阶</Option>
              <Option value={3}>LV.3 高级</Option>
              <Option value={4}>LV.4 专家</Option>
              <Option value={5}>LV.5 大师</Option>
            </Select>
          </Form.Item>
          
          <Form.Item
            name="commissionRate"
            label="佣金比例"
            rules={[{ required: true, message: '请输入佣金比例' }]}
          >
            <Input 
              type="number" 
              min={0} 
              max={1} 
              step={0.01}
              addonAfter="%"
              placeholder="0.05 表示 5%"
            />
          </Form.Item>

          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                保存
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

export default UserList;