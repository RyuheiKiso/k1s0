---
runbook_id: MONTHLY-002
title: DR 演習（table-top / 実機）
category: OPS
severity: 該当なし（定常運用）
owner: 起案者（採用後の運用拡大時で DR 専任ロール）
automation: manual
alertmanager_rule: 該当なし
estimated_recovery: 2〜4 時間（四半期 table-top / 半期 実機）
last_updated: 2026-05-02
---

# MONTHLY-002: DR 演習

DR 体制の継続性を確保するため、table-top exercise（四半期）と staging 実機演習（半期）を実施する。
[`docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md) §DS-OPS-ENV-012 に対応。

## 1. 前提条件

- 実行者は起案者 + 協力者 + EM。
- 必要ツール: 演習用 Slack チャンネル / staging 環境（実機演習時）。
- 直近 backup が 24h 以内に成功している（[`../daily/backup-verification.md`](../daily/backup-verification.md)）。

## 2. 対象事象

| 種別 | 頻度 | 参加者 |
|---|---|---|
| Table-top exercise | 四半期に 1 回（3 / 6 / 9 / 12 月最終水曜） | 起案者 + 協力者 + EM |
| 実機演習（staging） | 半期に 1 回（4 / 10 月） | 起案者 + 協力者 + 採用組織 SRE |
| 本番 DR 発動演習 | 年 1 回（10 月） | 全関係者（Product Council 召集） |

## 3. 初動手順（5 分以内）

事前準備:

```bash
# 演習シナリオを選定（過去 1 年のインシデントから）
# RB-DR-001 (cluster-rebuild) を中心に、関連 RB-* との連鎖を含める

# staging 環境の Health 確認
kubectl --context=k1s0-staging get nodes
argocd app list --argocd-server staging.argocd.example.jp
```

## 4. 原因特定手順

該当しない（定常運用）。

## 5. 復旧手順

### Table-top exercise（4 時間）

1. **シナリオ提示**（30 分）: 起案者が「クラスタ全壊、RPO 24h、外部監視のみ生存」等を提示。
2. **意思決定 walkthrough**（2 時間）: 各メンバが [`../../dr/scenarios/RB-DR-001-cluster-rebuild.md`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md) を順次なぞる。
3. **問題抽出**（1 時間）: Runbook の曖昧箇所、必要 ツール の不足、連絡手段の欠落を全件記録。
4. **改善 PR 起案**（30 分）: 抽出した問題ごとに Issue または PR を起案。

### 実機演習（4〜8 時間）

1. staging 環境を意図的に破壊（例: namespace 削除、PVC 削除）。
2. [`RB-DR-001`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md) を実行、RTO 4h 以内達成を測定。
3. 結果を [`../../dr/drills/DR-drill-YYYY-Qn.md`](../../dr/drills/) に記録。

### 本番 DR 発動演習

採用後の運用拡大時で実施。詳細は別途プラン化。

## 6. 検証手順

- 演習レポートが `dr/drills/DR-drill-YYYY-Qn.md` に記録済み。
- 抽出された問題に対して Issue / PR が起案済み（2 週間以内に対応）。
- 実機演習で RTO 4h を達成（達成できなければ改善 plan を起案）。
- 全関係者にレビュー結果が共有済み。

## 7. 予防策

- 演習頻度を採用後の運用拡大時で四半期 → 月次（table-top）に上げる。
- 演習で発見した問題のうち重要度高は次月パッチサイクルに織り込む。
- 演習の参加者を毎回 1 名以上ローテーション（属人化回避）。

## 8. 関連 Runbook

- [`../../dr/scenarios/RB-DR-001-cluster-rebuild.md`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md)
- [`../../dr/drills/`](../../dr/drills/)
- [`patch-management.md`](patch-management.md)
- [`infra-disposal.md`](infra-disposal.md)
