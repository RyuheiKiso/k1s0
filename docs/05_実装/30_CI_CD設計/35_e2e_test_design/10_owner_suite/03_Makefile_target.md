# 03. owner suite Makefile target

本ファイルは ADR-TEST-008 で確定した `make e2e-owner-*` 8 target の実装契約を実装段階の正典として固定する。各 target の起動経路（`tools/e2e/owner/up.sh` 専用スクリプト連携）、cluster ライフサイクル（起動 / 部分実行 / cleanup）、go test 引数、artifact 出力先を ID として採番する。

## 本ファイルの位置付け

ADR-TEST-008 で「make e2e-owner-* 系列の 8 target を Makefile に追加する」と決定したが、各 target の cluster 起動責務（cluster を毎回新規起動するのか、既存 cluster を再利用するのか）と部分実行時の挙動が未確定だった。本ファイルでは 8 target の起動経路を決定し、cluster 起動コスト（約 30 分）を部分実行で繰り返さない設計を採る。

## 8 target の責務

| target | 起動範囲 | go test target | 想定所要時間 |
|---|---|---|---|
| `make e2e-owner-full` | cluster 新規起動 + 全 8 部位実行 + cleanup | `./...` | 約 1 時間 45 分 |
| `make e2e-owner-platform` | cluster 既存利用 + platform/ のみ | `./platform/...` | 約 8 分 |
| `make e2e-owner-observability` | cluster 既存利用 + observability/ のみ | `./observability/...` | 約 5 分 |
| `make e2e-owner-security` | cluster 既存利用 + security/ のみ | `./security/...` | 約 5 分 |
| `make e2e-owner-ha-dr` | cluster 既存利用 + ha-dr/ のみ | `./ha-dr/...` | 約 15 分 |
| `make e2e-owner-upgrade` | cluster 既存利用 + upgrade/ のみ | `./upgrade/...` | 約 30 分 |
| `make e2e-owner-sdk-roundtrip` | cluster 既存利用 + sdk-roundtrip/ のみ | `./sdk-roundtrip/...` | 約 12 分 |
| `make e2e-owner-tier3-web` | cluster 既存利用 + tier3-web/ のみ | `./tier3-web/...` | 約 8 分 |
| `make e2e-owner-perf` | cluster 既存利用 + perf/ のみ | `./perf/...` | 約 10 分 |

`make e2e-owner-full` のみ cluster の起動 + cleanup を含み、他 7 target は **既存 cluster を前提** とする。これは multipass × 5 + kubeadm の起動が約 30 分かかるため、部分実行のたびに cluster を再起動すると test 時間より起動時間が支配的になる問題への対処である。

## cluster ライフサイクルの取り扱い

owner suite の cluster は以下の 3 段階で扱う。

```text
1. 起動: tools/e2e/owner/up.sh
   → multipass × 5 起動 + kubeadm init/join + Cilium/Longhorn/MetalLB/フルスタック install
   （フルスタック install は tools/local-stack/install/<component>/ を helper として再利用）
2. 部分実行: make e2e-owner-{部位} → 既存 cluster で go test 実行（cluster は触らない）
3. 終了: tools/e2e/owner/down.sh
   → multipass delete × 5（VM ごと削除）
```

`make e2e-owner-full` は 1 → 各部位順次実行 → 3 を 1 シーケンスで実行する。部分実行を繰り返したい場合は 1 を手動実行 → make e2e-owner-{部位} を任意回数 → 完了時に手動で 3、を Runbook で案内する。

cluster の状態を意図せず改変する test（例: ha-dr/ で control-plane を kill する）は、test 実行後に必ず cluster 状態を回復させる responsibility を test 側が負う。回復不能な状態にした場合は test を `t.Fatal` で終了させ、Makefile target も exit 1 を返す。利用者は `down.sh` → `up.sh` で cluster を再構築する。

## go test の引数規約

各 target は以下の go test 引数で実行する。

```text
cd tests/e2e/owner && \
  go test \
    -tags=owner_e2e \
    -timeout=N分 \           # 部位ごとに調整（platform=10m, ha-dr=20m, upgrade=40m）
    -v \                      # 詳細 log（artifact 用）
    -json \                   # 機械可読出力（owner-e2e-results.md 集計用）
    -count=1 \                # cache 無効化（毎回 fresh 実行）
    ./{部位}/... \
    | tee {部位}_result.json
```

`-tags=owner_e2e` build tag で owner suite 専用 test であることを表現する。これにより user suite の go test と build tag が衝突しない。`-json` 出力を `owner-e2e-results.md` の自動更新 script（採用初期で整備）が parse する。

## artifact 出力先

各 target が出力する artifact は `tests/.owner-e2e/<YYYY-MM-DD>/{部位}/` 配下に集約する。本構造は ADR-TEST-011 の release tag ゲート（cut.sh が sha256 検証する対象）と整合する。

```text
tests/.owner-e2e/<YYYY-MM-DD>/
├── full-result.tar.zst                 # make e2e-owner-full 完了時に全部位を tar.zst 化
├── cluster-info.txt                    # kubectl version / nodes / get all -A / Cilium / Longhorn / MetalLB status
├── dmesg.txt                           # 実走中の OOM / kernel error 監視ログ
├── platform/
│   ├── result.json
│   └── stdout.log
├── observability/
│   ├── result.json
│   ├── trace_propagation.json          # 検証 1 の Tempo span tree dump
│   └── ...
├── security/
├── ha-dr/
├── upgrade/
├── sdk-roundtrip/
├── tier3-web/
│   ├── result.json
│   └── screenshots/                    # chromedp で取得した failure screenshot
└── perf/
    ├── result.json
    └── k6_summary.json
```

`<YYYY-MM-DD>` は実走日。同日に複数回実行した場合は `<YYYY-MM-DD>-<HHMM>` でディレクトリを分ける。古い artifact は git LFS で 12 ヶ月版管理（`02_artifact_保管.md` 参照）。

## 部分実行時の前提確認

`make e2e-owner-{部位}` 実行時、Makefile target が **cluster 存在確認** を行い、未起動なら exit 1 で `tools/e2e/owner/up.sh` を案内する。これは「cluster がない状態で test を走らせて謎の失敗で 5 分浪費する」事故の予防策である。

```makefile
e2e-owner-platform:
	@./tools/e2e/owner/check.sh || \
	  (echo "[error] owner cluster が起動していません。先に './tools/e2e/owner/up.sh' を実行してください。" && exit 1)
	@cd tests/e2e/owner && go test -tags=owner_e2e -timeout=10m -v -json -count=1 ./platform/... | tee ../../.owner-e2e/$(shell date +%Y-%m-%d)/platform/result.json
```

`tools/e2e/owner/check.sh` は kubeconfig context が `k1s0-owner-e2e` であること + 全 5 VM が Running + 全 node が Ready を確認する。

## IMP ID

| ID | 内容 | 配置 |
|---|---|---|
| IMP-CI-E2E-006 | make e2e-owner-* 8 target 実装契約 | 本ファイル |

## 対応 ADR / 関連設計

- ADR-TEST-008（e2e owner / user 二分構造）— Makefile target の起源
- `01_環境契約.md`（同章）— cluster 起動の前提環境
- `02_ディレクトリ構造.md`（同章）— go test target の `./{部位}/...` の根拠
- `04_観測性5検証.md`（同章）— `make e2e-owner-observability` の test 内容
- `40_release_tag_gate/02_artifact_保管.md` — artifact 保管経路
- `tools/local-stack/up.sh` / `down.sh` / `check.sh` — 実装本体
