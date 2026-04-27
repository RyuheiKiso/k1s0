# @k1s0/ui

tier3 web 共通 UI コンポーネント。リリース時点 では `Button` / `Card` / `Spinner` の 3 種のみ提供。

## 利用例

```tsx
import { Button, Card, Spinner } from '@k1s0/ui';

function Example() {
  return (
    <Card title="承認">
      <Button variant="primary" onClick={() => alert('clicked')}>
        承認する
      </Button>
      <Spinner size={32} />
    </Card>
  );
}
```

## 設計判断

- React は `peerDependencies` で受ける（apps が install 主体）。
- スタイリングは Tailwind 前提（class 名指定）。Tailwind が使えない環境の場合は `className` で上書きする。
- 大きな依存（shadcn/ui / Radix UI）は採用後の運用拡大時に追加する。リリース時点 では「shadcn/ui 派生のフルセット」に向けたコア API のみ揃える。

## 関連

- 配置: `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/02_web_pnpm_workspace配置.md`
