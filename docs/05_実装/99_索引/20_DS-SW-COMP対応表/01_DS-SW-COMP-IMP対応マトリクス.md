# 99. 索引 / 20. DS-SW-COMP 対応表 / 01. DS-SW-COMP-IMP 対応マトリクス

本ファイルは `04_概要設計/` 配下で管理される **DS-SW-COMP-\*** 設計 ID（Software Component の概要設計レベル ID）と、本実装段階で採番された IMP-\* ID の対応を「DS-SW-COMP から IMP を逆引き」する方向で表現する。DS-SW-COMP は `04_概要設計/80_トレーサビリティ/` に自律管理されるため、本章では読み取り専用の対応表として扱う（IMP-TRACE-POL-005 に基づく双方向リンクの一方）。

## マトリクスの読み方

DS-SW-COMP は概要設計段階で確定した「論理コンポーネント」の ID で、3 つの階層区分に分かれる。**tier1 俯瞰（001-019）** は公開 11 API や Observability Core などの論理機能、**Dapr ファサード（020-049 + 141）** は State / PubSub / Secret 等の Dapr Building Block の面、**Rust 自作領域（050-099）** は ZEN Engine / crypto / 採用側組織の固有機能、**SDK / 配信（120-135）** は SDK と Harbor / ArgoCD / Backstage などの配信系を指す。

実装段階（05_実装/）の IMP-\* はこれらの論理 ID に対して「どの物理配置・ビルド設定・運用手順で具体化されるか」を定義する関係にある。したがって 1 DS-SW-COMP : n IMP の関係が基本で、1 つの論理コンポーネントに複数の実装判断がぶら下がる。本ファイルはその n を示し、概要設計改訂時の実装影響範囲を逆引きできる状態にする。

リリース時点で本章にて採番された IMP-\* が直接参照する DS-SW-COMP は 17 種（001〜006、019、085、119〜135、141）で、これ以外の DS-SW-COMP は 00 章 `00_ディレクトリ設計/` 側で受けるか、リリース時点 以降の実装節で採番される。

## tier1 俯瞰層（DS-SW-COMP-001〜019）

tier1 の公開 11 API と Observability Core、SDK 統合点などの論理機能 ID。リリース時点 本章で直接参照されるのは 4 件。

### DS-SW-COMP-001（tier1 公開 API 集合）

tier1 の公開 11 API を束ねた論理コンポーネント。本章では SLO 定義の源泉として参照される。

- 直接: `IMP-OBS-SLO-040〜047`（tier1 公開 11 API の SLO 階層設計）
- 間接: `IMP-OBS-POL-003`（Google SRE Book 準拠の SLO 思想） / `IMP-REL-POL-003`（AnalysisTemplate の SLO 判定源）

### DS-SW-COMP-002（tier1 Observability Core）

tier1 の観測性取り込み論理コンポーネント。OTel トレース・メトリクス・ログの集約点。

- 直接: `IMP-OBS-SLO-040〜047`（内部 SLO 99.9% の計測点）
- 間接: `IMP-OBS-OTEL-010〜019`（Collector が Observability Core の物理実装）

### DS-SW-COMP-003（tier1 共通基盤）

tier1 全般で共通して使われる基盤ライブラリ群（ロガー / メトリクス抽象 / エラー型）。

- 直接: `IMP-BUILD-CW-010〜017`（Rust 共通 crate の workspace 配置） / `IMP-BUILD-GM-020〜027`（Go 共通 module の分離）
- 間接: `IMP-CODEGEN-BUF-010〜017`（Protobuf 共通型の生成経路）

### DS-SW-COMP-006（Secret Store）

Dapr Secret Store ビルディングブロックの論理面。OpenBao が物理実装。

- 直接: `IMP-SEC-POL-004`（OpenBao Secret 集約） / `IMP-SEC-KC-010〜022`（Keycloak の DB 接続情報を OpenBao 参照）
- 間接: `IMP-DEV-DC-015`（ローカル OpenBao dev server）

### DS-SW-COMP-019（tier2 共通業務ロジック）

tier2 の全サービスで共通する業務ロジック層（ドメイン共通）の論理 ID。

- 直接: 本章では未結合（00 章 `00_ディレクトリ設計/30_tier2レイアウト/` で IMP-DIR-T2-\* が受ける）
- 間接: `IMP-DEV-GP-020`（tier2 テンプレートからの Golden Path）

## Dapr ファサード層（DS-SW-COMP-020〜049 + 141）

Dapr Building Block（State / PubSub / Binding / Secret / Configuration 等）と Control Plane の論理面。リリース時点 本章で直接参照されるのは 085 と 141。

### DS-SW-COMP-085（OTel Collector Gateway）

観測性の Gateway 論理面。tail sampling と PII transform の責務を持つ。

- 直接: `IMP-OBS-OTEL-010〜019`（OTel Collector 配置 10 ID、Gateway 部分）
- 間接: `IMP-OBS-POL-002`（AGPL 分離維持、LGTM export 経路）

### DS-SW-COMP-141（Observability + Security 統合監査）

可用性インシデントとセキュリティインシデントの統合管理論理面。

- 直接: `IMP-OBS-INC-060〜071`（Incident Taxonomy 12 ID）
- 間接: `IMP-SEC-REV-050〜059`（退職 revoke Runbook、Security 側 HIGH インシデント） / `IMP-SUP-FOR-040〜048`（Forensics Runbook）

### DS-SW-COMP-020〜084, 086〜099, 100〜119 の未結合分

リリース時点 本章では直接参照されないが、以下で間接対応する。

- DS-SW-COMP-020〜049（Dapr Building Block 群）: `IMP-DEV-GP-022`（Dapr components.yaml 同梱）で横断参照
- DS-SW-COMP-050〜099（Rust 自作領域）: 00 章 `IMP-DIR-T1-024`（rust_workspace 配置）で受ける
- DS-SW-COMP-119（src/ 層別分割）: 00 章 IMP-DIR-ROOT-009 で受ける

## 配信系・SDK 系（DS-SW-COMP-120〜135）

SDK の 4 言語配布 / Harbor / ArgoCD / Backstage / Scaffold CLI などの配信インフラ論理面。本章で最も密度高く結合する領域。

### DS-SW-COMP-120（tier1 Dapr ファサード配置）

tier1 内の Dapr ファサード層の物理配置。Go 実装で、infra/dapr/ と deploy/ と ops/ の 3 階層に分散配置される。

- 直接: `IMP-BUILD-GM-020, 027`（tier1 Go module の境界） / `IMP-BUILD-POL-002`（workspace 境界 = tier 境界）
- 間接: `IMP-SUP-FOR-040〜048`（Forensics Runbook が image hash を tier1 配置に逆引き）

### DS-SW-COMP-121（contracts Protobuf 定義）

`src/contracts/` に配置される Protobuf 契約の論理面。tier1 公開 11 API + internal API の源泉。

- 直接: `IMP-CODEGEN-BUF-010〜017`（buf 生成パイプライン 8 ID）
- 間接: `IMP-BUILD-POL-004`（path-filter の契約変更伝播）

### DS-SW-COMP-122（Protobuf 生成コード配置）

Protobuf から生成される言語別コード（Rust / Go / TS / C#）の物理配置の論理面。

- 直接: `IMP-CODEGEN-POL-001〜007`（コード生成方針 7 件） / `IMP-CODEGEN-BUF-010〜017`（buf 生成 8 ID） / `IMP-BUILD-POL-007`（生成物 commit と隔離）
- 間接: `IMP-BUILD-CW-010〜017`（Rust 生成コードの workspace 配置） / `IMP-BUILD-GM-020〜027`（Go 生成コードの module 配置）

### DS-SW-COMP-123（tier1 Observability 契約）

tier1 の観測性（トレース / メトリクス / ログ）用の Protobuf 契約。

- 直接: 本章では未結合（00 章 IMP-DIR-T1-022 で受ける）
- 間接: `IMP-OBS-OTEL-010〜019`（Collector が契約に沿って propagate）

### DS-SW-COMP-124（tier1 Go Dapr ファサード）

tier1 の Dapr ファサード層の Go 実装物。stable Dapr Go SDK を使用（ADR-TIER1-001）。

- 直接: `IMP-BUILD-GM-020〜027`（Go module 8 ID）
- 間接: `IMP-OBS-POL-001`（Dapr 経由のトレース propagation） / `IMP-SEC-POL-006`（Istio Ambient で mTLS 強制） / `IMP-SEC-SP-020〜035`（SPIRE Workload API 統合）

### DS-SW-COMP-125〜128（tier1 Go サブ領域）

tier1 Go 側のサブコンポーネント群（Dapr Pluggable Component / API Server / middleware / observability）。

- 直接: 本章では未結合（00 章 IMP-DIR-T1-023 で受ける）
- 間接: `IMP-BUILD-GM-020〜027`（Go module 分離戦略）

### DS-SW-COMP-129（tier1 Rust 自作領域）

tier1 の Rust 実装物。Decision / Audit / PII の 3 バイナリを出力する crate 群。

- 直接: `IMP-BUILD-CW-010〜017`（Rust workspace 8 ID）全て / `IMP-BUILD-POL-001, 002`（言語ネイティブ / workspace 境界）
- 間接: `IMP-DEP-POL-004`（Rust の SPDX ライセンス表示） / `IMP-DEV-DC-011`（Dev Container の tier1-rust-dev）

### DS-SW-COMP-130（tier1 Rust decision crate）

ZEN Engine を統合した Decision 評価 crate。

- 直接: 本章では未結合（`IMP-DIR-T1-024` で受ける）
- 間接: `IMP-DEV-GP-025`（リリース時点 の decision-example で参照）

### DS-SW-COMP-131（tier1 Rust audit crate）

監査ログ（WORM 保存）の Rust 実装 crate。

- 直接: 本章では未結合（`IMP-DIR-T1-024` で受ける）
- 間接: `IMP-SEC-REV-054`（退職 revoke の監査ログ WORM 保存との整合）

### DS-SW-COMP-132（SDK の 4 言語配布）

SDK（Rust / Go / TS / C#）の 4 言語配布物の論理面。tier2 / tier3 から見える唯一の tier1 面。

- 直接: `IMP-DEV-POL-002`（Scaffold 経由） / `IMP-DEV-GP-020〜026`（Golden Path examples が SDK のみ使用） / `IMP-CODEGEN-SCF-030〜037`（Scaffold CLI が SDK 入りテンプレートを生成）
- 間接: `IMP-BUILD-CW-010`（SDK Rust workspace 分離） / `IMP-BUILD-GM-027`（SDK Go module name 固定）

### DS-SW-COMP-133（SDK generators）

SDK 4 言語の生成器群（tonic-build / protoc-gen-go / ts-proto / Grpc.Tools）。

- 直接: 本章では未結合（IMP-CODEGEN-BUF-010〜017 が buf 生成として受けるが、リリース時点 で CODEGEN サブ接頭辞 SDK が拡張）
- 間接: `IMP-CODEGEN-BUF-011, 012`（SDK 生成先分離 / internal 除外）

### DS-SW-COMP-134（tier1 operator）

tier1 のカスタム Operator（CRD 定義と controller 実装）。

- 直接: 本章では未結合（`IMP-DIR-T1-024` で受ける）
- 間接: `IMP-REL-ARG-010〜017`（Operator の ArgoCD Application 定義）

### DS-SW-COMP-135（配信系インフラ = Harbor / ArgoCD / Backstage / Scaffold）

Harbor / ArgoCD / Backstage / Scaffold CLI / OTel Collector などの配信・運用基盤を束ねた論理面。本章で最も結合密度の高い DS-SW-COMP。

- 直接: `IMP-CI-POL-001〜007`（CI/CD 方針 7 件） / `IMP-CI-RWF-010〜021`（reusable workflow 12 ID） / `IMP-CI-HAR-040〜051`（Harbor 運用 12 ID） / `IMP-REL-POL-001〜007`（リリース方針 7 件） / `IMP-REL-ARG-010〜017`（ArgoCD App 8 ID） / `IMP-REL-PD-020〜028`（Argo Rollouts 9 ID） / `IMP-SUP-POL-001〜007`（サプライチェーン方針 7 件） / `IMP-SUP-COS-010〜018`（cosign 9 ID）
- 間接: `IMP-DEV-POL-002`（Scaffold 経由） / `IMP-CODEGEN-SCF-030〜037`（Scaffold CLI 8 ID）

### DS-SW-COMP-136〜140（リリース時点 以降）

採用初期 で追加される SDK / 配信系コンポーネント。リリース時点 本章では未結合。

## DS-SW-COMP 対応カバレッジ

本章が直接/間接で結びつく DS-SW-COMP は 17 種（001 / 002 / 003 / 006 / 019 / 085 / 119 / 120 / 121 / 122 / 123 / 124 / 125 / 126 / 127 / 128 / 129 / 132 / 133 / 134 / 135 / 141）。リリース時点 確定の DS-SW-COMP が 120〜135 + 141 + tier1 俯瞰 019 前後 = 約 35 件であるため、本章のカバレッジは **約 50%**（残りは 00 章 `IMP-DIR-*` および リリース時点 以降の実装節で受ける）。

最も結合密度が高いのは **DS-SW-COMP-135（配信系インフラ）** で、12 章のうち 30 / 85 / 70 / 80 の 4 章（CI/CD・Identity・リリース・サプライチェーン）から 67 ID が結合する。これは「配信経路が全章横断で効く」という実装ドキュメントの自然な構造であり、DS-SW-COMP 改訂時の影響範囲が最大になる領域であることを示す。

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- IMP-ID 台帳: [`../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- ADR 対応: [`../10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../10_ADR対応表/01_ADR-IMP対応マトリクス.md)
- 上流の DS-SW-COMP 管理: [`../../../04_概要設計/80_トレーサビリティ/`](../../../04_概要設計/80_トレーサビリティ/)
- 並列索引の DS-SW-COMP 対応: [`../../00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md`](../../00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md)
