---
runbook_id: MONTHLY-001
title: パッチ管理（OS / OSS / k8s / アプリ依存）
category: OPS
severity: 該当なし（定常運用）
owner: 協力者
automation: manual（Renovate 自動 PR + 手動承認、ADR-DEP-001）
alertmanager_rule: 該当なし
estimated_recovery: 半日〜1 日（毎月第 2 火曜）
last_updated: 2026-05-02
---

# MONTHLY-001: パッチ管理

毎月第 2 火曜（Microsoft Patch Tuesday と同期）に、OS / OSS / k8s / アプリ依存のパッチを適用する。
ADR-DEP-001（Renovate 自動 PR）と連動。

## 1. 前提条件

- 実行者は協力者または SRE lead。
- 必要ツール: `gh`（GitHub CLI）/ Renovate dashboard / staging 環境。

## 2. 対象事象

毎月第 2 火曜 09:00 JST。緊急パッチ（CVE 9.0+）は随時実施。

## 3. 初動手順（5 分以内）

```bash
# Renovate dashboard で月次 PR を確認
gh pr list --label renovate --state open
```

## 4. 原因特定手順

該当しない（定常運用）。

## 5. 復旧手順

実行手順:

1. **CVE 緊急度の判定**: GitHub Dependabot Security Advisories を確認、CVSS 7.0+ は当日対応。
2. **staging 適用**: Renovate PR を staging 環境に先行マージ、24h テスト。
3. **Production 適用**: 採用後の運用拡大時 で staging 結果 OK なら Argo CD で本番 sync（Progressive Delivery）。
4. **k8s クラスタ自体のパッチ**: kubeadm upgrade / GKE auto-upgrade のスケジュール確認。
5. **Node OS パッチ**: cordon → drain → reboot → uncordon の順で 1 Node ずつ。
6. **OSS バージョンアップ**: 採用後の運用拡大時で staging 環境を 1 週間 soak test 後に本番。

## 6. 検証手順

- staging で 24h soak test 実施済み。
- 本番デプロイ後、SLO 達成度に劣化なし（[`../weekly/slo-burn-rate-review.md`](../weekly/slo-burn-rate-review.md) 参照）。
- すべての PR が `Merged` または `Closed`（理由付き）。

## 7. 予防策

- CVE 9.0+ は 24h SLA、5.0-8.9 は 7 日 SLA、5.0 未満は月次に統合。
- Renovate auto-merge を minor / patch に限定（major は手動承認）。
- 月次レビューで未マージ PR の棚卸し。

## 8. 関連 Runbook

- 関連 ADR: [ADR-DEP-001 Renovate](../../../docs/02_構想設計/adr/ADR-DEP-001-renovate.md)
- [`dr-drill.md`](dr-drill.md) — DR 演習と同じ月次レビュー
