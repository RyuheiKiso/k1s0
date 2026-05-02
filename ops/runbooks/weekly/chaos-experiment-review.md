---
runbook_id: WEEKLY-001
title: Chaos Engineering 実験結果 週次レビュー
category: OPS
severity: 該当なし（定常運用）
owner: 起案者（採用後の運用拡大時で SRE lead に委譲）
automation: manual
alertmanager_rule: 該当なし
estimated_recovery: 60 分（毎週金曜 16:00 JST）
last_updated: 2026-05-02
---

# WEEKLY-001: Chaos Engineering 実験結果 週次レビュー

毎週金曜 16:00 JST に、その週の Chaos 実験結果をレビューし、Runbook 改善 PR にフィードバックする。
[`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §「Chaos Drill」に対応。

## 1. 前提条件

- 実行者は起案者（または SRE lead）。
- 必要ツール: `kubectl` / Grafana / GitHub PR 権限。
- LitmusChaos ChaosResult が直近 1 週間分蓄積されていること。

## 2. 対象事象

毎週金曜 16:00 JST。前週土曜〜今週金曜の ChaosWorkflow 実行結果を対象。

## 3. 初動手順（5 分以内）

```bash
# 直近 1 週間の ChaosResult を取得
kubectl get chaosresult -A --sort-by=.metadata.creationTimestamp \
  | awk -v d="$(date -d '7 days ago' +%Y-%m-%d)" '$1 >= d'
```

## 4. 原因特定手順

各 ChaosResult について:
- `verdict: Pass` → Runbook が機能している。
- `verdict: Fail` → 該当 RB-* に問題あり、改善必要。

## 5. 復旧手順

実行手順:

1. 各実験の verdict / probe 結果を確認。
2. Failed な実験について該当 RB-* を特定し、Runbook 著者に Slack で通知。
3. Runbook 改善 PR の起案を依頼（受領者は 2 営業日以内に対応）。
4. レビュー結果サマリを `ops/chaos/results/<YYYY-MM-DD>-weekly-review.md` に記録。

## 6. 検証手順

- 直近 1 週間の全 ChaosResult を確認済み。
- Failed な実験 each に対して GitHub Issue または PR が起案済み。
- レビュー結果が `ops/chaos/results/` に記録済み。

## 7. 予防策

- ChaosWorkflow のスケジュール頻度を採用後の運用拡大時で週次 → 日次に上げる。
- 過去 4 週間の Failed 件数を Grafana ダッシュボードで可視化。

## 8. 関連 Runbook

- [`../../chaos/README.md`](../../chaos/README.md)
- [`slo-burn-rate-review.md`](slo-burn-rate-review.md) — 同じ週次レビュー会議で実施
