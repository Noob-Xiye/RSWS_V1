// 跨平台资金转换页面
import React, { useState } from 'react';
import { Card, Form, Select, InputNumber, Button, Alert, Divider } from 'antd';
import { SwapOutlined } from '@ant-design/icons';

const CrossPlatformPage: React.FC = () => {
  const [form] = Form.useForm();
  const [exchangeRate, setExchangeRate] = useState(1.0);
  const [fromPlatform, setFromPlatform] = useState<'paypal' | 'usdt'>('paypal');
  const [toPlatform, setToPlatform] = useState<'paypal' | 'usdt'>('usdt');

  const handleSwap = () => {
    const temp = fromPlatform;
    setFromPlatform(toPlatform);
    setToPlatform(temp);
    form.resetFields();
  };

  const onFinish = async (values: any) => {
    try {
      // 调用跨平台转换API
      console.log('转换请求:', values);
    } catch (error) {
      console.error('转换失败:', error);
    }
  };

  return (
    <div className="cross-platform-page">
      <Card title="跨平台资金转换">
        <Alert
          message="转换说明"
          description="支持PayPal与USDT之间的资金转换，转换会收取一定手续费"
          type="info"
          showIcon
          style={{ marginBottom: 24 }}
        />

        <Form form={form} onFinish={onFinish} layout="vertical">
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <Form.Item label="从" style={{ flex: 1 }}>
              <Select
                value={fromPlatform}
                onChange={setFromPlatform}
                options={[
                  { label: 'PayPal', value: 'paypal' },
                  { label: 'USDT', value: 'usdt' },
                ]}
              />
            </Form.Item>

            <Button
              type="text"
              icon={<SwapOutlined />}
              onClick={handleSwap}
              style={{ marginTop: 8 }}
            />

            <Form.Item label="到" style={{ flex: 1 }}>
              <Select
                value={toPlatform}
                onChange={setToPlatform}
                options={[
                  { label: 'PayPal', value: 'paypal' },
                  { label: 'USDT', value: 'usdt' },
                ]}
              />
            </Form.Item>
          </div>

          <Form.Item
            name="amount"
            label="转换金额"
            rules={[{ required: true, message: '请输入转换金额' }]}
          >
            <InputNumber
              style={{ width: '100%' }}
              placeholder="请输入金额"
              min={0.01}
              precision={2}
              addonAfter={fromPlatform === 'paypal' ? 'USD' : 'USDT'}
            />
          </Form.Item>

          <Divider />

          <div style={{ background: '#f5f5f5', padding: 16, borderRadius: 8 }}>
            <p>汇率: 1 {fromPlatform === 'paypal' ? 'USD' : 'USDT'} = {exchangeRate} {toPlatform === 'paypal' ? 'USD' : 'USDT'}</p>
            <p>手续费: 2%</p>
            <p>预计到账: -- {toPlatform === 'paypal' ? 'USD' : 'USDT'}</p>
          </div>

          <Form.Item style={{ marginTop: 24 }}>
            <Button type="primary" htmlType="submit" block size="large">
              确认转换
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
};

export default CrossPlatformPage;