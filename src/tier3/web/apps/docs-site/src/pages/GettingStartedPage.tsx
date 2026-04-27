// Getting Started ページ。

import { Card } from '@k1s0/ui';

export function GettingStartedPage() {
  return (
    <Card title="Getting Started">
      <ol>
        <li>git clone https://github.com/k1s0/k1s0.git</li>
        <li>./tools/local-stack/up.sh</li>
        <li>./tools/sparse/checkout-role.sh tier1-go-dev</li>
        <li>cd src/tier1/go &amp;&amp; go test ./...</li>
      </ol>
      <p>詳細は <code>docs/05_実装/50_開発者体験設計/</code> を参照してください。</p>
    </Card>
  );
}
