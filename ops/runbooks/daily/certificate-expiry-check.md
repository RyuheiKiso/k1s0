---
runbook_id: DAILY-003
title: 証明書期限確認（cert-manager / SPIRE / Strimzi）
category: OPS
severity: 該当なし（定常運用）
owner: 当番 SRE
automation: manual（CronJob 化済み: cert-expiry-watcher、本 Runbook は手動レビュー用）
alertmanager_rule: 該当なし（失敗時は CertExpiringSoon）
estimated_recovery: 5 分
last_updated: 2026-05-02
---

# DAILY-003: 証明書期限確認

毎営業日の朝に cert-manager / SPIRE / Strimzi の証明書残余有効期間を確認し、7 日以内に切れる証明書を早期検知する。

## 1. 前提条件

- 実行者は SRE オンコール当番。
- 必要ツール: `kubectl` / `cmctl` / `openssl`。
- kubectl context が `k1s0-prod`。

## 2. 対象事象

毎営業日 09:20 JST 実施（[`backup-verification.md`](backup-verification.md) 直後）。

## 3. 初動手順（5 分以内）

```bash
# cert-manager 管理の Certificate を残余有効期間でソート
kubectl get certificate -A -o json | jq -r '
  .items[] | "\(.status.notAfter)\t\(.metadata.namespace)\t\(.metadata.name)"' \
  | sort | head -20
```

```bash
# SPIRE Server / Agent の SVID 残余時間
kubectl exec -n spire-system deploy/spire-server -- \
  /opt/spire/bin/spire-server bundle list -format=pem | openssl x509 -noout -dates
```

```bash
# Strimzi Kafka の cluster-ca 残余時間
kubectl get secret k1s0-kafka-cluster-ca-cert -n kafka -o jsonpath='{.data.ca\.crt}' \
  | base64 -d | openssl x509 -noout -dates
```

## 4. 原因特定手順

7 日以内に切れる証明書がある場合:
1. cert-manager auto-renew が動作しているか確認（`renewBefore` 設定）。
2. 動作していない場合は [`../incidents/RB-SEC-002-cert-expiry.md`](../incidents/RB-SEC-002-cert-expiry.md) を起動。

## 5. 復旧手順

該当 Runbook（`RB-SEC-002`）起動。本 Runbook 自体は確認専用。

## 6. 検証手順

- 全 Certificate の残余有効期間が `renewBefore`（720h = 30 日）超。
- SPIRE bundle が 24h 以内に更新済み。
- Kafka cluster-ca が 30 日以内に再生成予定（Strimzi 自動）。

## 7. 予防策

- 7 日以内警告 + 1 日以内アラート の二段階監視ルールを Mimir に整備済み。
- 月次レビューで `renewBefore` 設定の正しさを確認（全 Certificate に設定されているか）。

## 8. 関連 Runbook

- [`../incidents/RB-SEC-002-cert-expiry.md`](../incidents/RB-SEC-002-cert-expiry.md)
