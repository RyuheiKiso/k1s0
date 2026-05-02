---
runbook_id: RB-TEST-001
title: Flaky test の quarantine 運用（fail 率 ≥ 5% 自動検出 → quarantine 解除フロー）
category: OPS
severity: SEV3
owner: 起案者
automation: argo-workflow
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 quarantine PR merge / 恒久 解除（修正完了後）
last_updated: 2026-05-02
---

# RB-TEST-001: Flaky test quarantine 運用

ADR-TEST-007（テスト属性タグ + CI 実行フェーズ分離）の IMP-CI-TAG-004 / 005 に対応する Runbook。`tools/qualify/flaky-detector.py` が直近 20 PR で fail 率 ≥ 5% を検出した時に、tests/.flaky-quarantine.yaml に追加する PR を自動提出 → SRE / QA リードがレビューする運用を定める。詳細は採用初期で完成、本リリース時点は skeleton。

## 1. 前提条件

- `flaky-report.yml` workflow が日次起動（cron 04:00 JST）
- `tools/qualify/flaky-detector.py` が GitHub Actions API 経由で test 結果を集計
- CODEOWNERS で `tests/.flaky-quarantine.yaml` のレビュアーが SRE / QA リードに紐付け済（採用初期で）

## 2. 対象事象

- flaky-report.yml の自動 PR が起案者 / SRE に通知された
- nightly E2E が連続 4 週で同 test に fail（pattern 検出）

## 3. 初動手順（5 分以内）

```bash
# 自動 PR の差分確認
gh pr list --label flaky-quarantine
gh pr diff <pr-number>
# 該当 test の最新 fail log 確認
gh run view <run-id> --log
```

Slack `#qa` に「flaky candidate <test-name> 検出」を投稿。

## 4. 原因特定手順

- race condition: `go test -race` で再実行
- 環境依存: kind cluster の resource 競合
- timing 依存: `time.Sleep` を含む test が CI runner 性能で flaky 化

## 5. 復旧手順

```bash
# quarantine 入りの場合: 自動 PR を merge して PR ゲートから除外
gh pr merge <pr-number> --squash

# quarantine 解除の場合: 修正 commit + 連続 4 週 PASS 確認後に PR で yaml から削除
git revert <quarantine-add-commit>
gh pr create --title "test: <test-name> quarantine 解除" --body "連続 4 週 PASS 確認済"
```

## 6. 検証手順

- quarantine 入り後: PR ゲートが該当 test を skip して exit 0
- quarantine 解除後: nightly で連続 4 週 PASS（flaky-detector.py で確認）

## 7. 予防策

- ADR-TEST-007 の 4 タグ最低セット（@slow / @flaky / @security / @nightly）の徹底
- `tools/lint/test-tag-lint.sh` を pre-push hook で強制（採用初期）
- race condition は `go test -race` を必須化

## 8. 関連 Runbook

- [RB-CHAOS-001 pod-kill](incidents/RB-CHAOS-001-pod-kill.md) — chaos 経由の flaky 化対策
- ADR-TEST-007 / IMP-CI-TAG-004 / IMP-CI-TAG-005 / `tools/qualify/flaky-detector.py`
