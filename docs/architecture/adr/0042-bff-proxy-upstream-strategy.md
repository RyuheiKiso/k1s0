# ADR-0042: BFF-Proxy upstream 拡張戦略

## ステータス

承認済み

## コンテキスト

BFF-Proxy（`regions/system/server/go/bff-proxy`）は React SPA および Flutter アプリケーションの
Backend-for-Frontend として機能し、OAuth2/OIDC 認証フローとリバースプロキシを担う。

外部技術監査（H-05）において、BFF-Proxy の upstream（上流サービス）への
プロキシルーティングの拡張戦略が明文化されていないという指摘を受けた。

### 現行の upstream 設定

Phase 1 の BFF-Proxy は以下のフローに絞って実装されている：

- `/auth/*`: OAuth2/OIDC 認証フロー（Login/Callback/Session/Exchange/Logout）
- `/api/*path`: upstream へのリバースプロキシ（Kong API Gateway 経由）

リバースプロキシ先は `config.yaml` の `upstream.base_url` 一点に集約されており、
パス別の upstream 振り分けは実装されていない。

### 拡張要望の背景

今後以下のユースケースへの対応が想定されている：

1. **複数 upstream への振り分け**: System Tier サービスと Service Tier サービスで
   異なる Kong ルートまたは直接サービスへの転送
2. **WebSocket サポート**: リアルタイム通知（notification-server）への ws/wss 転送
3. **ストリーミング**: ファイルアップロード・ダウンロードの大容量通信
4. **Service Tier への直接ルーティング**: Kong を介さない内部通信の最適化

## 決定

**Phase 1（2026-Q1）では auth フローのみをフルサポートし、プロキシルーティングは単一 upstream に集約する。**
**Phase 2（2026-Q2）でプロキシルーティングを拡張し、複数 upstream への振り分けを実装する。**

### Phase 1 の範囲（現行実装）

```yaml
# config.yaml（Phase 1 設定例）
upstream:
  base_url: "http://kong:8000"  # 全プロキシリクエストを Kong に集約
auth:
  discovery_url: "http://keycloak:8080/realms/k1s0/.well-known/openid-configuration"
```

- 認証フロー（PKCE、セッション管理、トークンリフレッシュ、モバイル交換コード）を完全実装
- `/api/*path` はすべて `upstream.base_url` + パスに転送
- WebSocket、ストリーミングは対象外

### Phase 2 の範囲（2026-Q2 実装予定）

```yaml
# config.yaml（Phase 2 設定例、予定）
upstreams:
  - path_prefix: "/api/v1/system"
    base_url: "http://kong-system:8000"
  - path_prefix: "/api/v1/service"
    base_url: "http://kong-service:8000"
  - path_prefix: "/ws/"
    base_url: "http://notification:8081"
    websocket: true
```

- パスプレフィックスによる upstream 振り分け
- WebSocket プロキシ（`golang.org/x/net/websocket` または `gorilla/websocket`）
- upstream ヘルスチェックとサーキットブレーカー統合

## 理由

### Phase 1 で絞り込む根拠

1. **コア機能の安定化優先**: 認証フロー（PKCE/セッション管理）の品質が最重要であり、
   プロキシ拡張との同時開発は品質リスクを高める
2. **Kong による集約**: 現在すべてのサービスルーティングは Kong API Gateway で管理されており、
   BFF 側での振り分けは Kong との責務が重複する
3. **段階的複雑性管理**: upstream 振り分けロジックの複雑性は段階的に導入することで
   テストカバレッジを維持できる

### Phase 2 のタイミング

2026-Q2 は以下の前提条件が揃うタイミングである：
- Phase 1 の認証フローの本番稼働実績（約1四半期）
- Service Tier の WebSocket 要件の確定
- 複数 upstream の設計仕様レビュー完了

## 影響

**ポジティブな影響**:

- Phase 1 は実装範囲が明確で、認証品質の集中管理が可能
- Phase 2 のロードマップが明示されることでチームの見通しが立つ

**ネガティブな影響・トレードオフ**:

- Phase 2 まで WebSocket やストリーミングが BFF-Proxy を経由できない
- 暫定的に一部クライアントが Kong に直接アクセスする必要が生じる可能性がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: Phase 1 で全機能実装 | プロキシ拡張も同時実装 | 認証品質への集中が妨げられる |
| 案 B: Envoy/Nginx に委任 | プロキシ機能を外部ツールに分離 | 認証情報の共有が複雑になる |
| 案 C: Phase 2 をスキップ | 単一 upstream のまま維持 | WebSocket/ストリーミング要件に対応不可 |

## 参考

- [docs/servers/system/bff-proxy/implementation.md](../../servers/system/bff-proxy/implementation.md)
- `regions/system/server/go/bff-proxy/internal/upstream/reverse_proxy.go`
- 外部技術監査報告書 H-05: "BFF-Proxy upstream 拡張戦略の明示を求める"
- [ADR-0043: Service Tier GraphQL 統合方針](0043-service-tier-graphql-integration.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-28 | 初版作成（H-05 監査対応） | 監査対応チーム |
| 2026-03-29 | Phase 1 完了確認。`config.docker.yaml` の `upstream.base_url: "http://auth-rust:8080"` は BFF-Proxy の開発環境設定。`config.yaml` では `http://kong:8000` を参照するため設計通り。Phase 2 は 2026-Q2 に実施予定 | 監査対応チーム |
