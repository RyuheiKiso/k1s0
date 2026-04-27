// admin app のトップレベル。
//
// admin-bff の REST のみ呼ぶ（GraphQL は portal-bff のみ）。

import { Routes, Route, Link } from 'react-router-dom';
import { Card } from '@k1s0/ui';
import { TenantListPage } from './pages/TenantListPage';
import { AuditLogsPage } from './pages/AuditLogsPage';

export function App() {
  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: 16 }}>
      <header style={{ marginBottom: 24 }}>
        <h1 style={{ fontSize: 24, fontWeight: 600 }}>k1s0 admin</h1>
        <nav style={{ display: 'flex', gap: 12, marginTop: 8 }}>
          <Link to="/">テナント一覧</Link>
          <Link to="/audit">監査ログ</Link>
        </nav>
      </header>
      <main>
        <Routes>
          <Route path="/" element={<TenantListPage />} />
          <Route path="/audit" element={<AuditLogsPage />} />
          <Route
            path="*"
            element={
              <Card title="エラー">
                <p>404: page not found</p>
              </Card>
            }
          />
        </Routes>
      </main>
    </div>
  );
}
