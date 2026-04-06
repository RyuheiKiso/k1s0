# ADR-0112: 外部技術監査対応（2026-04-06）

## ステータス

承認済み

## コンテキスト

2026-04-06 に外部技術監査チームから以下の指摘を受けた:

- **HIGH**: GO-001〜003（bff-proxy セキュリティ強化）、Rust/インフラ系多数
- **MEDIUM**: RUST-001/002（notification テナント分離、navigation readyz）、GO-001（bff-proxy テナント分離）、FE-003（Flutter 証明書ピンニング）、INFRA-002（アラート閾値ドキュメント）

本 ADR は 2026-04-06 実施分の外部技術監査への対応内容を記録する。

## 決定

以下の修正を全件実施する。既存の動作を壊さないことを最優先とし、
最小変更・日本語コメント付きで実装する。

### HIGH 対応

#### HIGH-GO-001: bff-proxy /metrics を内部ポートに移動

**問題**: `/metrics` エンドポイントが公開ルーターに登録されており、
認証なしで内部 Prometheus メトリクスが取得可能な状態だった。

**対応**:
- `ServerConfig` に `InternalPort`（デフォルト 9090）フィールドを追加
- `/metrics` を公開ルーター（8080）から除去
- 内部専用サーバー（127.0.0.1:9090）を goroutine で起動
- graceful shutdown 対応

**変更ファイル**:
- `regions/system/server/go/bff-proxy/internal/config/config.go`
- `regions/system/server/go/bff-proxy/cmd/server/main.go`

#### HIGH-GO-002: ReverseProxy に ErrorHandler + ModifyResponse 追加

**問題**: デフォルトの ErrorHandler がバックエンドエラー詳細をそのままクライアントに返す可能性があり、
内部ネットワーク情報が漏洩するリスクがあった。また `Server` ヘッダーでバックエンドサーバー種別が露出していた。

**対応**:
- `ErrorHandler`: バックエンドエラーを隠蔽し 502 のみ返す（詳細はサーバーログに記録）
- `ModifyResponse`: `X-Internal-*` プレフィックスヘッダーと `Server` ヘッダーを除去

**変更ファイル**:
- `regions/system/server/go/bff-proxy/internal/upstream/reverse_proxy.go`

#### HIGH-GO-003: トークンエンドポイントエラーボディをログから除外

**問題**: トークンエンドポイントのエラーレスポンスボディ（クライアントシークレット等を含む可能性）が
エラーメッセージに含まれており、ログへの機密情報漏洩リスクがあった。

**対応**:
- `tokenRequest` のエラーメッセージからボディを除外し、ステータスコードのみ返す

**変更ファイル**:
- `regions/system/server/go/bff-proxy/internal/oauth/client.go`

### MEDIUM 対応

#### MEDIUM-RUST-001: notification の "system" テナントハードコード修正

**問題**: `create_channel` ハンドラーでテナント ID が `"system"` にハードコードされており、
マルチテナント分離が実現できていなかった。

**対応**:
- `ExtractClaims` メソッドに加え `ExtractFullClaims` を追加（`IDTokenClaims` 構造体で tenant_id を含む全クレームを返す）
- `create_channel` ハンドラーで `Extension<Claims>` から `tenant_id()` を取得
- 未認証環境では "system" にフォールバック

**変更ファイル**:
- `regions/system/server/rust/notification/src/adapter/handler/notification_handler.rs`

#### MEDIUM-RUST-002: navigation readyz に実チェック追加

**問題**: `readyz` エンドポイントが常に `"status": "healthy"` を返しており、
実際のサービス準備状態を反映していなかった。

**対応**:
- navigation サービスは DB を持たないため、`navigation.yaml` のロード可否をヘルスチェックとして使用
- `GetNavigationUseCase.check_config_loadable()` メソッドを追加
- `readyz` がファイルロード失敗時に `503 SERVICE_UNAVAILABLE` を返すよう変更

**変更ファイル**:
- `regions/system/server/rust/navigation/src/adapter/handler/health.rs`
- `regions/system/server/rust/navigation/src/usecase/get_navigation.rs`

#### MEDIUM-GO-001: bff-proxy テナント分離実装

**問題**: bff-proxy が上流 API に `X-Tenant-ID` ヘッダーを転送しておらず、
マイクロサービスがテナントを識別できなかった。

**対応**:
- `IDTokenClaims` 構造体に `TenantID` フィールドを追加
- Keycloak の `tenant_id` カスタムクレームをセッションに格納
- `ProxyHandler` がセッションの `TenantID` を `X-Tenant-ID` ヘッダーとして上流に転送

**変更ファイル**:
- `regions/system/server/go/bff-proxy/internal/oauth/client.go`
- `regions/system/server/go/bff-proxy/internal/usecase/auth_usecase.go`
- `regions/system/server/go/bff-proxy/internal/session/types.go`
- `regions/system/server/go/bff-proxy/internal/handler/proxy_handler.go`

#### MEDIUM-FE-003: Flutter 証明書ピンニング ADR 作成

実装方針を ADR-0111 として記録。

#### MEDIUM-INFRA-002: alerting_rules ドキュメント化

**問題**: Prometheus アラートルールの閾値根拠が不明確で、
本番環境への移行時に適切な閾値調整が行われないリスクがあった。

**対応**:
- 本番環境移行チェックリストをコメントとして追記
- 各閾値の調整ガイドラインと算出根拠を明記

**変更ファイル**:
- `infra/docker/prometheus/alerting_rules.yaml`

## 理由

外部技術監査は年次セキュリティレビューの一環として実施されており、
指摘事項は全件対応を原則としている。

本 ADR では優先度 HIGH/MEDIUM の全件について、
既存の動作・インターフェースを破壊しない範囲で最小限の変更を適用した。

## 影響

**ポジティブな影響**:

- bff-proxy のセキュリティ強度向上（メトリクス内部化・エラー隠蔽・機密情報ログ除外）
- notification/navigation サービスのテナント分離強化
- Prometheus アラートルールの可読性・保守性向上

**ネガティブな影響・トレードオフ**:

- bff-proxy の `ServerConfig` に `InternalPort` フィールドが追加される。
  既存の設定ファイル（config.yaml）に `internal_port` が未設定の場合はデフォルト 9090 を使用する。
- bff-proxy の Prometheus スクレイプ設定を 8080 → 9090 に変更する必要がある。
- notification の `create_channel` が `Extension<Claims>` を受け取るようになるため、
  axum の認証ミドルウェアが Claims を Extension として挿入していない場合は `None` になる。
  認証有効時は Claims から取得、無効時（開発環境）は "system" にフォールバックするため後方互換性を維持する。

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| /metrics に認証を追加 | Bearer トークンで /metrics を保護 | Prometheus の設定変更が必要。内部ポート分離の方が単純 |
| ErrorHandler をログのみ | エラーを 502 で返さずパススルー | クライアントへの内部情報露出リスクが残存する |
| notification に JWT 必須化 | 認証なし環境でエラーを返す | 開発環境の破壊なしに対応可能な方法を優先した |
| navigation DB チェック追加 | DB を追加して実チェックを実施 | アーキテクチャの複雑化を避け、YAML ロードチェックで十分 |

## 参考

- [ADR-0110: Outbox BYPASSRLS Publisher Role](0110-outbox-bypassrls-publisher-role.md)
- [ADR-0109: Vault EventMonitor Maintenance Tenant Isolation](0109-vault-eventmonitor-maintenance-tenant-isolation.md)
- [ADR-0111: Flutter Certificate Pinning](0111-flutter-certificate-pinning.md)
- 外部技術監査レポート 2026-04-06

## 対応一覧

| 問題 ID | 概要 | 変更ファイル |
|---------|------|------------|
| HIGH-GO-001 | /metrics を内部ポートに移動 | config/config.go, main.go |
| HIGH-GO-002 | ErrorHandler + ModifyResponse 追加 | upstream/reverse_proxy.go |
| HIGH-GO-003 | トークンエラーボディをログから除外 | oauth/client.go |
| MEDIUM-RUST-001 | notification テナント分離 | notification_handler.rs |
| MEDIUM-RUST-002 | navigation readyz 実チェック | health.rs, get_navigation.rs |
| MEDIUM-GO-001 | bff-proxy X-Tenant-ID 転送 | client.go, auth_usecase.go, types.go, proxy_handler.go |
| MEDIUM-FE-003 | Flutter 証明書ピンニング ADR | ADR-0111 |
| MEDIUM-INFRA-002 | alerting_rules ドキュメント化 | alerting_rules.yaml |

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-06 | 初版作成（外部技術監査 2026-04-06 対応） | @kiso-ryuhei |
