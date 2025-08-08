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
// 更新路由配置以匹配新的目录结构
import { createBrowserRouter } from 'react-router-dom';
import MainLayout from '../components/layout/MainLayout';
import ProtectedRoute from '../components/common/ProtectedRoute';
import PublicRoute from '../components/common/PublicRoute';

// 认证相关
import Login from '../views/auth/Login';
import Register from '../views/auth/Register';
import ForgotPassword from '../views/auth/ForgotPassword';
import ResetPassword from '../views/auth/ResetPassword';

// 仪表板
import Dashboard from '../views/dashboard/Index';
import Profile from '../views/dashboard/Profile';
import Security from '../views/dashboard/Security';
import Notifications from '../views/dashboard/Notifications';

// 资源管理
import ResourceList from '../views/resource/List';
import ResourceDetail from '../views/resource/Detail';
import ResourceUpload from '../views/resource/Upload';
import ResourceEdit from '../views/resource/Edit';
import MyResources from '../views/resource/MyResources';
import Favorites from '../views/resource/Favorites';
import ResourceSearch from '../views/resource/Search';

// 钱包管理
import WalletOverview from '../views/wallet/Overview';
import WalletAddresses from '../views/wallet/Addresses';
import WalletTransactions from '../views/wallet/Transactions';
import WalletDeposit from '../views/wallet/Deposit';
import WalletWithdraw from '../views/wallet/Withdraw';
import PayPalAccount from '../views/wallet/PayPalAccount';
import CrossPlatform from '../views/wallet/CrossPlatform';
import Commission from '../views/wallet/Commission';

// 支付相关
import PaymentOrders from '../views/payment/Orders';
import PaymentOrderDetail from '../views/payment/OrderDetail';
import PaymentHistory from '../views/payment/PaymentHistory';
import Refunds from '../views/payment/Refunds';

// 请求管理
import RequestList from '../views/request/List';
import RequestCreate from '../views/request/Create';
import RequestDetail from '../views/request/Detail';
import MyRequests from '../views/request/MyRequests';

export const router = createBrowserRouter([
  {
    path: '/auth',
    element: <PublicRoute />,
    children: [
      { path: 'login', element: <Login /> },
      { path: 'register', element: <Register /> },
      { path: 'forgot-password', element: <ForgotPassword /> },
      { path: 'reset-password', element: <ResetPassword /> },
    ],
  },
  {
    path: '/',
    element: <ProtectedRoute><MainLayout /></ProtectedRoute>,
    children: [
      { index: true, element: <Dashboard /> },
      
      // 仪表板
      { path: 'dashboard', element: <Dashboard /> },
      { path: 'profile', element: <Profile /> },
      { path: 'security', element: <Security /> },
      { path: 'notifications', element: <Notifications /> },
      
      // 资源管理
      {
        path: 'resources',
        children: [
          { index: true, element: <ResourceList /> },
          { path: 'search', element: <ResourceSearch /> },
          { path: 'upload', element: <ResourceUpload /> },
          { path: 'my', element: <MyResources /> },
          { path: 'favorites', element: <Favorites /> },
          { path: ':id', element: <ResourceDetail /> },
          { path: ':id/edit', element: <ResourceEdit /> },
        ],
      },
      
      // 钱包管理
      {
        path: 'wallet',
        children: [
          { index: true, element: <WalletOverview /> },
          { path: 'addresses', element: <WalletAddresses /> },
          { path: 'transactions', element: <WalletTransactions /> },
          { path: 'deposit', element: <WalletDeposit /> },
          { path: 'withdraw', element: <WalletWithdraw /> },
          { path: 'paypal', element: <PayPalAccount /> },
          { path: 'cross-platform', element: <CrossPlatform /> },
          { path: 'commission', element: <Commission /> },
        ],
      },
      
      // 支付相关
      {
        path: 'payment',
        children: [
          { path: 'orders', element: <PaymentOrders /> },
          { path: 'orders/:id', element: <PaymentOrderDetail /> },
          { path: 'history', element: <PaymentHistory /> },
          { path: 'refunds', element: <Refunds /> },
        ],
      },
      
      // 请求管理
      {
        path: 'requests',
        children: [
          { index: true, element: <RequestList /> },
          { path: 'create', element: <RequestCreate /> },
          { path: 'my', element: <MyRequests /> },
          { path: ':id', element: <RequestDetail /> },
        ],
      },
    ],
  },
]);