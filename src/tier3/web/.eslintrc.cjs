// tier3 web 共通の ESLint 設定。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/02_web_pnpm_workspace配置.md
//
// scope:
//   tools/eslint-config の package を extends する。各 app / package は
//   自身の .eslintrc.cjs で `extends: ['@k1s0/eslint-config']` を指定する。

module.exports = {
  root: true,
  extends: ['@k1s0/eslint-config'],
  ignorePatterns: ['dist/**', 'node_modules/**', '*.config.cjs', '*.config.js'],
};
