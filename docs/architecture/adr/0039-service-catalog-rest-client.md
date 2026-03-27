# ADR-0039: graphql-gateway の service-catalog クライアントを gRPC から REST へ変更

## ステータス

承認済み

## コンテキスト

graphql-gateway は service-catalog サーバーへの接続に `ServiceCatalogGrpcClient`（tonic ベース）を使用していた。
しかし service-catalog サーバーは HTTP/axum で実装されており、tonic/prost 依存も gRPC サーバーの定義も存在しない。
そのため graphql-gateway が起動して gRPC 接続を試みると接続エラーが確定する状態にあった。

外部技術監査 C-4 でこの不整合が指摘された。

## 決定

graphql-gateway の service-catalog クライアントを gRPC クライアントから reqwest ベースの REST クライアント（`ServiceCatalogHttpClient`）に変更する。

- 新クライアントは `regions/system/server/rust/graphql-gateway/src/infrastructure/http/service_catalog_client.rs` に配置する
- 公開インターフェース（メソッド名・シグネチャ・戻り値型）は旧 gRPC クライアントと同一に保ち、usecase 層・resolver 層の変更を最小化する
- `infrastructure/http/mod.rs` を新設し、REST クライアント群のモジュールとする
- `infrastructure/grpc/mod.rs` から `service_catalog_client` の参照を削除する

## 理由

- service-catalog は HTTP/axum で実装されており、gRPC サーバーを持たないため gRPC クライアントでは接続できない
- reqwest は既に graphql-gateway の依存関係に含まれており、追加コストなしに導入できる
- インターフェースを同一に保つことで呼び出し側（usecase・resolver）の変更量を最小化できる
- REST クライアントに変更することで実際のサービス実装と通信プロトコルが一致する

## 影響

**ポジティブな影響**:

- graphql-gateway が service-catalog に正常に接続できるようになる
- tonic による proto コンパイルが不要になり、ビルド依存が減少する
- service-catalog の REST API エンドポイントと通信プロトコルが一致し、保守性が向上する

**ネガティブな影響・トレードオフ**:

- REST クライアントは gRPC クライアントに比べて型安全性がやや低い（JSON デシリアライズ時のエラーが実行時に発生する）
- 一括ヘルスチェックエンドポイントが service-catalog に存在しないため、全サービスのヘルス取得は一覧取得→個別取得の 2 段階になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| service-catalog に gRPC サーバーを追加 | tonic/prost を service-catalog に追加して gRPC を実装する | 実装コストが大きく、アーキテクチャの方針（HTTP/axum）を変更する必要がある |
| proto なし tonic raw channel | proto 定義なしに tonic で接続する | service-catalog 側に gRPC エンドポイントが存在しないため無意味 |

## 参考

- [service-catalog サーバー設計](../../servers/system/service-catalog/server.md)
- [graphql-gateway サーバー設計](../../servers/system/graphql-gateway/server.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-27 | 初版作成 | 外部技術監査 C-4 対応 |
