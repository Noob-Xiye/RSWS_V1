import React, { useState, useEffect } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Menu, Avatar, Dropdown, Button, Badge } from 'antd';
import { 
  UserOutlined, 
  ShoppingCartOutlined, 
  BellOutlined,
  MenuOutlined,
  CodeOutlined,
  CheckOutlined
} from '@ant-design/icons';
import styled from 'styled-components';
import { motion } from 'framer-motion';
import { Input } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';

const HeaderContainer = styled(motion.header)`
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  background: rgba(12, 12, 12, 0.95);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid ${props => props.theme.colors.border};
  padding: 0 2rem;
  height: 70px;
  display: flex;
  align-items: center;
  justify-content: space-between;
`;

const Logo = styled(Link)`
  display: flex;
  align-items: center;
  text-decoration: none;
  color: ${props => props.theme.colors.text.primary};
  font-size: 1.5rem;
  font-weight: bold;
  
  .logo-icon {
    margin-right: 0.5rem;
    color: ${props => props.theme.colors.primary};
    font-size: 2rem;
  }
  
  .logo-text {
    background: linear-gradient(45deg, ${props => props.theme.colors.primary}, ${props => props.theme.colors.secondary});
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
`;

const Navigation = styled.nav`
  display: flex;
  align-items: center;
  gap: 2rem;
  
  @media (max-width: 768px) {
    display: none;
  }
`;

const NavLink = styled(Link)<{ $active?: boolean }>`
  color: ${props => props.$active ? props.theme.colors.primary : props.theme.colors.text.secondary};
  text-decoration: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  transition: all 0.3s ease;
  position: relative;
  
  &:hover {
    color: ${props => props.theme.colors.primary};
    background: rgba(0, 212, 255, 0.1);
  }
  
  ${props => props.$active && `
    &::after {
      content: '';
      position: absolute;
      bottom: -2px;
      left: 50%;
      transform: translateX(-50%);
      width: 20px;
      height: 2px;
      background: ${props.theme.colors.primary};
      border-radius: 1px;
    }
  `}
`;

const UserActions = styled.div`
  display: flex;
  align-items: center;
  gap: 1rem;
`;

const SearchButton = styled(Button)`
  border: 1px solid ${props => props.theme.colors.border};
  background: rgba(26, 26, 46, 0.5);
  color: ${props => props.theme.colors.text.secondary};
  
  &:hover {
    border-color: ${props => props.theme.colors.primary};
    background: rgba(0, 212, 255, 0.1);
    color: ${props => props.theme.colors.primary};
  }
`;

const SearchBar = styled.div`
  flex: 1;
  max-width: 400px;
  margin: 0 20px;
  
  .ant-input-search {
    .ant-input {
      background: rgba(255, 255, 255, 0.1);
      border: 1px solid rgba(255, 255, 255, 0.2);
      color: white;
      border-radius: 20px;
      
      &::placeholder {
        color: rgba(255, 255, 255, 0.5);
      }
      
      &:hover, &:focus {
        border-color: #00d4ff;
        background: rgba(255, 255, 255, 0.15);
      }
    }
    
    .ant-btn {
      background: transparent;
      border: none;
      color: #00d4ff;
      
      &:hover {
        color: white;
        background: #00d4ff;
      }
    }
  }
`;

const Header: React.FC = () => {
  const navigate = useNavigate();
  const { user, logout } = useAuth();
  
  const handleSearch = (value: string) => {
    if (value.trim()) {
      navigate(`/search?q=${encodeURIComponent(value.trim())}`);
    }
  };
  const location = useLocation();
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 50);
    };
    
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  const userMenuItems = [
    {
      key: 'profile',
      label: <Link to="/user">个人中心</Link>,
      icon: <UserOutlined />,
    },
    {
      key: 'orders',
      label: '我的订单',
      icon: <ShoppingCartOutlined />,
    },
    {
      type: 'divider',
    },
    {
      key: 'logout',
      label: '退出登录',
      danger: true,
    },
  ];

  return (
    <HeaderContainer
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      transition={{ duration: 0.6, ease: 'easeOut' }}
      style={{
        background: scrolled ? 'rgba(12, 12, 12, 0.98)' : 'rgba(12, 12, 12, 0.95)',
        boxShadow: scrolled ? '0 4px 20px rgba(0, 0, 0, 0.3)' : 'none'
      }}
    >
      <Logo to="/">
        <ShieldCheckOutlined className="logo-icon" />
        <span className="logo-text">RSWS</span>
      </Logo>
      
      <Navigation>
        <NavLink to="/" $active={location.pathname === '/'}>
          首页
        </NavLink>
        <NavLink to="/resources" $active={location.pathname === '/resources'}>
          <CodeOutlined style={{ marginRight: '0.5rem' }} />
          编程资源
        </NavLink>
        <NavLink to="/security" $active={location.pathname === '/security'}>
          <ShieldCheckOutlined style={{ marginRight: '0.5rem' }} />
          网络安全
        </NavLink>
        <NavLink to="/tutorials" $active={location.pathname === '/tutorials'}>
          教程中心
        </NavLink>
      </Navigation>
      
      <UserActions>
        <SearchButton 
          icon={<SearchOutlined />} 
          shape="circle" 
          size="large"
        />
        
        <Badge count={3} size="small">
          <Button 
            icon={<BellOutlined />} 
            shape="circle" 
            size="large"
            style={{
              border: '1px solid rgba(0, 212, 255, 0.3)',
              background: 'rgba(26, 26, 46, 0.5)',
              color: '#b0b0b0'
            }}
          />
        </Badge>
        
        <Dropdown menu={{ items: userMenuItems }} placement="bottomRight">
          <Avatar 
            size="large" 
            icon={<UserOutlined />} 
            style={{
              background: 'linear-gradient(45deg, #00d4ff, #0099cc)',
              cursor: 'pointer'
            }}
          />
        </Dropdown>
      </UserActions>
    </HeaderContainer>
  );
};

export default Header;