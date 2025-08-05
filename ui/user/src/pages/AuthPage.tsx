import React, { useState } from 'react';
import { Form, Input, Button, Tabs, Card, message, Checkbox } from 'antd';
import { UserOutlined, LockOutlined, MailOutlined } from '@ant-design/icons';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { AuthService } from '../utils/api';
import { useNavigate } from 'react-router-dom';

const AuthContainer = styled.div`
  min-height: 100vh;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  position: relative;
  overflow: hidden;

  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><defs><pattern id="grid" width="10" height="10" patternUnits="userSpaceOnUse"><path d="M 10 0 L 0 0 0 10" fill="none" stroke="%23ffffff" stroke-width="0.1" opacity="0.1"/></pattern></defs><rect width="100" height="100" fill="url(%23grid)"/></svg>');
    opacity: 0.3;
  }
`;

const AuthCard = styled(Card)`
  width: 100%;
  max-width: 400px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  position: relative;
  z-index: 1;

  .ant-card-body {
    padding: 40px;
  }

  .ant-tabs-tab {
    color: rgba(255, 255, 255, 0.7) !important;
    font-weight: 500;
    
    &.ant-tabs-tab-active {
      color: #00d4ff !important;
    }
  }

  .ant-tabs-ink-bar {
    background: linear-gradient(90deg, #00d4ff, #0099cc) !important;
  }

  .ant-form-item-label > label {
    color: rgba(255, 255, 255, 0.8);
    font-weight: 500;
  }

  .ant-input {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: white;
    border-radius: 8px;
    
    &:hover, &:focus {
      border-color: #00d4ff;
      background: rgba(255, 255, 255, 0.15);
    }

    &::placeholder {
      color: rgba(255, 255, 255, 0.5);
    }
  }

  .ant-btn-primary {
    background: linear-gradient(135deg, #00d4ff, #0099cc);
    border: none;
    border-radius: 8px;
    height: 45px;
    font-weight: 600;
    font-size: 16px;
    
    &:hover {
      background: linear-gradient(135deg, #0099cc, #007399);
      transform: translateY(-2px);
      box-shadow: 0 8px 25px rgba(0, 212, 255, 0.3);
    }
  }

  .ant-checkbox-wrapper {
    color: rgba(255, 255, 255, 0.7);
    
    .ant-checkbox-checked .ant-checkbox-inner {
      background-color: #00d4ff;
      border-color: #00d4ff;
    }
  }
`;

const Title = styled.h1`
  text-align: center;
  color: white;
  font-size: 28px;
  font-weight: 700;
  margin-bottom: 30px;
  background: linear-gradient(135deg, #00d4ff, #ffffff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
`;

const AuthPage: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const [loginForm] = Form.useForm();
  const [registerForm] = Form.useForm();

  const handleLogin = async (values: any) => {
    setLoading(true);
    try {
      const response = await AuthService.login(values.email, values.password);
      if (response.success) {
        localStorage.setItem('token', response.data.token);
        localStorage.setItem('user', JSON.stringify(response.data.user));
        message.success('登录成功！');
        navigate('/');
      } else {
        message.error(response.message || '登录失败');
      }
    } catch (error) {
      message.error('登录失败，请检查网络连接');
    } finally {
      setLoading(false);
    }
  };

  const handleRegister = async (values: any) => {
    setLoading(true);
    try {
      const response = await AuthService.register({
        username: values.username,
        email: values.email,
        password: values.password
      });
      if (response.success) {
        message.success('注册成功！请登录');
        registerForm.resetFields();
      } else {
        message.error(response.message || '注册失败');
      }
    } catch (error) {
      message.error('注册失败，请检查网络连接');
    } finally {
      setLoading(false);
    }
  };

  const loginTab = (
    <Form
      form={loginForm}
      name="login"
      onFinish={handleLogin}
      layout="vertical"
      size="large"
    >
      <Form.Item
        name="email"
        label="邮箱"
        rules={[
          { required: true, message: '请输入邮箱' },
          { type: 'email', message: '请输入有效的邮箱地址' }
        ]}
      >
        <Input
          prefix={<MailOutlined />}
          placeholder="请输入邮箱"
        />
      </Form.Item>

      <Form.Item
        name="password"
        label="密码"
        rules={[{ required: true, message: '请输入密码' }]}
      >
        <Input.Password
          prefix={<LockOutlined />}
          placeholder="请输入密码"
        />
      </Form.Item>

      <Form.Item>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Checkbox style={{ color: 'rgba(255, 255, 255, 0.7)' }}>记住我</Checkbox>
          <a href="#" style={{ color: '#00d4ff' }}>忘记密码？</a>
        </div>
      </Form.Item>

      <Form.Item>
        <Button
          type="primary"
          htmlType="submit"
          loading={loading}
          block
        >
          登录
        </Button>
      </Form.Item>
    </Form>
  );

  const registerTab = (
    <Form
      form={registerForm}
      name="register"
      onFinish={handleRegister}
      layout="vertical"
      size="large"
    >
      <Form.Item
        name="username"
        label="用户名"
        rules={[
          { required: true, message: '请输入用户名' },
          { min: 3, message: '用户名至少3个字符' }
        ]}
      >
        <Input
          prefix={<UserOutlined />}
          placeholder="请输入用户名"
        />
      </Form.Item>

      <Form.Item
        name="email"
        label="邮箱"
        rules={[
          { required: true, message: '请输入邮箱' },
          { type: 'email', message: '请输入有效的邮箱地址' }
        ]}
      >
        <Input
          prefix={<MailOutlined />}
          placeholder="请输入邮箱"
        />
      </Form.Item>

      <Form.Item
        name="password"
        label="密码"
        rules={[
          { required: true, message: '请输入密码' },
          { min: 6, message: '密码至少6个字符' }
        ]}
      >
        <Input.Password
          prefix={<LockOutlined />}
          placeholder="请输入密码"
        />
      </Form.Item>

      <Form.Item
        name="confirmPassword"
        label="确认密码"
        dependencies={['password']}
        rules={[
          { required: true, message: '请确认密码' },
          ({ getFieldValue }) => ({
            validator(_, value) {
              if (!value || getFieldValue('password') === value) {
                return Promise.resolve();
              }
              return Promise.reject(new Error('两次输入的密码不一致'));
            },
          }),
        ]}
      >
        <Input.Password
          prefix={<LockOutlined />}
          placeholder="请再次输入密码"
        />
      </Form.Item>

      <Form.Item
        name="agreement"
        valuePropName="checked"
        rules={[
          { validator: (_, value) => value ? Promise.resolve() : Promise.reject(new Error('请同意用户协议')) }
        ]}
      >
        <Checkbox style={{ color: 'rgba(255, 255, 255, 0.7)' }}>
          我已阅读并同意 <a href="#" style={{ color: '#00d4ff' }}>用户协议</a> 和 <a href="#" style={{ color: '#00d4ff' }}>隐私政策</a>
        </Checkbox>
      </Form.Item>

      <Form.Item>
        <Button
          type="primary"
          htmlType="submit"
          loading={loading}
          block
        >
          注册
        </Button>
      </Form.Item>
    </Form>
  );

  return (
    <AuthContainer>
      <motion.div
        initial={{ opacity: 0, y: 50 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
      >
        <AuthCard>
          <Title>RSWS 资源平台</Title>
          <Tabs
            defaultActiveKey="login"
            centered
            items={[
              {
                key: 'login',
                label: '登录',
                children: loginTab
              },
              {
                key: 'register',
                label: '注册',
                children: registerTab
              }
            ]}
          />
        </AuthCard>
      </motion.div>
    </AuthContainer>
  );
};

export default AuthPage;