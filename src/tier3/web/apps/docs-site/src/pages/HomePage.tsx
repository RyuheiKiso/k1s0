// Home ページ。

import { Card } from '@k1s0/ui';

export function HomePage() {
  return (
    <Card title="k1s0 ドキュメント">
      <p>本サイトは k1s0 採用検討者・利用開発者向けのドキュメントポータルです。</p>
      <p>リリース時点 では静的ページ集として最小提供。リリース時点 で Docusaurus / VitePress に移行し、`docs/` 配下の Markdown を統合配信します。</p>
      <ul>
        <li><strong>Getting Started</strong>: clone から最初のテストパスまで</li>
        <li><strong>Architecture</strong>: tier1 / tier2 / tier3 / SDK / contracts の関係</li>
      </ul>
    </Card>
  );
}
