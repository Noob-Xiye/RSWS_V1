import React from 'react';
import { Button, Card, Row, Col, Statistic, Typography } from 'antd';
import { 
  RocketOutlined, 
  ShieldCheckOutlined, 
  CodeOutlined,
  DownloadOutlined,
  StarOutlined,
  UserOutlined
} from '@ant-design/icons';
import styled from 'styled-components';
import { motion } from 'framer-motion';
import { Link } from 'react-router-dom';

const { Title, Paragraph } = Typography;

const PageContainer = styled.div`
  padding-top: 70px;
  min-height: 100vh;
`;

const HeroSection = styled(motion.section)`
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  position: relative;
  overflow: hidden;
  
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: 
      radial-gradient(circle at 50% 50%, rgba(0, 212, 255, 0.1) 0%, transparent 70%),
      linear-gradient(45deg, transparent 30%, rgba(0, 153, 204, 0.05) 50%, transparent 70%);
    animation: pulse 4s ease-in-out infinite;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
  }
`;

const HeroContent = styled(motion.div)`
  position: relative;
  z-index: 2;
  max-width: 800px;
  padding: 0 2rem;
`;

const HeroTitle = styled(Title)`
  &.ant-typography {
    color: ${props => props.theme.colors.text.primary} !important;
    font-size: 4rem !important;
    font-weight: 700 !important;
    margin-bottom: 1.5rem !important;
    
    .highlight {
      background: linear-gradient(45deg, ${props => props.theme.colors.primary}, ${props => props.theme.colors.secondary});
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    @media (max-width: 768px) {
      font-size: 2.5rem !important;
    }
  }
`;

const HeroSubtitle = styled(Paragraph)`
  &.ant-typography {
    color: ${props => props.theme.colors.text.secondary} !important;
    font-size: 1.25rem !important;
    margin-bottom: 3rem !important;
    line-height: 1.6 !important;
  }
`;

const CTAButtons = styled.div`
  display: flex;
  gap: 1rem;
  justify-content: center;
  flex-wrap: wrap;
`;

const PrimaryButton = styled(Button)`
  height: 50px;
  padding: 0 2rem;
  font-size: 1.1rem;
  background: linear-gradient(45deg, ${props => props.theme.colors.primary}, ${props => props.theme.colors.secondary});
  border: none;
  border-radius: 25px;
  box-shadow: ${props => props.theme.shadows.glow};
  
  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 0 30px rgba(0, 212, 255, 0.5);
  }
`;

const SecondaryButton = styled(Button)`
  height: 50px;
  padding: 0 2rem;
  font-size: 1.1rem;
  background: transparent;
  border: 2px solid ${props => props.theme.colors.primary};
  color: ${props => props.theme.colors.primary};
  border-radius: 25px;
  
  &:hover {
    background: rgba(0, 212, 255, 0.1);
    transform: translateY(-2px);
  }
`;

const FeaturesSection = styled.section`
  padding: 5rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
`;

const FeatureCard = styled(motion(Card))`
  background: rgba(26, 26, 46, 0.6) !important;
  border: 1px solid ${props => props.theme.colors.border} !important;
  border-radius: 16px !important;
  backdrop-filter: blur(10px);
  
  .ant-card-body {
    padding: 2rem !important;
    text-align: center;
  }
  
  &:hover {
    border-color: ${props => props.theme.colors.primary} !important;
    box-shadow: ${props => props.theme.shadows.glow} !important;
    transform: translateY(-5px);
  }
`;

const FeatureIcon = styled.div`
  font-size: 3rem;
  color: ${props => props.theme.colors.primary};
  margin-bottom: 1rem;
`;

const StatsSection = styled.section`
  padding: 3rem 2rem;
  background: rgba(26, 26, 46, 0.3);
  backdrop-filter: blur(10px);
`;

const StatsContainer = styled.div`
  max-width: 1000px;
  margin: 0 auto;
`;

const StatCard = styled(Card)`
  background: rgba(12, 12, 12, 0.8) !important;
  border: 1px solid ${props => props.theme.colors.border} !important;
  border-radius: 12px !important;
  text-align: center;
  
  .ant-statistic-content {
    color: ${props => props.theme.colors.primary} !important;
  }
  
  .ant-statistic-title {
    color: ${props => props.theme.colors.text.secondary} !important;
  }
`;

const HomePage: React.FC = () => {
  const features = [
    {
      icon: <CodeOutlined />,
      title: '编程资源',
      description: '最新的编程教程、源码、工具和框架，助力您的开发之路'
    },
    {
      icon: <ShieldCheckOutlined />,
      title: '网络安全',
      description: '专业的安全工具、渗透测试资源和防护方案'
    },
    {
      icon: <RocketOutlined />,
      title: '高质量内容',
      description: '精心筛选的优质资源，确保每一份内容都有价值'
    }
  ];

  const stats = [
    { title: '注册用户', value: 12580, suffix: '+' },
    { title: '资源数量', value: 3420, suffix: '+' },
    { title: '下载次数', value: 89600, suffix: '+' },
    { title: '用户评分', value: 4.9, precision: 1 }
  ];

  return (
    <PageContainer>
      <HeroSection
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 1 }}
      >
        <HeroContent
          initial={{ y: 50, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.8, delay: 0.2 }}
        >
          <HeroTitle level={1}>
            专业的<span className="highlight">编程</span>与
            <span className="highlight">网络安全</span>资源平台
          </HeroTitle>
          <HeroSubtitle>
            汇聚最新的编程教程、安全工具和实战项目，
            为开发者和安全专家提供高质量的学习资源
          </HeroSubtitle>
          <CTAButtons>
            <Link to="/resources">
              <PrimaryButton 
                type="primary" 
                size="large"
                icon={<RocketOutlined />}
              >
                开始探索
              </PrimaryButton>
            </Link>
            <SecondaryButton 
              size="large"
              icon={<DownloadOutlined />}
            >
              免费试用
            </SecondaryButton>
          </CTAButtons>
        </HeroContent>
      </HeroSection>

      <FeaturesSection>
        <motion.div
          initial={{ y: 50, opacity: 0 }}
          whileInView={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.6 }}
          viewport={{ once: true }}
        >
          <Title level={2} style={{ textAlign: 'center', marginBottom: '3rem', color: '#ffffff' }}>
            为什么选择 RSWS？
          </Title>
          <Row gutter={[32, 32]}>
            {features.map((feature, index) => (
              <Col xs={24} md={8} key={index}>
                <FeatureCard
                  initial={{ y: 50, opacity: 0 }}
                  whileInView={{ y: 0, opacity: 1 }}
                  transition={{ duration: 0.6, delay: index * 0.1 }}
                  viewport={{ once: true }}
                  whileHover={{ y: -5 }}
                >
                  <FeatureIcon>{feature.icon}</FeatureIcon>
                  <Title level={4} style={{ color: '#ffffff', marginBottom: '1rem' }}>
                    {feature.title}
                  </Title>
                  <Paragraph style={{ color: '#b0b0b0', margin: 0 }}>
                    {feature.description}
                  </Paragraph>
                </FeatureCard>
              </Col>
            ))}
          </Row>
        </motion.div>
      </FeaturesSection>

      <StatsSection>
        <StatsContainer>
          <motion.div
            initial={{ y: 50, opacity: 0 }}
            whileInView={{ y: 0, opacity: 1 }}
            transition={{ duration: 0.6 }}
            viewport={{ once: true }}
          >
            <Title level={2} style={{ textAlign: 'center', marginBottom: '3rem', color: '#ffffff' }}>
              平台数据
            </Title>
            <Row gutter={[24, 24]}>
              {stats.map((stat, index) => (
                <Col xs={12} md={6} key={index}>
                  <StatCard>
                    <Statistic
                      title={stat.title}
                      value={stat.value}
                      suffix={stat.suffix}
                      precision={stat.precision}
                    />
                  </StatCard>
                </Col>
              ))}
            </Row>
          </motion.div>
        </StatsContainer>
      </StatsSection>
    </PageContainer>
  );
};

export default HomePage;