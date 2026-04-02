# ADR-0068: readyz エンドポイントレスポンス形式の標準化

## ステータス

承認済み

## コンテキスト

各サービスの `/readyz` エンドポイントが独自形式で実装されており、レスポンスのステータス文字列・チェック構造がサービスごとに不統一であった。
この状態では監視ダッシュボードや自動アラートルールを共通クエリで記述できず、監視自動化が困難であった。

具体的な問題点:
- サービスによって `"status": "ok"` / `"status": "healthy"` / `"status": "up"` など文字列が混在
- チェック項目の構造（配列形式・オブジェクト形式）が統一されていない
- タイムスタンプの有無・形式もサービスによって異なる
- 外部技術監査（2026-04-01）で MED-001 として指摘

## 決定

`regions/system/library/go/k1s0-health` ライブラリの `HealthResponse` 構造体を標準フォーマットとし、
今後全サービスの `/readyz` エンドポイントはこの形式でレスポンスを返す。

### 標準フォーマット

```json
{
  "status": "healthy",
  "checks": {
    "<service_name>": "<ok | error: <detail>>"
  },
  "timestamp": "<ISO 8601>"
}
```

#### ステータス値の定義

| 値 | 意味 |
|----|------|
| `healthy` | 全チェック項目が正常 |
| `degraded` | 一部チェック項目が異常だが、主要機能は継続可能 |
| `unhealthy` | 重要なチェック項目が異常で、正常動作不可 |

#### checks フィールドの定義

- キー: チェック対象のサービス名（例: `database`, `keycloak`, `kafka`）
- 値: 正常時は `"ok"`、異常時は `"error: <詳細メッセージ>"` 形式

#### timestamp フィールドの定義

- ISO 8601 形式（例: `"2026-04-02T12:00:00Z"`）
- UTC タイムゾーン必須

### レスポンス例

```json
{
  "status": "healthy",
  "checks": {
    "database": "ok",
    "keycloak": "ok",
    "kafka": "ok"
  },
  "timestamp": "2026-04-02T12:00:00Z"
}
```

```json
{
  "status": "unhealthy",
  "checks": {
    "database": "ok",
    "keycloak": "error: connection refused to https://keycloak:8443",
    "kafka": "ok"
  },
  "timestamp": "2026-04-02T12:00:00Z"
}
```

### HTTP ステータスコードの定義

| HealthResponse.status | HTTP ステータスコード |
|----------------------|-------------------|
| `healthy` | 200 OK |
| `degraded` | 200 OK（監視ツールがボディを解析して判断） |
| `unhealthy` | 503 Service Unavailable |

## 理由

- `k1s0-health` ライブラリはすでに Go サービスで利用されており、新たな実装コストが最小である
- 標準形式を定めることで Prometheus の `probe_success` に加え、ボディ内容のアラートルールを共通化できる
- `degraded` 状態を設けることで、部分障害時にもサービスを継続しながら監視で検知できる
- HIGH-001 対応として graphql-gateway の readyz が先行実装済みであり、実運用上の検証が取れている

## 影響

**ポジティブな影響**:

- 監視ダッシュボードのクエリを単一の標準形式で記述できるようになる
- Alertmanager のルールを全サービス共通化できる
- 新規サービスの readyz 実装が `k1s0-health` ライブラリを使うだけで完了する

**ネガティブな影響・トレードオフ**:

- 既存サービスは段階的移行が必要であり、移行期間中は形式が混在する
- 監視ダッシュボードの既存クエリを標準形式に更新する作業が発生する
- Dart（Flutter）クライアントなど他言語では `k1s0-health` ライブラリを参照できないため、独自実装が必要

## 移行方針

1. **新規サービス**: 即時適用。`k1s0-health` ライブラリの `HealthResponse` を使用すること
2. **既存 Go サービス**: `k1s0-health` ライブラリをインポートして段階的に移行する
3. **既存 Rust サービス**: `k1s0-health` に相当する Rust 実装（`k1s0-server-common` の health モジュール）を同一フォーマットに準拠させる
4. **先行実装**: HIGH-001 対応として graphql-gateway が標準形式で readyz を実装済みであり、参考実装として利用できる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|--------------------|
| 案 A | 各サービスが独自実装を維持する | 監視自動化が困難な現状を解消できない |
| 案 B | Kubernetes 標準の liveness/readiness probe のみに依存する | ボディの詳細情報が得られず、根本原因の特定が難しい |
| 案 C | OpenTelemetry Health Check Protocol を採用する | 既存実装との乖離が大きく、移行コストが高い |

## 参考

- [ADR-0001: テンプレート](0001-template.md)
- [k1s0-health ライブラリ](../../../regions/system/library/go/k1s0-health/)
- graphql-gateway readyz 実装（HIGH-001 対応）
- 外部技術監査 2026-04-01 MED-001 指摘

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-02 | 初版作成（外部監査 MED-001 対応） | @k1s0-team |
