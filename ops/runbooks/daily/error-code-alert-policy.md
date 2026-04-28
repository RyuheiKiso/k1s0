# エラーコード別アラートポリシー Runbook

> **severity**: SEV2〜SEV3
> **owner**: tier1-platform-team
> **estimated_mttr**: 2h（SEV2）/ 24h（SEV3）
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

Loki アラートルール（`infra/monitoring/loki/alerts/`）が以下のエラーコードパターンを監視し、
Alertmanager 経由で Slack `#alert-<tier>` チャンネルへ通知する。

```bash
# 現在のアラートルール一覧を確認
kubectl get prometheusrule -A | grep k1s0
kubectl get alertingrule -n monitoring
```

## 2. 初動 (Immediate Action)

### エラーコード × 閾値 × 通知先マトリクス

#### tier1（内部 gRPC API）

| エラーコード | 閾値 | 継続時間 | Severity | 通知先 |
|---|---|---|---|---|
| `K1s0Error.Unauthorized` | >10 req/min | 5 min | SEV2 | `#alert-tier1` + Security SRE |
| `K1s0Error.Forbidden` | >5 req/min | 5 min | SEV2 | `#alert-tier1` + Security SRE |
| `K1s0Error.InternalError` | >1% エラーレート | 10 min | SEV2 | `#alert-tier1` |
| `K1s0Error.Unavailable` | >5% エラーレート | 5 min | SEV1 | `#incident-alert` + PagerDuty |
| `K1s0Error.NotFound` | >50 req/min | 10 min | SEV3 | `#alert-tier1` |
| `K1s0Error.RateLimitExceeded` | >100 req/min | 1 min | SEV3 | `#alert-tier1` |

#### tier2（SDK / ファサード層）

| エラーコード | 閾値 | 継続時間 | Severity | 通知先 |
|---|---|---|---|---|
| gRPC `UNAVAILABLE` | >3% エラーレート | 5 min | SEV2 | `#alert-tier2` |
| gRPC `UNAUTHENTICATED` | >20 req/min | 5 min | SEV2 | `#alert-tier2` + Security SRE |
| gRPC `RESOURCE_EXHAUSTED` | >50 req/min | 3 min | SEV2 | `#alert-tier2` |
| Dapr サイドカー障害 | >2 Pod | 即時 | SEV2 | `#alert-tier2` |
| DB 接続エラー | >5 req/min | 3 min | SEV1 | `#incident-alert` |

#### tier3（Web / BFF 層）

| エラーコード | 閾値 | 継続時間 | Severity | 通知先 |
|---|---|---|---|---|
| HTTP 5xx | >2% エラーレート | 5 min | SEV2 | `#alert-tier3` |
| HTTP 401 | >50 req/min | 5 min | SEV3 | `#alert-tier3` |
| HTTP 403 | >20 req/min | 5 min | SEV2 | `#alert-tier3` + Security SRE |
| HTTP 429 | >200 req/min | 1 min | SEV3 | `#alert-tier3` |
| レスポンスタイム p99 > 5s | - | 10 min | SEV3 | `#alert-tier3` |

### アラート受信後の対応手順

1. アラートの Grafana リンクを開き、エラーレートの傾向を確認する。
2. 該当 tier の Pod ログを確認する。

   ```bash
   kubectl logs -n k1s0-tier1 -l app=tier1-facade --since=15m | grep ERROR | tail -50
   kubectl logs -n k1s0-tier2 -l app=tier2-sdk --since=15m | grep ERROR | tail -50
   kubectl logs -n k1s0-tier3 -l app=tier3-web --since=15m | grep ERROR | tail -50
   ```

3. `K1s0Error.Unauthorized` / `Forbidden` が急増している場合は Security SRE に報告し、
   `auth-abuse-detection.md` を参照する。
4. DB 接続エラーが続く場合は `severity-decision-tree.md` で SEV1 昇格を判定する。

## 3. 復旧 (Recovery)

- エラーレートが閾値を下回り 15 分継続した場合にアラートをクローズする。
- 原因特定前にアラートをサイレンスする場合は Alertmanager で期限付きサイレンスを設定し、
  必ず理由を記入する。

  ```bash
  amtool silence add --alertname="K1s0Unauthorized" \
    --comment="調査中: auth-abuse 疑い。15:00 まで" \
    --duration=1h
  ```

- Pod 再起動が必要な場合はローリングリスタートを使用する。

  ```bash
  kubectl rollout restart deployment/tier1-facade -n k1s0-tier1
  kubectl rollout status deployment/tier1-facade -n k1s0-tier1
  ```

## 4. 原因調査 (Root Cause Analysis)

- Loki でエラーコード別の時系列クエリを実行する。

  ```bash
  logcli query 'sum by (error_code) (rate({namespace="k1s0-tier1"}
    |= "K1s0Error" [5m]))' --since=2h
  ```

- 直前のデプロイとの相関を Argo CD で確認する。
- エラーコードが `Forbidden` / `Unauthorized` の場合は Keycloak の監査ログも確認する。

## 5. 事後処理 (Post-incident)

- SEV2 以上のアラートはインシデント記録を作成する（Backstage チケット）
- アラート閾値が適切かを週次 SLO レビューで評価し、調整 PR を出す
- 新エラーコード追加時はこの Runbook と Loki アラートルールを同時に更新する

## 関連

- 関連設計書: docs/03_要件定義/30_非機能要件/E_セキュリティ.md (NFR-E-AC-001, NFR-E-MON-003)
- 関連設計書: docs/03_要件定義/30_非機能要件/I_SLI_SLO_エラーバジェット.md
- 関連 ADR: ADR-SEC-001 (Keycloak)
- 関連 Runbook: severity-decision-tree.md, auth-abuse-detection.md
