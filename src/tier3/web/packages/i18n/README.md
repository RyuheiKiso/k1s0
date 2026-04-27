# @k1s0/i18n

tier3 web 用の軽量 i18n。i18next 依存なしの最小実装で、翻訳キー解決と `{var}` 補間を提供する。

## 利用例

```ts
import { createI18n } from '@k1s0/i18n';

const i18n = createI18n('ja');
console.log(i18n.t('common.welcome')); // -> "ようこそ"
console.log(i18n.t('hello {name}', { name: '世界' })); // -> "hello 世界"
```

## ロケール

`src/locales/` 配下の JSON ファイルがロケール辞書。新しいロケールを追加する場合は `<locale>.json` を追加して `src/index.ts` の `translations` map に登録する。

## 制約 / 拡張ポイント

- 複数形 / pluralization は未対応（リリース時点 で必要なら i18next-react に移行）
- 日付 / 数値整形は未対応（`Intl.DateTimeFormat` / `Intl.NumberFormat` を直接利用する）
