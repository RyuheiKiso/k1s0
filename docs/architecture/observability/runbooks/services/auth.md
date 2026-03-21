# アラート名: auth サービス障害

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical |
| **影響範囲** | 全サービス（認証が通らなくなるため全体に波及） |
| **通知チャネル** | Microsoft Teams #alert-critical |
| **対応 SLA** | SEV1: 15分以内 / SEV2: 30分以内 |

## アラート発火条件

```promql
# サービスダウン
up{job="auth"} == 0

# エラーレート高騰
sum(rate(http_requests_total{job="auth",status=~"5.."}[5m]))
  / sum(rate(http_requests_total{job="auth"}[5m])) > 0.05

# JWT 検証失敗レート高騰
rate(auth_jwt_validation_failures_total[5m]) > 10
```

## 初動対応（5分以内）

### 1. 状況確認

```bash
# Pod の状態確認
kubectl get pods -n k1s0-system -l app=auth

# 直近のエラーログ確認
# Grafana → Explore → Loki: {app="auth", level="error"} | 直近10分

# Keycloak の疎通確認（auth は Keycloak に依存している）
kubectl exec -n k1s0-system deploy/auth -- \
  wget -qO- http://keycloak.k1s0-system.svc.cluster.local:8080/health

# Redis セッションストアの状態確認
kubectl exec -n k1s0-system deploy/auth -- \
  redis-cli -h redis-session.k1s0-system.svc.cluster.local ping
```

### 2. Grafana ダッシュボード確認

- [auth サービスダッシュボード](http://grafana.k1s0.internal/d/k1s0-auth/auth-dashboard)
- [SLO ダッシュボード](http://grafana.k1s0.internal/d/k1s0-slo/slo-dashboard)

### 3. 即時判断

- [ ] auth Pod が全て落ちている → SEV1（即時エスカレーション）
- [ ] Keycloak が応答しない → SEV1（Keycloak 障害）
- [ ] JWT 検証失敗が急増 → SEV2（証明書期限切れまたは Keycloak 設定変更）
- [ ] 一部 Pod のみ障害 → SEV2（Pod 再起動で対応試みる）

## 詳細調査

### よくある原因

1. **Keycloak への接続失敗**: Keycloak サービスのダウン、ネットワーク障害
2. **JWT 公開鍵の期限切れ**: Keycloak の realm 公開鍵ローテーション後に auth が古いキャッシュを参照
3. **Redis セッションストア障害**: セッション検証が失敗し全認証が拒否される
4. **OOM Kill**: メモリ不足による Pod の強制終了

### 調査コマンド

```bash
# Pod の詳細情報（OOM Kill の確認）
kubectl describe pod -n k1s0-system -l app=auth

# Keycloak への接続テスト
kubectl exec -n k1s0-system deploy/auth -- \
  curl -s http://keycloak.k1s0-system.svc.cluster.local:8080/realms/k1s0/.well-known/openid-configuration

# JWT 公開鍵の確認
kubectl exec -n k1s0-system deploy/auth -- \
  curl -s http://keycloak.k1s0-system.svc.cluster.local:8080/realms/k1s0/protocol/openid-connect/certs

# auth Pod のメモリ使用量
kubectl top pods -n k1s0-system -l app=auth
```

## 復旧手順

### パターン A: Pod 障害（CrashLoopBackOff）の場合

```bash
# Pod を再起動する
kubectl rollout restart deployment/auth -n k1s0-system
kubectl rollout status deployment/auth -n k1s0-system
```

### パターン B: Keycloak 接続失敗の場合

```bash
# Keycloak の状態を確認する
kubectl get pods -n k1s0-system -l app=keycloak

# Keycloak が落ちている場合は再起動する
kubectl rollout restart deployment/keycloak -n k1s0-system

# Keycloak 復旧後、auth も再起動して接続を再確立する
kubectl rollout restart deployment/auth -n k1s0-system
```

### パターン C: JWT 公開鍵キャッシュ不整合の場合

```bash
# auth を再起動して公開鍵キャッシュをクリアする
kubectl rollout restart deployment/auth -n k1s0-system

# 環境変数 KEYCLOAK_PUBLIC_KEY_REFRESH_INTERVAL が設定されているか確認する
kubectl get configmap auth-config -n k1s0-system -o yaml
```

### パターン D: Redis セッションストア障害の場合

[service-down Runbook](../common/service-down.md) を参照して Redis を復旧する。

## エスカレーション基準

以下の条件に該当する場合は上位にエスカレーションする:

- [ ] auth Pod が全台 CrashLoopBackOff から回復しない
- [ ] Keycloak が復旧しない
- [ ] 15分以内に復旧の見通しが立たない
- [ ] データ損失が疑われる

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md) のエスカレーションパスを参照。

## 根本原因分析のポイント

- Keycloak の公開鍵ローテーション周期と auth のキャッシュTTLの設定を確認する
- Redis セッションストアの接続プール設定と最大接続数を確認する
- OOM Kill の場合はメモリ requests/limits の見直しを検討する

## 関連ドキュメント

- [デプロイ手順書](../../../infrastructure/kubernetes/デプロイ手順書.md)
- [可観測性設計](../../可観測性設計.md)
- [インシデント管理設計](../インシデント管理設計.md)
