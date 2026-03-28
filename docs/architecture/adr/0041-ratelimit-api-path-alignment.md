# ADR-0041: レートリミット API パスのクライアント・サーバー間統一

## ステータス

承認済み

## コンテキスト

レートリミットサーバー（`regions/system/server/rust/ratelimit`）と4言語のクライアントライブラリ（Rust, Go, TypeScript, Dart）の API パスが完全に不一致であった。

**サーバー側の実装（正）**:
- `POST /api/v1/ratelimit/check` + ボディ: `{scope, identifier, window}`
- `POST /api/v1/ratelimit/reset` + ボディ: `{scope, identifier}`
- `GET /api/v1/ratelimit/usage`

**クライアント側の実装（不正）**:
- `POST /api/v1/ratelimit/{key}/check` + ボディ: `{cost}`
- `POST /api/v1/ratelimit/{key}/consume` + ボディ: `{cost}`
- `GET /api/v1/ratelimit/{key}/policy`

問題点:
1. パスにキーを含めるパターンとボディに含めるパターンが不一致
2. `consume` エンドポイントはサーバーに存在しない
3. `policy` エンドポイントはサーバーに存在しない（`usage` が対応する）
4. リクエストボディの構造が完全に異なる（`cost` vs `scope + identifier + window`）
5. クライアントのクラス名が `GrpcRateLimitClient` だが実際は HTTP REST を使用

この不一致は外部技術監査（C-02, L-16）で指摘された。

## 決定

クライアント側の API パス・リクエスト構造をサーバー実装に合わせて修正する。サーバー側は変更しない。

具体的な変更:
1. API パスからキーを除去し、ボディに `scope` + `identifier` を含める
2. `consume` メソッドは `check` エンドポイントで代用する
3. `get_limit` / `getLimit` メソッドは `/api/v1/ratelimit/usage` を使用する
4. クラス名を `GrpcRateLimitClient` → `HttpRateLimitClient` にリネーム
5. 既存コードとの後方互換性のため、旧名称の型エイリアスを deprecated として残す

## 理由

- サーバー側は実装・テスト・デプロイ済みで変更コストが大きい
- クライアントライブラリは各サービスの依存関係として使用されるが、まだ本番デプロイされていない
- サーバーの API 設計（scope + identifier 分離）のほうがセマンティクスとして正しい
- `GrpcRateLimitClient` という名称は実態（HTTP REST）と乖離しており混乱を招く

## 影響

**ポジティブな影響**:

- クライアントとサーバーの API が完全に一致し、疎通エラーが解消される
- クラス名が実装（HTTP REST）を正確に反映する
- 後方互換エイリアスにより既存コードの段階的移行が可能

**ネガティブな影響・トレードオフ**:

- 3言語（Rust, Go, TypeScript）のクライアントライブラリを同時修正する必要がある
- `consume` メソッドは `check` で代用するため、原子的な消費操作ではなくなる
- 既存の `key` ベースの API を使用していたコードは `scope:identifier` 形式に移行する必要がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| サーバー側をクライアントに合わせる | サーバーに `/{key}/check` パスを追加 | サーバー変更コストが大きく、既存テスト・デプロイに影響する |
| 両方のパスをサポート | サーバーに新旧両方のルートを追加 | メンテナンスコストが増加し、どちらが正かの混乱が生じる |
| API Gateway でパス変換 | Kong 等でパスリライト | 運用の複雑性が増加し、デバッグが困難になる |

## 参考

- [ratelimit サーバー handler](../../regions/system/server/rust/ratelimit/src/adapter/handler/mod.rs)
- [ratelimit Rust クライアント](../../regions/system/library/rust/ratelimit-client/src/grpc.rs)
- [ratelimit Go クライアント](../../regions/system/library/go/ratelimit-client/ratelimit_client.go)
- [ratelimit TypeScript クライアント](../../regions/system/library/typescript/ratelimit-client/src/index.ts)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-28 | 初版作成（C-02, L-16 監査対応） | k1s0-team |
