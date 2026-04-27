// Architecture ページ。

import { Card } from '@k1s0/ui';

export function ArchitecturePage() {
  return (
    <Card title="Architecture">
      <pre style={{ background: '#f4f4f4', padding: 12, borderRadius: 4, overflow: 'auto' }}>
{`tier3 Web/Native ──→ tier3 BFF (portal/admin) ──┐
                                                 ├──→ tier1 公開 12 API
tier3 Web/Native ──→ tier2 ドメインサービス ─────┘
                     (ApprovalFlow / InvoiceGenerator / TaxCalculator /
                      stock-reconciler / notification-hub)

依存方向: tier3 → tier2 → (sdk ← contracts) → tier1 → infra
`}
      </pre>
      <p>本図は単純化版。詳細は <code>docs/02_構想設計/01_アーキテクチャ/</code>。</p>
    </Card>
  );
}
