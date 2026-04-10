# ADR-0113: Tauri GUI の CSP に `unsafe-inline` を許可する

## ステータス

承認済み

## コンテキスト

k1s0 デスクトップ CLI（`CLI/crates/k1s0-gui/`）は Tauri フレームワークを使用したデスクトップアプリケーションである。Tauri は WebView をフロントエンドとして使用し、Content Security Policy (CSP) でスクリプト・スタイルの実行を制御する。

外部監査（MED-009）において、CSP に `style-src 'self' 'unsafe-inline'` が含まれており、クロスサイトスクリプティング（XSS）リスクとして指摘された。

## 決定

Tauri GUI の CSP を以下のように設定し、`style-src` に `'unsafe-inline'` を許可する:

```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
}
```

## 理由

`unsafe-inline` を `style-src` のみに限定する理由は以下の通り:

1. **Tauri フレームワークの制約**: Tauri の WebView は、フレームワーク内部でインラインスタイルを動的に生成するケースがある。Rust 側から WebView への UI 状態の更新（プログレスバー、カラーテーマ等）はインラインスタイルを通じて実装されており、`unsafe-inline` なしでは UI レンダリングが破損する。

2. **`script-src` には適用しない**: XSS の主要な攻撃ベクターはスクリプト実行であり、`script-src 'self'` のみを許可することで、任意スクリプト実行のリスクは排除されている。インラインスタイルは JavaScript の実行能力を持たないため、リスクは限定的。

3. **デスクトップアプリの性質**: Tauri GUI は Web サーバーとして公開されるアプリではなく、ローカルデスクトップアプリである。外部からの入力がインラインスタイルとして注入される経路は存在しない（`tauri://localhost` スキームで動作）。

4. **代替手段のコスト**: Tauri の動的スタイル生成を全て CSS クラスベースに書き換えることは、フレームワーク内部実装の変更が必要であり、メンテナンスコストが著しく高い。

## 影響

**ポジティブな影響**:

- Tauri GUI のレンダリングが正常に動作する
- フレームワーク標準の実装パターンを維持できる

**ネガティブな影響・トレードオフ**:

- `style-src 'unsafe-inline'` により、将来的に XSS 脆弱性が発見された場合にスタイル注入が可能になるリスクがある（スクリプト実行は不可）
- 外部セキュリティ監査での指摘事項となりうる

**軽減策**:
- `script-src 'self'` は厳格に維持し、インラインスクリプトは一切許可しない
- Tauri の Rust 側での入力バリデーションを維持する
- `default-src 'self'` により外部リソースの読み込みを遮断する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| CSS クラスベース完全移行 | 動的スタイルを全て CSS クラスで実装し `unsafe-inline` を除去する | Tauri フレームワーク内部実装の変更が必要で、アップグレード毎に対応が必要になる |
| Nonce ベース CSP | `style-src 'nonce-xxx'` でインラインスタイルを制御する | Tauri v1/v2 の CSP 実装は nonce の自動付与をサポートしていない |
| CSP 廃止 | CSP を設定しない | セキュリティリスクが過大。`script-src` の保護が失われる |

## 参考

- [Tauri Security 設計](https://tauri.app/v1/references/security/)
- [`CLI/crates/k1s0-gui/tauri.conf.json`](../../../../CLI/crates/k1s0-gui/tauri.conf.json) — 現行 CSP 設定
- `docs/cli/TauriGUI設計.md` — Tauri GUI 設計方針

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-06 | 初版作成（MED-009 外部監査対応） | kiso ryuhei |
