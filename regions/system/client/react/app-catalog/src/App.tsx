import { BrowserRouter, Routes, Route, Link } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AppListPage } from './pages/AppListPage';
import { AppDetailPage } from './pages/AppDetailPage';
import { AdminPage } from './pages/AdminPage';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      retry: 1,
    },
  },
});

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <header className="app-header">
          <nav className="app-header__nav">
            <Link to="/" className="app-header__logo">
              App Catalog
            </Link>
            <Link to="/admin">管理</Link>
          </nav>
        </header>
        <main className="app-main">
          <Routes>
            <Route path="/" element={<AppListPage />} />
            <Route path="/apps/:appId" element={<AppDetailPage />} />
            <Route path="/admin" element={<AdminPage />} />
          </Routes>
        </main>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
