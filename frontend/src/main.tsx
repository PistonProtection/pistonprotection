import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { ThemeProvider } from 'next-themes'
import { Toaster } from 'sonner'

import './styles/globals.css'

// Layouts
import RootLayout from './layouts/RootLayout'
import DashboardLayout from './layouts/DashboardLayout'
import AuthLayout from './layouts/AuthLayout'

// Pages
import DashboardOverview from './pages/dashboard/Overview'
import DashboardBackends from './pages/dashboard/Backends'
import DashboardFilters from './pages/dashboard/Filters'
import DashboardAnalytics from './pages/dashboard/Analytics'
import DashboardSettings from './pages/dashboard/Settings'
import DashboardBilling from './pages/dashboard/Billing'
import AuthLogin from './pages/auth/Login'
import AuthRegister from './pages/auth/Register'
import AuthForgotPassword from './pages/auth/ForgotPassword'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      retry: 1,
    },
  },
})

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
        <BrowserRouter>
          <Routes>
            <Route element={<RootLayout />}>
              {/* Auth routes */}
              <Route path="/auth" element={<AuthLayout />}>
                <Route index element={<Navigate to="/auth/login" replace />} />
                <Route path="login" element={<AuthLogin />} />
                <Route path="register" element={<AuthRegister />} />
                <Route path="forgot-password" element={<AuthForgotPassword />} />
              </Route>

              {/* Dashboard routes */}
              <Route path="/dashboard" element={<DashboardLayout />}>
                <Route index element={<DashboardOverview />} />
                <Route path="backends" element={<DashboardBackends />} />
                <Route path="filters" element={<DashboardFilters />} />
                <Route path="analytics" element={<DashboardAnalytics />} />
                <Route path="settings" element={<DashboardSettings />} />
                <Route path="billing" element={<DashboardBilling />} />
              </Route>

              {/* Default redirect */}
              <Route path="/" element={<Navigate to="/dashboard" replace />} />
              <Route path="*" element={<Navigate to="/dashboard" replace />} />
            </Route>
          </Routes>
        </BrowserRouter>
        <Toaster richColors position="top-right" />
      </ThemeProvider>
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  )
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
