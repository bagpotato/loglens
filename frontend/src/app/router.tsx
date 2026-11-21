import { createBrowserRouter, Navigate } from 'react-router'
import { ChatPage } from '@/features/ai'
import { LoginPage, RequireAuth } from '@/features/auth'
import { LogsPage } from '@/features/logs'
import { OverviewPage } from '@/features/overview'
import { Layout } from './Layout'

export const router = createBrowserRouter([
  { path: '/login', element: <LoginPage /> },
  {
    path: '/',
    element: (
      <RequireAuth>
        <Layout />
      </RequireAuth>
    ),
    children: [
      { index: true, element: <OverviewPage /> },
      { path: 'logs', element: <LogsPage /> },
      { path: 'chat', element: <ChatPage /> },
    ],
  },
  { path: '*', element: <Navigate to="/" replace /> },
])
