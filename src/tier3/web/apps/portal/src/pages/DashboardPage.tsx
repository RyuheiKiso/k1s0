// ダッシュボード画面（リリース時点 minimum）。

import { Card } from '@k1s0/ui';
import { createI18n } from '@k1s0/i18n';

const i18n = createI18n('ja');

// DashboardPage はテナントの主要メトリクスを表示する画面（リリース時点 ではプレースホルダ）。
export function DashboardPage() {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: 16 }}>
      <Card title={i18n.t('common.welcome')}>
        <p>k1s0 portal へようこそ。本ページは tier3 web の最小実装です。</p>
        <p>リリース時点 で以下を順次追加します。</p>
        <ul>
          <li>テナントメトリクス（PubSub publish 数 / 失敗率）</li>
          <li>承認フロー一覧（ApprovalFlow tier2 サービス連携）</li>
          <li>請求書ダッシュボード（InvoiceGenerator tier2 サービス連携）</li>
        </ul>
      </Card>
      <Card title={i18n.t('approval.title')}>
        <p>承認待ち: <strong>--</strong> 件</p>
        <p>承認済（今月）: <strong>--</strong> 件</p>
      </Card>
    </div>
  );
}
