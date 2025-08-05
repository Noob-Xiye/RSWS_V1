import React, { useState, useEffect } from 'react';
import { Form, Input, Switch, Button, Card, message, InputNumber, Select, Tag } from 'antd';
import { paymentConfigApi } from '../../api/paymentConfig';

const { Option } = Select;
const { TextArea } = Input;

interface BlockchainConfigData {
  network: string;
  network_name: string;
  api_url: string;
  api_key?: string;
  usdt_contract: string;
  wallet_addresses: string[];
  min_confirmations: number;
  min_amount: number;
  max_amount: number;
  fee_rate: number;
  is_active: boolean;
}

const BlockchainConfig: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [selectedNetwork, setSelectedNetwork] = useState<string>('tron');
  const [walletAddresses, setWalletAddresses] = useState<string[]>([]);

  useEffect(() => {
    loadConfig(selectedNetwork);
  }, [selectedNetwork]);

  const loadConfig = async (network: string) => {
    setLoading(true);
    try {
      const config = await paymentConfigApi.getBlockchainConfig(network);
      if (config) {
        form.setFieldsValue({
          ...config,
          api_key: '', // 不显示敏感信息
        });
        setWalletAddresses(config.wallet_addresses || []);
      } else {
        // 设置默认值
        const defaults = {
          tron: {
            network_name: 'TRON',
            api_url: 'https://api.trongrid.io',
            usdt_contract: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
            min_confirmations: 1,
          },
          ethereum: {
            network_name: 'Ethereum',
            api_url: 'https://api.etherscan.io/api',
            usdt_contract: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
            min_confirmations: 12,
          },
        };
        
        form.setFieldsValue({
          network,
          ...defaults[network as keyof typeof defaults],
          min_amount: 1,
          max_amount: 50000,
          fee_rate: 0,
          is_active: true,
        });
        setWalletAddresses([]);
      }
    } catch (error) {
      message.error('加载配置失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (values: BlockchainConfigData) => {
    setSaving(true);
    try {
      await paymentConfigApi.updateBlockchainConfig(selectedNetwork, {
        ...values,
        wallet_addresses: walletAddresses,
      });
      message.success('区块链配置更新成功');
    } catch (error) {
      message.error('更新配置失败');
    } finally {
      setSaving(false);
    }
  };

  const handleAddWallet = (address: string) => {
    if (address && !walletAddresses.includes(address)) {
      setWalletAddresses([...walletAddresses, address]);
    }
  };

  const handleRemoveWallet = (address: string) => {
    setWalletAddresses(walletAddresses.filter(addr => addr !== address));
  };

  return (
    <Card title="区块链配置" loading={loading}>
      <div style={{ marginBottom: 16 }}>
        <Select
          value={selectedNetwork}
          onChange={setSelectedNetwork}
          style={{ width: 200 }}
        >
          <Option value="tron">TRON (TRC20)</Option>
          <Option value="ethereum">Ethereum (ERC20)</Option>
        </Select>
      </div>

      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
      >
        <Form.Item
          name="network_name"
          label="网络名称"
          rules={[{ required: true, message: '请输入网络名称' }]}
        >
          <Input placeholder="网络显示名称" />
        </Form.Item>

        <Form.Item
          name="api_url"
          label="API URL"
          rules={[{ required: true, message: '请输入API URL' }]}
        >
          <Input placeholder="区块链API地址" />
        </Form.Item>

        <Form.Item name="api_key" label="API Key">
          <Input.Password placeholder="API密钥 (可选)" />
        </Form.Item>

        <Form.Item
          name="usdt_contract"
          label="USDT合约地址"
          rules={[{ required: true, message: '请输入USDT合约地址' }]}
        >
          <Input placeholder="USDT代币合约地址" />
        </Form.Item>

        <Form.Item label="钱包地址">
          <Input.Search
            placeholder="输入钱包地址"
            enterButton="添加"
            onSearch={handleAddWallet}
          />
          <div style={{ marginTop: 8 }}>
            {walletAddresses.map(address => (
              <Tag
                key={address}
                closable
                onClose={() => handleRemoveWallet(address)}
                style={{ marginBottom: 4 }}
              >
                {address}
              </Tag>
            ))}
          </div>
        </Form.Item>

        <Form.Item
          name="min_confirmations"
          label="最小确认数"
          rules={[{ required: true, message: '请输入最小确认数' }]}
        >
          <InputNumber min={1} style={{ width: '100%' }} />
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

export default BlockchainConfig;