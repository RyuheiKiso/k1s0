---
runbook_id: RB-INC-001
title: Severity 判定フロー（インシデント横断ヘルパー）
category: INC
severity: SEV1〜SEV3（判定対象）
owner: 起案者
automation: manual
alertmanager_rule: alert_severity_*（Loki ルール群）
fmea_id: 横断
estimated_recovery: 判定完了まで 15 分
last_updated: 2026-05-02
---

# RB-INC-001: Severity 判定フロー（インシデント横断ヘルパー）

本 Runbook は他の RB-* インシデント Runbook 起動前の **Severity 判定** を一元化する横断ヘルパーである。
SEV1 / SEV2 / SEV3 を 15 分以内に確定し、該当 Runbook と [`oncall/escalation.md`](../../oncall/escalation.md) の起動をトリガーする。
NFR-E-SIR-001 / NFR-C-OPS-001 に対応する。

## 1. 前提条件

- 実行者は SRE オンコール当番、または Slack `#incident-alert` の購読者で 15 分以内に応答可能なこと。
- 必要ツール: `kubectl` / `logcli`（Loki）/ `argocd` / Grafana ダッシュボードへのアクセス。
- 判定者の権限要件は最小（read-only）。誰でも本フローを起動可能だが、SEV1 認定後は CTO 承認が必要。
- 経営報告のためのコミュニケーション手段（Slack / 電話）が利用可能。

## 2. 対象事象

以下のいずれかが起動トリガーとなる:

- Loki アラート（`alert_severity_*` ルール）が Slack `#incident-alert` チャンネルに着信。
- 外部ユーザー・採用組織からの障害報告（Backstage サポートチケット / メール）。
- 監視当番による手動検知（死活監視ダッシュボード Grafana `/d/k1s0-health`）。
- CI/CD パイプライン失敗による本番デプロイ停止。
- セキュリティツール（Falco / gitleaks）からの自動通報。

## 3. 初動手順（5 分以内）

判定者はインシデント認知後 **15 分以内** に Severity を確定し、対応 Runbook を起動する。最初の 5 分で判定フロー Step 1〜2 を完了させる。

### 判定フロー

```
┌─────────────────────────────────────────────────────┐
│ STEP 1: 影響サービスを特定                            │
│   kubectl get pods -A | grep -v Running              │
│   → 複数 tier または全テナントに影響？                │
└──────────────────┬──────────────────────────────────┘
                   │ Yes → SEV1 候補
                   │ No  → STEP 2 へ
┌──────────────────▼──────────────────────────────────┐
│ STEP 2: 継続時間とエラーレートを確認                  │
│   Grafana SLO ダッシュボード:                         │
│     - エラーバジェット消費率 > 20%/h  → SEV1         │
│     - エラーレート > 5% かつ > 10 分 → SEV2          │
│     - エラーレート < 5% または < 10 分 → SEV3        │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│ STEP 3: セキュリティ・法的影響の確認                  │
│   - PII / 個人情報の漏えい疑い？ → SEV1              │
│   - テナント越境アクセス検知？  → SEV1              │
│   - 法的開示要求（令状等）あり？ → SEV1              │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│ STEP 4: 経営報告閾値の確認                            │
│   SEV1: 即時 CTO へ電話報告、30 分以内に役員 Slack   │
│   SEV2: 1 時間以内に Engineering Manager へ Slack    │
│   SEV3: 当番が対応、翌朝の定例で報告                 │
└─────────────────────────────────────────────────────┘
```

### Severity 判定基準早見表

| 指標 | SEV1 | SEV2 | SEV3 |
|------|------|------|------|
| サービス全停止 | Yes | - | - |
| 複数テナント影響 | Yes | 部分的 | 単一テナント |
| エラーバジェット消費 | >20%/h | 5〜20%/h | <5%/h |
| 継続時間 | >15min | 10〜60min | <10min |
| PII 漏えい疑い | Yes | - | - |
| テナント越境 | Yes | - | - |
| MTTR 目標 | 2h | 8h | 24h |
| 経営報告 | CTO 即時 | EM 1h以内 | 定例報告 |

## 4. 原因特定手順

判定後の根本原因調査（次段の Runbook で実施するが、判定段階で初期データを収集する）:

```bash
# Loki ログで影響開始時刻を特定
logcli query '{namespace="k1s0-tier1"}' --since=2h --limit=1000 \
  | grep -E "(ERROR|CRITICAL)" | head -50
```

- Grafana でエラーレート急上昇タイミングと直前のデプロイを照合。
- Argo CD で直近 2 時間のデプロイ履歴を確認:

```bash
argocd app history k1s0-tier1 --last 10
```

判定段階で原因を確定する必要はない（次段 Runbook の役割）。判定段階の責務は「Severity 確定 + 該当 Runbook の起動」のみ。

## 5. 復旧手順（該当 Runbook 起動）

Severity 確定後、該当 Runbook を **5 分以内** に起動する:

| Severity | 起動先 |
|---|---|
| SEV1 セキュリティ系 | [`RB-SEC-005-pii-leak-detection.md`](RB-SEC-005-pii-leak-detection.md) / [`RB-SEC-006-tenant-boundary-breach.md`](RB-SEC-006-tenant-boundary-breach.md) / [`RB-COMP-001-legal-disclosure.md`](RB-COMP-001-legal-disclosure.md) |
| SEV1 可用性系 | [`RB-DB-002`](RB-DB-002-postgres-primary-failover.md) / [`RB-SEC-001`](RB-SEC-001-openbao-raft-failover.md)（予定） / [`RB-SEC-002`](RB-SEC-002-cert-expiry.md) / [`RB-NET-002`](RB-NET-002-envoy-dos.md)（予定） |
| SEV1 全壊 | [`RB-DR-001`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md) |
| SEV2 認証系 | [`RB-AUTH-002-auth-abuse-detection.md`](RB-AUTH-002-auth-abuse-detection.md) / [`RB-AUTH-001`](RB-AUTH-001-keycloak-db-failover.md)（予定） |
| SEV2 メッセージング系 | [`RB-MSG-001-kafka-broker-failover.md`](RB-MSG-001-kafka-broker-failover.md) / [`RB-MSG-002-dlq-backlog.md`](RB-MSG-002-dlq-backlog.md) |
| SEV2 運用系 | [`daily/error-code-alert-policy.md`](../daily/error-code-alert-policy.md) |
| SEV3 | [`daily/error-code-alert-policy.md`](../daily/error-code-alert-policy.md) |

並行アクション:

1. SEV1/2 はインシデント指揮者（Incident Commander）を指名（必須）。
2. 対応進捗を Slack `#incident-<YYYYMMDD>-<slug>` チャンネルで共有（SEV1 は 15 分ごと）。
3. SEV1 確定で [`oncall/escalation.md`](../../oncall/escalation.md) を起動。

Severity ダウングレード条件: SLO 回復 + 根本原因が封じ込め済みであること。

## 6. 検証手順

判定の妥当性を検証する基準:

- 起動した Runbook の §6. 検証手順 を満たしてインシデントが Resolved。
- ポストモーテムで Severity 判定が「適切」「過剰」「過少」のいずれかに分類される。
- Severity 判定の所要時間が 15 分以内（NFR-E-SIR-001 整合）。
- 経営報告閾値が遵守された（CTO 即時 / EM 1h / 定例）。
- 該当する場合、Status Page 更新が SEV1 では 15 分以内、SEV2 では 1h 以内に完了。

## 7. 予防策

- ポストモーテム作成（SEV1: 24h / SEV2: 72h / SEV3: 1 週間）。
- Severity 判定精度のレビュー（過剰判定・過少判定を記録）。
- Runbook へのフィードバック反映 PR。
- 四半期ごとに判定基準を見直し、アラートルールと整合させる。
- Loki アラート `alert_severity_*` ルールの誤検知率を計測し、20% 超なら閾値見直し。
- 月次レビュー（[`weekly/slo-burn-rate-review.md`](../weekly/slo-burn-rate-review.md) 予定）で Severity 分布を確認。

## 8. 関連 Runbook

- 関連 NFR: [NFR-E-SIR-001 / NFR-E-SIR-002](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md), [NFR-A-SLO 全般](../../../docs/03_要件定義/30_非機能要件/I_SLI_SLO_エラーバジェット.md)
- 関連 ADR: [ADR-SEC-001（Keycloak）](../../../docs/02_構想設計/adr/ADR-SEC-001-keycloak.md), [ADR-SEC-002（OpenBao）](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md), [ADR-OBS-003 Incident Taxonomy](../../../docs/02_構想設計/adr/ADR-OBS-003-incident-taxonomy.md)
- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md)
- 連鎖 Runbook: 上記 §5 の Severity 別 Runbook 一覧
- エスカレーション: [`../../oncall/escalation.md`](../../oncall/escalation.md)
