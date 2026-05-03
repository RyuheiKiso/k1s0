---
runbook_id: RB-TEST-003
title: user e2e smoke 実走（利用者向け sanity check）
category: OPS
severity: SEV3
owner: 利用者|起案者
automation: manual|argo-workflow
alertmanager_rule: "（PR / nightly CI で機械検証、Alertmanager rule なし）"
fmea_id: 間接対応
estimated_recovery: 事後型 Runbook（実走完遂までの所要、約 3〜5 分）
last_updated: 2026-05-03
---

# RB-TEST-003: user e2e smoke 実走（利用者向け sanity check）

本 Runbook は ADR-TEST-008（owner / user 二分構造）で確定した user e2e の smoke check を、k1s0 SDK を使うアプリ開発者（利用者）が自分の host で実行する手順を定める。所要は約 3〜5 分（kind 起動 60 秒 + minimum stack 60 秒 + smoke test 60〜180 秒）。

## 1. 前提条件

- 実行者は k1s0 SDK を採用するアプリ開発者、または k1s0 起案者（CI 失敗の手元再現用）。
- 実行 host: 16GB RAM 推奨（minimum stack 約 1.7GB + kind 約 4GB + 利用者 dev 余裕）。
- devcontainer 内でも実行可能（multipass 不要、owner 経路と異なる）。
- 必須ツール:
  - `kind` v0.23+
  - `kubectl` v1.30+
  - `helm` v3.16+
  - `make`
- staging 検証は不要（user smoke は kind cluster 上の最小成立形）。

## 2. 対象事象

本 Runbook は **事後型**（reactive）。利用者が自分の k1s0 SDK 採用アプリで「k1s0 install が動いているか不安」「PR が CI で通らない原因切り分け」を行う時に起動する。

検知シグナル:

- 利用者: 自アプリ test で「k1s0 SDK の Setup が timeout する」「State.Get が 401 / 403 を返す」等
- owner: GitHub Actions の `e2e-user-smoke` job が PR で fail し、CI ログだけでは原因不明な時

ダッシュボード: GitHub Actions の `pr.yml` / `nightly.yml` 実行履歴。
通知経路: PR comment（Actions が自動投稿）。

## 3. 初動手順（5 分以内）

最初の 5 分で **環境前提と既存 cluster の有無** を確認する。

```bash
# 環境前提確認
kind version
kubectl version --client
helm version
free -h | head -2  # MemAvail が 4GB 以上あるか確認

# 既存 cluster の有無
kind get clusters
# k1s0-user-e2e が含まれていれば再利用可（KEEP_CLUSTER=1）、無ければ新規起動
```

```bash
# 既に k1s0-user-e2e cluster があり、状態が怪しい場合は事前削除
./tools/e2e/user/down.sh
```

## 4. 原因特定手順

smoke 実行が失敗した場合の原因特定:

```bash
# kind cluster の Pod 状態確認
kubectl get pods -A --sort-by=.metadata.creationTimestamp | tail -30

# minimum stack 各 Pod の event 確認
kubectl get events -A --sort-by='.lastTimestamp' | tail -20

# tier1 facade Pod の log 確認
kubectl logs -n tier1-state -l app=tier1-state --tail=100

# Keycloak 起動の確認
kubectl get pods -n keycloak
kubectl logs -n keycloak -l app=keycloak --tail=50
```

よくある原因:

1. **devcontainer 内で host network mode 不可**: kind が devcontainer 内 docker daemon に接続できない。対処: host OS の bash から `./tools/e2e/user/up.sh` 実行、または `KIND_EXPERIMENTAL_PROVIDER=podman` で podman 切替。
2. **CNPG operator が install 中で smoke test が timeout**: minimum stack の起動順序待機を 5 分まで増やす。対処: `make e2e-user-smoke` の前に手動で `kubectl wait --for=condition=Ready pod -A --timeout=300s`。
3. **16GB host で memory pressure**: 他のメモリ重いプロセス（ブラウザ / VS Code 等）を停止。dmesg で OOM event 確認。
4. **kind / image pull rate limit**: docker.io 100 pulls/6h 制限。対処: cilium / quay.io ミラー使用に切替（既に local-stack/lib/apply-layers.sh で実装済）。

エスカレーション: 上記に該当しない場合、起案者に GitHub issue で報告（template: tests/e2e/user/smoke 関連）。

## 5. 復旧手順

user smoke の実走経路。

```bash
# Step 1: cluster 起動 + smoke 実行 + cleanup を 1 sequence で
make e2e-user-smoke
```

任意 stack を追加する場合（例: workflow の動作確認）:

```bash
# Step 1a: 任意 stack を追加で起動（既存 kind cluster を再利用）
KEEP_CLUSTER=1 ./tools/e2e/user/up.sh --add workflow
# Step 2a: smoke だけ実行（cluster cleanup を skip）
cd tests/e2e/user && go test -tags=user_e2e -timeout=10m -v ./smoke/...
```

各 step は約 3 分 / 1 分以内。3 分超で kind 起動が進まない場合は Step 4 の原因特定に分岐。

## 6. 検証手順

user smoke PASS の判定基準:

- `make e2e-user-smoke` が exit 0
- `tests/.user-e2e/<日付>/smoke/result.json` で `"Action":"fail"` が 0 件
- kind cluster が cleanup で削除されている（`kind get clusters` で `k1s0-user-e2e` 不在）

```bash
# 結果サマリの確認
cat tests/.user-e2e/$(date -u +%Y-%m-%d)/smoke/result.json | jq -r 'select(.Action=="pass") | .Test' | sort -u
```

## 7. 予防策

- 利用者が自アプリ repo で SDK fixtures（ADR-TEST-010）を import する経路を README で導線化
- CI で `e2e-user-smoke` job が fail した時の triage 手順を本 Runbook §4 として継続更新
- nightly cron が user full で fail した場合、`docs/40_運用ライフサイクル/user-e2e-results.md` に root cause を集約

## 8. 関連 Runbook

- 関連設計書:
  - [`docs/05_実装/30_CI_CD設計/35_e2e_test_design/20_user_suite/`](../../docs/05_実装/30_CI_CD設計/35_e2e_test_design/20_user_suite/)
  - [`tools/e2e/`](../../tools/e2e/) — owner / user の cluster orchestration
- 関連 ADR:
  - [ADR-TEST-008](../../docs/02_構想設計/adr/ADR-TEST-008-e2e-owner-user-bisection.md)
  - [ADR-TEST-010](../../docs/02_構想設計/adr/ADR-TEST-010-test-fixtures-sdk-bundled.md)
- 連鎖 Runbook:
  - [`RB-TEST-OWNER-E2E-FULL.md`](RB-TEST-OWNER-E2E-FULL.md) — owner 側の全 8 部位実走
- live document: [`docs/40_運用ライフサイクル/user-e2e-results.md`](../../docs/40_運用ライフサイクル/user-e2e-results.md)
