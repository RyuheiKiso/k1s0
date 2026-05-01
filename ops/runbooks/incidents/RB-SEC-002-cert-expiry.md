---
runbook_id: RB-SEC-002
title: 証明書期限切れ対応（cert-manager / SPIRE / Strimzi 共通）
category: SEC
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: CertExpiringSoon / CertExpired
fmea_id: FMEA-010
estimated_recovery: 暫定 30 分 / 恒久 2 時間
last_updated: 2026-05-02
---

# RB-SEC-002: 証明書期限切れ対応（cert-manager / SPIRE / Strimzi 共通）

本 Runbook は cert-manager・SPIRE・Strimzi が発行する証明書の自動更新失敗および期限切れに対する対応を定める。証明書期限切れは API 接続不可・サービス間通信停止を引き起こすため SEV1。NFR-A-CONT-001 / NFR-E-ENC-001 / FMEA-010 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole + `cert-manager.io` の Certificate edit 権限を保持。
- 必要ツール: `kubectl` / `openssl` / `jq` / `cmctl`（cert-manager CLI、推奨）。
- kubectl context が `k1s0-prod`。
- cert-manager Operator（`kubectl get deploy -n cert-manager cert-manager` が `1/1`）と SPIRE Server（`kubectl get deploy -n spire-system spire-server` が `1/1`）が起動済み。
- ClusterIssuer `k1s0-internal-ca` が存在し、`Status: Ready` であること。

## 2. 対象事象

- Alertmanager `CertExpiringSoon` 発火（`certmanager_certificate_expiration_timestamp_seconds - time() < 259200`、72h 以内）、または
- Alertmanager `CertExpired` 発火（既に期限切れ）、または
- SPIRE Agent SVID の残余有効期間が 1h を切る（`spire_agent_svid_renewal_seconds < 3600`）、または
- Istio Ambient で 503 ハンドシェイク失敗率が 5% 超。

検知シグナル:

```promql
# cert-manager が管理する証明書の残余有効期間（72 時間以内でアラート）
certmanager_certificate_expiration_timestamp_seconds{namespace=~".*"} - time() < 259200

# SPIRE Agent の SVID 残余有効期間（1 時間以内で警告）
spire_agent_svid_renewal_seconds{namespace="spire-system"} < 3600

# Istio Ambient での TLS ハンドシェイク失敗率
sum(rate(istio_requests_total{response_code="503", namespace=~"k1s0.*"}[5m])) > 0.05
```

ダッシュボード: **Grafana → k1s0 Certificate Health**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#incident-mtls`。

## 3. 初動手順（5 分以内）

```bash
# cert-manager の Certificate リソース一覧と有効期限を確認
kubectl get certificate -A -o custom-columns="NS:.metadata.namespace,NAME:.metadata.name,READY:.status.conditions[0].status,EXPIRY:.status.notAfter"
```

```bash
# SPIRE Agent の SVID 状態
kubectl exec -n spire-system ds/spire-agent -- \
  /opt/spire/bin/spire-agent api fetch x509 | grep -i "expiry\|spiffe"
```

```bash
# SPIRE Server のログで SVID 発行エラーを確認
kubectl logs -n spire-system deploy/spire-server --tail=100 | grep -i "error\|expired\|denied"
```

```bash
# tier1 facade の TLS エラー
kubectl logs -n k1s0 deploy/tier1-facade --tail=100 | grep -i "certificate\|tls\|handshake\|x509"
```

```bash
# Istio Ambient ztunnel / waypoint の cert キャッシュ
kubectl exec -n k1s0 deploy/tier1-facade -c istio-proxy -- \
  pilot-agent request GET /certs | jq '.certificates[] | {uri: .identity, expiry: .expiration_time}'
```

ステークホルダー通知: SEV1 確定で `oncall/escalation.md` を起動、Slack `#status` に「mTLS 証明書期限切れ、サービス間通信影響あり」を投稿。

## 4. 原因特定手順

```bash
# cert-manager Operator ログ
kubectl logs -n cert-manager deploy/cert-manager --tail=200 | grep -i "renew\|error\|expired"

# ClusterIssuer の状態
kubectl describe clusterissuer k1s0-internal-ca

# CA 証明書の有効期限
kubectl get secret k1s0-internal-ca-key-pair -n cert-manager -o jsonpath='{.data.tls\.crt}' \
  | base64 -d | openssl x509 -noout -dates
```

よくある原因:

1. **cert-manager `renewBefore` の設定不足**: `renewBefore: 720h` が設定されていない Certificate は失効直前に更新される。`kubectl describe certificate <name>` で `renewBefore` を確認。
2. **SPIRE upstream authority（cert-manager CA）の証明書期限切れ**: `k1s0-internal-ca-key-pair` Secret の CA 証明書が失効していると SPIRE の SVID 発行全停止。
3. **SPIFFE ID の trust domain 不一致**: SPIRE Server の `trustDomain: k1s0.example.com` と tier1 gRPC クライアントの期待値が一致しない。`infra/security/spire/values.yaml` を確認。
4. **Istio Ambient の waypoint proxy が古い証明書をキャッシュ**: waypoint proxy Pod を再起動することで解消する。
5. **ClockSkew**: Node の NTP がずれて証明書の NotBefore チェックが失敗。`timedatectl status` でノードの時刻同期を確認。NTP ずれは `RB-OPS-003` を並行起動。

## 5. 復旧手順

cert-manager 管理の証明書を手動更新:

```bash
# Certificate を削除すると cert-manager が即時再発行する
kubectl delete certificate <cert-name> -n <namespace>
# または annotations で更新をトリガー
kubectl annotate certificate <cert-name> -n <namespace> \
  cert-manager.io/issue-temporary-certificate="true" --overwrite
```

CA 自体が期限切れの場合（k1s0-internal-ca）:

```bash
# CA Secret を削除すると cert-manager が ClusterIssuer 設定に基づき再発行
kubectl delete secret k1s0-internal-ca-key-pair -n cert-manager
# 再発行確認
kubectl get clusterissuer k1s0-internal-ca -o yaml | grep -A5 status
```

SPIRE Server の CA ローテーション:

```bash
# SPIRE Server のバンドルを更新（upstream authority = cert-manager k1s0-internal-ca）
kubectl rollout restart deployment/spire-server -n spire-system
kubectl rollout status deployment/spire-server -n spire-system

# Agent を再起動して新 SVID を取得させる
kubectl rollout restart daemonset/spire-agent -n spire-system
```

SPIFFE ID 不一致の場合（Pod が別 namespace に移動した等）:

```bash
# SPIRE Controller Manager が自動で SpiffeID CR を作成しているか確認
kubectl get spiffeid -A

# 手動で SVID を強制再取得
kubectl delete pod -n <namespace> <pod-name>
```

Kafka の TLS listener 証明書が期限切れの場合:

```bash
# Strimzi が管理する Kafka cluster-ca を強制ローテーション
kubectl annotate secret k1s0-kafka-cluster-ca-cert \
  strimzi.io/force-renew=true -n kafka --overwrite
# Strimzi Operator が自動で broker を rolling restart する
kubectl get pods -n kafka -w
```

## 6. 検証手順

復旧完了の判定基準:

- 全 Certificate が `READY=True` かつ `notAfter` が `now() + renewBefore` 以上（`kubectl get certificate -A` で確認）。
- `certmanager_certificate_expiration_timestamp_seconds - time()` が全 namespace で 720h 超。
- SPIRE Agent SVID 残余時間が 24h 超（`spire_agent_svid_renewal_seconds > 86400`）。
- Istio Ambient の 503 率が 1% 未満に戻り、5 分間継続。
- tier1 facade の `/healthz` が 200、直近 10 分の Loki クエリ `{namespace="k1s0"} |= "x509" |= "ERROR"` が 0 件。
- Kafka broker 間 mTLS 接続が成功（`kafka_server_replicamanager_underreplicatedpartitions == 0`）。

## 7. 予防策

- ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-RB-SEC-002.md`）。
- cert-manager の `renewBefore` が全 Certificate に設定されているか PR で一括点検（`infra/security/cert-manager/`）。
- SPIRE CA / cert-manager CA の有効期限を Grafana ダッシュボードに追加（現在未可視の場合）。
- `certmanager_certificate_expiration_timestamp_seconds` の alert ルールを 72h → 168h（7 日）に引き上げることを検討。
- NFR-A-REC-002 の MTTR ログを更新（目標: 暫定 30 分以内、恒久 2 時間以内）。
- 月次 Chaos Drill 対象に「証明書 1h で期限切れ」シナリオを追加（`ops/chaos/experiments/cert-expiry.yaml`）。

## 8. 関連 Runbook

- 関連設計書: `infra/security/spire/values.yaml`、`infra/security/cert-manager/cluster-issuer.yaml`
- 関連 ADR: [ADR-SEC-001](../../../docs/02_構想設計/adr/ADR-SEC-001-keycloak.md)、[ADR-SEC-003](../../../docs/02_構想設計/adr/ADR-SEC-003-spire.md)
- 関連 NFR: [NFR-E-ENC-001](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md)
- 関連 FMEA: [FMEA-010](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`RB-DB-002-postgres-primary-failover.md`](RB-DB-002-postgres-primary-failover.md) — SPIRE が CNPG に依存
  - [`RB-MSG-001-kafka-broker-failover.md`](RB-MSG-001-kafka-broker-failover.md) — Kafka mTLS 期限切れと連鎖
  - [`RB-OPS-003-ntp-drift.md`](RB-OPS-003-ntp-drift.md) — ClockSkew 起因の場合
