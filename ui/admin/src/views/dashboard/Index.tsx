import React, { useState, useEffect } from 'react';
import {
  Row,
  Col,
  Card,
  Statistic,
  Table,
  Progress,
  List,
  Avatar,
  Tag,
  Space,
  DatePicker,
  Select,
  Button
} from 'antd';
import {
  UserOutlined,
  FileTextOutlined,
  DollarOutlined,
  ShoppingCartOutlined,
  TrendingUpOutlined,
  TrendingDownOutlined,
  EyeOutlined
} from '@ant-design/icons';
import { Line, Column, Pie } from '@ant-design/plots';
import { adminAPI } from '../../api';
import type { RangePickerProps } from 'antd/es/date-picker';

const { RangePicker } = DatePicker;
const { Option } = Select;

interface DashboardStats {
  totalUsers: number;
  totalResources: number;
  totalRevenue: number;
  totalOrders: number;
  userGrowth: number;
  resourceGrowth: number;
  revenueGrowth: number;
  orderGrowth: number;
}

interface RecentActivity {
  id: string;
  type: 'user_register' | 'resource_upload' | 'order_create' | 'payment_complete';
  user: {
    id: string;
    name: string;
    avatar?: string;
  };
  description: string;
  createdAt: string;
}

interface TopResource {
  id: string;
  title: string;
  author: string;
  views: number;
  downloads: number;
  revenue: number;
}

const AdminDashboard: React.FC = () => {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [recentActivities, setRecentActivities] = useState<RecentActivity[]>([]);
  const [topResources, setTopResources] = useState<TopResource[]>([]);
  const [revenueData, setRevenueData] = useState<any[]>([]);
  const [userGrowthData, setUserGrowthData] = useState<any[]>([]);
  const [categoryData, setCategoryData] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [dateRange, setDateRange] = useState<[string, string]>(['', '']);

  useEffect(() => {
    fetchDashboardData();
  }, [dateRange]);

  const fetchDashboardData = async () => {
    setLoading(true);
    try {
      const [statsRes, activitiesRes, resourcesRes, revenueRes, userGrowthRes, categoryRes] = await Promise.all([
        adminAPI.getDashboardStats(dateRange[0], dateRange[1]),
        adminAPI.getRecentActivities(),
        adminAPI.getTopResources(),
        adminAPI.getRevenueChart(dateRange[0], dateRange[1]),
        adminAPI.getUserGrowthChart(dateRange[0], dateRange[1]),
        adminAPI.getCategoryStats()
      ]);

      setStats(statsRes.data);
      setRecentActivities(activitiesRes.data);
      setTopResources(resourcesRes.data);
      setRevenueData(revenueRes.data);
      setUserGrowthData(userGrowthRes.data);
      setCategoryData(categoryRes.data);
    } catch (error) {
      console.error('获取仪表板数据失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleDateRangeChange: RangePickerProps['onChange'] = (dates, dateStrings) => {
    setDateRange([dateStrings[0], dateStrings[1]]);
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'user_register': return <UserOutlined style={{ color: '#52c41a' }} />;
      case 'resource_upload': return <FileTextOutlined style={{ color: '#1890ff' }} />;
      case 'order_create': return <ShoppingCartOutlined style={{ color: '#fa8c16' }} />;
      case 'payment_complete': return <DollarOutlined style={{ color: '#eb2f96' }} />;
      default: return <EyeOutlined />;
    }
  };

  const getActivityColor = (type: string) => {
    switch (type) {
      case 'user_register': return 'green';
      case 'resource_upload': return 'blue';
      case 'order_create': return 'orange';
      case 'payment_complete': return 'purple';
      default: return 'default';
    }
  };

  const topResourceColumns = [
    {
      title: '资源标题',
      dataIndex: 'title',
      key: 'title',
      ellipsis: true,
    },
    {
      title: '作者',
      dataIndex: 'author',
      key: 'author',
    },
    {
      title: '浏览量',
      dataIndex: 'views',
      key: 'views',
      render: (views: number) => views.toLocaleString(),
    },
    {
      title: '下载量',
      dataIndex: 'downloads',
      key: 'downloads',
      render: (downloads: number) => downloads.toLocaleString(),
    },
    {
      title: '收入',
      dataIndex: 'revenue',
      key: 'revenue',
      render: (revenue: number) => `$${revenue.toFixed(2)}`,
    },
  ];

  const revenueConfig = {
    data: revenueData,
    xField: 'date',
    yField: 'revenue',
    smooth: true,
    color: '#1890ff',
    point: {
      size: 3,
      shape: 'circle',
    },
    tooltip: {
      formatter: (datum: any) => {
        return { name: '收入', value: `$${datum.revenue?.toFixed(2)}` };
      },
    },
  };

  const userGrowthConfig = {
    data: userGrowthData,
    xField: 'date',
    yField: 'count',
    color: '#52c41a',
    columnWidthRatio: 0.6,
    tooltip: {
      formatter: (datum: any) => {
        return { name: '新增用户', value: datum.count };
      },
    },
  };

  const categoryConfig = {
    data: categoryData,
    angleField: 'value',
    colorField: 'category',
    radius: 0.8,
    label: {
      type: 'outer',
      content: '{name} {percentage}',
    },
    interactions: [{ type: 'element-active' }],
  };

  if (!stats) {
    return <div>加载中...</div>;
  }

  return (
    <div className="admin-dashboard">
      {/* 顶部工具栏 */}
      <Row justify="space-between" align="middle" style={{ marginBottom: 24 }}>
        <Col>
          <h2>管理员仪表板</h2>
        </Col>
        <Col>
          <Space>
            <RangePicker onChange={handleDateRangeChange} />
            <Button onClick={fetchDashboardData} loading={loading}>
              刷新数据
            </Button>
          </Space>
        </Col>
      </Row>

      {/* 统计卡片 */}
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="总用户数"
              value={stats.totalUsers}
              prefix={<UserOutlined />}
              suffix={
                <span style={{ fontSize: '12px', color: stats.userGrowth >= 0 ? '#52c41a' : '#ff4d4f' }}>
                  {stats.userGrowth >= 0 ? <TrendingUpOutlined /> : <TrendingDownOutlined />}
                  {Math.abs(stats.userGrowth)}%
                </span>
              }
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="总资源数"
              value={stats.totalResources}
              prefix={<FileTextOutlined />}
              suffix={
                <span style={{ fontSize: '12px', color: stats.resourceGrowth >= 0 ? '#52c41a' : '#ff4d4f' }}>
                  {stats.resourceGrowth >= 0 ? <TrendingUpOutlined /> : <TrendingDownOutlined />}
                  {Math.abs(stats.resourceGrowth)}%
                </span>
              }
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="总收入"
              value={stats.totalRevenue}
              precision={2}
              prefix={<DollarOutlined />}
              suffix={
                <span style={{ fontSize: '12px', color: stats.revenueGrowth >= 0 ? '#52c41a' : '#ff4d4f' }}>
                  {stats.revenueGrowth >= 0 ? <TrendingUpOutlined /> : <TrendingDownOutlined />}
                  {Math.abs(stats.revenueGrowth)}%
                </span>
              }
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="总订单数"
              value={stats.totalOrders}
              prefix={<ShoppingCartOutlined />}
              suffix={
                <span style={{ fontSize: '12px', color: stats.orderGrowth >= 0 ? '#52c41a' : '#ff4d4f' }}>
                  {stats.orderGrowth >= 0 ? <TrendingUpOutlined /> : <TrendingDownOutlined />}
                  {Math.abs(stats.orderGrowth)}%
                </span>
              }
            />
          </Card>
        </Col>
      </Row>

      {/* 图表区域 */}
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={24} lg={16}>
          <Card title="收入趋势" loading={loading}>
            <Line {...revenueConfig} height={300} />
          </Card>
        </Col>
        <Col xs={24} lg={8}>
          <Card title="资源分类分布" loading={loading}>
            <Pie {...categoryConfig} height={300} />
          </Card>
        </Col>
      </Row>

      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={24} lg={12}>
          <Card title="用户增长" loading={loading}>
            <Column {...userGrowthConfig} height={250} />
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card title="热门资源" loading={loading}>
            <Table
              columns={topResourceColumns}
              dataSource={topResources}
              rowKey="id"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
      </Row>

      {/* 最近活动 */}
      <Row>
        <Col span={24}>
          <Card title="最近活动" loading={loading}>
            <List
              itemLayout="horizontal"
              dataSource={recentActivities}
              renderItem={(item) => (
                <List.Item>
                  <List.Item.Meta
                    avatar={
                      <Avatar 
                        src={item.user.avatar} 
                        icon={getActivityIcon(item.type)}
                      />
                    }
                    title={
                      <Space>
                        {item.user.name}
                        <Tag color={getActivityColor(item.type)}>
                          {item.type.replace('_', ' ')}
                        </Tag>
                      </Space>
                    }
                    description={item.description}
                  />
                  <div>{new Date(item.createdAt).toLocaleString()}</div>
                </List.Item>
              )}
            />
          </Card>
        </Col>
      </Row>
    </div>
  );
};

export default AdminDashboard;