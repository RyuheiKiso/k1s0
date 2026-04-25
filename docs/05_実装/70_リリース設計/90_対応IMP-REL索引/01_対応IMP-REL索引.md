# 01. 対応 IMP-REL 索引

本ファイルは `70_リリース設計/` 配下で採番された全 54 件の `IMP-REL-*` ID を一覧化し、ADR / DS-SW-COMP / NFR との対応をリバース参照可能にする索引である。`99_索引/` 配下の横断マトリクスとは異なり、本ファイルは章内に閉じた粒度でのナビゲーションを担う。

## なぜ章内索引を持つのか

`99_索引/` の全章横断マトリクスは IMP-* ID と ADR / NFR の対応を網羅するが、章内ナビゲーション（「リリース章のどの節に何の ID があるか」）には粒度が粗すぎる。本ファイルは章内に閉じた索引として、新規節追加時の番号衝突回避と、レビュー時の章内整合確認を支える。

## サブ接頭辞別の ID マップ

リリース章は次の 6 つのサブ接頭辞に分かれる。サブ接頭辞ごとに採番範囲を予約し、追加採番は範囲内で行う。

| サブ接頭辞 | 範囲 | 目的 | 物理位置 |
|---|---|---|---|
| `IMP-REL-POL-*` | 001〜009 | 7 軸原則 | `00_方針/01_リリース原則.md` |
| `IMP-REL-ARG-*` | 010〜019 | Argo CD App 構造 | `10_ArgoCD_App構造/01_ArgoCD_App構造.md` |
| `IMP-REL-PD-*` | 020〜029 | Argo Rollouts PD | `20_ArgoRollouts_PD/01_ArgoRollouts_PD設計.md` |
| `IMP-REL-FFD-*` | 030〜039 | flagd フィーチャーフラグ | `30_flagd_フィーチャーフラグ/01_flagd_フィーチャーフラグ設計.md` |
| `IMP-REL-AT-*` | 040〜049 | AnalysisTemplate | `40_AnalysisTemplate/01_AnalysisTemplate設計.md` |
| `IMP-REL-RB-*` | 050〜059 | rollback runbook | `50_rollback_runbook/01_rollback_runbook設計.md` |

7 軸原則のみ 7 ID で範囲が緩く、他は概ね 8〜10 ID で密度が均一。新規節（例: 60_release_train）を追加する際は次の予約範囲（060〜069）から採番する。

## 全 IMP-REL ID 一覧

### IMP-REL-POL-001〜007（7 軸原則）

| ID | 主題 | 原則の核心 |
|---|---|---|
| POL-001 | GitOps 原則 | クラスタ反映は Argo CD のみ、`kubectl apply` 直打ち禁止 |
| POL-002 | PD 必須化 | 例外は内部ツール / バッチ / emergency rollback のみ |
| POL-003 | AnalysisTemplate SLI 連動 | 手動「次へ進める」操作禁止、Mimir SLI で自動判定 |
| POL-004 | flagd cosign 署名必須 | 未署名フラグは Kyverno admission で reject |
| POL-005 | 15 分以内 rollback runbook | 演習を四半期で実施、超過時は構成改善必須 |
| POL-006 | canary 最低 3 段階 | 5%/25%/100% の中間段階観測窓を保証 |
| POL-007 | Release Notes Backstage 紐付け | image hash と 1:1 対応で Forensics 起点となる |

### IMP-REL-ARG-010〜017（Argo CD App 構造）

| ID | 主題 |
|---|---|
| ARG-010 | `deploy/apps/root-app.yaml` の app-of-apps 構成 |
| ARG-011 | 6 ApplicationSet（tier1 / 2 / 3 / infra / observability / security）と sync wave |
| ARG-012 | 環境別 sync policy（dev automated / staging no-selfHeal / prod manual） |
| ARG-013 | Helm + Kustomize 二層構成（chart 出力を overlay で env 差分） |
| ARG-014 | Argo CD HA（server x3 / repo-server x3 / controller x3 shard） |
| ARG-015 | backing store を Valkey Sentinel に統一 |
| ARG-016 | image-updater opt-in（dev / staging の tier2 / tier3 のみ） |
| ARG-017 | Argo CD Notifications（drift / health / sync の PagerDuty / Slack 連動） |

### IMP-REL-PD-020〜028（Argo Rollouts PD）

| ID | 主題 |
|---|---|
| PD-020 | Canary 3 段階既定（5% → 25% → 100%、各 5 分） |
| PD-021 | tier1 公開 11 API は リリース時点 で 10 段階細分化（30 分以内） |
| PD-022 | AnalysisTemplate 共通セット 5 本の `deploy/rollouts/analysis/` 配置 |
| PD-023 | Mimir Prometheus 互換 API を provider に統一（failureLimit 2） |
| PD-024 | 例外経路（rolling-internal / rolling-batch / rolling-emergency）と Kyverno 検証 |
| PD-025 | Blue-Green は `tier3/native/` MAUI 配布のみ |
| PD-026 | flagd 3 パターン連動（Release / Kill / Experiment） |
| PD-027 | 手動 rollback の 1 コマンド化と四半期演習 |
| PD-028 | Release Notes と image hash 連動（cosign subject + Backstage TechDocs） |

### IMP-REL-FFD-030〜039（flagd フィーチャーフラグ）

| ID | 主題 |
|---|---|
| FFD-030 | `infra/feature-management/flags/` の 4 種別ファイル構造 |
| FFD-031 | CODEOWNERS による reviewer 分離（kill switch は SRE オンコール 1 名） |
| FFD-032 | JSON Schema + 構文チェックの CI 必須化 |
| FFD-033 | cosign keyless 署名 + OCI Artifact 化 + Rekor 記録 |
| FFD-034 | Kyverno admission policy による cosign verify-blob |
| FFD-035 | flagd sidecar 配置（30 秒キャッシュ、p99 5ms 以内） |
| FFD-036 | OpenFeature SDK 4 言語統合（Rust / Go / Node / .NET） |
| FFD-037 | 3 パターン責務分離（Release / Kill / Experiment） |
| FFD-038 | 評価ログ OTel span 化（PII hash 化）と Loki 30 日保管 |
| FFD-039 | kill switch 発動の PagerDuty Sev2 自動連動 |

### IMP-REL-AT-040〜049（AnalysisTemplate）

| ID | 主題 |
|---|---|
| AT-040 | `at-common-error-rate` baseline 2σ 超過判定 |
| AT-041 | `at-common-latency-p99` SLO 値超過判定（args 経由） |
| AT-042 | `at-common-cpu` 80% 10 分継続判定（count 10 + failureLimit 8） |
| AT-043 | `at-common-dependency-down` 短絡判定（failureLimit 1） |
| AT-044 | `at-common-error-budget-burn` burn rate 2x 超過判定（IMP-OBS-EB-053 連動） |
| AT-045 | ClusterAnalysisTemplate（共通） / AnalysisTemplate（固有）スコープ分離 |
| AT-046 | Mimir provider 統一（`mimir-query-frontend.k1s0-observability:8080`） |
| AT-047 | Scaffold CLI による共通テンプレ自動挿入 |
| AT-048 | 月次カバレッジ計測（Backstage TechInsights 連動） |
| AT-049 | サービス固有テンプレの SRE 承認必須化 |

### IMP-REL-RB-050〜059（rollback runbook）

| ID | 主題 |
|---|---|
| RB-050 | 5 段階タイムライン（検知 2 + revert 3 + sync 5 + AT 5 + 観測 5 = 15 分） |
| RB-051 | `ops/scripts/rollback.sh` 1 コマンド化と GitHub OIDC 認証 |
| RB-052 | SRE 承認の Branch Protection 強制（4-eyes 原則の構造的保証） |
| RB-053 | Phase 3 の Helm sync + Rollout undo 並列実行 |
| RB-054 | 第二経路（forward fix）の二者承認 |
| RB-055 | 第三経路（全停止 / Maintenance Page）の三者承認 |
| RB-056 | 四半期演習（staging chaos + 実時間計測） |
| RB-057 | サービス別 runbook の `ops/runbooks/incidents/rollback-<service>/` 配置 |
| RB-058 | Postmortem PR 自動生成（24 時間期限） |
| RB-059 | Incident メタデータ必須記録と四半期可視化 |

## ADR からの逆引き

リリース章で参照している ADR ごとの IMP-REL ID 分布。`99_索引/10_ADR対応表/` は全章横断版で、本表は本章内の分布に絞った索引。

| ADR | 参照する IMP-REL ID |
|---|---|
| ADR-CICD-001（Argo CD） | POL-001 / ARG-010〜017 / FFD-033 / RB-050〜053 |
| ADR-CICD-002（Argo Rollouts） | POL-002 / 003 / 006 / PD-020〜028 / AT-040〜049 / RB-050〜053 |
| ADR-CICD-003（Kyverno） | POL-004 / FFD-034 / PD-024 |
| ADR-FM-001（flagd / OpenFeature） | POL-004 / PD-026 / FFD-030〜039 |
| ADR-OBS-001（Grafana LGTM） | AT-046 / RB-059 |
| ADR-REL-001（PD 必須化） | POL-002 / PD-020〜028 / AT-040〜049 |
| ADR-STOR-001（Longhorn） | ARG-014 |
| ADR-DATA-004（Valkey） | ARG-015 |
| ADR-SEC-001（Keycloak） | ARG-014 |
| ADR-0001（Istio Ambient） | PD-020 / RB-055 |
| ADR-0003（AGPL 分離 / 監査） | （横断、本章では明示参照なし） |

## DS-SW-COMP からの逆引き

| DS-SW-COMP ID | 該当エンティティ | 参照する IMP-REL ID |
|---|---|---|
| 135 | 配信系インフラ | POL-001〜007 / ARG-010〜017 / PD-020〜028 / FFD-030〜039 / RB-050〜059 / AT-046 |
| 141 | 多層防御統括（Observability + Security） | FFD-034 / FFD-039 / RB-055 / RB-059 |
| 085 | OTel Gateway / Mimir | AT-040〜044 / AT-046 / RB-059 |

DS-SW-COMP-135 の densitiy は 54 ID で、本章は配信系の中核を構成する。

## NFR からの逆引き

| NFR ID | 主題 | 参照する IMP-REL ID |
|---|---|---|
| NFR-A-CONT-001 | SLA 99% | POL-001〜007 / ARG-010〜017 / PD-020〜028 / AT-040〜049 / RB-050〜059 |
| NFR-A-FT-001 | 自動復旧 15 分 | POL-005 / PD-027 / RB-050〜059 |
| NFR-C-IR-001 | Incident Response | RB-050〜059 |
| NFR-C-IR-002 | Circuit Breaker | POL-006 / PD-020 / PD-021 / RB-050〜055 |
| NFR-D-MTH-002 | Canary / Blue-Green | POL-006 / PD-020〜025 / AT-040〜049 |
| NFR-H-INT-001 | Cosign 署名 | POL-004 / FFD-033 / FFD-034 |
| NFR-E-MON-002 | 操作監査 | FFD-038 / FFD-039 / RB-051 / RB-059 |
| NFR-I-EB-001 | エラーバジェット | AT-044 |

## 関連章との境界

- [`../README.md`](../README.md) — 章全体の節構成と RACI
- [`../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md) — 全章横断の IMP-ID 台帳
- [`../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md) — ADR ↔ IMP の全章横断対応
- [`../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md`](../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md) — DS-SW-COMP ↔ IMP 対応
- [`../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md`](../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md) — NFR ↔ IMP 対応
