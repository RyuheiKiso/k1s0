---
runbook_id: RB-OPS-001
title: CI/CD パイプライン停止対応
category: OPS
severity: SEV3
owner: 協力者
automation: manual
alertmanager_rule: GitHubActionsConsecutiveFailures
fmea_id: 間接対応
estimated_recovery: 暫定 1 時間 / 恒久 4 時間
last_updated: 2026-05-02
---

# RB-OPS-001: CI/CD パイプライン停止対応

本 Runbook は GitHub Actions の連続失敗、または Argo CD Sync 失敗急増によるデプロイメント不能時の対応を定める。
本番影響なし、開発生産性影響のみ SEV3。NFR-C-NOP-001 に対応する。

## 1. 前提条件

- 実行者は `k1s0/k1s0` リポジトリの GitHub Actions 管理権限 + Argo CD admin。
- 必要ツール: `gh`（GitHub CLI）/ `kubectl` / `argocd`。
- kubectl context が `k1s0-prod`。

## 2. 対象事象

- GitHub Actions の連続失敗（直近 5 件中 4 件以上が失敗）、または
- Argo CD ダッシュボードで Sync 失敗が急増。

検知シグナル:

```bash
# 直近 GitHub Actions の失敗件数
gh run list --limit 10 --json conclusion | jq '[.[] | select(.conclusion=="failure")] | length'

# Argo CD の Sync 失敗 app 数
argocd app list -o json | jq '[.[] | select(.status.sync.status!="Synced")] | length'
```

通知経路: Slack `#dev-ops` → 開発チームへ周知。

## 3. 初動手順（5 分以内）

```bash
# 失敗 workflow の特定
gh run list --workflow=ci.yml --limit 10
gh run view <run-id> --log-failed

# Argo CD 失敗 app の特定
argocd app list | grep -v Synced
```

ステークホルダー通知: Slack `#dev-ops` に「CI/CD 停止、調査中」を投稿。本番影響なしを明記。

## 4. 原因特定手順

よくある原因:

1. **外部依存障害**: pkg.go.dev / npmjs.com / crates.io の障害。Status Page 確認。
2. **GitHub Actions ランナー障害**: hosted runner の障害（GitHub Status）または self-hosted runner ダウン。
3. **テストコード問題**: 直前の PR でテスト追加され環境差で失敗。
4. **GHCR 認証失敗**: PAT の期限切れ、または OIDC 設定変更。

## 5. 復旧手順

### 外部依存障害の場合

- GitHub Status / npm / pkg.go.dev を確認、復旧待ち（通常 30 分以内）。
- 緊急の場合は依存を vendoring または近接 mirror に切替。

### Runner 障害の場合

```bash
# self-hosted runner の再起動
kubectl rollout restart deployment/github-runner -n ci-system

# hosted runner の場合は Status Page で復旧待ち
```

### テスト失敗の場合

- 直前の PR を revert または hotfix。
- ランナーの kernel / Docker version を Pin。

### Argo CD 失敗の場合

- 該当 app を手動 sync。
- Manifest 競合は手動修正後 commit。

## 6. 検証手順

- GitHub Actions の直近 3 件が成功。
- Argo CD 全 app が `Synced` `Healthy`。
- 開発チームが PR をマージできる状態。

## 7. 予防策

- ポストモーテム起票（1 週間以内、`postmortems/<YYYY-MM-DD>-RB-OPS-001.md`）。
- 外部依存の vendoring 検討。
- self-hosted runner の冗長化。

## 8. 関連 Runbook

- 関連 ADR: [ADR-CICD-001 Argo CD](../../../docs/02_構想設計/adr/ADR-CICD-001-argocd.md), [ADR-CICD-002 Rollouts](../../../docs/02_構想設計/adr/ADR-CICD-002-rollouts.md)
- 連鎖 Runbook:
  - [`RB-OPS-002-argocd-out-of-sync.md`](RB-OPS-002-argocd-out-of-sync.md)
