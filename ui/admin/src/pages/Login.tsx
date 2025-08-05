import React, { useState } from 'react';
import { Form, Input, Button, message } from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { login } from '../api/auth';
import '../styles/login.css';

const Login: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const onFinish = async (values: { email: string; password: string }) => {
    try {
      setLoading(true);
      const response = await login(values.email, values.password);
      localStorage.setItem('adminToken', response.token);
      localStorage.setItem('adminInfo', JSON.stringify(response.admin));
      message.success('登录成功');
      navigate('/dashboard');
    } catch (error: any) {
      message.error(error.response?.data?.error || '登录失败，请检查邮箱和密码');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="admin-login-container">
      <div className="admin-login-form">
        <div className="admin-login-logo">
          <h1>资源管理系统</h1>
          <h2>管理员后台</h2>
        </div>
        <Form
          name="admin_login"
          className="login-form"
          initialValues={{ remember: true }}
          onFinish={onFinish}
        >
          <Form.Item
            name="email"
            rules={[{ required: true, message: '请输入管理员邮箱!' }]}
          >
            <Input 
              prefix={<UserOutlined />} 
              placeholder="管理员邮箱" 
              size="large" 
            />
          </Form.Item>
          <Form.Item
            name="password"
            rules={[{ required: true, message: '请输入密码!' }]}
          >
            <Input
              prefix={<LockOutlined />}
              type="password"
              placeholder="密码"
              size="large"
            />
          </Form.Item>

          <Form.Item>
            <Button 
              type="primary" 
              htmlType="submit" 
              className="login-form-button" 
              size="large"
              loading={loading}
              block
            >
              登录
            </Button>
          </Form.Item>
        </Form>
      </div>
    </div>
  );
};

export default Login;