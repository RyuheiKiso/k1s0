---
runbook_id: RB-NET-002
title: Envoy Gateway DoS / 異常リクエストレート対応
category: NET
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: GatewayRequestRateAbnormal / GatewayErrorRateHigh
fmea_id: 間接対応
estimated_recovery: 暫定 15 分 / 恒久 1 時間
last_invoked: 2026-05-02
last_updated: 2026-05-02
---

# RB-NET-002: Envoy Gateway DoS / 異常リクエストレート対応

本 Runbook は Envoy Gateway（k1s0 の北向き入口）が DoS / DDoS 攻撃 / 異常リクエストレートを受けた時の対応を定める。
正規利用者も影響を受けるため SEV1。NFR-E-AC-006 / NFR-A-CONT-001 に対応する。

## 1. 前提条件

- 実行者は `security-sre` ClusterRole + Envoy Gateway SecurityPolicy の編集権限を保持。
- 必要ツール: `kubectl` / `logcli` / Cloudflare API（DDoS 対策有効化用、契約済の場合）。
- kubectl context が `k1s0-prod`。
- Envoy Gateway namespace は `k1s0-ingress`、HTTPRoute 名は `k1s0-public`。
- Cloudflare DDoS 防御サービスとの契約状況を [`oncall/contacts.md`](../../oncall/contacts.md) §インフラ・ベンダ で確認可能。

## 2. 対象事象

- Alertmanager `GatewayRequestRateAbnormal` 発火（RPS が直近 24h 平均の 10 倍を超える）、または
- `GatewayErrorRateHigh`（5xx エラー率が 5% を 5 分継続）、または
- 利用者からの「サービスが応答しない」報告。

検知シグナル:

```promql
# Envoy Gateway リクエストレート（RPS）
sum(rate(envoy_http_downstream_rq_total{namespace="k1s0-ingress"}[1m]))

# 5xx エラー率
sum(rate(envoy_http_downstream_rq_total{response_code=~"5.."}[5m])) /
sum(rate(envoy_http_downstream_rq_total[5m]))

# Connection 数（同時接続）
envoy_http_downstream_cx_active{namespace="k1s0-ingress"}
```

ダッシュボード: **Grafana → k1s0 Envoy Gateway**。
通知経路: PagerDuty `security-sre` → Slack `#incident-sev1`。

## 3. 初動手順（5 分以内）

```bash
# Envoy Gateway Pod 状態
kubectl get pods -n k1s0-ingress -l gateway.envoyproxy.io/owning-gateway-name=k1s0-public

# 攻撃元 IP の特定（RPS Top 20）
logcli query '{namespace="k1s0-ingress", job="envoy-gateway"}
  | json | line_format "{{.client_ip}} {{.path}}"' \
  --since=10m | awk '{print $1}' | sort | uniq -c | sort -rn | head -20

# 攻撃パターン判定
logcli query '{namespace="k1s0-ingress"} |= "GET" or |= "POST"
  | json | line_format "{{.path}}"' \
  --since=5m | sort | uniq -c | sort -rn | head -10
```

ステークホルダー通知（即時）:

- SEV1 即時宣言、Slack `#incident-sev1` に「Envoy Gateway 異常リクエストレート、攻撃源 <IP>、対応中」。
- [`oncall/escalation.md`](../../oncall/escalation.md) 起動、CTO に連絡。
- Status Page を「アクセス遅延発生中」に更新。

## 4. 原因特定手順

攻撃パターン分類:

| パターン | 指標 | 緊急度 |
|---|---|---|
| Single IP | 単一 IP からの高 RPS | 中（IP ブロックで対処可） |
| Botnet（分散） | 多数 IP から低〜中 RPS、合計大 | 高（rate limit ＋ Cloudflare 必須） |
| Layer 7 DDoS | 特定エンドポイント（例: `/login`）への集中攻撃 | 高（path-based rate limit） |
| Slowloris | Connection 数大、RPS 低 | 中（connection timeout 短縮） |

```bash
# 攻撃が分散か集中か判定
logcli query '{namespace="k1s0-ingress"}' --since=5m \
  | jq -r '.client_ip' | sort -u | wc -l   # ユニーク IP 数
```

エスカレーション: Layer 7 DDoS で正規利用者影響が拡大する場合、上位 ISP / Cloudflare DDoS 防御を有効化（CTO 判断）。

## 5. 復旧手順

### Step 1: 緊急 Rate Limit 投入（〜5 分）

```bash
kubectl apply -f - <<EOF
apiVersion: gateway.envoyproxy.io/v1alpha1
kind: BackendTrafficPolicy
metadata:
  name: emergency-rate-limit-$(date +%Y%m%d)
  namespace: k1s0-ingress
spec:
  targetRef:
    group: gateway.networking.k8s.io
    kind: HTTPRoute
    name: k1s0-public
  rateLimit:
    type: Global
    global:
      rules:
      - clientSelectors:
        - sourceCIDR:
            type: Distinct
            value: "0.0.0.0/0"
        limit:
          requests: 10
          unit: Second
EOF
```

### Step 2: 攻撃源 IP のブロック（Single IP 攻撃の場合）

```bash
kubectl apply -f - <<EOF
apiVersion: gateway.envoyproxy.io/v1alpha1
kind: SecurityPolicy
metadata:
  name: block-attacker-ip-$(date +%Y%m%d)
  namespace: k1s0-ingress
spec:
  targetRef:
    group: gateway.networking.k8s.io
    kind: HTTPRoute
    name: k1s0-public
  authorization:
    defaultAction: Allow
    rules:
    - action: Deny
      principal:
        clientCIDRs:
        - <attacker-ip>/32
EOF
```

### Step 3: Cloudflare DDoS 防御の有効化（Botnet / Layer 7 DDoS の場合）

```bash
# Cloudflare API で「Under Attack」モードを有効化
curl -X PATCH "https://api.cloudflare.com/client/v4/zones/<zone-id>/settings/security_level" \
  -H "Authorization: Bearer ${CF_API_TOKEN}" \
  -H "Content-Type: application/json" \
  --data '{"value":"under_attack"}'
```

### Step 4: アップストリーム遮断（最終手段）

正規利用者への影響が許容できないレベルになった場合、ISP に依頼して該当 ASN を遮断（CTO 判断）。
連絡先は [`oncall/contacts.md`](../../oncall/contacts.md) §インフラ・ベンダ。

### Step 5: Envoy Gateway のスケールアウト

```bash
kubectl scale deployment/envoy-k1s0-public -n k1s0-ingress --replicas=10
```

## 6. 検証手順

復旧完了の判定基準:

- RPS が直近 24h 平均の 1.5 倍以下に収束、5 分間継続。
- 5xx エラー率が 1% 未満。
- 正規利用者からのアクセス成功（手動でテスト URL を叩く）。
- Connection 数が通常範囲（通常上限の 80% 以下）。
- Cloudflare DDoS モードが解除可能か判定（「Medium」以下に戻せるか）。
- 直近 30 分の Loki クエリ `{namespace="k1s0-ingress"} |= "DDoS"` が 0 件。

## 7. 予防策

- ポストモーテム起票（24 時間以内、`postmortems/<YYYY-MM-DD>-RB-NET-002.md`）。
- 緊急 Rate Limit ポリシーの恒久化検討（攻撃パターンに応じて）。
- ブロック IP リストの四半期見直し（誤検知の場合は解除）。
- Cloudflare DDoS 防御の事前設定見直し。
- WAF（Web Application Firewall）の導入検討（Coraza WAF、`infra/security/`）。
- 月次 Chaos Drill 対象に「200x RPS Spike」シナリオを追加。
- NFR-A-REC-002 の MTTR ログを更新。

## 8. 関連 Runbook

- 関連設計書: `infra/mesh/envoy-gateway/`
- 関連 ADR: [ADR-MIG-002（API Gateway）](../../../docs/02_構想設計/adr/ADR-MIG-002-api-gateway.md)
- 関連 NFR: [NFR-E-AC-006](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md), [NFR-A-CONT-001](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 連鎖 Runbook:
  - [`RB-AUTH-002-auth-abuse-detection.md`](RB-AUTH-002-auth-abuse-detection.md) — `/login` エンドポイントへの DDoS が認証悪用と並行する場合
  - [`RB-API-001-tier1-latency-high.md`](RB-API-001-tier1-latency-high.md) — Backend が過負荷で応答遅延の場合
- エスカレーション: [`../../oncall/escalation.md`](../../oncall/escalation.md)
