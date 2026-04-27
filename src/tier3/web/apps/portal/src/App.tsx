// portal app のトップレベル。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/02_web_pnpm_workspace配置.md

import { Routes, Route, Link } from 'react-router-dom';
import { Card } from '@k1s0/ui';
import { createI18n } from '@k1s0/i18n';
import { DashboardPage } from './pages/DashboardPage';
import { StateExplorerPage } from './pages/StateExplorerPage';

// グローバル i18n（リリース時点 では ja 固定、リリース時点 でロケール選択を追加）。
const i18n = createI18n('ja');

export function App() {
  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: 16 }}>
      <header style={{ marginBottom: 24 }}>
        <h1 style={{ fontSize: 24, fontWeight: 600 }}>{i18n.t('common.appName')} portal</h1>
        <nav style={{ display: 'flex', gap: 12, marginTop: 8 }}>
          <Link to="/">Dashboard</Link>
          <Link to="/state">State Explorer</Link>
        </nav>
      </header>
      <main>
        <Routes>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/state" element={<StateExplorerPage />} />
          <Route
            path="*"
            element={
              <Card title={i18n.t('common.error')}>
                <p>404: page not found</p>
              </Card>
            }
          />
        </Routes>
      </main>
    </div>
  );
}
