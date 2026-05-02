# 01. 対応 IMP-CI 索引

本ファイルは [`05_実装/30_CI_CD設計/`](../README.md) 章配下で採番された全 `IMP-CI-*` ID を 1 ページに集約する横断索引である。各 ID から所在ファイル・対応原則・関連 ADR / DS / NFR への逆引きが可能で、PR レビュー時の影響範囲確認や、新規 ID 採番時の重複チェックを最短動線で行うために用意する。本索引は `IMP-CI-*` の正典とし、各章本文と齟齬が出た場合は本索引を改訂後に各章を更新する運用とする。

## 採番ルール

`IMP-CI-*` ID は次の規約で採番する。`20_コード生成設計/90_対応IMP-CODEGEN索引/` の規約と整合させ、接頭辞 → 番号レンジ → 連番の 3 段で運用する。

- 形式: `IMP-CI-<接頭辞>-<番号>`
  - 接頭辞は本章配下のサブディレクトリ単位で割り当てる（`POL` = 方針、`RWF` = reusable workflow、`PF` = path-filter、`HAR` = Harbor + Trivy + push、`QG` = quality gate、`BP` = branch protection）
  - 番号は 3 桁ゼロ埋めで本章全体を通じて重複しない（接頭辞横断の単一番号空間）
- 接頭辞別の番号レンジは 10 単位で予約し、欠番が出ても再利用しない
- 採番時は本索引の対応表を**先に**更新し、その後で本文ファイルに ID を埋め込む
- ID の意味（説明文）は本索引と本文ファイルで完全一致させる

接頭辞と章の対応は以下とする。

| 接頭辞 | 略称 | 所在 | 番号レンジ |
|---|---|---|---|
| `POL` | Policy | [`00_方針/01_CI_CD原則.md`](../00_方針/01_CI_CD原則.md) | 001 〜 009 |
| `RWF` | Reusable Workflow | [`10_reusable_workflow/01_reusable_workflow設計.md`](../10_reusable_workflow/01_reusable_workflow設計.md) | 010 〜 029 |
| `PF` | Path Filter | [`20_path_filter選択ビルド/01_path_filter選択ビルド.md`](../20_path_filter選択ビルド/01_path_filter選択ビルド.md) | 030 〜 039 |
| `HAR` | Harbor + Trivy + push | [`40_Harbor_Trivy_push/01_Harbor_Trivy_push設計.md`](../40_Harbor_Trivy_push/01_Harbor_Trivy_push設計.md) | 040 〜 059 |
| `QG` | Quality Gate | [`30_quality_gate/01_quality_gate.md`](../30_quality_gate/01_quality_gate.md) | 060 〜 069 |
| `BP` | Branch Protection | [`50_branch_protection/01_branch_protection.md`](../50_branch_protection/01_branch_protection.md) | 070 〜 079 |
| `LCDT` | Local Dev Tooling | [`60_ローカル開発ツール/01_ローカル検査と自動修正設計.md`](../60_ローカル開発ツール/01_ローカル検査と自動修正設計.md) | 080 〜 089 |

QG が章番号 30 でありながら ID レンジが 060 始まりなのは、HAR（章番号 40）の初期採番が 040〜051 まで広がり 040〜049 のレンジを超過したためレンジを 040〜059 に拡張した結果である。本章全体で「番号は本章全体を通じて重複しない」原則を守るため、QG は HAR レンジの直後 060 から開始した。章番号と ID レンジの不一致は採番時の混乱を招くため、本索引で必ず両者の対応表を確認して採番すること。

## 全 ID 一覧（接頭辞別）

### POL: CI/CD 原則（7 件）

| ID | 概要 |
|---|---|
| `IMP-CI-POL-001` | CI 責務は Harbor push まで（クラスタ反映は Argo CD） |
| `IMP-CI-POL-002` | quality gate は reusable workflow で統制 |
| `IMP-CI-POL-003` | path-filter による選択ビルド（4 軸: tier / 言語 / workspace / contracts） |
| `IMP-CI-POL-004` | Harbor 門番の Trivy Critical 拒否 |
| `IMP-CI-POL-005` | cosign keyless 署名で完結（Fulcio + Rekor） |
| `IMP-CI-POL-006` | branch protection 経由のマージのみ |
| `IMP-CI-POL-007` | Renovate PR の自動ビルドと patch 自動マージ |

### RWF: Reusable Workflow（12 件）

| ID | 概要 |
|---|---|
| `IMP-CI-RWF-010` | reusable workflow 4 本（lint / test / build / push）構成と 1 言語 1 job 原則 |
| `IMP-CI-RWF-011` | docs 単独変更時の lint-only 経路 |
| `IMP-CI-RWF-012` | path-filter golden test による filter 定義変更保護 |
| `IMP-CI-RWF-013` | Karpenter + ARC による runner 自動スケール |
| `IMP-CI-RWF-014` | CI secret の最小集合化と注入経路固定 |
| `IMP-CI-RWF-015` | env 明示・secret echo 禁止の lint |
| `IMP-CI-RWF-016` | cache キー規約（`os-tier-language-hash(lockfile)`）と共有 backend |
| `IMP-CI-RWF-017` | reusable workflow の tag 固定参照と Renovate 連携 |
| `IMP-CI-RWF-018` | coverage 閾値の段階導入（リリース時点 計測のみ → 採用初期 80% → 運用拡大 90%） |
| `IMP-CI-RWF-019` | 失敗時の可読性（failure_reason outputs と Loki / Mimir 連携） |
| `IMP-CI-RWF-020` | workflow リポジトリ分離のリリース時点不採用 |
| `IMP-CI-RWF-021` | composite action の内部実装扱いと `tools/ci/actions/` 配置 |

### PF: Path Filter（8 件）

| ID | 概要 |
|---|---|
| `IMP-CI-PF-030` | `dorny/paths-filter@v3` の採用と `@v3` major 固定 |
| `IMP-CI-PF-031` | `tools/ci/path-filter/filters.yaml` の単一真実源化（10 ビルド章と本章で共有） |
| `IMP-CI-PF-032` | 4 軸（tier / 言語 / workspace / contracts 横断）の判定構造 |
| `IMP-CI-PF-033` | `contracts=true → sdk-all=true` 強制昇格の物理化 |
| `IMP-CI-PF-034` | `_reusable-*.yml` への filter outputs 伝搬と起動条件 |
| `IMP-CI-PF-035` | `tools/ci/path-filter/run-golden-test.sh` による filter 変更保護 |
| `IMP-CI-PF-036` | 集約 job `ci-overall` 1 本のみを必須 status check とする運用 |
| `IMP-CI-PF-037` | キャッシュキーへの tier / 言語伝搬による衝突回避 |

### HAR: Harbor + Trivy + push（12 件）

| ID | 概要 |
|---|---|
| `IMP-CI-HAR-040` | Harbor 物理配置と CloudNativePG バックエンド |
| `IMP-CI-HAR-041` | 5 Harbor project（tier1 / tier2 / tier3 / infra / sdk）と RBAC 分離 |
| `IMP-CI-HAR-042` | quarantine プロジェクトへの自動隔離 |
| `IMP-CI-HAR-043` | robot アカウントの 60 日自動ローテ |
| `IMP-CI-HAR-044` | CVSS 連動の 4 段階閾値運用（Trivy CVE Critical 拒否） |
| `IMP-CI-HAR-045` | Trivy DB の日次更新とオフラインミラー |
| `IMP-CI-HAR-046` | allowlist 例外の 30 日時限 + Security 承認 |
| `IMP-CI-HAR-047` | cosign keyless 署名と Rekor 記録 |
| `IMP-CI-HAR-048` | Harbor DR replication と段階展開 |
| `IMP-CI-HAR-049` | Harbor / Trivy / cosign の SLI 計測と SLO 定義 |
| `IMP-CI-HAR-050` | 手動 push 禁止と緊急時の一時付与手順 |
| `IMP-CI-HAR-051` | Retention / GC policy とスナップショット管理 |

### QG: Quality Gate（8 件）

| ID | 概要 |
|---|---|
| `IMP-CI-QG-060` | 4 ゲート（fmt / lint / unit-test / coverage）の順序固定 |
| `IMP-CI-QG-061` | fmt は `--check` モード固定で自動修正禁止 |
| `IMP-CI-QG-062` | lint は warning 全て error 化（`-D warnings` / `--max-warnings 0`） |
| `IMP-CI-QG-063` | `tools/lint/` 配下の単一真実源化（tier 別独自ルール禁止） |
| `IMP-CI-QG-064` | unit-test は外部依存モック必須（integration-test は別 workflow） |
| `IMP-CI-QG-065` | カバレッジ段階導入（リリース時点 計測のみ → 採用初期 80% → 運用拡大 90%） |
| `IMP-CI-QG-066` | Cobertura XML 統一とベンダー SaaS 非送信 |
| `IMP-CI-QG-067` | 各ゲート failed 時の後段 skip 伝搬構造 |

### BP: Branch Protection（8 件）

| ID | 概要 |
|---|---|
| `IMP-CI-BP-070` | 必須 status check は `ci-overall` 1 本のみ（個別 job 不可） |
| `IMP-CI-BP-071` | strict mode 有効化（main 最新 commit を含む状態でのみ merge） |
| `IMP-CI-BP-072` | 必須レビュー数（PoC 1 / 拡大期 2）と CODEOWNERS 自動指名 |
| `IMP-CI-BP-073` | squash merge 強制 / linear history 必須 |
| `IMP-CI-BP-074` | 署名コミット必須（SSH / GPG / Web UI） |
| `IMP-CI-BP-075` | 管理者にも適用 / direct push 禁止 / merge queue は採用拡大期 |
| `IMP-CI-BP-076` | terraform-provider-github による rule の Git 管理（IaC 化） |
| `IMP-CI-BP-077` | `release/*` ブランチに main と同一 rule を適用 |

### LCDT: Local Dev Tooling（3 件）

| ID | 概要 |
|---|---|
| `IMP-CI-LCDT-080` | ローカル検査 orchestrator（`make verify` / `verify-quick` / `tools/ci/verify-local.sh`） |
| `IMP-CI-LCDT-081` | PR title 自動補正 workflow（`.github/workflows/pr-title-autofix.yml`） |
| `IMP-CI-LCDT-082` | branch 命名規約（`<type>/<scope>/<subject>` 形式を入力源にする） |

### CONF: CNCF Conformance（5 件、ADR-TEST-003 で確定）

ADR-CNCF-001 の「移行・対応事項」（CNCF Conformance テスト sonobuoy を kind multi-node で定期実行する CI を整備）を ADR-TEST-003 が具体化。Sonobuoy 実行・cluster 構成・実行頻度・report 保管・採用検討者向け公開を 5 ID で確定。

| ID | 概要 |
|---|---|
| `IMP-CI-CONF-001` | Sonobuoy v0.57+ を `--mode certified-conformance` で実行 |
| `IMP-CI-CONF-002` | kind multi-node（control-plane 1 + worker 3）+ Calico CNI 構成（ADR-NET-001 整合） |
| `IMP-CI-CONF-003` | 月次実行（cron 毎月 1 日 03:00 JST）+ workflow_dispatch（`.github/workflows/conformance.yml`） |
| `IMP-CI-CONF-004` | results.tar.gz + summary.md を 12 ヶ月分 git LFS で `tests/.conformance/<YYYY-MM>/` に版管理 |
| `IMP-CI-CONF-005` | failure 時に `docs/40_運用ライフサイクル/conformance-results.md` で公開、起案者に notify |

### TAG: テスト属性タグ + CI 実行フェーズ分離（5 件、ADR-TEST-007 で確定）

ADR-TEST-007 で確定したテストケース粒度の属性タグと 4 段実行フェーズ（PR / nightly / weekly / release）を 5 ID で実装段階に展開。IMP-CI-PF-031（path-filter）と orthogonal 並立。

| ID | 概要 |
|---|---|
| `IMP-CI-TAG-001` | 4 タグ最低セット（`@slow` / `@flaky` / `@security` / `@nightly`）の正典化 |
| `IMP-CI-TAG-002` | 4 段実行フェーズ（PR / nightly / weekly / release tag）の起動 trigger 一意化 |
| `IMP-CI-TAG-003` | 言語別属性タグ実装（Rust ignore / Go build tag / xUnit Trait / Vitest filter） |
| `IMP-CI-TAG-004` | flaky 自動検出（`tools/qualify/flaky-detector.py` で直近 20 PR fail 率 ≥ 5% を quarantine 自動追加） |
| `IMP-CI-TAG-005` | `tests/.flaky-quarantine.yaml` の PR レビュー必須化（quarantine 解除も同様） |

## 採番済み件数まとめ

| 接頭辞 | 件数 | 残レンジ | 次番 |
|---|---|---|---|
| `POL` | 7 | 002 件（008-009） | `IMP-CI-POL-008` |
| `RWF` | 12 | 008 件（022-029） | `IMP-CI-RWF-022` |
| `PF` | 8 | 002 件（038-039） | `IMP-CI-PF-038` |
| `HAR` | 12 | 008 件（052-059） | `IMP-CI-HAR-052` |
| `QG` | 8 | 002 件（068-069） | `IMP-CI-QG-068` |
| `BP` | 8 | 002 件（078-079） | `IMP-CI-BP-078` |
| `LCDT` | 3 | 007 件（083-089） | `IMP-CI-LCDT-083` |
| `CONF` | 5 | 005 件（006-010） | `IMP-CI-CONF-006` |
| `TAG` | 5 | 005 件（006-010） | `IMP-CI-TAG-006` |
| **合計** | **68** | **41** | — |

POL は OSS 公開時点の確定 7 原則で採番が一段落しており、新規原則の追加はリリース戦略の節目（採用拡大期 / メジャーバージョン）でしか発生しない見通し。RWF / HAR は 8 件残っており、運用観測（60 章）や リリース（70 章）からの参照増加に耐える余裕がある。CONF / TAG は ADR-TEST-003 / 007 で本体決定確定、実装段階の詳細記述は採用初期で各 ID に対応する設計書節を追加する。

## 対応 ADR 逆引き

`IMP-CI-*` から参照される ADR を逆引きで一覧化する。各 ADR がどの IMP-CI ID に影響するかを把握する時に使う。

| ADR | 影響する IMP-CI ID |
|---|---|
| [ADR-CICD-001](../../../02_構想設計/adr/ADR-CICD-001-argocd.md)（Argo CD） | POL-001（CI/CD 境界）, RWF-010〜021, PF-030〜037, QG-060〜067, BP-070〜077（GitHub Actions 前提） |
| [ADR-CICD-002](../../../02_構想設計/adr/ADR-CICD-002-argo-rollouts.md)（Argo Rollouts） | POL-001（CI/CD 境界の確認） |
| [ADR-CICD-003](../../../02_構想設計/adr/ADR-CICD-003-kyverno.md)（Kyverno） | POL-005, HAR-047（cosign 署名検証連携） |
| [ADR-DIR-003](../../../02_構想設計/adr/ADR-DIR-003-sparse-checkout-cone-mode.md)（sparse checkout cone） | PF-031（filters.yaml の cone 配置）, BP-076（infra/github の cone 配置） |
| [ADR-TIER1-001](../../../02_構想設計/adr/ADR-TIER1-001-go-rust-hybrid.md)（Go + Rust ハイブリッド） | RWF-010（言語別 reusable workflow）, QG-060〜063（4 言語 toolchain）, TAG-003（言語別属性タグ実装） |
| [ADR-TIER1-002](../../../02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md)（Protobuf gRPC） | POL-003（contracts 軸 path-filter）, PF-033（sdk-all 強制昇格） |
| [ADR-CNCF-001](../../../02_構想設計/adr/ADR-CNCF-001-cncf-conformance.md)（CNCF Conformance） | CONF-001〜005（Sonobuoy 月次実行体制） |
| [ADR-NET-001](../../../02_構想設計/adr/ADR-NET-001-cni-selection.md)（CNI 選定） | CONF-002（kind multi-node Calico） |
| [ADR-TEST-001](../../../02_構想設計/adr/ADR-TEST-001-test-pyramid-and-testcontainers.md)（Test Pyramid + testcontainers） | RWF-010（言語別 test）, QG-060〜067（4 ゲート）, QG-065（coverage 段階導入） |
| [ADR-TEST-003](../../../02_構想設計/adr/ADR-TEST-003-cncf-conformance-sonobuoy.md)（Sonobuoy 月次） | CONF-001〜005（本 ADR で確定） |
| [ADR-TEST-007](../../../02_構想設計/adr/ADR-TEST-007-test-tag-and-ci-phase-split.md)（テスト属性タグ + フェーズ分離） | TAG-001〜005（本 ADR で確定）, PF-031（path-filter と orthogonal 並立） |

新規 ADR 起票時は本逆引き表と本文両方を同期更新する。

## 対応 DS-SW-COMP 逆引き

| DS-SW-COMP | 影響する IMP-CI ID |
|---|---|
| DS-SW-COMP-135（CI/CD 配信系：Harbor / ArgoCD / Backstage / Scaffold の起動条件統制） | 全 ID（本章の主たる対応 DS） |
| DS-SW-COMP-122（contracts → 4 言語生成 / SDK） | PF-033（contracts 強制昇格） |
| DS-SW-COMP-140（外部 IF 設計） | HAR-040〜051（Harbor を外部に出すかの境界） |

## 対応 NFR 逆引き

| NFR | 影響する IMP-CI ID |
|---|---|
| [NFR-C-NOP-004](../../../03_要件定義/30_非機能要件/C_運用保守性.md)（ビルド時間：リリース時点 30 分 / 採用初期 20 分以内） | RWF-013（自動スケール）, RWF-016（cache）, PF-030〜037（選択ビルド）, QG-060〜067（並列実行） |
| [NFR-C-MGMT-001](../../../03_要件定義/30_非機能要件/C_運用保守性.md)（設定 Git 管理） | PF-031（filters.yaml）, BP-076（terraform-provider-github） |
| [NFR-C-MNT-003](../../../03_要件定義/30_非機能要件/C_運用保守性.md)（保守性） | QG-060〜067（lint / coverage 機械化） |
| [NFR-C-QLT-002](../../../03_要件定義/30_非機能要件/C_運用保守性.md)（品質） | QG-065（カバレッジ段階導入） |
| [NFR-H-INT-001](../../../03_要件定義/30_非機能要件/E_セキュリティ.md)（Cosign 署名） | POL-005, HAR-047, BP-074（署名コミット必須） |
| [NFR-H-INT-002](../../../03_要件定義/30_非機能要件/E_セキュリティ.md)（SBOM 添付） | POL-005, HAR-040〜051（push 経路で SBOM 生成） |
| NFR-E-MON-004（Flag/Decision 変更監査） | BP-076（terraform 履歴で rule 変更監査） |

## 上位索引との連携

本索引は [`05_実装/30_CI_CD設計/`](../README.md) 章内の局所索引である。`IMP-CI-*` 全件を含むより上位の索引は以下に置かれており、本索引はそこへ集約される位置付けとなる。

- [`05_実装/99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md) — `IMP-*` 全接頭辞（DIR / BUILD / CODEGEN / CI / DEP / DEV / OBS / REL / SUP / SEC / POL / DX / TRACE）の横断索引
- [`05_実装/99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md) — ADR と IMP-* の双方向マトリクス
- [`05_実装/99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md`](../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md) — 概要設計 ID と実装 ID の対応
- [`05_実装/99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md`](../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md) — NFR と IMP-* のトレース

新規章追加・新規接頭辞追加・新規 ID 採番のたびに本索引と上位索引の両方を同期更新する。同期忘れは PR レビュー（[`docs-review-checklist`](../../../00_format/review_checklist.md)）の必須チェック項目とする。

## 関連章 / 参照

- [`README.md`](../README.md) — 本章の章構成・段階確定範囲
- [`00_方針/01_CI_CD原則.md`](../00_方針/01_CI_CD原則.md) — POL ID の本文
- [`10_reusable_workflow/01_reusable_workflow設計.md`](../10_reusable_workflow/01_reusable_workflow設計.md) — RWF ID の本文
- [`20_path_filter選択ビルド/01_path_filter選択ビルド.md`](../20_path_filter選択ビルド/01_path_filter選択ビルド.md) — PF ID の本文
- [`30_quality_gate/01_quality_gate.md`](../30_quality_gate/01_quality_gate.md) — QG ID の本文
- [`40_Harbor_Trivy_push/01_Harbor_Trivy_push設計.md`](../40_Harbor_Trivy_push/01_Harbor_Trivy_push設計.md) — HAR ID の本文
- [`50_branch_protection/01_branch_protection.md`](../50_branch_protection/01_branch_protection.md) — BP ID の本文
- [`../20_コード生成設計/90_対応IMP-CODEGEN索引/01_対応IMP-CODEGEN索引.md`](../../20_コード生成設計/90_対応IMP-CODEGEN索引/01_対応IMP-CODEGEN索引.md) — 隣接章の対応索引（書式統一の参照元）
