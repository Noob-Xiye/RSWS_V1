import React, { useState } from 'react';
import { Form, Input, Button, Card, message, Result } from 'antd';
import { MailOutlined, ArrowLeftOutlined } from '@ant-design/icons';
import { Link } from 'react-router-dom';
import { authAPI } from '../../api';

const ForgotPassword: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [emailSent, setEmailSent] = useState(false);

  const onFinish = async (values: { email: string }) => {
    setLoading(true);
    try {
      await authAPI.forgotPassword(values.email);
      setEmailSent(true);
      message.success('重置密码邮件已发送');
    } catch (error) {
      message.error('发送失败，请重试');
    } finally {
      setLoading(false);
    }
  };

  if (emailSent) {
    return (
      <div className="auth-container">
        <Card style={{ maxWidth: 400, margin: '0 auto' }}>
          <Result
            status="success"
            title="邮件已发送"
            subTitle="请检查您的邮箱，点击邮件中的链接重置密码"
            extra={[
              <Link to="/auth/login" key="back">
                <Button type="primary">返回登录</Button>
              </Link>
            ]}
          />
        </Card>
      </div>
    );
  }

  return (
    <div className="auth-container">
      <Card 
        title="忘记密码" 
        style={{ maxWidth: 400, margin: '0 auto' }}
        extra={
          <Link to="/auth/login">
            <Button type="text" icon={<ArrowLeftOutlined />}>
              返回登录
            </Button>
          </Link>
        }
      >
        <Form form={form} onFinish={onFinish} layout="vertical">
          <Form.Item
            name="email"
            label="邮箱地址"
            rules={[
              { required: true, message: '请输入邮箱地址' },
              { type: 'email', message: '请输入有效的邮箱地址' }
            ]}
          >
            <Input 
              prefix={<MailOutlined />}
              placeholder="请输入注册邮箱" 
              size="large"
            />
          </Form.Item>
          <Form.Item>
            <Button 
              type="primary" 
              htmlType="submit" 
              loading={loading}
              block 
              size="large"
            >
              发送重置邮件
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
};

export default ForgotPassword;