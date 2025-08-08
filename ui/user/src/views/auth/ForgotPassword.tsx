import React, { useState } from 'react';
import { Form, Input, Button, Card, message, Result } from 'antd';
import { MailOutlined, ArrowLeftOutlined } from '@ant-design/icons';
import { Link } from 'react-router-dom';
import { authAPI } from '../../api/auth';

const ForgotPassword: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [emailSent, setEmailSent] = useState(false);
  const [email, setEmail] = useState('');

  const onFinish = async (values: { email: string }) => {
    setLoading(true);
    try {
      await authAPI.forgotPassword(values.email);
      setEmail(values.email);
      setEmailSent(true);
      message.success('重置密码邮件已发送');
    } catch (error: any) {
      message.error(error.response?.data?.message || '发送失败，请重试');
    } finally {
      setLoading(false);
    }
  };

  if (emailSent) {
    return (
      <div className="forgot-password-container" style={{ padding: '50px 20px', maxWidth: 500, margin: '0 auto' }}>
        <Result
          icon={<MailOutlined style={{ color: '#1890ff' }} />}
          title="邮件已发送"
          subTitle={`我们已向 ${email} 发送了重置密码的邮件，请查收并按照邮件中的指引重置密码。`}
          extra={[
            <Button type="primary" key="login">
              <Link to="/auth/login">返回登录</Link>
            </Button>,
            <Button key="resend" onClick={() => setEmailSent(false)}>
              重新发送
            </Button>,
          ]}
        />
      </div>
    );
  }

  return (
    <div className="forgot-password-container" style={{ padding: '50px 20px', maxWidth: 400, margin: '0 auto' }}>
      <Card 
        title="忘记密码" 
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
              block 
              size="large"
              loading={loading}
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