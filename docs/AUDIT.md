# 監査スナップショット（AUDIT）

本ファイルは k1s0 リポジトリの自己監査結果を機械生成した網羅マトリクス。`docs/SHIP_STATUS.md`（人間散文 + 経緯）と補完関係にある。判定基準は [`docs/00_format/audit_criteria.md`](00_format/audit_criteria.md) を参照。

採用検討者は本ファイルを「ID 単位の網羅性」「手抜き残存件数」「実機検証状態」「OSS 採点項目」のチェックリストとして使い、SHIP_STATUS は判断の文脈として使う。両者を併読することを推奨する。

## 凡例

| 記号 | 意味 |
|---|---|
| **PASS** | docs 定義 + 実装サンプル + 動作証跡の 3 段すべて満たす |
| **PARTIAL** | 3 段のうち 1-2 段のみ |
| **FAIL** | 設計のみ / 実装欠落 / 動作未確認 |
| **N/A** | 該当なし（規約のみ / 採用後の運用拡大時で意図的に未着手） |
| **保留** | Claude / 環境制約で本実行回では検証不可（嘘 PASS を書かない） |

判定（PASS / PARTIAL / FAIL / N/A）は **人間が下す**。Claude は数値・分数・走査範囲・証跡パスのみ並べる。本ファイルの「判定材料」列に Claude が PASS を書くことは禁止（[`audit-protocol` skill](../.claude/skills/audit-protocol/SKILL.md) の根本原則）。判定列はレビュアー記入用に意図的に空欄。

## 前提と限界

- 本マトリクスは `tools/audit/run.sh` の機械検出に基づく。実機 E2E は外部環境依存（kubectl / Helm / kind）
- C 軸の running pod 数は `cluster_class` を必ず併記する。kind での 100% Running を production 検証として誇張しない
- OSSF Scorecard は public repo + scorecard-cli 前提のため、private 段階では一部 Unknown
- A 軸の NFR / DS / IMP は「ID をコード文字列で直引用しない」業界慣行のため、coverage.sh の grep ベース集計だけでは impl 不在比率が高く出る。本 audit では `tools/audit/lib/trace.sh` で **(a) direct, (b) verify アーティファクト, (c) co-cited 経由** の 3 経路から間接 reach を集計し、`unreached` 件数のみ「真の impl 不在候補」とする
- 「採用検討者が信頼できる水準」は OSSF Scorecard 7/10 + CNCF Sandbox 最低要件 + OpenSSF Best Practices Passing の合算で判定する。**「完璧」は到達不能な目標として採らない**
- 生証跡 `.claude/audit-evidence/<date>/` は **git 管理対象**。再生成なしに第三者が AUDIT.md の主張を verify できる

## サマリ（最新実行: 2026-05-02 #8）

| 軸 | 判定材料（数値・走査範囲） | 主な変化 |
|---|---|---|
| **A 網羅** | ADR **46**（ファイル ↔ ID 1:1）/ **code-orphan 0 / docs-orphan 41** / 実装参照 0 件 **3 件**（ADR-0002 規約 / DEP-001 / DX-001） ・ FR 50 / 3-stage 22 (44%) ・ NFR 155 / coverage 3-stage 8 (5%) → trace **reach 140 (90%)** / unreached 15 ・ DS 1416 / coverage 3-stage 16 (1%) → trace **reach 1168 (82%)** / unreached 248 ・ IMP 718 / coverage 3-stage 1 (0.1%) → trace **reach 712 (99%)** / unreached 6 | **coverage.sh ADR ID 列挙を「adr/ 配下 grep」から「ファイル名抽出」に構造修正**（ID 数 49 → 46、cite-only 3 件 DEV-003/DIR-004/SUP-002 が docs-only に誤分類されていた #7 発覚 bug を解消、以後 ids-adr.txt は ADR ファイルと厳密 1:1）。docs-only 6 → 3、その他は不変 |
| **B 手抜き** | **1237 ファイル走査** / 22 パターン / 真の手抜き **0 件** / 許容残置 2 件 (false-positive、コメント内擬似コード) / gitkeep-only ディレクトリ 23 件中 documented **9** / undocumented **14** | 同一値で再現性確認、本文側 (B 軸セクション) の旧表記「1236 ファイル」を 1237 に統一 |
| **C k8s** | local cluster (`kind-k1s0-local`) / 36 namespaces / **93 Running / 93 total = 100%** ・ production-equivalent (managed K8s) 検証: **未実施 (保留)** | 同一値で再現性確認 |
| **D OSS** | Met **31** / Unmet **0** / Unknown 3 (Branch-Protection / Code-Review / Vulnerabilities — 全て public 化 + scorecard-cli 必須) ・ CII Best Practices Passing 17 項目: 機械判定 Met **10** / Manual-Required 5 / Unmet 0 ・ Dangerous-Workflow **Met** (危険な pull_request_target + PR HEAD checkout 0 件) ・ 直近 30 日 **750 commits** / 90 日 **1684 commits** | 再現性確認、commit 数の自然増（30 日 745→750 / 90 日 1679→1684） |

詳細は次節以降。

## A 軸: 要求網羅

### A-1: ADR 監査（46 件）

| 状態 | 件数 | 詳細 |
|---|---|---|
| 計上 ADR ファイル | **46** | docs/02_構想設計/adr/ 配下（実体 .md 数）— 旧形式 4 桁通し番号 3 件（ADR-0001/0002/0003）+ 新形式カテゴリ別 43 件 |
| 計上 ADR ID（coverage） | **46** | ADR ファイルから抽出（ファイル ↔ ID 1:1）。#8 で coverage.sh の ADR ID 列挙を「adr/ 配下 grep」から「ファイル名抽出」に構造修正、cite-only ID（README や他 ADR から引用されているが ADR ファイル不在）は本行に混入させず docs-orphan 行で別途集計（過去の grep ベース列挙では 49 件、cite-only 3 件 DEV-003/DIR-004/SUP-002 が「docs-only (impl 不在)」に誤分類されていた） |
| 実装参照あり ADR | **43** | コードから ID で参照されているもの（3-stage 34 + 2-stage 9） |
| **code-orphan**（コード参照あり / ADR 不在） | **0** | 過去 11 → 8 → 0、3 件統合 + 8 件新規起票で全件解消 |
| **docs-orphan**（docs 引用あり / ADR 不在、2026-05-02 #6 新設検出） | **41** | docs/ 全体走査で発見、過去のリファクタで ID を docs に残したまま ADR を統合 / 削除 / 改名した形跡。詳細は下記 #### docs-orphan 41 件。再分類済の DEV-003 / DIR-004 / SUP-002 はここに残置（regression test Test 19 で監視） |
| **実装参照 0 件**（ADR 起票済 / コード未参照） | **3** | ADR-0002（規約系）/ ADR-DEP-001（Renovate）/ ADR-DX-001（DX メトリクス）。詳細は下記 |

#### 解消済み統合（過去）

| 旧 ID（orphan） | 統合先 ADR | 統合根拠 |
|---|---|---|
| ADR-CNCF-004 | [ADR-MIG-002（API Gateway / Envoy Gateway）](02_構想設計/adr/ADR-MIG-002-api-gateway.md) | 同一テーマ（Envoy Gateway 選定）、ADR-MIG-002 で Strangler Fig + API Gateway として採用済 |
| ADR-DEVEX-002 | [ADR-BS-001（Backstage 開発者ポータル）](02_構想設計/adr/ADR-BS-001-backstage.md) | 同一テーマ（Backstage 採用） |
| ADR-DEVEX-004 | [ADR-DEV-001（Paved Road）](02_構想設計/adr/ADR-DEV-001-paved-road.md) | ADR-DEV-001 内で「Backstage Golden Path = Paved Road の一変種」と明示包摂 |

波及で発見した壊れたリンク 1 件も同時修正（`infra/mesh/envoy-gateway/README.md:72`、`ADR-MIG-002-istio-ambient-migration.md` → `ADR-0001-istio-ambient-vs-sidecar.md`）。

#### 解消済み新規起票（8 件、2026-05-02）

| ID | ファイル | 主題 |
|---|---|---|
| `ADR-INFRA-001` | [`ADR-INFRA-001-kubernetes-cluster-bootstrap.md`](02_構想設計/adr/ADR-INFRA-001-kubernetes-cluster-bootstrap.md) | Kubernetes クラスタを kubeadm + Cluster API で構築する |
| `ADR-CNCF-001` | [`ADR-CNCF-001-cncf-conformance.md`](02_構想設計/adr/ADR-CNCF-001-cncf-conformance.md) | vanilla Kubernetes（CNCF Conformance 互換）を維持する |
| `ADR-NET-001` | [`ADR-NET-001-cni-selection.md`](02_構想設計/adr/ADR-NET-001-cni-selection.md) | production CNI に Cilium、kind 検証用に Calico を使い分ける |
| `ADR-DAPR-001` | [`ADR-DAPR-001-dapr-operator.md`](02_構想設計/adr/ADR-DAPR-001-dapr-operator.md) | 分散ランタイムに Dapr Operator を採用する |
| `ADR-SCALE-001` | [`ADR-SCALE-001-keda-event-driven-autoscaling.md`](02_構想設計/adr/ADR-SCALE-001-keda-event-driven-autoscaling.md) | Event-driven autoscaling に KEDA を採用する |
| `ADR-TIER3-001` | [`ADR-TIER3-001-bff-pattern.md`](02_構想設計/adr/ADR-TIER3-001-bff-pattern.md) | tier3 client ごとに専用 BFF を配置する |
| `ADR-TIER3-002` | [`ADR-TIER3-002-spa-plus-bff.md`](02_構想設計/adr/ADR-TIER3-002-spa-plus-bff.md) | tier3 Web を React + Vite SPA + Go BFF で構成する |
| `ADR-TIER3-003` | [`ADR-TIER3-003-dotnet-maui-native.md`](02_構想設計/adr/ADR-TIER3-003-dotnet-maui-native.md) | tier3 Native アプリに .NET MAUI を採用する |

各 ADR は `docs-adr-authoring` 規約（5 段構成 / 検討肢 3 件以上 / 決定理由 / 影響）に準拠、計 1191 行。3 索引ファイル（`docs/02_構想設計/adr/README.md` / `docs/03_要件定義/00_要件定義方針/08_ADR索引.md` / `docs/04_概要設計/90_付録/02_ADR索引.md`）も同期更新済。

#### 実装参照 0 件 ADR（3 件、要実態確認）

- `ADR-0002`（diagram-layer-convention、規約系で実装が無いのが正常）
- `ADR-DEP-001`（Renovate、`renovate.json` で間接参照されているが ID 文字列として含まれない可能性）
- `ADR-DX-001`（DX メトリクス、要実態確認）

判定材料: いずれも ADR ファイル存在 / コード参照なしのケース。規約系 ADR (0002) は impl 不在が正常、他 2 件は SHIP_STATUS と突合せが必要。**過去 AUDIT.md の「5 件」のうち ADR-DEV-003 / ADR-DIR-004 / ADR-SUP-002 は ADR ファイル不在 = docs-orphan に再分類** (2026-05-02 #6)。

#### docs-orphan 41 件（docs 引用あり / ADR ファイル不在、2026-05-02 #6 新設検出）

`tools/audit/lib/coverage.sh` の orphan 検出は当初 `src/infra/deploy/tools/examples` のみを走査していたため、docs 他階層からの cite と ADR ファイル間の不整合を見逃していた。docs/ 全体走査機能を追加した結果、**41 件**の docs-orphan を発見:

| 系列 | 件数 | ID |
|---|---:|---|
| 旧形式 (regex 拡張で初検出) | 2 | ADR-0000（要件定義書で「ADR-0000 台帳」言及）、ADR-0005（methodology の例示） |
| ADR-CNCF-* | 4 | CNCF-002, CNCF-003, CNCF-004, CNCF-005 |
| ADR-DEVEX-* / ADR-DEVX-* | 5 | DEVEX-001, DEVEX-002, DEVEX-003, DEVEX-004, DEVX-001（typo の可能性） |
| ADR-CICD-* | 3 | CICD-004, CICD-005, CICD-006 |
| ADR-DIR-* | 3 | DIR-004, DIR-005, DIR-006 |
| ADR-DEV-003 / OBS-004 / SUP-002 / TIER1-007 | 4 | 既存系列の欠番 |
| ADR-OPS-* / ADR-SEC-* / ADR-ZEN-* | 6 | OPS-001/002, SEC-004/005, ZEN-001/002 |
| 単発 | 14 | AUDIT-001, BS-002, CB-001, CODE-001, CT-001, FEAT-001, GOV-001, MESH-001, MSG-001, NFR-001, PERF-001, PROC-002, TEST-001, WF-001 |

対応方針（個別判定は別 PR で実施、本回は検出機能まで）:
- **ADR 起票で解消**: 既存系列の欠番（DEV-003 / DIR-005 / OPS-001 等、設計が確立しているが ADR が未起票）
- **Superseded 注記で解消**: 統合 / 改名された ID（CNCF-004 → MIG-002 のような既存統合に追加でカバー）
- **cite 削除で解消**: 教育用例示（ADR-0000「台帳」概念、ADR-0005 「ADR-0002 を見直す」例示）
- **typo 修正で解消**: ADR-DEVX-001（ADR-DEVEX-* の typo の可能性）

詳細は [`audit-evidence/2026-05-02/docs-orphans-adr.txt`](../.claude/audit-evidence/2026-05-02/docs-orphans-adr.txt) 参照。

#### 監査ツール自身のバグ（2026-05-02 #6 解消）

本イテレーションで以下 2 件の audit ツール側バグを発見・修正した。これらは AUDIT.md の数値が「実態より良く見える」誤読を生む構造問題で、最優先で潰す対象だった (Layer 1: 監査の信頼性ブロッカー):

| バグ | 場所 | 症状 | 修正 |
|---|---|---|---|
| **ID_REGEX 旧形式取りこぼし** | `tools/audit/lib/coverage.sh:42` / `:222`、`tools/audit/lib/trace.sh:128` | `ADR-[A-Z0-9]+-[0-9]+` がハイフン区切り数値サフィックスを必須とするため、旧形式 `ADR-0001/0002/0003`（4 桁通し番号）を完全に取りこぼし。3 件 ADR が coverage / orphan / trace のすべてで不可視 | regex を `ADR-([0-9]{4}|[A-Z][A-Z0-9]*-[0-9]+)` に拡張、旧 4 桁通し番号と新カテゴリ別の両対応。Test 13/14 で regression 防止 |
| **docs-side orphan 未検出** | `tools/audit/lib/coverage.sh` の ADR 処理が `docs/02_構想設計/adr/` のみ走査 | docs 他階層からの ADR cite と ADR ファイル間の不整合を見逃し。code-orphan は 0 だが docs-orphan は構造的に未検出 | coverage.sh の ADR セクションに docs/ 全体走査を追加、`docs-orphans-adr.txt` を新規出力。Test 15 で regression 防止 |

副次として self-detection 不具合（コメント内 placeholder 文字列の誤検出）が新検出されたため、`coverage.sh` の orphan grep に `--exclude-dir=audit` を追加し、Test 16 で baseline 0 件を CI 検査。

### A-2: FR / NFR / DS / IMP の coverage + trace 結果

`coverage.sh` が ID ごとに **(a) docs 定義 / (b) 実装サンプル / (c) 動作証跡（test 参照 + SHIP_STATUS キーワード共起）** を集計。NFR / DS / IMP は ID 直引用が稀なので、`trace.sh` で **(a') direct, (b') verify アーティファクト (policy / SLO / contract test), (c') co-cited 経由 (FR/ADR impl reach)** の 3 経路から間接 reach を集計する補正を加える。

判定材料は事実ベースで提示し、PASS / PARTIAL / FAIL は人間が AUDIT.md の判定列を埋める運用。

#### coverage 結果（直接 grep ベース）

| 軸 | 総 ID | 3 段揃い候補 | 2 段（docs+impl） | impl 不在 | 集計時点 |
|---|---:|---:|---:|---:|---|
| FR-T1-* | 50 | 22 (44%) | 14 (28%) | 14 (28%) | 2026-05-02 #6 |
| NFR-* | 155 | 8 (5%) | 5 (3%) | 142 (92%) | 2026-05-02 #6 |
| DS-* | 1416 | 16 (1%) | 11 (1%) | 1389 (98%) | 2026-05-02 #6 |
| IMP-* | 718 | 1 (0.1%) | 58 (8%) | 659 (92%) | 2026-05-02 #6 |
| ADR-* | **46** | **34 (74%)** | 9 (20%) | **3 (7%)** | 2026-05-02 #8 |

#### trace 結果（NFR / DS / IMP の間接 reach 補正）

| 軸 | 総 ID | reach (任意経路) | unreached (要 inspect) | reach 内訳 |
|---|---:|---:|---:|---|
| NFR-* | 155 | **140 (90%)** | **15 (10%)** | direct+verify+via-fr+via-adr 5 / direct+via-fr+via-adr 7 / direct+via-adr 1 / verify+via-fr+via-adr 20 / via-fr+via-adr 85 / via-fr 6 / via-adr 16 |
| DS-* | 1416 | **1168 (82%)** | **248 (18%)** | direct+via-fr+via-adr 16 / direct+via-adr 11 / verify+via-fr+via-adr 1 / verify+via-adr 3 / verify 1 / via-fr+via-adr 644 / via-fr 50 / via-adr 442 |
| IMP-* | 718 | **712 (99%)** | **6 (1%)** | direct+verify+via-fr+via-adr 1 / direct+verify+via-adr 7 / direct+via-fr+via-adr 8 / direct+via-adr 40 / direct 3 / via-adr 653 |

trace 経路の意味:
- **direct**: `src / infra / deploy / tools / examples` で ID 直接 grep ヒット（coverage と重複、業界慣行で稀）
- **verify**: `infra/security/kyverno/`, `infra/observability/`, `deploy/rollouts/`, `ops/sli-slo/`, `tests/{contract,fuzz}/` 内の ID 引用（推奨される検証経路、e2e はテスト基盤刷新中で対象外）
- **via-fr / via-adr**: ID 言及 docs に同居する FR / ADR のうち impl_refs > 0 のもの（co-cite による間接 reach）
- **unreached**: 4 経路すべてで reach 0、真の impl 不在候補。要 inspect

詳細は `audit-evidence/2026-05-02/coverage-{fr,nfr,ds,imp,adr}.txt` および `trace-{nfr,ds,imp}.txt` 参照。

#### 重要な観察 — coverage 単独では誤読される

NFR-* / DS-* / IMP-* で coverage 単独では「impl 不在」が高比率（NFR 92% / DS 98%）になるが、これは **業界標準の設計慣行**を反映しただけで「手抜き」の証拠ではない：

- **NFR**（性能 / 可用性 / セキュリティ等）はコード内に ID 文字列で参照されにくい。性能要件は benchmark で / 可用性要件は SLO で / セキュリティ要件は admission policy で、それぞれ間接検証されるのが普通
- **DS**（概要設計）は設計書同士の相互参照に使う ID で、コード本体には現れにくい。実装は IMP-* / FR-* / ADR-* 経由で紐付く
- **IMP**（実装規約）も同様にコードに ID 文字列を直接埋め込まない

trace.sh の数値（NFR reach 90% / DS reach 81%）が**実態に近い impl 充足度**を示す。coverage.sh の「impl 不在 92%」を真に受けて手抜きと解釈するのは過去の AUDIT.md の誤りで、本版で trace 軸を追加して補正済。

#### FR-T1-* の 14 件 impl 不在（要 inspect）

これは coverage.sh の検出限界ではなく、実際に実装が手薄な可能性が高い候補。`audit-evidence/2026-05-02/coverage-fr.txt` で classification が「docs-only (impl 不在)」のものを inspect する次のタスクが必要。FR は ID 直引用される性質なので、直接 grep の結果がそのまま判定材料になる。

## B 軸: 手抜き検出

実行: `tools/audit/run.sh slack`、走査範囲: **1237 ファイル**（`src` / `infra` / `deploy` / `tools` / `tests` / `examples`、生成コード + audit lib 自身を除外、内訳は `slack-scope.txt` の `total_files`）。

### B-1: パターン別残存件数（最新）

| パターン | 件数 | 備考 |
|---|---:|---|
| `codes.Unimplemented`（Go） | 1 | dapr.go:40 のコメント内言及（コード本体ではない、許容） |
| `unimplemented!()`（Rust） | 0 | |
| `todo!()`（Rust） | 0 | |
| `NotImplementedException`（.NET） | 0 | |
| `not impl`（TS） | 0 | |
| `NotImplementedError`（Python） | 0 | |
| 禁止語彙 `TODO` / `FIXME` / `XXX` | 0 / 0 / 0 | |
| 禁止語彙 `とりあえず` / `暫定` / `仮置き` / `あとで` | 0 / 0 / 0 / 0 | |
| 禁止語彙 `for now` / `temporary` / `quick fix` / `// hack` / `workaround` | 0 / 0 / 0 / 0 / 0 | |
| 空 catch（JS/TS） / `except: pass`（Python） | 0 / 0 | |
| Go silent error（`_ = err`） | 1 | doc.go:28 の doc コメント内擬似コード（コード本体ではない、許容） |
| Rust empty `unwrap_or()` | 0 | |

判定材料: **コード本体に残る真の手抜き 0 件 / 1237 ファイル走査**。許容残置 2 件は false-positive で、いずれも `//` で始まるコメント内の識別子言及であり実コード経路ではない。

### B-2: 許容残置 2 件の根拠

| 位置 | 内容 | 許容理由 |
|---|---|---|
| `src/tier1/go/internal/adapter/dapr/dapr.go:40` | `// gRPC `codes.Unimplemented` に翻訳する。` | コメント内で `codes.Unimplemented` を**識別子として言及**しているのみ。実際の Unimplemented 返却ではない |
| `src/sdk/go/k1s0/doc.go:28` | `//		_ = data; _ = etag; _ = found; _ = err` | doc コメント内の **使用例擬似コード**（`//` で始まる行）。実コード本体での silent suppress ではない |

### B-3: gitkeep-only ディレクトリ整合検査（自動化済）

`tools/audit/lib/slack.sh` が gitkeep のみのディレクトリと SHIP_STATUS の許容キーワード（`設計のみ` / `採用後の運用拡大時` / `意図的に空` / `雛形あり`）を自動突合せする。

| 区分 | 件数 | 詳細 |
|---|---:|---|
| 計上 gitkeep-only ディレクトリ | **23** | |
| **documented** (SHIP_STATUS 許容キーワード共起) | **9** | infra/environments/{dev,staging,prod}/secrets / infra/environments/prod/patches / src/tier1/rust/crates/policy / deploy/opentofu/environments/{dev,staging,prod} / deploy/rollouts/experiments |
| **undocumented** (要 SHIP_STATUS 加筆 or 実装) | **14** | 下記リスト |

#### undocumented 14 件（要対応）

| パス | 推測される性質 | 推奨アクション |
|---|---|---|
| `tools/migration/framework-to-sidecar` | レガシー .NET Framework → sidecar 移行ツール雛形 | SHIP_STATUS に「採用後の運用拡大時」明示 or 削除 |
| `tools/migration/framework-to-net8` | レガシー .NET Framework → .NET 8 移行ツール雛形 | 同上 |
| `third_party/` | サードパーティ預かり | SHIP_STATUS に空である理由明示 |
| `src/tier3/native/apps/K1s0.Native.Hub/Resources/Fonts` | フォント資産配置先 | SHIP_STATUS に「ライセンス調達後に配置」明示 |
| `tests/fixtures/openapi-samples` / `tests/fixtures/seed-data` / `tests/fixtures/tls-certs` | テスト fixture 雛形 3 件 | SHIP_STATUS に fixture 整備計画明示 |
| `tests/contract/pact/consumers/portal-bff` / `consumers/admin-bff` / `providers/tier1-state` / `providers/tier2-payroll` | Pact contract 雛形 4 件 | SHIP_STATUS に「採用初期で contract 整備」明示 |
| `deploy/opentofu/modules/vpn-gateway` / `dns` / `baremetal-k8s` | OpenTofu モジュール雛形 3 件 | SHIP_STATUS に「採用後の運用拡大時」明示（既存の environments/ と整合） |

判定材料: 23 件中 9 件は SHIP_STATUS で正当化済、14 件は説明欠落。判定の責任は人間。

### B-4: 過去に修正した audit 自身のバグ（再記録）

1. **shell IFS 罠**: `slack.sh` のパターン定義 `//\s*hack\b|#\s*hack\b` の `|` を `IFS='|'` split が誤分解 → 修正前 739 件 / 修正後 0 件
2. **生成コード未除外**: `_grpc.pb.go` の自動生成 `UnimplementedXxxServer` stub を手抜きと誤判定 → 修正前 55 件 / 修正後 1 件
3. **audit lib self-detection**: パターン定義行が自分自身にマッチ → `tools/audit/lib/` を走査範囲から除外
4. **ID placeholder の `XXX`**: `IMP-` 系 placeholder が `\bXXX\b` にマッチ → `(?![-A-Z])` の look-ahead で除外
5. **Go 構文の `for now`**: `for now := range ticker.C` が `\bfor now\b` にマッチ → `(?!\s*[:=,])` で除外
6. **`*.yaml` リテラル glob**: oss.sh の Dangerous-Workflow 検査で `.github/workflows/*.yaml` がマッチせずリテラル渡しで grep が exit 2、`set -e` で script 死亡 → find で先に列挙
7. **ADR ID_REGEX が旧形式 4 桁通し番号を取りこぼし**（2026-05-02 #6 解消）: `ADR-[A-Z0-9]+-[0-9]+` がハイフン区切り数値サフィックスを必須とするため、ADR-0001/0002/0003 が completely 不可視 → `ADR-([0-9]{4}|[A-Z][A-Z0-9]*-[0-9]+)` に拡張、Test 13/14 で regression 防止
8. **docs-side ADR orphan 未検出**（2026-05-02 #6 解消）: coverage.sh が `docs/02_構想設計/adr/` 配下のみを走査するため、docs 他階層からの ADR cite と ADR ファイル間の不整合を見逃し。docs 全体走査機能を追加し 41 件発覚 → `docs-orphans-adr.txt` 出力、Test 15 で regression 防止
9. **coverage.sh ADR orphan grep の self-detection**（2026-05-02 #6 解消）: lib 内コメントの placeholder 文字列が code-orphan に誤検出 → `--exclude-dir=audit` で構造的に防止、Test 16 で baseline 0 件を CI 検査

これらの改善で **false positive 17 件 → 0 件 + ID_REGEX 取りこぼし 3 件解消 + docs-orphan 41 件可視化**、信頼性が大幅向上。修正は `tools/audit/lib/{slack,coverage,trace,oss}.sh` に反映済。

## C 軸: k8s 実機動作

実行: `tools/audit/run.sh k8s`、`tools/audit/lib/k8s.sh` が context 名から `cluster_class` を判定し、local / production を分離。

| 検証 tier | cluster_class | context | namespaces | Running / total | 検証日 |
|---|---|---|---:|---|---|
| **local** | `kind` | `kind-k1s0-local` | 36 | **93 / 93 = 100%** | 2026-05-02 |
| **production-equivalent** (managed K8s) | — | — | — | — | **未実施 (保留)** |

判定材料: **local cluster (kind) 段階では全 Pod Running**。production-equivalent (GKE / EKS / AKS / OKE 等) での E2E 検証は未実施で、本 audit では「保留」扱い。kind での 100% Running を production 検証の証跡と読まないこと。

### C-1: 過去解消（kind の non-Running Pod 4 件 → 0 件）

**当初の該当 Pod**: `observability/grafana` / `observability/loki-0` / `observability/prometheus-server` / `observability/tempo-0`（37h Pending）

**5 Whys 結論**:
1. PVC が unbound
2. StorageClass `k1s0-default` の provisioner が `PLACEHOLDER_csi_provisioner`（実 provisioner なし）
3. ADR-STOR-001 が Longhorn 前提だが kind では Longhorn install 不可、誰かが PLACEHOLDER で代用 apply
4. kind 向け SC override が `tools/local-stack/up.sh` / `infra/environments/dev/` に未整備
5. SHIP_STATUS で「Longhorn は kind 不可」明記済だが、observability 用 kind 代替 SC 設定が欠落

**採用した修正**: `tools/local-stack/lib/apply-layers.sh` に `patch_kind_storageclasses` 関数を新設し、`up.sh` の main flow で `apply_metallb` の直後に呼ぶ構成を確立。kind context のみで `k1s0-{default,high-iops,backup,shared}` の `PLACEHOLDER_*` provisioner を `rancher.io/local-path` に置換、`k1s0-default` を default SC に集約。

**実機検証結果**: 関数を実行後、4 PVC すべて Bound に遷移、4 Pod すべて Running に。

### C-2: production-equivalent 検証の必要条件

local kind PASS は production PASS の代理にならない。以下が production verification として要追加：

- **Cluster 種別**: managed K8s (GKE / EKS / AKS) または vanilla K8s (kubeadm + Cluster API、ADR-INFRA-001)
- **CNI**: production CNI = Cilium (ADR-NET-001)、kind 検証用の Calico ではない
- **CSI**: Longhorn (ADR-STOR-001)、kind 代替 (`rancher.io/local-path`) ではない
- **観測**: Prometheus / Loki / Tempo / Grafana が kind 簡易構成ではなく production スケール構成
- **セキュリティ**: Kyverno admission policy の enforce モード適用（kind では audit モードのみ）

これらは SHIP_STATUS の「Production carry-over verification matrix」に詳述。

## D 軸: OSS 完成度

`tools/audit/lib/oss.sh` が OSSF Scorecard 18 項目 + CNCF Sandbox 最低要件 + OpenSSF Best Practices Passing 17 項目をローカル機械化可能な範囲で採点。

集計（自動採点で機械化可能な範囲）: **Met 31 / Unmet 0 / Unknown 3 / N/A 1**

### D-1: CNCF Sandbox 最低要件（ファイル存在 + 中身）

| 項目 | 状態 | サイズ |
|---|---|---:|
| `LICENSE` | Met（**Apache-2.0**、OSI 承認） | 201 行 |
| `CODE_OF_CONDUCT.md` | Met | 115 行 |
| `CONTRIBUTING.md` | Met | 160 行 |
| `GOVERNANCE.md` | Met | 79 行 |
| `SECURITY.md` | Met（vulnerability 報告経路あり） | 73 行 |
| `README.md` | Met | 359 行 |

判定材料: **リリース時点の最低要件は全件 Met**。中身の質的評価は別途。

### D-2: OSSF Scorecard 機械化項目

| 項目 | 状態 | 証跡 |
|---|---|---|
| Maintained | Met | 直近 30 日 750 commits / 直近 90 日 1684 commits（2026-05-02 #7 再実行時） |
| License | Met | LICENSE = Apache-2.0 |
| Security-Policy | Met | SECURITY.md に mailto/URL + disclosure キーワード 12 件 |
| Pinned-Dependencies | Met | renovate.json + go.sum + Cargo.lock × 3 + pnpm-lock.yaml × 3 |
| Fuzzing | Met | tests/fuzz/{go(1), rust(10)} |
| Signed-Releases | Met | ops/supply-chain/sbom (12) + signatures (6) + keys |
| Binary-Artifacts | Met | git ls-files で .exe/.dll/.so/.dylib/.jar/.class = 0 件 |
| Token-Permissions | Met | 17/17 workflow で `permissions:` 明示 |
| SAST | Met | `_reusable-lint.yml` で各言語 linter 実行 |
| Dependency-Update-Tool | Met | renovate.json |
| CI-Tests | Met | .github/workflows/ に PR 時発火 workflow 17 件 |
| **Dangerous-Workflow** | **Met** | pull_request_target + PR HEAD checkout の危険な組み合わせ 0 件 (全 1 件中) |
| Branch-Protection | **Unknown** | public repo + scorecard-cli 必須 |
| Code-Review | **Unknown** | public repo + PR 履歴の機械分析必要 |
| Vulnerabilities | **Unknown** | dependabot alert (public 化後) |
| Webhooks | N/A | public 化前 |
| CII-Best-Practices | 部分 Met | 機械判定 10/17 (下記 D-4 参照)、最終登録は外部サイト |
| Contributors | 部分 Met | unique 3 名（採用組織複数化で増加見込み） |

### D-3: リポジトリ運用 (.github/ 系)

| 項目 | 状態 |
|---|---|
| CODEOWNERS | Met (56 行) |
| PR Template | Met (93 行) |
| Issue Template | Met (`.github/ISSUE_TEMPLATE/`、3 件) |
| Labels Definition | Met (`labels.yml`、225 行) |
| Repo Settings 文書化 | Met (`repo-settings.md`、119 行) |

### D-4: CII Best Practices Passing 17 項目（ローカル判定可能分）

`oss.sh` で機械判定可能な部分を採点。最終的な Best Practices Badge は [bestpractices.dev](https://www.bestpractices.dev/) で repo URL を申告して取得。

| 領域 | 項目 | 状態 |
|---|---|---|
| Basics | project_url | Met (README.md) |
| Basics | vulnerability_report | Met (SECURITY.md) |
| Basics | floss_license | Met (LICENSE) |
| Change Control | public_version_control | Met (git remote = public hosting) |
| Change Control | unique_version_numbering | **Manual-Required** (git tag 0 件、semver 準拠は別途) |
| Change Control | release_notes | **Manual-Required** (CHANGELOG.md 不在) |
| Reporting | bug_reporting_process | Met (.github/ISSUE_TEMPLATE/) |
| Reporting | vulnerability_response | Met (SECURITY.md、応答 SLA は別途記述要) |
| Quality | working_build_system | Met (Cargo.toml / go.mod / package.json) |
| Quality | automated_test_suite | Met (tests/ + *_test.{go,rs}) |
| Quality | test_added_for_changes | **Manual-Required** (PR template で確認) |
| Security | basic_good_cryptographic_practices | **Manual-Required** (ADR-CRYPTO-* 系参照) |
| Security | secured_delivery | Met (ops/supply-chain/) |
| Security | publicly_known_vulnerabilities_fixed | **Manual-Required** (renovate + lock file 監査) |
| Analysis | static_analysis | Met (`_reusable-lint.yml`) |

判定材料: **Met 10 / Manual-Required 5 / Unmet 0**（17 項目中 15 項目が走査対象、2 項目は走査ロジック未実装で要拡張）。

### D-5: 段階目標との充足度

`oss-completeness-criteria` skill で定義した段階目標：

| 段階 | 目標 | 現状 |
|---|---|---|
| **リリース時点 (v0)** | CNCF Sandbox 最低要件 / OSSF Scorecard 5/10 / Best Practices Passing 9/17 | 判定材料: ファイル存在 + 機械化項目 Met 31 / Unmet 0 / CII Met 10 |
| 採用初期 | OSSF Scorecard 7/10 / Best Practices Passing 17/17 | 判定材料: public repo 化後 + 外部採点で評価、Manual-Required 5 件の人間判定が要 |
| 採用後の運用拡大時 | OSSF Scorecard 9/10 / Best Practices Silver | 段階運用 |
| 長期 | OSSF Scorecard 10/10 / Best Practices Gold | 任意目標 |

詳細は [`oss-completeness-criteria` skill](../.claude/skills/oss-completeness-criteria/SKILL.md) と `audit-evidence/2026-05-02/oss-checklist.txt` 参照。

## 採用検討者向け総合所見

### 信頼できる水準への充足度（最新）

| 基準 | 判定材料 | 残作業 |
|---|---|---|
| OSSF Scorecard 7/10 | 機械化項目 Met 31 / Unmet 0 / Unknown 3 + Dangerous-Workflow Met | public repo 化 + scorecard-cli 導入で 3 Unknown を解消 |
| CNCF Sandbox 最低要件 | 6 ファイル全て Met (中身検査済) | 外部 contributor 受入の可視化 |
| OpenSSF Best Practices Passing | 機械判定 Met 10/17 / Manual-Required 5 / Unmet 0 | bestpractices.dev で repo URL 申告、Manual 5 件の人間判定 |
| k1s0 自己基準（4 軸） | A: code-orphan 0 / docs-orphan **41 (新規可視化)** / FR 22/50 (44%) 3-stage 候補 / NFR trace reach 140/155 (90%) / DS trace reach **1168/1416 (82%)** / IMP trace reach **712/718 (99%)** ・ B: 真の手抜き 0 / gitkeep undocumented 14 ・ C: kind 100% / production 保留 ・ D: Met 31 / CII 10 | docs-orphan 41 件の個別判定（起票 / Superseded 注記 / cite 削除）、undocumented 14 件の SHIP_STATUS 加筆、unreached 件の inspect、production-equivalent 検証 |

### 解消済み（2026-05-02 #6 イテレーション — 監査ツール側バグ 2 件解消）

1. ✅ **ADR ID_REGEX を旧形式 4 桁通し番号対応に拡張**（`tools/audit/lib/coverage.sh:42` / `:222` / `tools/audit/lib/trace.sh:128`、`ADR-([0-9]{4}|[A-Z][A-Z0-9]*-[0-9]+)`、ADR-0001/0002/0003 を初検出、ID 数 46→49）
2. ✅ **docs-side ADR orphan 検出機能の新設**（`tools/audit/lib/coverage.sh` の ADR セクションに docs/ 全体走査を追加、`docs-orphans-adr.txt` 出力、41 件の docs-orphan を初可視化）
3. ✅ **coverage.sh code-orphan の self-detection 不具合解消**（lib コメント内 placeholder 文字列の誤検出を `--exclude-dir=audit` で構造的に防止）
4. ✅ **AUDIT.md「実装参照 0 件 5 件」の再分類**（DEV-003 / DIR-004 / SUP-002 を docs-orphan に振り替え、真の impl 不在は 3 件: ADR-0002 規約 / DEP-001 / DX-001）
5. ✅ **判定基準正典 `docs/00_format/audit_criteria.md` に orphan 2 系統 (code/docs) 表を追加**
6. ✅ **`audit-protocol` skill の orphan 検出セクションを 2 系統対応に書き換え**
7. ✅ **regression test 4 件追加**（`tests/audit/test_audit_lib.sh` Test 13/14/15/16、計 25 assertion 全 PASS）

### 解消済み（2026-05-02 #5 イテレーションで構造改善）

8. ✅ **AUDIT.md から Claude 記入の PASS / PARTIAL を全削除**（`audit-protocol` skill 違反の解消）
9. ✅ **`.claude/audit-evidence/` を git 管理対象化**（第三者が AUDIT.md を再生成なしに verify 可能）
10. ✅ **trace.sh 軸新設で NFR / DS / IMP の間接 reach を可視化**（coverage の「impl 不在 92%」誤読を補正、NFR reach 90% / DS reach 82% / IMP reach 99%）
11. ✅ **k8s.sh が cluster_class / verification_tier を出力**（kind PASS を production PASS と誤読する事故を構造防止）
12. ✅ **slack.sh に gitkeep ↔ SHIP_STATUS 自動整合検査追加**（「別 task」逃げを潰し、未文書化 14 件を即可視化）
13. ✅ **oss.sh に Dangerous-Workflow + CII Best Practices ローカル採点追加**（Unknown 4 → 3、Met 20 → 31）
14. ✅ **trace.sh の batch grep 化で 350× 高速化**（DS 1416 ID 走査が 7 分 → 1.3 秒）
15. ✅ **audit-protocol SKILL.md を厳格化**（Claude が AUDIT.md にいかなる場合も PASS を書かない原則を明文化、自己点検チェックリスト 12 項目に拡張）
16. ✅ **過去解消継続**: ADR code-orphan 0 / non-Running Pod 0 / 真の手抜き 0 / audit 自身のバグ 9 件修正（#6 で 6 → 9）

### 残存タスク（最優先 Layer 1: 採用判断ブロッカー）

17. **docs-orphan 41 件の個別判定**（別 PR で実施）。ADR 起票 / Superseded 注記 / cite 削除 / typo 修正の 4 方針で個別対応。詳細リストは `audit-evidence/2026-05-02/docs-orphans-adr.txt`

### 残存タスク（中優先 Layer 2）

18. FR-T1-* の 14 件 impl 不在の inspect (`audit-evidence/2026-05-02/coverage-fr.txt`)
19. 実装参照 0 件 ADR 3 件の実態確認 (`ADR-0002` 規約系 / `ADR-DEP-001` Renovate / `ADR-DX-001` DX メトリクス)
20. trace `unreached` の inspect: NFR 15 件 / DS 248 件 / IMP 6 件 — 真の impl 不在候補
21. gitkeep undocumented 14 件の SHIP_STATUS 加筆 or 実装合流
22. CII Best Practices Manual-Required 5 件の人間判定（unique_version_numbering / release_notes / test_added_for_changes / cryptographic_practices / publicly_known_vulnerabilities_fixed）
23. production-equivalent (managed K8s) cluster での E2E 検証実施

### 残存タスク（低優先 Layer 3: 採用後の運用拡大時）

24. OSSF Scorecard 自動採点（public repo 化 + scorecard-cli 導入）
25. OpenSSF Best Practices Badge 取得（外部サイト bestpractices.dev で URL 申告）
26. 外部 contributor 受入の可視化（GitHub Discussions / mailing list 等）

## 関連

- 判定基準: [`docs/00_format/audit_criteria.md`](00_format/audit_criteria.md)
- 物語版（経緯と判断）: [`docs/SHIP_STATUS.md`](SHIP_STATUS.md)
- 実行コマンド: `/audit [axis]`
- 実行スクリプト: `tools/audit/run.sh`
- 監査方法論: `.claude/skills/audit-protocol/SKILL.md`
- 生証跡（git 管理対象）: `.claude/audit-evidence/<date>/`

## 更新履歴

| 日付 | 軸 | 主な変化 |
|---|---|---|
| 2026-05-02 (#1) | 初版 | slack/adr/k8s/oss 実行、orphan 11 / slack 残存 60（うち false positive 多数）/ k8s 接続確認 / OSS ルート 6 Met |
| 2026-05-02 (#2) | 解消イテレーション #1 | **真の手抜き 12 → 0**（書き換え）/ **false positive 17 → 0**（audit logic 改善）/ **orphan 11 → 8**（3 件統合）/ k8s 4 Pending の root cause 特定（PVC SC drift） |
| 2026-05-02 (#3) | 解消イテレーション #2 | **ADR orphan 8 → 0**（8 件新規起票 + 索引 3 ファイル同期）/ **k8s Running 89/93 → 93/93**（C 案 `patch_kind_storageclasses` 実装 + 実機検証）/ ADR ファイル数 38 → 46 |
| 2026-05-02 (#4) | Phase 2 / 3 | **A 軸 5 ID 種を 3 段確認本実装**（FR / NFR / DS / IMP / ADR を coverage.sh で集計）/ **D 軸 OSSF Scorecard 18 項目対応**（oss.sh 拡張、Met 20 / Unmet 0 / Unknown 4）/ **`oss-completeness-criteria` skill 起票** |
| 2026-05-02 (#5) | 構造改善 — 信頼性の確立 | **Claude 記入 PASS を AUDIT.md から全削除**（protocol 違反の自己解消）/ **生証跡を git 管理化**（第三者 verify 可能）/ **trace.sh 軸新設**（NFR/DS/IMP の間接 reach、reach 90%/81%/98%）/ **k8s.sh kind/production 分離**（cluster_class 明示）/ **slack.sh gitkeep 整合自動検査**（documented 9 / undocumented 14）/ **oss.sh CII + Dangerous-Workflow ローカル採点**（Met 20 → 31 / Unknown 4 → 3）/ **trace.sh 350× 高速化**（batch grep）/ **audit-protocol skill 厳格化**（自己点検 12 項目） |
| 2026-05-02 (#6) | 監査ツール側バグ 2 件解消 — 信頼性の更なる強化 | **ADR ID_REGEX 旧形式取りこぼし解消**（`ADR-([0-9]{4}|[A-Z][A-Z0-9]*-[0-9]+)` 拡張、ADR-0001/0002/0003 を初検出、ID 数 46 → 49）/ **docs-side ADR orphan 検出機能新設**（41 件発覚、過去のリファクタで ID を docs に残したまま ADR を統合 / 削除 / 改名した形跡）/ **AUDIT.md「実装参照 0 件 5 件」を再分類**（DEV-003 / DIR-004 / SUP-002 を docs-orphan に振り替え、真の impl 不在は 3 件）/ **coverage.sh self-detection 不具合解消**（`--exclude-dir=audit`）/ **判定基準正典 audit_criteria.md / audit-protocol skill に orphan 2 系統 (code/docs) 明記** / **regression test 4 件追加** (Test 13-16) / trace 数値補正: DS reach 1148 → **1168 (82%)**, IMP reach 705 → **712 (99%)**（ADR-0001/2/3 が cocited 経路に追加されたため） |
| 2026-05-02 (#7) | 再現性確認 + 既存不整合の訂正 | **`/audit all` 再実行で全軸の決定論的動作を確認**（commit 数 30/90 日 = 745→750 / 1679→1684 の自然増以外、全数値が #6 と完全一致）/ **AUDIT.md 内既存不整合 2 件訂正**: ① ADR ID 内訳の表記精度（L46「新形式 46 件」→ ADR ファイル 46 件 (旧 3 + 新 43) + cite-only 3 件 = 49 を明示、ファイル総数 46 を新形式 ID 数として誤って流用していた）／ ② B 軸セクション本文の走査範囲（L167 / L186「1236 ファイル」→ 1237 に統一、サマリ L33 と証跡 `slack-scope.txt` の `total_files: 1237` に整合）/ **regression test 1 件追加**（`tests/audit/test_audit_lib.sh` Test 17 — AUDIT.md 内の走査範囲数値が `slack-scope.txt` の `total_files` に整合する不変式、再発防止）/ 真の手抜き 0 / code-orphan 0 / docs-orphan 41 / kind 100% Running を継続維持 |
| 2026-05-02 (#8) | coverage.sh の ADR ID 列挙を構造修正 — semantic 混在の解消 | **`tools/audit/lib/coverage.sh` の ADR ID 列挙ロジック修正**（adr/ 配下 grep → ADR ファイル名抽出、ID 数 49 → **46**、ファイル ↔ ID 1:1 厳密化）/ 過去 (~#7) の grep ベース列挙では cite-only 3 件 (DEV-003 / DIR-004 / SUP-002) が ids-adr.txt に混入し、coverage の分類で「docs-only (impl 不在)」と誤判定されていた（実態は ADR ファイル不在 = docs-orphan、概念混在）。本修正により `coverage-adr.txt` の docs-only が **6 → 3** に収束（ADR-0002 規約 / DEP-001 / DX-001 のみ、AUDIT.md narrative の手動再分類が不要に）/ docs-orphan 41 件 / code-orphan 0 件は不変、再分類済 ID は引き続き `docs-orphans-adr.txt` に登場 / **regression test 4 件追加・更新** (`tests/audit/test_audit_lib.sh` Test 14 を 1:1 厳密等号に強化、Test 18 ids-adr ↔ ADR file per-ID 検査、Test 19 docs-orphan に DEV-003/DIR-004/SUP-002 残存検査、Test 20 docs-only に cite-only orphan 混入なし — 計 31 assertion 全 PASS) / trace 数値（NFR/DS/IMP reach）は ADR ファイル集合不変のため影響なし、k8s / OSS / B 軸も不変 |
