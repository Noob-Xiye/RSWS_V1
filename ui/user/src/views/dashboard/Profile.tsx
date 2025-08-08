import React, { useState, useEffect } from 'react';
import { Card, Tabs, Avatar, Button, Table, Tag, message, Modal, Form, Input, Upload } from 'antd';
import { UserOutlined, DownloadOutlined, EyeOutlined, EditOutlined, UploadOutlined } from '@ant-design/icons';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { UserService, ResourceService } from '../utils/api';
import { User, Resource, Order } from '../types';

const UserCenterContainer = styled.div`
  min-height: 100vh;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
  padding: 20px;
`;

const ProfileCard = styled(Card)`
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  margin-bottom: 24px;

  .ant-card-body {
    padding: 30px;
  }

  .profile-header {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 20px;
  }

  .profile-info {
    flex: 1;
    
    h2 {
      color: white;
      margin: 0 0 8px 0;
      font-size: 24px;
    }
    
    p {
      color: rgba(255, 255, 255, 0.7);
      margin: 4px 0;
    }
  }

  .profile-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 20px;
    margin-top: 20px;
  }

  .stat-item {
    text-align: center;
    padding: 20px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    
    .stat-number {
      font-size: 28px;
      font-weight: bold;
      color: #00d4ff;
      display: block;
    }
    
    .stat-label {
      color: rgba(255, 255, 255, 0.7);
      margin-top: 8px;
    }
  }
`;

const ContentCard = styled(Card)`
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;

  .ant-card-body {
    padding: 24px;
  }

  .ant-tabs-tab {
    color: rgba(255, 255, 255, 0.7) !important;
    
    &.ant-tabs-tab-active {
      color: #00d4ff !important;
    }
  }

  .ant-tabs-ink-bar {
    background: linear-gradient(90deg, #00d4ff, #0099cc) !important;
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

const UserCenterPage: React.FC = () => {
  const [user, setUser] = useState<User | null>(null);
  const [myResources, setMyResources] = useState<Resource[]>([]);
  const [myOrders, setMyOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(true);
  const [editModalVisible, setEditModalVisible] = useState(false);
  const [form] = Form.useForm();

  useEffect(() => {
    loadUserData();
  }, []);

  const loadUserData = async () => {
    try {
      const userResponse = await UserService.getCurrentUser();
      if (userResponse.success) {
        setUser(userResponse.data);
        form.setFieldsValue(userResponse.data);
      }

      const resourcesResponse = await ResourceService.getMyResources();
      if (resourcesResponse.success) {
        setMyResources(resourcesResponse.data.items);
      }

      const ordersResponse = await UserService.getMyOrders();
      if (ordersResponse.success) {
        setMyOrders(ordersResponse.data.items);
      }
    } catch (error) {
      message.error('加载用户数据失败');
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateProfile = async (values: any) => {
    try {
      const response = await UserService.updateProfile(values);
      if (response.success) {
        setUser(response.data);
        setEditModalVisible(false);
        message.success('个人信息更新成功');
      } else {
        message.error(response.message || '更新失败');
      }
    } catch (error) {
      message.error('更新失败，请检查网络连接');
    }
  };

  const handleDownload = async (resourceId: string) => {
    try {
      const response = await ResourceService.downloadResource(resourceId);
      if (response.success) {
        // 处理文件下载
        const link = document.createElement('a');
        link.href = response.data.downloadUrl;
        link.download = response.data.filename;
        link.click();
        message.success('下载开始');
      } else {
        message.error(response.message || '下载失败');
      }
    } catch (error) {
      message.error('下载失败，请检查网络连接');
    }
  };

  const resourceColumns = [
    {
      title: '资源名称',
      dataIndex: 'title',
      key: 'title',
      render: (text: string, record: Resource) => (
        <div>
          <div style={{ color: 'white', fontWeight: 500 }}>{text}</div>
          <div style={{ color: 'rgba(255, 255, 255, 0.5)', fontSize: '12px' }}>
            {record.category}
          </div>
        </div>
      )
    },
    {
      title: '价格',
      dataIndex: 'price',
      key: 'price',
      render: (price: number) => (
        <span style={{ color: '#00d4ff', fontWeight: 'bold' }}>¥{price}</span>
      )
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const statusMap = {
          'active': { color: 'green', text: '已发布' },
          'pending': { color: 'orange', text: '审核中' },
          'rejected': { color: 'red', text: '已拒绝' }
        };
        const statusInfo = statusMap[status as keyof typeof statusMap] || { color: 'default', text: status };
        return <Tag color={statusInfo.color}>{statusInfo.text}</Tag>;
      }
    },
    {
      title: '操作',
      key: 'action',
      render: (_, record: Resource) => (
        <div style={{ display: 'flex', gap: '8px' }}>
          <Button
            type="link"
            icon={<EyeOutlined />}
            onClick={() => window.open(`/resources/${record.id}`, '_blank')}
          >
            查看
          </Button>
          <Button
            type="link"
            icon={<EditOutlined />}
            onClick={() => {/* 编辑资源逻辑 */}}
          >
            编辑
          </Button>
        </div>
      )
    }
  ];

  const orderColumns = [
    {
      title: '订单号',
      dataIndex: 'orderNumber',
      key: 'orderNumber',
      render: (text: string) => (
        <span style={{ color: '#00d4ff', fontFamily: 'monospace' }}>{text}</span>
      )
    },
    {
      title: '资源名称',
      dataIndex: 'resourceTitle',
      key: 'resourceTitle',
      render: (text: string) => (
        <span style={{ color: 'white' }}>{text}</span>
      )
    },
    {
      title: '金额',
      dataIndex: 'amount',
      key: 'amount',
      render: (amount: number) => (
        <span style={{ color: '#00d4ff', fontWeight: 'bold' }}>¥{amount}</span>
      )
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const statusMap = {
          'completed': { color: 'green', text: '已完成' },
          'pending': { color: 'orange', text: '待支付' },
          'cancelled': { color: 'red', text: '已取消' }
        };
        const statusInfo = statusMap[status as keyof typeof statusMap] || { color: 'default', text: status };
        return <Tag color={statusInfo.color}>{statusInfo.text}</Tag>;
      }
    },
    // 在orderColumns中更新操作列
    {
      title: '操作',
      key: 'action',
      render: (_, record: Order) => (
        <div style={{ display: 'flex', gap: '8px' }}>
          {record.status === 'completed' && (
            <Button
              type="link"
              icon={<DownloadOutlined />}
              onClick={() => handleDownload(record.resourceId)}
            >
              下载
            </Button>
          )}
          <Button
            type="link"
            icon={<EyeOutlined />}
            onClick={() => navigate(`/orders/${record.id}`)}
          >
            详情
          </Button>
        </div>
      )
    }
  ];

  if (loading) {
    return <div>加载中...</div>;
  }

  return (
    <UserCenterContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
      >
        <ProfileCard>
          <div className="profile-header">
            <Avatar size={80} icon={<UserOutlined />} src={user?.avatar} />
            <div className="profile-info">
              <h2>{user?.username}</h2>
              <p>邮箱: {user?.email}</p>
              <p>注册时间: {user?.createdAt ? new Date(user.createdAt).toLocaleDateString() : ''}</p>
            </div>
            <Button
              type="primary"
              icon={<EditOutlined />}
              onClick={() => setEditModalVisible(true)}
            >
              编辑资料
            </Button>
          </div>
          
          <div className="profile-stats">
            <div className="stat-item">
              <span className="stat-number">{myResources.length}</span>
              <div className="stat-label">我的资源</div>
            </div>
            <div className="stat-item">
              <span className="stat-number">{myOrders.length}</span>
              <div className="stat-label">购买记录</div>
            </div>
            <div className="stat-item">
              <span className="stat-number">¥{user?.balance || 0}</span>
              <div className="stat-label">账户余额</div>
            </div>
            <div className="stat-item">
              <span className="stat-number">{user?.points || 0}</span>
              <div className="stat-label">积分</div>
            </div>
          </div>
        </ProfileCard>

        <ContentCard>
          // 在Tabs组件中添加交易记录标签页
          <Tabs
            defaultActiveKey="resources"
            items={[
              {
                key: 'resources',
                label: '我的资源',
                children: (
                  <Table
                    columns={resourceColumns}
                    dataSource={myResources}
                    rowKey="id"
                    pagination={{ pageSize: 10 }}
                  />
                )
              },
              {
                key: 'orders',
                label: '购买记录',
                children: (
                  <Table
                    columns={orderColumns}
                    dataSource={myOrders}
                    rowKey="id"
                    pagination={{ pageSize: 10 }}
                  />
                )
              },
              {
                key: 'transactions',
                label: '交易记录',
                children: (
                  <div style={{ textAlign: 'right', marginBottom: '16px' }}>
                    <Button type="primary" onClick={() => navigate('/transactions')}>
                      查看全部交易记录
                    </Button>
                  </div>
                )
              }
            ]}
          />
        </ContentCard>
      </motion.div>

      <Modal
        title="编辑个人资料"
        open={editModalVisible}
        onCancel={() => setEditModalVisible(false)}
        footer={null}
        width={500}
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={handleUpdateProfile}
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
            name="bio"
            label="个人简介"
          >
            <Input.TextArea rows={4} placeholder="介绍一下自己..." />
          </Form.Item>
          
          <Form.Item
            name="avatar"
            label="头像"
          >
            <Upload
              name="avatar"
              listType="picture-card"
              showUploadList={false}
              beforeUpload={() => false}
            >
              <div>
                <UploadOutlined />
                <div style={{ marginTop: 8 }}>上传头像</div>
              </div>
            </Upload>
          </Form.Item>
          
          <Form.Item>
            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
              <Button onClick={() => setEditModalVisible(false)}>
                取消
              </Button>
              <Button type="primary" htmlType="submit">
                保存
              </Button>
            </div>
          </Form.Item>
        </Form>
      </Modal>
    </UserCenterContainer>
  );
};

export default UserCenterPage;