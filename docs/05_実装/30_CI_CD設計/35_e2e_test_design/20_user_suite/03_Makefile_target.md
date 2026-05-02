# 03. user suite Makefile target

本ファイルは ADR-TEST-008 で確定した `make e2e-user-{smoke,full}` の実装契約を実装段階の正典として固定する。各 target の起動経路（`tools/e2e/user/up.sh` 専用スクリプト連携）、kind cluster ライフサイクル、go test 引数、artifact 出力先を ID として採番する。

## 本ファイルの位置付け

user suite の Makefile target は owner suite と異なり、cluster 起動コストが小さい（kind 約 60 秒 vs multipass 30 分）ため、**target 毎に cluster を再起動する設計** を採る。これにより test 間の cluster 状態汚染が起きず、利用者が PR で 1 度走らせるたびに毎回 fresh cluster で test される。owner suite が cluster を共有再利用していた構造とは正反対。

## 2 target の責務

| target | 起動範囲 | go test target | 想定所要時間 |
|---|---|---|---|
| `make e2e-user-smoke` | kind 起動 + minimum stack install + smoke/ のみ + cleanup | `./smoke/...` | 約 3〜5 分 |
| `make e2e-user-full` | kind 起動 + minimum stack + 任意 stack（test ごと）+ smoke/ + examples/ 全件 + cleanup | `./smoke/...` `./examples/...` | 約 30〜45 分 |

`make e2e-user-smoke` は PR 5 分予算（ADR-TEST-001）に収まる軽量検証。`make e2e-user-full` は nightly cron で全件実走する。両者とも cluster 起動 → test → cleanup の 1 サイクルで完結する。

## kind cluster ライフサイクルの取り扱い

user suite は cluster 起動コストが小さいため、target 毎に cluster を再起動する。

```text
1. 起動: tools/e2e/user/up.sh
   → kind create cluster + Calico install + minimum stack（Dapr / tier1 facade / Keycloak / 1 backend）install
   （minimum stack install は tools/local-stack/install/<component>/ を helper として再利用）
2. test 実行: cd tests/e2e/user && go test ...
3. 終了: tools/e2e/user/down.sh
   → kind delete cluster
```

target は 1 → 2 → 3 を 1 シーケンスで実行する。途中で test code を変更して再実行する場合、`KEEP_CLUSTER=1 make e2e-user-smoke` で cluster を再利用する経路を提供する（開発者の iteration 効率向上のため）。

任意 stack（Workflow / Decision / Strimzi 等）は test 内で `testfixtures.Setup` の AddOns 引数で指定する（ADR-TEST-010）。Makefile target レベルでは minimum stack のみを起動し、task 毎の stack 追加は test code に委ねる。これは task 別の stack 要求が動的なため、Makefile で静的に決められない設計。

## go test の引数規約

```text
cd tests/e2e/user && \
  go test \
    -tags=user_e2e \
    -timeout=N分 \           # smoke=10m, full=45m
    -v \                      # 詳細 log
    -json \                   # 機械可読出力（CI artifact + user-e2e-results.md 集計用）
    -count=1 \                # cache 無効化
    -parallel=4 \             # examples/ は並列実行可能
    ./{部位}/... \
    | tee {部位}_result.json
```

`-tags=user_e2e` build tag で user suite 専用 test を表現する（owner_e2e との衝突回避）。`-parallel=4` は user suite では並列実行を許容（examples/ の各 test は独立 namespace で動くため衝突しない）。owner suite の HA / DR test は順次実行が必要だが、user suite はそのような制約がない。

## artifact 出力先

CI で実行する場合、artifact は GitHub Actions の `actions/upload-artifact@v4` で 14 日保管する。local 実行時は `tests/.user-e2e/<YYYY-MM-DD>/{部位}/` 配下に出力する。

```text
tests/.user-e2e/<YYYY-MM-DD>/   (local)
  または
artifacts/user-e2e/<run-id>/    (CI)
├── smoke/
│   ├── result.json
│   └── stdout.log
└── examples/
    ├── result.json
    ├── stdout.log
    ├── tier2-go-service-pod.log     # 失敗時の Pod log dump
    ├── tier3-web-portal-screenshots/ # 失敗時の screenshot（chromedp は使わない、利用者は Playwright fixtures、本 suite は Pod 経路のみ）
    └── ...
```

local 実行時の artifact は git LFS 管理ではない（user suite は CI で機械検証されるため、local artifact は ad-hoc）。

## CI 統合

`make e2e-user-{smoke,full}` は CI workflow（`_reusable-e2e-user.yml`、ADR-TEST-008 で新設）からも呼ばれる。CI と local で同 Makefile target を呼ぶことで、「local で動いたが CI で落ちる」/「CI で通ったが local で再現できない」が原理的に発生しない。詳細は `04_CI戦略.md`。

```text
# .github/workflows/_reusable-e2e-user.yml の core step
- name: run user e2e
  run: make e2e-user-smoke    # PR 経由
# または
- name: run user e2e full
  run: make e2e-user-full     # nightly 経由
```

## test 失敗時の挙動

`make e2e-user-{smoke,full}` 実行中に go test が失敗した場合、Makefile target は exit 1 を返す。cleanup（`down.sh`）は exit code 関係なく実行される（trap で）。

```makefile
e2e-user-smoke:
	@./tools/e2e/user/up.sh
	@trap './tools/e2e/user/down.sh' EXIT; \
	  cd tests/e2e/user && go test -tags=user_e2e -timeout=10m -v -json -count=1 ./smoke/... | tee ../../.user-e2e/$(shell date +%Y-%m-%d)/smoke/result.json
```

cleanup を確実に実行することで「test 失敗 + cluster 残存」状態を防ぐ。利用者の host 環境を清潔に保つ責務を Makefile target が負う。

## IMP ID

| ID | 内容 | 配置 |
|---|---|---|
| IMP-CI-E2E-007 | make e2e-user-{smoke,full} 実装契約 | 本ファイル |

## 対応 ADR / 関連設計

- ADR-TEST-008（e2e owner / user 二分構造）— Makefile target の起源
- `01_環境契約.md`（同章）— cluster 起動の前提環境
- `02_ディレクトリ構造.md`（同章）— go test target の `./{部位}/...` の根拠
- `04_CI戦略.md`（同章）— CI workflow からの呼び出し
- `tools/local-stack/up.sh` / `down.sh` — 実装本体
