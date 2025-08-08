import React from 'react';
import { Outlet } from 'react-router-dom';
import { Layout } from 'antd';
import styled from 'styled-components';
import Header from './Header';
import Footer from './Footer';

const { Content } = Layout;

const StyledLayout = styled(Layout)`
  min-height: 100vh;
  background: #0a0a0a;
`;

const StyledContent = styled(Content)`
  flex: 1;
  background: transparent;
`;

const MainLayout: React.FC = () => {
  return (
    <StyledLayout>
      <Header />
      <StyledContent>
        <Outlet />
      </StyledContent>
      <Footer />
    </StyledLayout>
  );
};

export default MainLayout;