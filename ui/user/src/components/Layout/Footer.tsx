import React from 'react';
import { Layout, Row, Col, Typography, Space, Divider } from 'antd';
import { 
  GithubOutlined, 
  TwitterOutlined, 
  WechatOutlined,
  MailOutlined,
  PhoneOutlined,
  EnvironmentOutlined
} from '@ant-design/icons';
import styled from 'styled-components';
import { Link } from 'react-router-dom';

const { Footer: AntFooter } = Layout;
const { Title, Text } = Typography;

const StyledFooter = styled(AntFooter)`
  background: rgba(12, 12, 12, 0.95) !important;
  border-top: 1px solid ${props => props.theme.colors.border};
  padding: 3rem 2rem 1rem !important;
  margin-top: 5rem;
`;

const FooterContent = styled.div`
  max-width: 1200px;
  margin: 0 auto;
`;

const FooterSection = styled.div`
  .ant-typography-title {
    color: ${props => props.theme.colors.text.primary} !important;
    font-size: 1.2rem !important;
    margin-bottom: 1rem !important;
  }
`;

const FooterLink = styled(Link)`
  color: ${props => props.theme.colors.text.secondary};
  text-decoration: none;
  display: block;
  padding: 0.25rem 0;
  transition: color 0.3s ease;
  
  &:hover {
    color: ${props => props.theme.colors.primary};
  }
`;

const SocialIcon = styled.a`
  color: ${props => props.theme.colors.text.secondary};
  font-size: 1.5rem;
  transition: all 0.3s ease;
  
  &:hover {
    color: ${props => props.theme.colors.primary};
    transform: translateY(-2px);
  }
`;

const Copyright = styled.div`
  text-align: center;
  padding-top: 2rem;
  border-top: 1px solid ${props => props.theme.colors.border};
  color: ${props => props.theme.colors.text.secondary};
`;

const Footer: React.FC = () => {
  return (
    <StyledFooter>
      <FooterContent>
        <Row gutter={[32, 32]}>
          <Col xs={24} sm={12} md={6}>
            <FooterSection>
              <Title level={4}>关于RSWS</Title>
              <Text style={{ color: '#b0b0b0', lineHeight: 1.6 }}>
                专业的编程与网络安全资源平台，致力于为开发者和安全专家提供高质量的学习资源和工具。
              </Text>
            </FooterSection>
          </Col>
          
          <Col xs={24} sm={12} md={6}>
            <FooterSection>
              <Title level={4}>快速链接</Title>
              <FooterLink to="/resources">编程资源</FooterLink>
              <FooterLink to="/security">网络安全</FooterLink>
              <FooterLink to="/tutorials">教程中心</FooterLink>
              <FooterLink to="/pricing">价格方案</FooterLink>
            </FooterSection>
          </Col>
          
          <Col xs={24} sm={12} md={6}>
            <FooterSection>
              <Title level={4}>用户服务</Title>
              <FooterLink to="/help">帮助中心</FooterLink>
              <FooterLink to="/contact">联系我们</FooterLink>
              <FooterLink to="/privacy">隐私政策</FooterLink>
              <FooterLink to="/terms">服务条款</FooterLink>
            </FooterSection>
          </Col>
          
          <Col xs={24} sm={12} md={6}>
            <FooterSection>
              <Title level={4}>联系方式</Title>
              <Space direction="vertical" size="small">
                <Text style={{ color: '#b0b0b0' }}>
                  <MailOutlined style={{ marginRight: '0.5rem' }} />
                  contact@rsws.com
                </Text>
                <Text style={{ color: '#b0b0b0' }}>
                  <PhoneOutlined style={{ marginRight: '0.5rem' }} />
                  +86 400-123-4567
                </Text>
                <Text style={{ color: '#b0b0b0' }}>
                  <EnvironmentOutlined style={{ marginRight: '0.5rem' }} />
                  北京市朝阳区科技园
                </Text>
              </Space>
              
              <Divider style={{ borderColor: 'rgba(255,255,255,0.1)', margin: '1rem 0' }} />
              
              <Space size="large">
                <SocialIcon href="#" target="_blank">
                  <GithubOutlined />
                </SocialIcon>
                <SocialIcon href="#" target="_blank">
                  <TwitterOutlined />
                </SocialIcon>
                <SocialIcon href="#" target="_blank">
                  <WechatOutlined />
                </SocialIcon>
              </Space>
            </FooterSection>
          </Col>
        </Row>
        
        <Copyright>
          <Text style={{ color: '#666' }}>
            © 2024 RSWS. All rights reserved. | 京ICP备12345678号
          </Text>
        </Copyright>
      </FooterContent>
    </StyledFooter>
  );
};

export default Footer;