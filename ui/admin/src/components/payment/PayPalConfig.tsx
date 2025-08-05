import React, { useState, useEffect } from 'react';
import { Form, Input, Switch, Button, Card, message, InputNumber } from 'antd';
import { paymentConfigApi } from '../../api/paymentConfig';

interface PayPalConfigData {
  client_id: string;
  client_secret: string;
  sandbox: boolean;
  webhook_id?: string;
  webhook_secret?: string;
  return_url: string;
  cancel_url: string;
  brand_name: string;
  min_amount: number;
  max_amount: number;
  fee_rate: number;
  is_active: boolean;
}

const PayPalConfig: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const config = await paymentConfigApi.getPayPalConfig();
      if (config) {
        form.setFieldsValue({
          ...config,
          client_secret: '', // 不显示敏感信息
          webhook_secret: '',
        });
      }
    } catch (error) {
      message.error('加载配置失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (values: PayPalConfigData) => {
    setSaving(true);
    try {
      await paymentConfigApi.updatePayPalConfig(values);
      message.success('PayPal配置更新成功');
    } catch (error) {
      message.error('更新配置失败');
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card title="PayPal配置" loading={loading}>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={{
          sandbox: true,
          min_amount: 0.01,
          max_amount: 10000,
          fee_rate: 0.0349,
          is_active: true,
          brand_name: 'RSWS',
        }}
      >
        <Form.Item
          name="client_id"
          label="Client ID"
          rules={[{ required: true, message: '请输入Client ID' }]}
        >
          <Input placeholder="PayPal Client ID" />
        </Form.Item>

        <Form.Item
          name="client_secret"
          label="Client Secret"
          rules={[{ required: true, message: '请输入Client Secret' }]}
        >
          <Input.Password placeholder="PayPal Client Secret" />
        </Form.Item>

        <Form.Item name="sandbox" label="沙盒模式" valuePropName="checked">
          <Switch />
        </Form.Item>

        <Form.Item name="webhook_id" label="Webhook ID">
          <Input placeholder="PayPal Webhook ID (可选)" />
        </Form.Item>

        <Form.Item name="webhook_secret" label="Webhook Secret">
          <Input.Password placeholder="PayPal Webhook Secret (可选)" />
        </Form.Item>

        <Form.Item
          name="return_url"
          label="返回URL"
          rules={[{ required: true, message: '请输入返回URL' }]}
        >
          <Input placeholder="https://your-domain.com/payment/success" />
        </Form.Item>

        <Form.Item
          name="cancel_url"
          label="取消URL"
          rules={[{ required: true, message: '请输入取消URL' }]}
        >
          <Input placeholder="https://your-domain.com/payment/cancel" />
        </Form.Item>

        <Form.Item
          name="brand_name"
          label="品牌名称"
          rules={[{ required: true, message: '请输入品牌名称' }]}
        >
          <Input placeholder="显示在PayPal页面的品牌名称" />
        </Form.Item>

        <Form.Item
          name="min_amount"
          label="最小金额"
          rules={[{ required: true, message: '请输入最小金额' }]}
        >
          <InputNumber min={0.01} step={0.01} style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item
          name="max_amount"
          label="最大金额"
          rules={[{ required: true, message: '请输入最大金额' }]}
        >
          <InputNumber min={1} step={1} style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item
          name="fee_rate"
          label="手续费率"
          rules={[{ required: true, message: '请输入手续费率' }]}
        >
          <InputNumber min={0} max={1} step={0.0001} style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item name="is_active" label="启用" valuePropName="checked">
          <Switch />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" loading={saving}>
            保存配置
          </Button>
        </Form.Item>
      </Form>
    </Card>
  );
};

export default PayPalConfig;