// テナント一覧画面（リリース時点 minimum、リリース時点 で admin-bff /api/admin/tenants を呼ぶ）。

import { Card } from '@k1s0/ui';

export function TenantListPage() {
  return (
    <Card title="テナント一覧">
      <p>本ページは admin-bff の REST endpoint を呼ぶ予定（リリース時点 で接続）。</p>
      <p>本リリース時点 はプレースホルダ表示。</p>
      <table style={{ marginTop: 16, width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr style={{ borderBottom: '1px solid #ddd' }}>
            <th style={{ textAlign: 'left', padding: 8 }}>Tenant ID</th>
            <th style={{ textAlign: 'left', padding: 8 }}>表示名</th>
            <th style={{ textAlign: 'left', padding: 8 }}>状態</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td style={{ padding: 8 }}>tenant-dev</td>
            <td style={{ padding: 8 }}>開発環境テナント</td>
            <td style={{ padding: 8 }}>active</td>
          </tr>
        </tbody>
      </table>
    </Card>
  );
}
