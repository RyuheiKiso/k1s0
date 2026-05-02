# 01. owner / user 責務分界

本ファイルは ADR-TEST-008（e2e owner / user 二分構造）で確定した方針を、実装段階の運用契約として固定する。e2e の責務を owner（k1s0 を OSS として完璧に検証する起案者）と user（k1s0 でアプリを開発する利用者）の 2 系統に物理分離し、各系統の責務範囲・host 環境制約・CI 経路を ID として採番する。

## 本ファイルの位置付け

過去 ADR-TEST-002（撤回済）は両者を 1 つの e2e suite に統合する設計で、ディレクトリも `tests/e2e/` 単一階層だった。実環境で運用するに、利用者の host RAM（16GB）でフルスタック起動が破綻する / オーナーが本番再現スタックで OSS 完成度検証する経路が CI 不可で塞がる、の 2 つの矛盾が判明し、責務を 1 経路で統合する設計は破綻する。本ファイルでは ADR-TEST-008 の二分構造を実装規約として固定し、後続節の環境契約・ディレクトリ構造・Makefile target・CI 戦略の前提とする。

## owner / user の責務軸

責務分界を 6 軸で固定する。各軸は ADR-TEST-008 の決定に対応し、本ファイルでは実装段階の運用契約として読み解く。

| 軸 | owner | user |
|---|---|---|
| 検証目的 | k1s0 自体が OSS として完璧に動くか（4 言語 SDK の cross-product / 観測性 5 検証 / security policy 強制 / HA / DR / upgrade drill / CNCF Conformance 連携） | 自分の tier2 / tier3 アプリが k1s0 SDK 越しに動くか |
| host RAM 要件 | 48GB 専用 | 16GB OK |
| K8s 実装 | multipass × 5 + kubeadm 3CP HA + 2W | kind（CP1 + W1） |
| 本番再現範囲 | フル（Cilium / Longhorn / MetalLB / multi-AZ 疑似 / フルスタック 11 components） | minimum（Dapr + tier1 facade + Keycloak + 1 backend） |
| CI 経路 | CI 不可（multipass nested virt 不可）/ release tag ゲート (ADR-TEST-011) で代替保証 | PR + nightly で機械検証 |
| 起動契機 | 不定期（release tag 切る前 / k8s upgrade 前 / SDK major 改訂後） | PR 毎（smoke）/ nightly cron（full） |

軸の対比は責務の物理的分離を表現する。例えば「16GB host で本番再現 fidelity を取る」は host RAM 不足で物理的に不可能、「48GB host での検証を CI で機械化する」は GitHub Actions runner の nested virt 不可で物理的に不可能。両系統の物理的制約を起動経路（`tools/local-stack/up.sh --role` 引数）に吸収することで、host 環境別のテスト網羅性が ADR レベルで確定する。

## 責務境界が曖昧なテストの判定基準

owner / user 両方に書ける test 項目（例: tier1 facade の State.Get round-trip）は、以下の判定基準で配置先を確定する。

- **本番 fidelity が必要か**: Cilium eBPF / Longhorn replication / MetalLB / multi-AZ topology に依存する検証は owner、これらを参照しない検証は user
- **4 言語 SDK の網羅が必要か**: 4 言語 × 12 RPC = 48 cross-product を網羅するテストは owner、特定 1 言語のみで成立するテストは user
- **CI 機械検証が必要か**: PR / nightly で必ず PASS が要求される項目は user、release tag 時のみ要求される項目は owner
- **観測性スタックが必要か**: Grafana LGTM フルスタックで Tempo / Loki / Mimir API を直接叩く検証は owner、メトリクス / log の存在のみ確認する検証は user

判定が割れる場合は **owner 側に置く** ことを既定とする。理由は、owner は CI 不可で実走頻度が低い分、より重い検証を許容できるためである。逆に user 側に過剰な検証を置くと PR 5 分予算 / nightly 30〜45 分予算を圧迫する。

## ディレクトリ間で test を移動する手順

owner → user / user → owner の test 移動が必要になった場合、以下の手順で行う。両者は別 Go module（`go.mod` 分離）であり、import path も異なるため、機械的なファイル移動だけでは成立しない。

1. 移動先の `go.mod` で必要な依存（testcontainers / chromedp / k6 等）を確認、不足なら追加
2. 移動先の helpers / fixtures を import するように test code を書き換え（owner = `tests/e2e/owner/helpers/`、user = `src/sdk/<lang>/test-fixtures/` 経由）
3. 起動経路（`tools/local-stack/up.sh --role`）の前提を移動先に合わせる（owner-e2e role の前提はフルスタック、user-e2e role は minimum）
4. CI 経路の確認（owner 移動後は `pr.yml` / `nightly.yml` から外れる、user 移動後は逆）
5. `02_test_layer_responsibility.md` に移動の根拠を追記（判定基準のどれが決め手か明記）

移動は物理的なファイル移動 + 上記 5 step を 1 PR で同時に行うことを必須とする。test の移動と CI 経路の改訂を別 PR に分けると、CI 失敗の状態で test code がマージされる事故が発生する。

## IMP ID

| ID | 内容 | 配置 |
|---|---|---|
| IMP-CI-E2E-001 | owner / user 二分原則と判定基準 | 本ファイル |

## 対応 ADR / 関連設計

- ADR-TEST-008（e2e owner / user 二分構造）— 本ファイルの起源
- ADR-TEST-001（Test Pyramid + testcontainers）— L4 の位置付け
- `02_test_layer_responsibility.md`（同章 30_quality_gate）— L4 / L5 責務分界の上位設計
- `01_環境契約.md`（10_owner_suite / 20_user_suite）— 本ファイルの 6 軸を環境構成に展開
