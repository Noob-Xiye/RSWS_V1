import React, { useState, useEffect } from 'react';
import {
  Table,
  Card,
  Button,
  Input,
  Select,
  Space,
  Tag,
  Avatar,
  Modal,
  Form,
  message,
  Popconfirm,
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
  ExportOutlined
} from '@ant-design/icons';
import { adminAPI } from '../../api';
import type { ColumnsType } from 'antd/es/table';
import type { TablePaginationConfig } from 'antd/es/table';

const { Search } = Input;
const { Option } = Select;
const { RangePicker } = DatePicker;

interface User {
  id: string;
  username: string;
  email: string;
  avatar?: string;
  status: 'active' | 'inactive' | 'banned';
  role: 'user' | 'vip' | 'admin';
  level: number;
  balance: number;
  totalSpent: number;
  resourceCount: number;
  createdAt: string;
  lastLoginAt?: string;
}

interface UserFilters {
  keyword?: string;
  status?: string;
  role?: string;
  level?: number;
  dateRange?: [string, string];
}

const UserManagement: React.FC = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [filters, setFilters] = useState<UserFilters>({});
  const [pagination, setPagination] = useState<TablePaginationConfig>({
    current: 1,
    pageSize: 20,
    total: 0,
  });
  const [stats, setStats] = useState({
    totalUsers: 0,
    activeUsers: 0,
    newUsersToday: 0,
    totalRevenue: 0,
  });
  const [form] = Form.useForm();

  useEffect(() => {
    fetchUsers();
    fetchStats();
  }, [pagination.current, pagination.pageSize, filters]);

  const fetchUsers = async () => {
    setLoading(true);
    try {
      const response = await adminAPI.getUsers({
        page: pagination.current || 1,
        pageSize: pagination.pageSize || 20,
        ...filters,
      });
      setUsers(response.data.users);
      setPagination(prev => ({
        ...prev,
        total: response.data.total,
      }));
    } catch (error) {
      message.error('获取用户列表失败');
    } finally {
      setLoading(false);
    }
  };

  const fetchStats = async () => {
    try {
      const response = await adminAPI.getUserStats();
      setStats(response.data);
    } catch (error) {
      console.error('获取用户统计失败:', error);
    }
  };

  const handleSearch = (value: string) => {
    setFilters(prev => ({ ...prev, keyword: value }));
    setPagination(prev => ({ ...prev, current: 1 }));
  };

  const handleFilterChange = (key: keyof UserFilters, value: any) => {
    setFilters(prev => ({ ...prev, [key]: value }));
    setPagination(prev => ({ ...prev, current: 1 }));
  };

  const handleTableChange = (newPagination: TablePaginationConfig) => {
    setPagination(newPagination);
  };

  const handleEdit = (user: User) => {
    setEditingUser(user);
    setModalVisible(true);
    form.setFieldsValue({
      username: user.username,
      email: user.email,
      role: user.role,
      level: user.level,
      status: user.status,
    });
  };

  const handleDelete = async (id: string) => {
    try {
      await adminAPI.deleteUser(id);
      message.success('删除用户成功');
      fetchUsers();
      fetchStats();
    } catch (error) {
      message.error('删除用户失败');
    }
  };

  const handleStatusChange = async (id: string, status: string) => {
    try {
      await adminAPI.updateUserStatus(id, status);
      message.success('更新用户状态成功');
      fetchUsers();
    } catch (error) {
      message.error('更新用户状态失败');
    }
  };

  const handleExport = async () => {
    try {
      const response = await adminAPI.exportUsers(filters);
      // 处理文件下载
      const url = window.URL.createObjectURL(new Blob([response.data]));
      const link = document.createElement('a');
      link.href = url;
      link.setAttribute('download', `users_${new Date().toISOString().split('T')[0]}.xlsx`);
      document.body.appendChild(link);
      link.click();
      link.remove();
      window