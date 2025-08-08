import React, { useState, useEffect } from 'react';
import { Form, Input, Button, Card, message, Result } from 'antd';
import { LockOutlined, CheckCircleOutlined } from '@ant-design/icons';
import { useSearchParams, useNavigate, Link } from 'react-router-dom';
import { authAPI } from '../../api/auth';

const ResetPassword: React.FC = () => {
  const [form] = Form.useForm();
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [resetSuccess, setResetSuccess] = useState(false);
  const [tokenValid, setTokenValid] = useState<boolean | null>(null);
  
  const token = searchParams.get('token');
  const email = searchParams.get('email');

  useEffect(() => {
    if (!token || !email) {
      setTokenValid(false);
      return;
    }
    
    // 验证token有效性
    authAPI.verifyResetToken(token, email)
      .then(() => setTokenValid(true))
      .catch(() => setTokenValid(false));
  }, [token, email]);

  const onFinish = async (values: { password: string; confirmPassword: string }) => {
    if (!token || !email) {
      message.error('重置链接无效');
      return;
    }

    setLoading(true);
    try {
      await authAPI.resetPassword({
        token,
        email,
        password: values.password
      });
      setResetSuccess(true);
      message.success('密码重置成功');
    } catch (error: any) {
      message.error(error.response?.data?.message || '重置失败，请重试');
    } finally {
      setLoading(false);
    }
  };

  if (tokenValid === false) {
    return (
      <div className="reset-password-container" style={{ padding: '50px 20px', maxWidth: 500, margin: '0 auto' }}>
        <Result
          status="error"
          title="重置链接无效"
          subTitle="该重置链接已过期或无效，请重新申请密码重置。"
          extra={[
            <Button type="primary" key="forgot">
              <Link to="/auth/forgot-password">重新申请</Link>
            </Button>,
            <Button key="login">
              <Link to="/auth/login">返回登录</Link>
            </Button>,
          ]}
        />
      </div>
    );
  }

  if (resetSuccess) {
    return (
      <div className="reset-password-container" style={{ padding: '50px 20px', maxWidth: 500, margin: '0 auto' }}>
        <Result
          icon={<CheckCircleOutlined style={{ color: '#52c41a' }} />}
          title="密码重置成功"
          subTitle="您的密码已成功重置，请使用新密码登录。"
          extra={[
            <Button type="primary" key="login" onClick={() => navigate('/auth/login')}>
              立即登录
            </Button>,
          ]}
        />
      </div>
    );
  }

  if (tokenValid === null) {
    return (
      <div className="reset-password-container" style={{ padding: '50px 20px', textAlign: 'center' }}>
        <Card>验证重置链接...</Card>
      </div>
    );
  }

  return (
    <div className="reset-password-container" style={{ padding: '50px 20px', maxWidth: 400, margin: '0 auto' }}>
      <Card title="重置密码">
        <Form form={form} onFinish={onFinish} layout="vertical">
          <Form.Item
            name="password"
            label="新密码"
            rules={[
              { required: true, message: '请输入新密码' },
              { min: 6, message: '密码至少6位' },
              { pattern: /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/, message: '密码必须包含大小写字母和数字' }
            ]}
          >
            <Input.Password 
              prefix={<LockOutlined />}
              placeholder="请输入新密码" 
              size="large"
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
              placeholder="请再次输入新密码" 
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
              重置密码
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
};

export default ResetPassword;