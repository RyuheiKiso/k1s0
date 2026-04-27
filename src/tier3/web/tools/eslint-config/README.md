# @k1s0/eslint-config

tier3 web 配下で共通 ESLint 設定を提供する内部パッケージ。`@typescript-eslint` を中心に、`react` / `react-hooks` 拡張は各 app 側で個別追加する。

## 利用例

各 app / package の `.eslintrc.cjs`:

```js
module.exports = {
  extends: ['@k1s0/eslint-config'],
};
```

## 公開しない

`"private": true` のため npm publish 対象外。tier3 web 内部のみ。
