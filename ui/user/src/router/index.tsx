import React from 'react';
import { createBrowserRouter, Navigate } from 'react-router-dom';
import { useAppContext } from '../store';

// 页面组件导入
import HomePage from '../pages/HomePage';
import AuthPage from '../pages/AuthPage';
import ResourcesPage from '../pages/ResourcesPage';
import ResourceDetailPage from '../pages/ResourceDetailPage';
import UserCenterPage from '../pages/UserCenterPage';
import UploadResourcePage from '../pages/UploadResourcePage';
import PaymentPage from '../pages/PaymentPage';
import SearchResultsPage from '../pages/SearchResultsPage';
import OrderDetailPage from '../pages/OrderDetailPage';
import TransactionsPage from '../pages/TransactionsPage';

// 布局组件
import MainLayout from '../components/Layout/MainLayout';

// 路由守卫组件
const ProtectedRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { state } = useAppContext();
  
  if (!state.isAuthenticated) {
    return <Navigate to="/auth" replace />;
  }
  
  return <>{children}</>;
};

const PublicRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { state } = useAppContext();
  
  if (state.isAuthenticated) {
    return <Navigate to="/" replace />;
  }
  
  return <>{children}</>;
};

// 路由配置
export const router = createBrowserRouter([
  {
    path: '/auth',
    element: (
      <PublicRoute>
        <AuthPage />
      </PublicRoute>
    )
  },
  {
    path: '/',
    element: <MainLayout />,
    children: [
      {
        index: true,
        element: <HomePage />
      },
      {
        path: 'resources',
        element: <ResourcesPage />
      },
      {
        path: 'resources/:id',
        element: <ResourceDetailPage />
      },
      {
        path: 'search',
        element: <SearchResultsPage />
      },
      {
        path: 'user',
        element: (
          <ProtectedRoute>
            <UserCenterPage />
          </ProtectedRoute>
        )
      },
      {
        path: 'upload',
        element: (
          <ProtectedRoute>
            <UploadResourcePage />
          </ProtectedRoute>
        )
      },
      {
        path: 'payment/:resourceId',
        element: (
          <ProtectedRoute>
            <PaymentPage />
          </ProtectedRoute>
        )
      },
      {
        path: 'orders/:orderId',
        element: (
          <ProtectedRoute>
            <OrderDetailPage />
          </ProtectedRoute>
        )
      },
      {
        path: 'transactions',
        element: (
          <ProtectedRoute>
            <TransactionsPage />
          </ProtectedRoute>
        )
      }
    ]
  },
  {
    path: '*',
    element: <Navigate to="/" replace />
  }
]);