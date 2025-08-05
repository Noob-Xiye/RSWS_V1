import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { ConfigProvider, theme } from 'antd';
import styled, { createGlobalStyle } from 'styled-components';
import { AuthProvider } from './hooks/useAuth';
import Header from './components/Layout/header';
import Footer from './components/Layout/Footer';
import HomePage from './pages/HomePage';
import ResourcesPage from './pages/ResourcesPage';
import ResourceDetailPage from './pages/ResourceDetailPage';
import AuthPage from './pages/AuthPage';
import UserCenterPage from './pages/UserCenterPage';
import UploadResourcePage from './pages/UploadResourcePage';
import PaymentPage from './pages/PaymentPage';
import SearchResultsPage from './pages/SearchResultsPage';

const GlobalStyle = createGlobalStyle`
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
    background: #0a0a0a;
    color: white;
    overflow-x: hidden;
  }

  .ant-layout {
    background: transparent !important;
  }

  .ant-menu {
    background: transparent !important;
    border: none !important;
  }

  .ant-menu-item {
    color: rgba(255, 255, 255, 0.8) !important;
    
    &:hover {
      color: #00d4ff !important;
      background: rgba(0, 212, 255, 0.1) !important;
    }
    
    &.ant-menu-item-selected {
      color: #00d4ff !important;
      background: rgba(0, 212, 255, 0.15) !important;
    }
  }

  .ant-btn {
    border-radius: 8px;
    font-weight: 500;
    transition: all 0.3s ease;
  }

  .ant-input, .ant-select-selector {
    border-radius: 8px;
  }

  .ant-card {
    border-radius: 12px;
  }

  .ant-modal-content {
    background: rgba(26, 26, 46, 0.95) !important;
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
  }

  .ant-modal-header {
    background: transparent !important;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1) !important;
    
    .ant-modal-title {
      color: white !important;
    }
  }

  .ant-modal-body {
    color: rgba(255, 255, 255, 0.8);
  }

  .ant-form-item-label > label {
    color: rgba(255, 255, 255, 0.8) !important;
  }

  .ant-input {
    background: rgba(255, 255, 255, 0.1) !important;
    border: 1px solid rgba(255, 255, 255, 0.2) !important;
    color: white !important;
    
    &:hover, &:focus {
      border-color: #00d4ff !important;
      background: rgba(255, 255, 255, 0.15) !important;
    }

    &::placeholder {
      color: rgba(255, 255, 255, 0.5) !important;
    }
  }

  .ant-select {
    .ant-select-selector {
      background: rgba(255, 255, 255, 0.1) !important;
      border: 1px solid rgba(255, 255, 255, 0.2) !important;
      color: white !important;
    }
    
    &:hover .ant-select-selector {
      border-color: #00d4ff !important;
    }
  }

  .ant-pagination {
    .ant-pagination-item {
      background: rgba(255, 255, 255, 0.1);
      border: 1px solid rgba(255, 255, 255, 0.2);
      
      a {
        color: rgba(255, 255, 255, 0.8);
      }
      
      &:hover {
        border-color: #00d4ff;
        
        a {
          color: #00d4ff;
        }
      }
      
      &.ant-pagination-item-active {
        background: #00d4ff;
        border-color: #00d4ff;
        
        a {
          color: white;
        }
      }
    }
    
    .ant-pagination-prev, .ant-pagination-next {
      .ant-pagination-item-link {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        color: rgba(255, 255, 255, 0.8);
        
        &:hover {
          border-color: #00d4ff;
          color: #00d4ff;
        }
      }
    }
  }
`;

const AppContainer = styled.div`
  min-height: 100vh;
  display: flex;
  flex-direction: column;
`;

const MainContent = styled.main`
  flex: 1;
`;

const App: React.FC = () => {
  return (
    <ConfigProvider
      theme={{
        algorithm: theme.darkAlgorithm,
        token: {
          colorPrimary: '#00d4ff',
          colorBgContainer: 'rgba(255, 255, 255, 0.05)',
          colorBorder: 'rgba(255, 255, 255, 0.1)',
          colorText: 'rgba(255, 255, 255, 0.8)',
          borderRadius: 8,
        },
      }}
    >
      <GlobalStyle />
      <AuthProvider>
        <Router>
          <AppContainer>
            <Routes>
              <Route path="/auth" element={<AuthPage />} />
              <Route path="/payment/:resourceId" element={<PaymentPage />} />
              <Route path="/*" element={
                <>
                  <Header />
                  <MainContent>
                    <Routes>
                      <Route path="/" element={<HomePage />} />
                      <Route path="/resources" element={<ResourcesPage />} />
                      <Route path="/resources/:id" element={<ResourceDetailPage />} />
                      <Route path="/search" element={<SearchResultsPage />} />
                      <Route path="/upload" element={<UploadResourcePage />} />
                      <Route path="/user-center" element={<UserCenterPage />} />
                      // 在路由配置中添加新的路由
                      <Routes>
                        {/* 现有路由 */}
                        <Route path="/orders/:orderId" element={<OrderDetailPage />} />
                        <Route path="/transactions" element={<TransactionsPage />} />
                      </Routes>
                  </MainContent>
                  <Footer />
                </>
              } />
            </Routes>
          </AppContainer>
        </Router>
      </AuthProvider>
    </ConfigProvider>
  );
};

export default App;