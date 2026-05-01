---
runbook_id: WEEKLY-002
title: SLO burn rate 週次レビュー
category: OPS
severity: 該当なし（定常運用）
owner: 起案者（採用後の運用拡大時で SRE lead に委譲）
automation: manual
alertmanager_rule: 該当なし
estimated_recovery: 60 分（毎週金曜 16:00 JST）
last_updated: 2026-05-02
---

# WEEKLY-002: SLO burn rate 週次レビュー

毎週金曜 16:00 JST に、その週の SLO 達成度・error budget 消費率を確認し、改善 PR を起案する。

## 1. 前提条件

- 実行者は起案者（または SRE lead）。
- 必要ツール: ブラウザ（Grafana SLO ダッシュボード）/ Loki アクセス。

## 2. 対象事象

毎週金曜 16:00 JST。前週土曜〜今週金曜の SLO 達成度を対象。

## 3. 初動手順（5 分以内）

Grafana → **k1s0 SLO Overview** で以下を確認:

- 7 日間の SLO 達成度（NFR-A-SLA-001 99% / NFR-A-CONT-002 99.9%）
- error budget 消費率（週次累計）
- top 5 エラーバジェット消費イベント（週内のインシデント）

## 4. 原因特定手順

エラーバジェット消費上位 5 件を特定:

```bash
# Loki でエラー時間帯のイベントを集計
logcli query '{namespace="k1s0"} |= "ERROR" | json
  | line_format "{{.app}} {{.error_code}}"' \
  --since=168h | sort | uniq -c | sort -rn | head -20
```

## 5. 復旧手順

実行手順:

1. SLO 違反があれば原因 RB-* を特定。
2. error budget 消費 > 50% の場合は変更凍結（feature freeze）を提案。
3. 上位 5 イベントについて Runbook 改善 PR を起案。
4. レビュー結果サマリを Slack `#status` + `ops/runbooks/weekly-review-<YYYY-Wnn>.md`（採用後の運用拡大時 整備）に記録。

## 6. 検証手順

- 7 日間の SLO 達成度が 99% 以上（NFR-A-SLA-001）。
- 100% 達成（Audit 完整性 NFR-H-AUD-001 系）。
- error budget 消費が想定範囲内（<50%）。

## 7. 予防策

- error budget 消費が高い場合の Action Plan（採用後の運用拡大時で正式な変更凍結プロセスを規定）。
- 月次レビューで 4 週間の傾向を Product Council に報告。

## 8. 関連 Runbook

- [`chaos-experiment-review.md`](chaos-experiment-review.md) — 同じ週次レビュー会議で実施
- [`../incidents/RB-INC-001-severity-decision-tree.md`](../incidents/RB-INC-001-severity-decision-tree.md)
