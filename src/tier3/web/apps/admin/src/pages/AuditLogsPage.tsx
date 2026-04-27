// 監査ログ画面（リリース時点 minimum）。

import { Card } from '@k1s0/ui';

export function AuditLogsPage() {
  return (
    <Card title="監査ログ">
      <p>tier1 Audit Service と admin-bff 経由で連携する想定（リリース時点 で接続）。</p>
      <p>WORM ストレージから過去 30 日分の監査イベントを表示する設計。</p>
    </Card>
  );
}
