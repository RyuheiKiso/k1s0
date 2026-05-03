---
runbook_id: RB-TEST-002
title: owner full e2e 実走 + release tag ゲート PASS 証跡作成
category: OPS
severity: SEV3
owner: 起案者
automation: manual
alertmanager_rule: "（手動 trigger、Alertmanager rule なし）"
fmea_id: 間接対応
estimated_recovery: 事後型 Runbook（recovery 軸ではなく実走完遂までの所要、約 1 時間 45 分）
last_updated: 2026-05-03
---

# RB-TEST-002: owner full e2e 実走 + release tag ゲート PASS 証跡作成

本 Runbook は ADR-TEST-008（owner / user 二分構造）と ADR-TEST-011（release tag ゲート代替保証）で確定した owner full e2e の不定期実走を、起案者（owner）が host OS の WSL2 native shell 上で実行する手順を定める。owner full は CI 不可（multipass nested virtualization 制約）のため、本 Runbook が実走の唯一の正規入口となる。所要は約 1 時間 45 分（multipass × 5 起動 30 分 + 全 8 部位実行 60 分 + cleanup 15 分）。

## 1. 前提条件

- 実行者は k1s0 起案者（owner）または同等権限を持つ協力者。
- 実行 host: WSL2 + 48GB RAM + Hyper-V nested virtualization 有効化（multipass 動作の前提）。
- 必須ツール:
  - `multipass` v1.13+（VM 起動）
  - `kubectl` v1.30+（cluster 操作）
  - `helm` v3.16+（Cilium / Longhorn / MetalLB install）
  - `git lfs` 設定済（artifact 12 ヶ月版管理、ADR-TEST-011 §4）
  - `zstd`（artifact 圧縮、不在時は gzip フォールバック）
- 起動契機（ADR-TEST-008 §6 推奨）:
  - release tag を切る前
  - k8s minor version upgrade 前
  - 4 言語 SDK の major 改訂後
  - tier1 12 service の互換破壊変更後
- 直前確認: `git status` がクリーンであること（`git stash` で待避してから実走）。
- staging 検証は不要（owner full は staging 相当の本番再現スタックを起動する）。

## 2. 対象事象

本 Runbook は **事後型**（reactive ではなく proactive）。Alertmanager 発火を起点としない。owner の判断で「次の release を切る前」「重大変更後の再検証」等の契機で起動する。

検知シグナル:

```bash
# Last PASS からの経過日数を確認（30 日超で release tag ゲート fail のリスク）
LAST_PASS_DATE="$(grep -m1 '^### [0-9]' docs/40_運用ライフサイクル/owner-e2e-results.md | sed 's/^### //')"
DAYS_DIFF=$(( ($(date +%s) - $(date -d "$LAST_PASS_DATE" +%s)) / 86400 ))
echo "Last owner full PASS: ${LAST_PASS_DATE} (${DAYS_DIFF} 日前)"
```

ダッシュボード: なし（owner 個人運用のため、Grafana ダッシュボード化は採用後の運用拡大時で SRE 増員後に整備）。
通知経路: なし（owner の判断で起動）。

## 3. 初動手順（5 分以内）

最初の 5 分で **環境前提を確認** する。multipass / kubectl / helm の存在 + 48GB host + WSL2 native shell であることを確認する。

```bash
# 環境前提確認
multipass version
kubectl version --client
helm version
free -h | head -2  # MemTotal が 48GB 程度か確認
echo "Shell: $SHELL / WSL: $(uname -r | grep -i microsoft)"
```

```bash
# 既存 VM があれば事前に削除（前回中断された残骸の clean）
./tools/e2e/owner/down.sh --keep-artifacts
```

ステークホルダー通知: なし（owner 個人作業）。

## 4. 原因特定手順

本 Runbook は事後型のため「失敗時の原因特定」を扱う。実走中に問題が発生した場合、以下の順で特定する。

```bash
# multipass VM 起動失敗の原因確認
multipass list  # 5 VM の State を確認、Stopped / Errored を特定
journalctl -u snap.multipass.* --since "1 hour ago" | tail -50  # multipass daemon log

# kubeadm init 失敗時
multipass exec k1s0-owner-cp-1 -- sudo journalctl -u kubelet --since "30 minutes ago" | tail -50

# Cilium / Longhorn install 失敗時
KUBECONFIG=tests/.owner-e2e/$(date -u +%Y-%m-%d)/kubeconfig kubectl get pods -A | grep -v Running
KUBECONFIG=tests/.owner-e2e/$(date -u +%Y-%m-%d)/kubeconfig kubectl describe pod -n cilium-system <crashing-pod>
```

よくある原因:

1. **multipass nested virt 不可**: `host nested virtualization not enabled` の error log。対処: `bcdedit /set hypervisorlaunchtype auto` で Windows 側 Hyper-V 有効化 → host 再起動。
2. **kubeadm init で certs upload 失敗**: certificate key の取得 timing 問題。対処: `tools/e2e/owner/down.sh` → `up.sh` で fresh 起動を再試行。
3. **Cilium image pull rate limit (docker.io)**: quay.io ミラー使用に切替（既に local-stack/lib/apply-layers.sh で対応済、Cilium も同経路）。
4. **48GB host で OOM**: `tools/e2e/owner/up.sh` の Step 7 でフルスタック install 中に kernel OOM 発生。対処: `dmesg` 確認 + Step 8 の `--skip-stack` で部分 install して原因切り分け。

エスカレーション: 上記いずれにも該当しない場合、issue 起票 + ADR-TEST-008 / 009 / 010 / 011 の決定を見直す（再策定の起点）。

## 5. 復旧手順

owner full の実走経路（本 Runbook は事後型のため recovery 軸ではなく完遂までの 1 sequence を示す）。所要約 1 時間 45 分。

```bash
# Step 1: cluster 起動（約 35 分）
./tools/e2e/owner/up.sh
# 進捗を観察（dmesg で OOM 監視）
dmesg -w | grep -i "out of memory" &  # 別 terminal で監視
```

```bash
# Step 2: 全 8 部位実行（約 60 分、make e2e-owner-full は up.sh + 全部位 + cleanup を含む 1 sequence）
# up.sh が既に走っている場合は make から up.sh の再実行を skip（idempotent）
make e2e-owner-full
```

```bash
# Step 3: artifact 集約 + sha256 計算 + owner-e2e-results.md entry 追記
make e2e-owner-full  # archive.sh + update-results.sh が含まれる
```

各 step は約 35 分 / 60 分 / 1 分。35 分超で multipass 起動が進まない場合は Step 4 の原因特定に分岐。

## 6. 検証手順

owner full PASS の判定基準（全項目満たして release tag を切れる状態）:

- 全 8 部位の go test が exit 0（result.json で `"Action":"fail"` が 0 件）
- `tests/.owner-e2e/<日付>/full-result.tar.zst` が生成され、sha256 が正しい
- `docs/40_運用ライフサイクル/owner-e2e-results.md` に新 entry が `判定: PASS` で追記されている
- `dmesg` に OOM event なし（`dmesg | grep -i "out of memory"` が 0 件）
- `multipass list` で 5 VM すべて Running（または `--keep-cluster` 指定の場合）

```bash
# release tag dry-run で検証経路を確認
./tools/release/cut.sh --dry-run v0.0.0-test
# → "[ok] owner full PASS 検証: YYYY-MM-DD (N 日前) sha256=..." が出力されれば OK
```

## 7. 予防策

- 月次に owner full を実走する目処を立てる（cron 起動はしないが、release cycle と同期）
- `OWNER_E2E_FRESHNESS_DAYS` env で鮮度閾値を運用環境に合わせて調整可（既定 30 日）
- artifact の git LFS 累積容量を月次レビュー（12 ヶ月超は release asset 昇格 + cold storage 移行）
- failure 時の root cause を `失敗詳細` に必ず記載（次回再実走の追跡可能性確保）
- ADR-TEST-008 / 011 の決定変更が発生した場合は本 Runbook を同 PR で更新

## 8. 関連 Runbook

- 関連設計書: [`docs/05_実装/30_CI_CD設計/35_e2e_test_design/`](../../docs/05_実装/30_CI_CD設計/35_e2e_test_design/)
- 関連 ADR:
  - [ADR-TEST-008](../../docs/02_構想設計/adr/ADR-TEST-008-e2e-owner-user-bisection.md)
  - [ADR-TEST-009](../../docs/02_構想設計/adr/ADR-TEST-009-observability-e2e-five-checks-owner-only.md)
  - [ADR-TEST-011](../../docs/02_構想設計/adr/ADR-TEST-011-release-tag-gate-as-owner-e2e-alternative.md)
- 関連 NFR: 観測性 5 検証経由で NFR-B-PERF-001〜007 / NFR-C-NOP-001〜003
- 連鎖 Runbook:
  - [`RB-UPGRADE-001-kubeadm-minor-upgrade.md`](RB-UPGRADE-001-kubeadm-minor-upgrade.md) — k8s upgrade 前後で owner full 実走
  - [`RB-TEST-USER-SMOKE.md`](RB-TEST-USER-SMOKE.md) — 利用者向け user smoke 経路
- live document: [`docs/40_運用ライフサイクル/owner-e2e-results.md`](../../docs/40_運用ライフサイクル/owner-e2e-results.md)
