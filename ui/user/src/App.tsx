import React from 'react';
import { RouterProvider } from 'react-router-dom';
import { ConfigProvider, theme } from 'antd';
import styled, { createGlobalStyle } from 'styled-components';
import { AppProvider } from './store';
import { router } from './router';

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
      <AppProvider>
        <RouterProvider router={router} />
      </AppProvider>
    </ConfigProvider>
  );
};

export default App;