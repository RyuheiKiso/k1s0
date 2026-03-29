# ADR-0049: JWT audience クレームの Vec<String> 化

## ステータス
承認済み

## コンテキスト
auth サーバーの domain entity (`regions/system/server/rust/auth/src/domain/entity/claims.rs`) では `aud: String`（単一文字列）のままだったが、JWT 仕様（RFC 7519）では `aud` クレームは文字列または文字列の配列が許容される。

k1s0-auth ライブラリ (`regions/system/library/rust/auth/src/claims.rs`) では既に `Audience(Vec<String>)` として実装されているが、auth サーバーの `jwks_adapter.rs:49` で `c.aud.0.first().cloned().unwrap_or_default()` と先頭要素のみを取得しており、マルチ audience JWT の検証が不完全だった。

## 決定
auth サーバーの domain entity の `Claims.aud` を `String` から `Vec<String>` に変更し、JWT ライブラリと一致させる。

## 理由
- RFC 7519 の `aud` 仕様準拠
- k1s0-auth ライブラリの `Audience(Vec<String>)` との一貫性
- マルチ audience JWT（Keycloak が発行する場合がある）の正しい検証

## 影響

**ポジティブな影響**:
- マルチ audience JWT が正しく検証される
- JWT ライブラリとの型の整合性が確保される

**ネガティブな影響・トレードオフ**:
- `Claims.aud` を参照する全箇所でコード変更が必要（単一文字列比較から Vec 比較に変更）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| String のまま | 最初の audience のみ検証 | RFC 7519 非準拠、マルチ audience JWT 対応不可 |

## 参考
- RFC 7519 Section 4.1.3 - "aud" (Audience) Claim
- `regions/system/library/rust/auth/src/claims.rs` - Audience 型定義

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成（外部監査 HIGH-01 対応） | k1s0-team |
