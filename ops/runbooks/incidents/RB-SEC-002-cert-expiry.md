# mTLS 証明書期限切れ / SPIFFE ID 不一致 Runbook

> **alert_id**: tier1.mtls.certificate.expiry-imminent
> **severity**: SEV2
> **owner**: tier1-platform-team
> **estimated_mttr**: 30m
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

**Mimir / Grafana** で以下を確認する。

PromQL（Mimir）:

```promql
# cert-manager が管理する証明書の残余有効期間（72 時間以内でアラート）
certmanager_certificate_expiration_timestamp_seconds{namespace=~".*"} - time() < 259200

# SPIRE Agent の SVID 残余有効期間（1 時間以内で警告）
spire_agent_svid_renewal_seconds{namespace="spire-system"} < 3600

# Istio Ambient での TLS ハンドシェイク失敗率
sum(rate(istio_requests_total{response_code="503", namespace=~"k1s0.*"}[5m])) > 0.05
```

ダッシュボード: **Grafana → k1s0 Certificate Health**。

alert チャンネル: PagerDuty `tier1-platform-team` → Slack `#incident-mtls`。

## 2. 初動 (Immediate Action, 〜15 分)

- [ ] cert-manager の Certificate リソース一覧と有効期限を確認する

  ```bash
  kubectl get certificate -A -o custom-columns="NS:.metadata.namespace,NAME:.metadata.name,READY:.status.conditions[0].status,EXPIRY:.status.notAfter"
  ```

- [ ] SPIRE Agent の SVID 状態を確認する

  ```bash
  kubectl exec -n spire-system ds/spire-agent -- \
    /opt/spire/bin/spire-agent api fetch x509 | grep -i "expiry\|spiffe"
  ```

- [ ] SPIRE Server のログで SVID 発行エラーを確認する

  ```bash
  kubectl logs -n spire-system deploy/spire-server --tail=100 | grep -i "error\|expired\|denied"
  ```

- [ ] tier1 間通信（gRPC）で TLS エラーが発生しているか確認する

  ```bash
  kubectl logs -n k1s0 deploy/tier1-facade --tail=100 | grep -i "certificate\|tls\|handshake\|x509"
  ```

- [ ] Istio の proxy ステータスを確認する

  ```bash
  kubectl exec -n k1s0 deploy/tier1-facade -c istio-proxy -- \
    pilot-agent request GET /certs | jq '.certificates[] | {uri: .identity, expiry: .expiration_time}'
  ```

## 3. 復旧 (Recovery, 〜60 分)

### cert-manager 管理の証明書を手動更新する

```bash
# Certificate を削除すると cert-manager が即時再発行する
kubectl delete certificate <cert-name> -n <namespace>
# または annotations で更新をトリガー
kubectl annotate certificate <cert-name> -n <namespace> \
  cert-manager.io/issue-temporary-certificate="true" --overwrite
```

### ClusterIssuer が `k1s0-internal-ca` の場合に CA Secret を確認する

```bash
kubectl get secret k1s0-internal-ca-key-pair -n cert-manager -o yaml | \
  grep tls.crt | awk '{print $2}' | base64 -d | openssl x509 -noout -dates
```

### SPIRE Server の CA ローテーションを実行する

```bash
# SPIRE Server のバンドルを更新（upstream authority = cert-manager k1s0-internal-ca）
kubectl rollout restart deployment/spire-server -n spire-system
kubectl rollout status deployment/spire-server -n spire-system

# Agent を再起動して新 SVID を取得させる
kubectl rollout restart daemonset/spire-agent -n spire-system
```

### SPIFFE ID 不一致の場合（pod が別 namespace に移動した等）

```bash
# SPIRE Controller Manager が自動で SpiffeID CR を作成しているか確認
kubectl get spiffeid -A

# 手動で SVID を強制再取得
kubectl delete pod -n <namespace> <pod-name>
```

### Kafka の TLS listener 証明書が期限切れの場合

```bash
# Strimzi が管理する Kafka cluster-ca を強制ローテーション
kubectl annotate secret k1s0-kafka-cluster-ca-cert \
  strimzi.io/force-renew=true -n kafka --overwrite
# Strimzi Operator が自動で broker を rolling restart する
kubectl get pods -n kafka -w
```

## 4. 原因調査 (Root Cause Analysis)

**よくある原因**:

1. **cert-manager `renewBefore` の設定不足**: `renewBefore: 720h` が設定されていない Certificate は失効直前に更新される。`kubectl describe certificate <name>` で `renewBefore` を確認。
2. **SPIRE upstream authority（cert-manager CA）の証明書が期限切れ**: `k1s0-internal-ca-key-pair` Secret の証明書が失効していると SPIRE の SVID 発行全停止。CA の有効期限を `openssl x509 -noout -dates` で確認。
3. **SPIFFE ID の trust domain 不一致**: SPIRE Server の `trustDomain: k1s0.example.com` と tier1 gRPC クライアントの期待値が合わない。`infra/security/spire/values.yaml` の `trustDomain` を確認。
4. **Istio Ambient の waypoint proxy が古い証明書をキャッシュ**: waypoint proxy Pod を再起動することで解消する。
5. **ClockSkew**: Node の NTP がずれて証明書の NotBefore チェックが失敗。`timedatectl status` でノードの時刻同期を確認。

## 5. 事後処理 (Post-incident)

- [ ] ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-mtls-cert-expiry.md`）
- [ ] cert-manager の `renewBefore` が全 Certificate に設定されているか確認・修正
- [ ] SPIRE CA / cert-manager CA の有効期限を Grafana ダッシュボードに追加（現在未可視の場合）
- [ ] `certmanager_certificate_expiration_timestamp_seconds` の alert ルールを 72h → 168h に引き上げることを検討
- [ ] NFR-A-REC-002 の MTTR ログを更新

## 関連

- 関連設計書: `infra/security/spire/values.yaml`、`infra/security/cert-manager/cluster-issuer.yaml`
- 関連 ADR: `docs/02_構想設計/adr/ADR-SEC-001`（cert-manager）、`docs/02_構想設計/adr/ADR-SEC-003`（SPIRE）
- 関連 Runbook: `ops/runbooks/incidents/postgresql-primary-down.md`（SPIRE が CNPG を使用）
