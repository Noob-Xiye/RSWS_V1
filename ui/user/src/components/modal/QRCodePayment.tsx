import React, { useState, useEffect } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { Card, Typography, Button, message, Spin } from 'antd';
import { CopyOutlined, ReloadOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

interface QRCodePaymentProps {
  paymentId: string;
  qrCode: string;
  amount: number;
  currency: string;
  onPaymentSuccess: () => void;
}

const QRCodePayment: React.FC<QRCodePaymentProps> = ({
  paymentId,
  qrCode,
  amount,
  currency,
  onPaymentSuccess
}) => {
  const [checking, setChecking] = useState(false);
  const [countdown, setCountdown] = useState(600); // 10分钟倒计时

  useEffect(() => {
    const timer = setInterval(() => {
      setCountdown(prev => {
        if (prev <= 1) {
          clearInterval(timer);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  const checkPaymentStatus = async () => {
    setChecking(true);
    try {
      const response = await PaymentService.verifyPayment(paymentId);
      if (response.data.success) {
        message.success('支付成功！');
        onPaymentSuccess();
      } else {
        message.info('支付尚未完成，请继续等待');
      }
    } catch (error) {
      message.error('检查支付状态失败');
    } finally {
      setChecking(false);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    message.success('已复制到剪贴板');
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <Card className="qr-payment-card">
      <div className="qr-payment-content">
        <Title level={4}>扫码支付</Title>
        
        <div className="qr-code-container">
          {qrCode ? (
            <img src={qrCode} alt="Payment QR Code" className="qr-code" />
          ) : (
            <QRCodeSVG value={paymentId} size={200} />
          )}
        </div>
        
        <div className="payment-info">
          <Text strong>支付金额: {amount} {currency}</Text>
          <br />
          <Text type="secondary">请使用支持{currency}的钱包扫码支付</Text>
        </div>
        
        <div className="payment-actions">
          <Button 
            type="primary" 
            icon={<ReloadOutlined />}
            loading={checking}
            onClick={checkPaymentStatus}
          >
            检查支付状态
          </Button>
          
          <Button 
            icon={<CopyOutlined />}
            onClick={() => copyToClipboard(paymentId)}
          >
            复制支付信息
          </Button>
        </div>
        
        <div className="countdown">
          <Text type="warning">
            支付剩余时间: {formatTime(countdown)}
          </Text>
        </div>
      </div>
    </Card>
  );
};

export default QRCodePayment;