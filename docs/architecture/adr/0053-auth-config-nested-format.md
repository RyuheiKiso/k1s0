# ADR-0053: AuthConfig ネスト形式統一

## ステータス

承認済み

## コンテキスト

外部監査 H-005、C-001、C-002 の指摘により、AuthConfig のスキーマ不一致が判明した。

### 現状の問題

- **saga / workflow サービス**: flat 形式（`auth.jwks_url`, `auth.issuer`, `auth.audience`）で Rust 構造体を定義
- **全サービスの config.docker.yaml**: nested 形式（`auth.jwt.issuer`, `auth.jwks.url`）で記述
- **auth サービス（正しい参照実装）**: `AuthConfig { jwt: JwtConfig, jwks: Option<JwksConfig> }` の nested 形式
- **server-common（`middleware/app.rs`）**: 古い flat 形式を提供

このスキーマ不一致により、saga / workflow サービスが起動不能となる（`auth: missing field 'jwks_url'` エラー）。

## 決定

全サービスの AuthConfig を nested 形式（auth サービス準拠）に統一する。

### 統一後の構造体定義

```rust
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub jwks: Option<JwksConfig>,
}

pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
}

pub struct JwksConfig {
    pub url: String,
    #[serde(default = "default_cache_ttl_secs")]
    pub cache_ttl_secs: u64,
}
```

### 対応内容

1. **server-common の `AuthConfig` を更新**: nested 形式に変更し、全サービスがこの定義を使用する
2. **saga / workflow の独自 AuthConfig を削除**: server-common の共通定義に統一する
3. **回帰防止テスト**: `include_str!` を使用した config parse テストを各サービスに追加し、YAML と構造体の整合性を継続的に検証する
4. **task / board / activity の `config/default.yaml` を修正**: フラット形式（`auth.jwks_url` 等）から nested 形式（`auth.jwt.*`, `auth.jwks.*`）へ変換し、Rust 構造体との整合性を回復する（CRIT-001 対応）

## 理由

- YAML 設定ファイルと Rust 構造体の整合性を確保し、起動不能を解消する
- auth サービスが既に正しい nested 形式を実装しており、これを正規フォーマットとすることで統一性が保たれる
- nested 形式は OIDC 標準に近く、JWT 設定と JWKS 設定の論理的な分離が明確である
- `include_str!` テストにより、将来の設定変更時にも不整合を即座に検出できる

## 影響

**ポジティブな影響**:

- saga / workflow サービスの起動不能が解消される
- 全サービスで単一の AuthConfig 定義を使用することで保守性が向上する
- config parse テストにより、YAML と構造体の不整合を CI で検出できる
- 将来の設定変更に対する安全性が確保される

**ネガティブな影響・トレードオフ**:

- 全サービスへの変更が必要となる（ただし cargo check で一括検証可能）
- server-common の破壊的変更により、全サービスの再ビルドが必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| YAML 側を flat 形式に変更 | config.docker.yaml を flat 形式に書き換える | auth サービスとの不整合が残る。OIDC 標準に近い nested 形式の方が保守性が高い |
| serde alias による両形式サポート | `#[serde(alias = "jwks_url")]` 等で両形式を受け付ける | 不一致を許容する設計となり、根本解決にならない。テストやドキュメントの複雑性が増す |
| サービスごとに個別対応 | 各サービスが独自の AuthConfig を維持 | 定義の分散により保守コストが増大する。不整合の再発リスクが残る |

## 参考

- [ADR-0008: JWT 秘密鍵ローテーション手順](0008-jwt-key-rotation.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成 | @team |
| 2026-04-01 | task / board / activity を適用対象として明示（CRIT-001 対応）。各サービスの config/default.yaml をフラット形式から nested 形式へ修正 | @team |
