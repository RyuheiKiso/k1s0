// docs-site のトップレベル。
//
// リリース時点 minimum: 静的ページ集。リリース時点 で Docusaurus / VitePress に置換予定。

import { Routes, Route, Link } from 'react-router-dom';
import { Card } from '@k1s0/ui';
import { HomePage } from './pages/HomePage';
import { GettingStartedPage } from './pages/GettingStartedPage';
import { ArchitecturePage } from './pages/ArchitecturePage';

export function App() {
  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', display: 'grid', gridTemplateColumns: '240px 1fr', minHeight: '100vh' }}>
      <aside style={{ padding: 16, borderRight: '1px solid #ddd', background: '#fafafa' }}>
        <h2 style={{ fontSize: 18, fontWeight: 600 }}>k1s0 docs</h2>
        <nav style={{ display: 'flex', flexDirection: 'column', gap: 8, marginTop: 16 }}>
          <Link to="/">Home</Link>
          <Link to="/getting-started">Getting Started</Link>
          <Link to="/architecture">Architecture</Link>
        </nav>
      </aside>
      <main style={{ padding: 24 }}>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/getting-started" element={<GettingStartedPage />} />
          <Route path="/architecture" element={<ArchitecturePage />} />
          <Route
            path="*"
            element={
              <Card title="404">
                <p>page not found</p>
              </Card>
            }
          />
        </Routes>
      </main>
    </div>
  );
}
