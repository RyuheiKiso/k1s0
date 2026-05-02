# ADR-TEST-009: 観測性 E2E を OTLP trace 貫通 / Prometheus cardinality / log↔trace 結合 / SLO alert 発火 / dashboard goldenfile の 5 検証で構造化し、owner suite に格納する

- ステータス: Proposed
- 起票日: 2026-05-03
- 決定日: -
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / オブザーバビリティ担当（採用初期）

## コンテキスト

前 ADR-TEST-006（撤回済）は観測性 E2E を 5 検証で構造化したが、配置を `tests/e2e/observability/` 単一階層に置き、利用者の 16GB host で起動可能な前提で書かれていた。実際には Grafana LGTM スタック（Mimir / Loki / Tempo / Grafana）と OTel Collector を 16GB host の kind cluster で起動すると、tier1 facade と Dapr を含めた合計で OOM が発生する。本 ADR では検証内容（5 検証）を継続採用しつつ、配置を ADR-TEST-008 の owner / user 二分構造に整合させ、owner suite に格納することで本番再現スタック上での実証経路を確立する。

ADR-OBS-001（Grafana LGTM 採用）/ ADR-OBS-002（OTel Collector）/ ADR-OBS-003（インシデント分類）の 3 つの観測性 ADR は、いずれも「実装かつ機械検証されている」ことが前提で意味を持つ決定である。Grafana LGTM スタックを採用すると決めても、Mimir で metrics が cardinality 爆発しないか / Loki で trace_id を埋めた log が検索できるか / Tempo で OTLP HTTP 経由の span tree が往復するかが継続的に検証されないと、ADR が「机上の決定」のまま実証されない。観測性 E2E はこの実証経路を担う層である。

加えて、ADR-OPS-001（Runbook 標準化）が要求する `runbook_url` 必須要件は、SLO burn rate alert 発火時に runbook URL が確実に含まれることが機械検証されないと履行確認できない。観測性 E2E の検証 4（SLO alert 発火）はこの履行確認の責務を持つ。

選択肢の対立軸は 3 つある。

第一に **検証範囲**。trace 貫通だけ（B 案、最小）/ 5 検証統合（A 案）/ L2 integration test に統合（C 案）/ 観測性 E2E 不実施（D 案、ADR 空洞化）。trace のみではダッシュボード破壊や cardinality regression を検出不能、L2 integration は単一プロセス前提で K8s 上の collector pipeline を検証不可、不実施は ADR-OBS-001 / 002 / 003 を実証なし状態に放置する。

第二に **配置**。owner suite（本 ADR で採用）/ user suite / 共通。利用者の 16GB host で Grafana LGTM フルスタック + tier1 facade + Dapr + OTel Collector + 自アプリ dev は OOM 不可避、user 配置は破綻。共通配置は ADR-TEST-008 の二分構造と矛盾。

第三に **検証 2 以降の射程拡大段階**。リリース時点で全 5 検証完備（A 案）/ 検証 1 のみ skeleton + 残り採用後（B 案）/ skeleton すら不実施（C 案）。リリース時点で全 5 完備は実装工数が爆発、不実施は ADR-OBS-* の空洞化、検証 1 skeleton + 残り採用後の段階拡張が現実的。

設計上の制約と前提:

- ADR-TEST-008 の決定により owner suite は本番再現スタック（multipass + kubeadm + Cilium + Longhorn + MetalLB + フルスタック、48GB host 専用）で動作
- Grafana LGTM スタック（Prometheus / Loki / Tempo / Mimir / Grafana）は ADR-OBS-001 で採用済、owner 環境のフルスタックに含まれる
- OTel Collector は ADR-OBS-002 で採用済、tier1→2→3 を貫通する trace_id 伝播の中継点
- 5 検証の各責務が 1 件ずつ独立した検証類型（trace / metric / log / alert / dashboard）を覆い、orthogonal に並立する
- ADR-OPS-001 の `runbook_url` 必須要件は検証 4（SLO alert）でしか機械検証経路を持たない
- リリース時点では skeleton 配置のみ、採用初期で検証 1 から段階的に real 化する射程

選定では以下を満たす必要がある:

- **ADR-OBS-001 / 002 / 003 の継続検証完備**: 3 つの観測性 ADR が「実装かつ機械検証されている」状態を保つ
- **ADR-OPS-001 の `runbook_url` 必須要件の機械検証**: alert 発火時に runbook URL が含まれることを継続検証
- **ADR-TEST-008 の owner / user 二分との整合**: 配置が `tests/e2e/owner/observability/` で 48GB 専用
- **個人 OSS の実装工数**: リリース時点で skeleton + 採用初期で段階拡張、起案者 1 人で持続可能
- **検証類型の独立性**: 5 検証が orthogonal で、1 件の修正が他に波及しない

## 決定

**観測性 E2E を 5 検証（OTLP trace 貫通 / Prometheus cardinality regression / Loki log↔trace 結合 / SLO burn rate alert 発火 / Grafana dashboard goldenfile）で構造化し、`tests/e2e/owner/observability/` 配下に Go test として配置する。** リリース時点では全 5 検証の skeleton（test 関数の枠 + Skip コメント）を配置、採用初期で検証 1 から real 実装、採用後の運用拡大時で 5 検証完備とする。owner full（不定期実走、ADR-TEST-008）の枠で起動し、CI では走らせない（multipass 制約）。

### 1. 5 検証の責務分界

各検証は独立した観測性類型を覆い、orthogonal に並立する。

| 検証 | 責務 | 検証対象 ADR | 実装ツール |
|---|---|---|---|
| 1. trace 貫通 | tier1→2→3 を貫通する trace_id が OTel Collector → Tempo まで往復する | ADR-OBS-002 | OTLP HTTP `/v1/traces` 送信 → Tempo HTTP API `/api/traces/<trace-id>` で取得、span tree を assert |
| 2. Prometheus cardinality | tier1 / tier2 が出す metric の cardinality が baseline の 1.2 倍を超えない | ADR-OBS-001 | `infra/observability/cardinality/baselines/<metric>.json` と Prometheus `/api/v1/series` の比較 |
| 3. log↔trace 結合 | 一定時間窓の log の 95% 以上に trace_id field が含まれる | ADR-OBS-001 / 002 / 003 | Loki LogQL `{service_name=~"k1s0-.*"} \| trace_id != ""` を全 log に対する比率で評価 |
| 4. SLO alert 発火 | 意図的な SLO 違反（k6 で latency 超過注入）で fast burn alert が 5 分窓内に発火、`runbook_url` 必須 | ADR-OPS-001 / ADR-OBS-003 | k6 で SLO 違反流量を投入 → Alertmanager `/api/v2/alerts` で active alert を取得、`labels.runbook_url` の存在を assert |
| 5. dashboard goldenfile | Grafana dashboard JSON が baseline と一致（panel / query / threshold の不変） | ADR-OBS-001 | `infra/observability/grafana/dashboards/*.json` を `baselines/*.json` と canonical diff |

5 検証は **orthogonal** であり、1 件の修正が他に波及しない構造とする。例えば検証 2（cardinality）の baseline 更新は検証 1（trace 貫通）の test code に影響しない。

### 2. ディレクトリ配置

```text
tests/e2e/owner/observability/
├── trace_propagation_test.go        # 検証 1
├── prometheus_cardinality_test.go   # 検証 2
├── log_trace_correlation_test.go    # 検証 3
├── slo_alert_test.go                # 検証 4
├── dashboard_goldenfile_test.go     # 検証 5
├── baselines/                       # 検証 2 / 5 の baseline
│   ├── cardinality/<metric>.json
│   └── dashboards/<dashboard>.json
└── helpers/
    ├── tempo_client.go              # 検証 1
    ├── prometheus_client.go         # 検証 2
    ├── loki_client.go               # 検証 3
    ├── alertmanager_client.go       # 検証 4
    └── grafana_client.go            # 検証 5
```

`tests/e2e/owner/` 配下に置くことで、ADR-TEST-008 の owner suite 環境契約（multipass + kubeadm + Cilium + Longhorn + MetalLB + Grafana LGTM フルスタック）を直接使う。kind 環境では Grafana LGTM の運用 fidelity（特に Mimir の cardinality 制御 / Tempo の trace TTL）が再現できないため user 配置は採用しない。

### 3. 段階拡張

リリース時点では全 5 検証を **skeleton** として配置（test 関数枠 + `t.Skip("PHASE: release-initial, real impl from 採用初期")` コメント）。実装は段階的に進める。

| Phase | 範囲 | 想定時期 |
|---|---|---|
| リリース時点 | 全 5 検証の skeleton 配置（Skip） | 本 ADR 起票時 |
| 採用初期 | 検証 1（trace 貫通）+ 検証 5（dashboard goldenfile）の real 実装 | 起案者の判断、不定期 |
| 採用後の運用拡大時 | 検証 2 / 3 / 4 の real 実装、`baselines/` 配下の整備 | SRE 増員 |
| 全件 real | 5 検証完備、owner full 実走で全件 PASS が前提 | 採用後の運用拡大時の終盤 |

採用初期で検証 1 / 5 を先行する理由は、(a) 検証 1（trace 貫通）が ADR-OBS-002（OTel Collector）の最小実証経路で他検証の前提、(b) 検証 5（dashboard goldenfile）が他検証類型と独立で実装が孤立的（baseline JSON と diff だけで成立）、の 2 点。検証 2 / 3 / 4 は baseline / SLO 違反注入 script / Alertmanager 整備が前提で実装工数が大きく、SRE 増員後に妥当。

### 4. 起動経路

owner full（ADR-TEST-008 の `make e2e-owner-full`）の subset として `make e2e-owner-observability` で起動する。

```text
make e2e-owner-observability    # tests/e2e/owner/observability/ のみ実行
```

owner full 全件実行時は本 5 検証も含まれる。CI では走らせない（multipass 不可）。結果は `docs/40_運用ライフサイクル/owner-e2e-results.md` に live document として記録（ADR-TEST-008 で新設）。

### 5. baseline 管理

検証 2（cardinality baseline）と検証 5（dashboard goldenfile）の baseline JSON は `tests/e2e/owner/observability/baselines/` 配下に git 管理する。baseline 更新は明示的な commit で行い、commit message に「ADR-TEST-009 baseline 更新: <metric/dashboard 名> / 更新理由」を記載することで、baseline の意図しない drift を防ぐ。`tools/audit/run.sh` で baseline 更新 commit の件数を集計し、AUDIT.md で報告する。

### 6. helpers の責務

`helpers/` 配下の 5 client（tempo / prometheus / loki / alertmanager / grafana）は本 ADR の検証類型ごとに独立した client を提供する。共通機能（HTTP retry / auth header 注入 / context propagation）は `tests/e2e/owner/observability/helpers/common.go` に括り出す。各 client の責務:

- `tempo_client.go`: OTLP HTTP `/v1/traces` 送信、Tempo HTTP API での span tree 取得
- `prometheus_client.go`: Prometheus `/api/v1/series` / `/api/v1/labels` での series / label 数取得
- `loki_client.go`: Loki LogQL での log 検索、trace_id field 比率計算
- `alertmanager_client.go`: Alertmanager `/api/v2/alerts` で active alert 取得、label 抽出
- `grafana_client.go`: Grafana HTTP API で dashboard JSON 取得（authenticated）

helpers は `tests/e2e/owner/` 内に閉じ、`src/sdk/<lang>/test-fixtures/`（ADR-TEST-010）には公開しない（観測性検証は owner 専用、利用者は触らない）。

## 検討した選択肢

### 選択肢 A: 5 検証統合 + owner suite 配置 + 段階拡張（採用）

- 概要: trace 貫通 / cardinality / log↔trace / SLO alert / dashboard の 5 検証を独立 Go test として `tests/e2e/owner/observability/` に配置、リリース時点 skeleton + 採用初期 / 運用拡大時で段階 real 化
- メリット:
  - 5 検証類型（trace / metric / log / alert / dashboard）が orthogonal に並立し、1 件修正で他に波及しない構造
  - ADR-OBS-001 / 002 / 003 / OPS-001 の決定が継続検証され、空洞化が防がれる
  - owner suite 配置で本番再現スタック上の運用 fidelity が直接検証される
  - 段階拡張で個人 OSS の実装工数を吸収（リリース時点 skeleton、real 化は SRE 増員後）
  - 検証 5（dashboard goldenfile）は本検証類型でしか実装できない独立価値（panel / query / threshold の不変を機械検証）
- デメリット:
  - 5 検証の skeleton コードが本 ADR 起票時に物理配置される一方、real 実装は SRE 増員を待つ構造のため、リリース直後はコードが Skip のまま長期間留まる状態が発生する（mitigation: AUDIT.md で skeleton 件数を報告、採用初期での着手を可視化、起案者の判断で着手契機を明示）
  - baseline JSON（cardinality / dashboard）の更新ルールが緩いと意図しない drift を生む（mitigation: baseline 更新は commit message で明示的に正当化、`tools/audit/run.sh` で件数監視）

### 選択肢 B: trace 貫通のみ（最小）

- 概要: 検証 1（trace 貫通）のみを `tests/e2e/owner/observability/` に配置、残 4 検証は不実施
- メリット:
  - 実装工数が最小（1 検証のみ）
  - リリース時点で real 実装まで完備可能
- デメリット:
  - **観測性 ADR の継続検証が片肺**: ADR-OBS-001（Mimir cardinality）/ ADR-OBS-003（インシデント分類）/ ADR-OPS-001（runbook_url）が機械検証されない
  - cardinality 爆発による Mimir パフォーマンス劣化が検出不能
  - dashboard 破壊（panel 削除 / query 変更）が検出不能で、採用組織が dashboard を頼った時に発覚
  - trace 貫通のみだと「観測性 ADR が動いている」の根拠として弱い

### 選択肢 C: L2 integration test に統合

- 概要: 観測性検証を L2 integration test（`tests/integration/`、testcontainers ベース、ADR-TEST-001）に統合
- メリット:
  - L2 で testcontainers を使えば Grafana LGTM 起動コストが軽い（PR で機械検証可能）
  - PR 毎に観測性検証が走る、検出が早い
- デメリット:
  - **L2 defining property（同一プロセス）を超える**: ADR-TEST-001 で L2 は「testcontainers + 単一プロセスの結合層」と定義済、観測性検証は K8s 上の collector pipeline を貫通する E2E 性質
  - L2 の 3 分予算（ADR-TEST-001）を観測性スタック起動 + 5 検証で超過、L2 全体の時間予算が破綻
  - testcontainers 上の Grafana LGTM は本番運用 fidelity が低く、owner full の本番再現スタックで再検証が必要 → 二重検証で工数増
  - kind / multipass の本番再現スタック上でしか検出できない fidelity 問題（cardinality / dashboard 破壊）が L2 では検出不能

### 選択肢 D: 観測性 E2E 不実施

- 概要: 観測性 E2E を整備せず、ADR-OBS-001 / 002 / 003 の決定を「採用組織の責務」として委ねる
- メリット:
  - 実装工数ゼロ
  - 起案者の運用工数がゼロ
- デメリット:
  - **ADR-OBS-001 / 002 / 003 の決定が実装かつ未検証状態で放置**: Grafana LGTM が「動いているか」の根拠が CI で取れず、ADR が空洞化
  - ADR-OPS-001 の `runbook_url` 必須要件が機械検証されず、Runbook URL 漏れが採用組織側でしか検出されない
  - 採用検討者が「k1s0 は観測性スタックを採用したと言うが、実証なし」と判定し、testing maturity 評価が低下
  - dashboard 破壊が CI で検出されず、採用組織が dashboard を頼った時に「panel が消えている」状態で発覚

### 選択肢 E: user suite に配置

- 概要: 観測性検証を `tests/e2e/user/observability/` に配置し、kind 上で実行
- メリット:
  - kind なので CI で機械検証可能、PR / nightly で検出が早い
  - 利用者が自アプリ開発時に観測性検証を共有できる
- デメリット:
  - **16GB host で OOM**: Grafana LGTM フルスタック + tier1 facade + Dapr + OTel Collector + 自アプリ dev は 16GB を超過し、利用者の 16GB host で起動不可
  - kind 上の Mimir は本番運用 fidelity が低く（特に長期 retention / cardinality 制御）、cardinality regression の検出能力が落ちる
  - ADR-TEST-008 の owner / user 二分（user = 自アプリ動作確認）と矛盾、観測性 ADR の継続検証は OSS 完成度検証（owner）の責務

## 決定理由

選択肢 A（5 検証統合 + owner suite 配置 + 段階拡張）を採用する根拠は以下。

- **ADR-OBS-001 / 002 / 003 / OPS-001 の同時履行**: 5 検証が ADR-OBS の 3 つと ADR-OPS-001 の `runbook_url` 必須要件を同時に機械検証する。選択肢 B（trace のみ）は片肺、選択肢 C（L2 統合）は本番運用 fidelity 不足、選択肢 D（不実施）は ADR 空洞化、選択肢 E（user 配置）は 16GB OOM
- **ADR-TEST-008 の owner / user 二分との整合**: 観測性検証は OSS 完成度検証（owner）の責務であり、user suite では本番運用 fidelity が取れない。owner 配置が ADR-TEST-008 の決定と直接整合
- **検証類型の orthogonal 並立**: 5 検証が独立した観測性類型（trace / metric / log / alert / dashboard）を覆い、1 件の修正が他に波及しない。選択肢 B（1 検証）は orthogonal 性が成立しない（1 件のみで何の orthogonal か議論できない）
- **段階拡張の現実性**: リリース時点で skeleton + 採用初期で検証 1 / 5 + 採用後の運用拡大時で 5 検証完備、という段階が個人 OSS の実装工数に整合。選択肢 A 全件 real 化を即時要求すると baseline / SLO 違反注入 script / Alertmanager 整備で工数爆発
- **dashboard goldenfile の独立価値**: 検証 5 は本検証類型でしか実装できない（dashboard 破壊は他のテスト類型では検出不能）。選択肢 B / D ではこの独立価値を取りこぼす
- **退路の確保**: 選択肢 A は将来の観測性 ADR 改訂（例: OpenTelemetry Profiling 追加）で 6 検証目を追加する余地を持つ。各検証が独立 Go test なので、追加で他検証に波及しない

## 影響

### ポジティブな影響

- ADR-OBS-001（Grafana LGTM）/ ADR-OBS-002（OTel Collector）/ ADR-OBS-003（インシデント分類）/ ADR-OPS-001（runbook_url 必須）の 4 つの ADR 決定が継続検証され、空洞化が防がれる
- 観測性検証が owner suite 配置で本番再現スタック上の運用 fidelity を直接検証
- 5 検証の orthogonal 並立で 1 件の修正が他に波及しない構造、保守性が高い
- 検証 5（dashboard goldenfile）が dashboard 破壊を機械検出可能にし、採用組織の dashboard 利用体験を保護
- 段階拡張により個人 OSS の実装工数が吸収され、リリース時点 skeleton で着手経路が確保される
- baseline 管理（commit message + AUDIT.md 監視）で意図しない drift が防がれる

### ネガティブな影響 / リスク

- 5 検証の skeleton コードがリリース時点で物理配置される一方、real 実装は SRE 増員を待つ構造のため、リリース直後はコードが Skip のまま長期間留まる状態が発生する（mitigation: AUDIT.md で skeleton 件数 / Skip 件数を報告、採用初期での着手契機を可視化、起案者の判断で着手契機を明示）
- baseline JSON（cardinality / dashboard）の更新が緩いと意図しない drift を生む（mitigation: baseline 更新 commit message での明示的正当化、`tools/audit/run.sh` での件数監視）
- 観測性 E2E は CI で走らない（owner suite、multipass 不可）ため、観測性スタックの破綻が release tag 切る時の owner full PASS まで検出されない（mitigation: ADR-TEST-011 の release tag ゲートで release 前必ず owner full PASS が要求される）
- 検証 4（SLO alert）の SLO 違反注入 script（k6 でラップ）が脆く、alert が「期待通り」発火するか「環境依存で」発火するかの切り分けが難しい場合がある（mitigation: SLO 違反注入 script を helpers/ 内に固定化、alert 発火閾値を baseline JSON で管理）
- helpers 5 client の保守工数が継続発生（Tempo / Prometheus / Loki / Alertmanager / Grafana の API 変更追従、mitigation: 各 OSS が API 互換性を高く保つ、k1s0 は LTS バージョンを `tools/local-stack/` で固定）

### 移行・対応事項

- `tests/e2e/owner/observability/` ディレクトリを新設、5 検証 skeleton（test 関数枠 + Skip）+ helpers/ 5 client + baselines/ ディレクトリを配置
- `Makefile` に `e2e-owner-observability` target を追加（ADR-TEST-008 の `make e2e-owner-*` 系列に追加）
- `tools/local-stack/up.sh --role owner-e2e` で Grafana LGTM スタック（Prometheus / Loki / Tempo / Mimir / Grafana / Alertmanager）+ OTel Collector が起動することを検証（既存 ADR-OBS-001 / 002 のフルスタック起動経路を踏襲）
- `tools/audit/run.sh` に baseline 更新 commit 件数の集計ロジックを追加し、AUDIT.md で報告
- `docs/40_運用ライフサイクル/owner-e2e-results.md` の月次サマリ template に「観測性 5 検証 PASS 数」を追加（ADR-TEST-008 で新設される doc に追記）
- `ADR-OBS-001 / 002 / 003` の「関連 ADR」セクションに本 ADR を追加（cross-reference 整備）
- `docs/03_要件定義/00_要件定義方針/08_ADR索引.md` の `ADR-TEST-006（撤回 / 再策定予定）` entry を本 ADR を cite する形に更新
- `docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` に本 ADR の対応 IMP-CI を追記
- 採用初期で検証 1（trace 貫通）と検証 5（dashboard goldenfile）の real 実装を起案者が着手、`docs/40_運用ライフサイクル/owner-e2e-results.md` に PASS 記録
- 採用後の運用拡大時で検証 2 / 3 / 4 の real 実装を SRE 増員後に着手

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— L2 integration と本 ADR の責務分界
- ADR-TEST-008（e2e owner / user 二分構造、別 commit で起票）— 本 ADR の配置基盤
- ADR-TEST-010（test-fixtures 4 言語 SDK 同梱、別 commit で起票予定）— 観測性 helpers は本 ADR で owner 内に閉じる根拠
- ADR-TEST-011（release tag ゲート代替保証、別 commit で起票予定）— 本 ADR の CI 不可を補完
- ADR-OBS-001（Grafana LGTM）— 検証 2 / 5 の対象
- ADR-OBS-002（OTel Collector）— 検証 1 の対象
- ADR-OBS-003（インシデント分類）— 検証 4 の SLO 分類軸
- ADR-OPS-001（Runbook 標準化）— 検証 4 の `runbook_url` 必須要件
- `infra/observability/grafana/dashboards/` — 検証 5 の baseline 元ファイル
- 関連 ADR（採用検討中）: ADR-TEST-004（Chaos Engineering）/ ADR-TEST-005（Upgrade / DR drill）/ ADR-TEST-007（テスト属性タグ + 実行フェーズ分離）
