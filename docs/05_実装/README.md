# 05. 実装

本ディレクトリは k1s0 の実装段階で参照する設計ドキュメントを格納する。概要設計（`docs/04_概要設計/`）で論理レベルの方式が確定した後、実装チーム（tier1 Rust / tier1 Go / tier2 / tier3 / infra / ops / docs）が具体的な作業に入る際のガイドラインを提供する。

## 本ディレクトリの位置付け

概要設計は「何をどの方式で実現するか」を論理レベルで固定するドキュメント群である。一方で実装段階では、ディレクトリ配置・ビルド単位・CI/CD 設定・運用スクリプト・スパースチェックアウトの cone 定義・サプライチェーン・ID 基盤・ガバナンスなど、物理配置と運用実務レベルの判断が大量に発生する。これらを概要設計に混ぜると、論理決定と物理配置の境界が ID 上で曖昧になり、改訂影響範囲の特定が困難になる。

本ディレクトリはその種の物理配置・運用実務ドキュメントを集約する。実装 ID は接頭辞 `IMP-` を持ち、概要設計 ID `DS-` と衝突しない。章ごとに接頭辞（`IMP-DIR-*` / `IMP-BUILD-*` / `IMP-CODEGEN-*` 等）を分割し、採番時の衝突と参照時の曖昧性を排除する。詳細は [../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md](../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md) の「実装 ID 体系」節および [00_ディレクトリ設計/00_設計方針/03_設計ID体系_IMP-DIR.md](00_ディレクトリ設計/00_設計方針/03_設計ID体系_IMP-DIR.md) を参照。

## 章構成

実装段階は 13 章で構成する。世界トップ企業事例（Google / Meta / Microsoft / Uber / Netflix / Shopify 等）の調査と 4 エージェント多ラウンド議論（docs/99_壁打ち/2026-04-23_05_実装フォルダ構成議論.md）を経て、以下の構成に確定した。

```
05_実装/
├── README.md                    # 本ファイル
├── 00_ディレクトリ設計/         # モノレポ物理配置（既存・IMP-DIR-*）
├── 10_ビルド設計/               # Cargo / go / pnpm / dotnet 選択ビルド（IMP-BUILD-*）
├── 20_コード生成設計/           # buf / openapi / Scaffold CLI（IMP-CODEGEN-*）
├── 30_CI_CD設計/                # GitHub Actions / path-filter / quality gate（IMP-CI-*）
├── 40_依存管理設計/             # Renovate / lockfile / vendoring（IMP-DEP-*）
├── 50_開発者体験設計/           # DevContainer / Golden Path / Backstage（IMP-DEV-*）
├── 60_観測性設計/               # LGTM + Pyroscope / SLO / Incident Taxonomy（IMP-OBS-*）
├── 70_リリース設計/             # Argo CD / Rollouts / flagd（IMP-REL-*）
├── 80_サプライチェーン設計/     # SLSA / cosign / SBOM / Forensics（IMP-SUP-*）
├── 85_Identity設計/             # Keycloak / SPIRE / OpenBao（IMP-SEC-*）
├── 90_ガバナンス設計/           # Kyverno / ADR / Threat Model（IMP-POL-*）
├── 95_DXメトリクス/             # DORA / SPACE / Scaffold 利用率（IMP-DX-*）
└── 99_索引/                     # IMP-* / ADR / DS-SW-COMP / NFR 横断索引（IMP-TRACE-*）
```

## 段階ごとの確定範囲

リリース時点（リリース時点）では以下の章を MUST とする。

- `00_ディレクトリ設計/` — モノレポレイアウト確定（既済）
- `10_ビルド設計/` — 4 言語ネイティブビルド境界
- `20_コード生成設計/` — buf / Scaffold CLI 最小実装
- `30_CI_CD設計/` — reusable workflow / quality gate
- `50_開発者体験設計/` — 10 役 DevContainer / Golden Path
- `60_観測性設計/` — SLO / Incident Taxonomy
- `70_リリース設計/` — Argo CD / Rollouts / flagd
- `80_サプライチェーン設計/` — cosign / SBOM / SLSA L2 / Forensics Runbook
- `85_Identity設計/` — Keycloak / SPIRE / OpenBao / cert-manager
- `95_DXメトリクス/` — DORA 4 keys / time-to-first-commit
- `99_索引/` — 横断索引スケルトン

リリース時点 以降で着手する章。

- `40_依存管理設計/` — Renovate 運用（リリース時点 SHOULD）
- `90_ガバナンス設計/` — Kyverno / Radar / 脅威モデル（リリース時点 SHOULD）

リリース時点+ で追加予定。

- `97_カオスとGameDay/` — LitmusChaos / 退職時 revoke 演習（リリース時点では除外）

## 実装段階の作業原則

- すべての物理配置・運用実務判断は本ディレクトリ配下のいずれかの Markdown に IMP-\* 接頭辞付き ID で記録する
- 物理配置・運用が対応する概要設計 ID（`DS-`）、ADR（`ADR-`）、NFR（`NFR-`）を本文末尾に明記し、双方向トレーサビリティを維持する
- 物理配置・運用の変更は概要設計の変更よりも頻度が高いことを許容するが、対応する `DS-` / `ADR-` を変更する際は本ディレクトリの該当 IMP-\* も同時更新する
- 各章の主担当（R）は RACI で明示し、他章の共担当（C）と境界を明文化する

## 関連ドキュメント

- [../../CLAUDE.md](../../CLAUDE.md) — プロジェクト共通規約
- [../00_format/document_standards.md](../00_format/document_standards.md) — ドキュメント書式規約
- [../04_概要設計/](../04_概要設計/) — 概要設計（論理方式）
- [../02_構想設計/adr/](../02_構想設計/adr/) — ADR（技術選定）
- ../99_壁打ち/2026-04-23_05_実装フォルダ構成議論.md — 本章構成の議論ログ
