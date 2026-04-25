# 95. DX メトリクス / 90. 対応 IMP-DX 索引 / 01. 対応 IMP-DX 索引

本ファイルは 95 章 DX メトリクスで採番された全 IMP-DX-\* ID を、サブ接頭辞ごと・節ごとに逆引きできる形で集約する。本章は IMP-TRACE-POL-002（接頭辞 001-099 の予約帯）の系として、全 57 件（POL 7 件 + 実装 50 件）を 99 枠内で運用する。

## 採番状況サマリ

| サブ接頭辞 | 範囲 | 採番済 | 所属節 | 適用段階 |
|---|---|---|---|---|
| POL | 001-007 | 7 | `00_方針/01_DXメトリクス原則.md` | 0 |
| DORA | 010-020 | 11 | `10_DORA_4keys/01_DORA_4keys計測.md` | リリース時点〜採用初期 |
| SPC | 021-029 | 9 | `20_SPACE/01_SPACE設計.md` | リリース時点〜運用拡大期 |
| SCAF | 030-039 | 10 | `30_Scaffold利用率/01_Scaffold利用率計測.md` | リリース時点〜運用拡大期 |
| TFC | 040-049 | 10 | `40_time_to_first_commit/01_time_to_first_commit計測.md` | リリース時点〜運用拡大期 |
| EMR | 050-059 | 10 | `50_EMレポート/01_EM月次レポート設計.md` | リリース時点（1 件のみ）〜運用拡大期 |

採番済合計 57 件 / 予約残 42 件。リリース時点で物理計測点として確定するのは 14 件（DORA 全 11 + SPC-022 / SPC-025 / SPC-028 + SCAF-030〜033 / SCAF-039 + TFC-040〜043 + EMR-059）。残りは採用初期 / 運用拡大期に段階的に有効化する。

## サブ接頭辞別 ID 一覧

### POL（00_方針）

| ID | 概要 |
|---|---|
| IMP-DX-POL-001 | DORA Four Keys を リリース時点 計測 |
| IMP-DX-POL-002 | Severity 別分離 |
| IMP-DX-POL-003 | Deploy Frequency 分母定義 |
| IMP-DX-POL-004 | MTTR ユーザ影響終点 |
| IMP-DX-POL-005 | time-to-first-commit 独自 SLI |
| IMP-DX-POL-006 | Backstage Scorecards 連動 |
| IMP-DX-POL-007 | 四半期レビュー |

### DORA（10_DORA_4keys）

10 章で採番済 11 件。DORA Four Keys（Lead Time / Deploy Frequency / Change Failure Rate / MTTR）の計測点・Severity 分離・postmortem 連動・Backstage Scorecards 連動を含む。詳細は `10_DORA_4keys/01_DORA_4keys計測.md` 参照。

### SPC（20_SPACE）

| ID | 概要 | 段階 |
|---|---|---|
| IMP-DX-SPC-021 | Survey Plugin 配信パイプライン（四半期 NPS） | 採用初期 |
| IMP-DX-SPC-022 | Activity span（PR / Review / Scaffold 起動）の OTel 変換 | リリース時点 |
| IMP-DX-SPC-023 | Communication ログ集約（PR review event / Slack thread） | 採用初期 |
| IMP-DX-SPC-024 | Efficiency opt-in 計測経路（Calendar / IDE telemetry） | 運用拡大期 |
| IMP-DX-SPC-025 | 個人特定排除のための PII transform 経路 | リリース時点 |
| IMP-DX-SPC-026 | 月次 MinIO Snapshot WORM 保管（経時比較用） | 採用初期 |
| IMP-DX-SPC-027 | Backstage Scorecards SPACE 5 軸ペイン | 採用初期 |
| IMP-DX-SPC-028 | チーム単位集計のみ可視化（個人ランキング化禁止） | リリース時点 |
| IMP-DX-SPC-029 | EM 評価プロセス組込時の対象軸明示（P / A のみ評価可） | 運用拡大期 |

### SCAF（30_Scaffold利用率）

| ID | 概要 | 段階 |
|---|---|---|
| IMP-DX-SCAF-030 | Scaffold CLI 起動の OTel span 化（template_id / outcome / role タグ） | リリース時点 |
| IMP-DX-SCAF-031 | Backstage Software Template UI 起動の Scaffolder action emit OTel 連動 | リリース時点 |
| IMP-DX-SCAF-032 | GitHub Webhook（repo:create / push）経由の Off-Path 検出 | リリース時点 |
| IMP-DX-SCAF-033 | 月次 Backstage Catalog 走査による catalog-info.yaml 不在 component 集計 | リリース時点 |
| IMP-DX-SCAF-034 | Paved Road 健全度の式定義と Mimir recording rule 化 | 採用初期 |
| IMP-DX-SCAF-035 | Backstage Scorecards への Adoption Rate 表示 | 採用初期 |
| IMP-DX-SCAF-036 | EM 月次レポートへの自動配信（IMP-DX-EMR-050〜059 連動） | 採用初期 |
| IMP-DX-SCAF-037 | Scaffold 利用率閾値による Paved Road 再整備トリガ（Sev3 Slack 通知） | 運用拡大期 |
| IMP-DX-SCAF-038 | template_id 別の Adoption Rate 分解（不人気テンプレートの early sign） | 採用初期 |
| IMP-DX-SCAF-039 | author を hash 化する PII transform 経路 | リリース時点 |

### TFC（40_time_to_first_commit）

| ID | 概要 | 段階 |
|---|---|---|
| IMP-DX-TFC-040 | Stage 0〜4 各境界の OTel span 出力 | リリース時点 |
| IMP-DX-TFC-041 | onboardingTimeFactRetriever の Backstage TechInsights 統合 | リリース時点 |
| IMP-DX-TFC-042 | new_joiner_hash による個人特定排除（HR システム連携） | リリース時点 |
| IMP-DX-TFC-043 | Mimir tfc_stage_duration_seconds histogram（cohort 別 p50 / p95） | リリース時点 |
| IMP-DX-TFC-044 | Day 1 4 時間達成率 SLI 化 | 採用初期 |
| IMP-DX-TFC-045 | onboarding-stumble label 月次集計 | 採用初期 |
| IMP-DX-TFC-046 | cohort 別経時推移分析 | 採用初期 |
| IMP-DX-TFC-047 | Stage 別劣化検出と Slack Sev3 通知 | 運用拡大期 |
| IMP-DX-TFC-048 | 採用拡大期 2 時間以内目標の達成測定と差分分析 | 運用拡大期 |
| IMP-DX-TFC-049 | EM 月次レポート連携 | 採用初期 |

### EMR（50_EMレポート）

| ID | 概要 | 段階 |
|---|---|---|
| IMP-DX-EMR-050 | EM Report Generator の Backstage backend job 実装 | 採用初期 |
| IMP-DX-EMR-051 | 4 入力統合の PromQL + LogQL + SQL クエリ単一真実源化 | 採用初期 |
| IMP-DX-EMR-052 | Markdown / HTML / JSON 3 形式生成テンプレート | 採用初期 |
| IMP-DX-EMR-053 | Slack 配信パイプライン（短縮版 + 4 軸 1 行 + リンク） | 採用初期 |
| IMP-DX-EMR-054 | Confluence 配信パイプライン（全文 5 章構成 HTML） | 採用初期 |
| IMP-DX-EMR-055 | Backstage Catalog DX-Report Entity 自動更新 | 採用初期 |
| IMP-DX-EMR-056 | 機械的閾値違反検出と自動アクション提案ロジック | 運用拡大期 |
| IMP-DX-EMR-057 | onboarding-stumble label 月次集計と章立て統合 | 採用初期 |
| IMP-DX-EMR-058 | 統計的有意性判定（cohort サイズ別閾値表） | 運用拡大期 |
| IMP-DX-EMR-059 | 個人特定排除の物理担保（hash 化済データのみ流入を CI で検証） | リリース時点 |

## 関連 ADR

- **ADR-DX-001（DX メトリクス分離原則 / 新規起票予定）**: DX メトリクスを観測性設計から独立させる方針 ADR。本章 5 節（10 / 20 / 30 / 40 / 50）全てが直接対応する。
- **ADR-BS-001（Backstage）**: TechInsights / Scorecards / Catalog Entity を 4 節（SPC-027 / SCAF-035 / TFC-041 / EMR-055）の表示基盤として使用する。
- **ADR-OBS-001（Grafana LGTM）**: 4 入力データ層（Mimir / Loki）の物理基盤。
- **ADR-DEV-001（Paved Road 思想）**: SCAF / TFC が Paved Road 健全度の物理計測点として直接対応する。

## 関連 NFR

- **NFR-C-NOP-001（採用側の小規模運用）**: 全 IMP-DX-\* が間接的に該当する。EM 月次レポートと TFC が直接対応。
- **NFR-C-NOP-002（可視性）**: Backstage Scorecards 表示と EM 配信パイプライン全件。
- **NFR-G-CLS-001（PII 取扱）**: SPC-025 / SCAF-039 / TFC-042 / EMR-059 の 4 件で物理担保。
- **`03_要件定義/50_開発者体験/03_DevEx指標.md`**: DORA Four Keys 要件全件。

## 関連 DS-SW-COMP

- **DS-SW-COMP-085（OTel Collector Gateway）**: SPC-022 / SCAF-030〜032 / TFC-040 の span 集約基盤。
- **DS-SW-COMP-132（platform）**: 章全体の基盤。
- **DS-SW-COMP-135（配信系インフラ = Backstage / Scorecards）**: SCAF-035 / TFC-041 / EMR-050〜055 の表示基盤。
- **DS-SW-COMP-141（多層防御統括）**: PII transform / WORM 保管経路の Security 統制側。

## 関連章

- `10_DORA_4keys/` — リリース時点 確定済の DORA Four Keys 11 ID 詳細
- `20_SPACE/` — SPACE 5 軸 9 ID 詳細
- `30_Scaffold利用率/` — Scaffold 利用率 10 ID 詳細
- `40_time_to_first_commit/` — TFC 10 ID 詳細
- `50_EMレポート/` — EM 月次レポート 10 ID 詳細

## 関連索引

- IMP-ID 台帳: [`../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- ADR 対応: [`../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md)
- DS-SW-COMP 対応: [`../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md`](../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md)
- NFR 対応: [`../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md`](../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md)
