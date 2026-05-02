# 02. owner full e2e artifact 保管（git LFS 12 ヶ月）

本ファイルは ADR-TEST-011 で確定した owner full e2e の artifact 保管規約を実装段階の正典として固定する。`tests/.owner-e2e/` のディレクトリ構造、git LFS による 12 ヶ月版管理、古い artifact の cold storage 移行経路、`.gitattributes` 設定を ID として採番する。

## 本ファイルの位置付け

ADR-TEST-011 で release tag ゲートが artifact sha256 を検証することを確定した。artifact が repo 内で安定的に取得可能でなければ cut.sh の検証経路が成立しないため、本ファイルでは git LFS による artifact 保管・retention・cleanup の運用契約を固定する。ADR-TEST-003 の conformance results 12 ヶ月管理と同パタンを踏襲する。

## ディレクトリ構造

owner full の artifact は実走日ごとにディレクトリを切る。同日複数回実行時は時刻 suffix を付ける。

```text
tests/.owner-e2e/
├── 2026-05-15/                               # 標準（1 日 1 実走）
│   ├── full-result.tar.zst                   # 全 8 部位の go test JSON + log
│   ├── cluster-info.txt                      # kubectl version / nodes / get all -A
│   ├── dmesg.txt                             # OOM / kernel error 監視ログ
│   ├── platform/result.json
│   ├── observability/result.json
│   ├── observability/trace_propagation.json  # 検証 1 の Tempo span tree
│   ├── observability/cardinality_diff.json   # 検証 2 の baseline 差分
│   ├── security/result.json
│   ├── ha-dr/result.json
│   ├── upgrade/result.json
│   ├── sdk-roundtrip/result.json
│   ├── tier3-web/result.json
│   ├── tier3-web/screenshots/                # chromedp 取得 failure screenshot
│   └── perf/result.json
├── 2026-05-15-1430/                          # 同日 2 回目（時刻 suffix）
│   └── ...
└── 2026-06-01/
    └── ...
```

artifact の中身は go test の `-json` 出力 + 各部位 helper が出す補助ファイル（screenshot / k6 summary / span tree dump 等）。`full-result.tar.zst` は全部位を tar.zst で集約した archive で、cut.sh の sha256 検証はこのファイルを対象とする。

## git LFS 設定

`tests/.owner-e2e/` 配下の binary / 大容量ファイルは git LFS で管理する。`.gitattributes` に以下を追加:

```text
# tests/.owner-e2e/ artifact (ADR-TEST-011)
tests/.owner-e2e/**/*.tar.zst       filter=lfs diff=lfs merge=lfs -text
tests/.owner-e2e/**/*.tar.gz        filter=lfs diff=lfs merge=lfs -text
tests/.owner-e2e/**/screenshots/*.png  filter=lfs diff=lfs merge=lfs -text
tests/.owner-e2e/**/*.log           filter=lfs diff=lfs merge=lfs -text
```

JSON / txt の小ファイル（result.json / cluster-info.txt / dmesg.txt）は通常 git 管理（diff 可能性 + LFS quota 節約）。binary ファイルのみ LFS 管理する。

## retention（12 ヶ月版管理）

artifact は **12 ヶ月分** を git LFS で保管する。13 ヶ月以上の古い artifact は git LFS から release asset への昇格 + cold storage 移行する（採用初期の Runbook で整備）。

理由:

- 12 ヶ月 = release cycle の 3〜4 周分（release を 3〜4 ヶ月ごとに想定）+ 採用組織の audit 期間
- 12 ヶ月超は git LFS の容量を圧迫（年間 12 entry × 約 100MB = 1.2GB、5 年で 6GB）
- 古い artifact は採用組織の audit 用途以外で使われない

retention 管理の Runbook は `ops/runbooks/RB-CI-002-owner-e2e-artifact-rotation.md`（採用初期で新設）で確定する。リリース時点では 12 ヶ月超の artifact が発生していないため、運用課題として明示するのみ。

## artifact 生成手順（make e2e-owner-full と整合）

`make e2e-owner-full` 実行完了時、artifact 生成 + sha256 計算 + owner-e2e-results.md 更新を 1 sequence で行う。

```bash
# Makefile の概略
e2e-owner-full:
	@./tools/e2e/owner/up.sh
	@trap './tools/e2e/owner/down.sh' EXIT; \
	  cd tests/e2e/owner && \
	  go test -tags=owner_e2e -timeout=120m -v -json -count=1 ./... \
	    | tee ../../.owner-e2e/$(shell date +%Y-%m-%d)/full-result.json
	@./tools/qualify/owner-e2e/archive.sh $(shell date +%Y-%m-%d)
	@./tools/qualify/owner-e2e/update-results.sh $(shell date +%Y-%m-%d)
```

`tools/qualify/owner-e2e/archive.sh` の責務:

1. `tests/.owner-e2e/<YYYY-MM-DD>/` 配下の全 file を `full-result.tar.zst` に集約
2. `sha256sum full-result.tar.zst` で sha256 を計算
3. `cluster-info.txt` を生成（kubectl version / nodes / get all -A / Cilium / Longhorn / MetalLB status）
4. `dmesg.txt` を生成（実走中の OOM / kernel log）

`tools/qualify/owner-e2e/update-results.sh` の責務:

1. `docs/40_運用ライフサイクル/owner-e2e-results.md` に新 entry を追記
2. entry の `artifact sha256` フィールドを archive.sh の出力で埋める
3. 各部位の PASS / FAIL を集計して entry に記載
4. git add で artifact + md 更新を staging（commit は手動）

実装は採用初期で `tools/qualify/owner-e2e/` 配下に配置する（リリース時点では skeleton 配置）。

## artifact の検証経路

cut.sh は release tag 切る時に artifact を検証する（ADR-TEST-011 の step 5）。検証経路:

1. `owner-e2e-results.md` の最新 entry から `artifact sha256` を抽出
2. `tests/.owner-e2e/<日付>/full-result.tar.zst` を git LFS から pull
3. 実 sha256sum と記録 sha256 が一致することを assert
4. 不一致なら exit 1

検証は git LFS pull が完了している前提のため、CI / cut.sh 実行環境で `git lfs install` + `git lfs pull` が事前に走っている必要がある。`tools/release/cut.sh` は実行開始時に LFS 状態を確認し、pull が必要なら自動で pull する。

## artifact のサイズ管理

owner full の 1 回実走で生成される artifact のサイズは推定 100〜200MB（go test JSON + screenshots + cluster info + log）。月 1 回実走なら年 12 entry × 200MB = 2.4GB の年間 LFS 消費。GitHub free tier の LFS storage 制限（1GB）を超えるため、採用初期で paid LFS plan に移行するか、release asset 昇格運用を整備する。

artifact の圧縮率を上げる経路:

- `full-result.tar.zst` は zstd level 19（high compression）で圧縮、未圧縮比 60-70% 削減
- screenshots は PNG → WebP 変換で約 30% 削減（採用初期で検討）
- log は gzip level 9 で約 80% 削減（zstd 集約で十分）

リリース時点では zstd level 19 のみ採用、他の最適化は採用初期での課題として明示。

## IMP ID

| ID | 内容 | 配置 |
|---|---|---|
| IMP-CI-E2E-015 | tests/.owner-e2e/ ディレクトリ + git LFS 12 ヶ月管理 | 本ファイル |

## 対応 ADR / 関連設計

- ADR-TEST-011（release tag ゲート代替保証）— 本ファイルの起源
- ADR-TEST-003（CNCF Conformance / Sonobuoy）— 12 ヶ月版管理の同パタン
- `01_cut_sh_拡張.md`（同章）— artifact sha256 検証経路
- `03_owner_e2e_results_template.md`（同章）— entry の artifact sha256 フィールド
- `10_owner_suite/03_Makefile_target.md` — `make e2e-owner-full` の artifact 生成
- `tools/qualify/owner-e2e/archive.sh`（採用初期で新設）— artifact 集約
- `tools/qualify/owner-e2e/update-results.sh`（採用初期で新設）— md 更新
- `ops/runbooks/RB-CI-002-owner-e2e-artifact-rotation.md`（採用初期で新設）— retention 運用
