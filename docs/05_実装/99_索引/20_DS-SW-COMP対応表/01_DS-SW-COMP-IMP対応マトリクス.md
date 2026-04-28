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

Dapr Secret Store ビルディングブロックの論理面。OpenBao が物理実装で、リリース時点 で Raft Integrated Storage 3 node HA / Auto-unseal AWS KMS / KV-v2・PKI・Transit 3 secret engine 構成を採る。cert-manager の証明書発行も Vault PKI を経由するため CRT 全 ID と接続する。

- 直接: `IMP-SEC-POL-004`（OpenBao Secret 集約） / `IMP-SEC-POL-005`（cert-manager 自動更新が OpenBao PKI 起点） / `IMP-SEC-KC-010〜022`（Keycloak の DB 接続情報を OpenBao 参照） / `IMP-SEC-OBO-040〜049`（OpenBao 10 ID）全て / `IMP-SEC-CRT-061`（Vault PKI ClusterIssuer） / `IMP-SEC-CRT-062`（Vault PKI 経由中間 CA 発行）
- 間接: `IMP-DEV-DC-015`（ローカル OpenBao dev server） / `IMP-SEC-CRT-060〜069`（cert-manager 全 10 ID は OpenBao PKI 経路を前提）

### DS-SW-COMP-019（tier2 共通業務ロジック）

tier2 の全サービスで共通する業務ロジック層（ドメイン共通）の論理 ID。

- 直接: 本章では未結合（00 章 `00_ディレクトリ設計/30_tier2レイアウト/` で IMP-DIR-T2-\* が受ける）
- 間接: `IMP-DEV-GP-020`（tier2 テンプレートからの Golden Path）

## Dapr ファサード層（DS-SW-COMP-020〜049 + 141）

Dapr Building Block（State / PubSub / Binding / Secret / Configuration 等）と Control Plane の論理面。リリース時点 本章で直接参照されるのは 085 と 141。

### DS-SW-COMP-085（OTel Collector Gateway）

観測性の Gateway 論理面。tail sampling と PII transform の責務を持つ。

- 直接: `IMP-OBS-OTEL-010〜019`（OTel Collector 配置 10 ID、Gateway 部分） / `IMP-OBS-LGTM-022`（NetworkPolicy で Gateway のみ ingress 許可） / `IMP-OBS-LGTM-028`（Gateway disk queue バッファリング） / `IMP-REL-AT-040〜044`（共通テンプレ 5 本が Mimir PromQL 経由で SLI 参照） / `IMP-REL-AT-046`（Mimir provider 統一） / `IMP-SUP-FLG-054`（flag 評価ログ cross-check が Loki 30 日保管に依存、四半期棚卸し Step 2） / `IMP-DX-SPC-022`（SPACE Activity span 集約） / `IMP-DX-SPC-025`（SPACE PII transform） / `IMP-DX-SCAF-030`（Scaffold CLI span 集約） / `IMP-DX-SCAF-039`（Scaffold author hash transform） / `IMP-DX-TFC-040`（TFC 5 ステージ span 集約） / `IMP-DX-TFC-042`（TFC new_joiner_hash transform） / `IMP-TRACE-CI-018`（月次 cron による trace-check 結果 Loki 転送 = 索引整合性の長期傾向集計）
- 間接: `IMP-OBS-POL-002`（AGPL 分離維持、LGTM export 経路） / `IMP-REL-RB-059`（Incident メタデータ集計）

### DS-SW-COMP-141（多層防御統括 / Observability + Security 統合監査）

可用性インシデントとセキュリティインシデントの統合管理論理面。本章 80 サプライチェーンの Forensics / 棚卸し / 監査ログ保管が全てここに帰属する。

- 直接: `IMP-OBS-INC-060〜071`（Incident Taxonomy 12 ID） / `IMP-OBS-RB-080〜089`（SLI ↔ Alert ↔ Runbook 1:1 結合 10 ID） / `IMP-OBS-EB-054`（セキュリティ hotfix bypass） / `IMP-REL-FFD-034`（Kyverno cosign verify-blob による未署名 flag 拒否） / `IMP-REL-FFD-039`（kill switch 発動の PagerDuty Sev2 自動連動） / `IMP-REL-RB-055`（第三経路 = 全停止の三者承認） / `IMP-REL-RB-059`（Incident メタデータ集計） / `IMP-SUP-POL-001`（SLSA 段階到達） / `IMP-SUP-POL-005〜007`（Forensics Runbook / SBOM 差分監視 / AGPL 分離エビデンス） / `IMP-SUP-COS-014`（Rekor インデックス Forensics 基盤化） / `IMP-SUP-COS-018`（月次 cluster 全 Pod 署名 cross-check） / `IMP-SUP-SBM-024〜029`（SBOM 差分監視 / 新規依存通知 / AGPL 検出 / CVE 連動 / 3 年保管 / WORM スナップショット） / `IMP-SUP-SLSA-034`（catalog-info SLSA level 表示） / `IMP-SUP-SLSA-036〜039`（採用後 L3 拡張 4 軸） / `IMP-SUP-FOR-040〜048`（Forensics Runbook 9 ID）全て / `IMP-SUP-FLG-053〜057`（flag 棚卸し 3 ステップ + Forensics 連携 2 ID） / `IMP-SEC-OBO-049`（OpenBao audit device の Loki 90 日 + S3 7 年 WORM 二段保管 = 統合監査の SEC 系基盤） / `IMP-SEC-CRT-064`（CertificateRequest の 7 年 WORM 保管 = 証明書発行履歴の統合監査帰着） / `IMP-DX-SPC-025`（DX SPACE PII transform） / `IMP-DX-SPC-026`（DX SPACE 月次 MinIO Snapshot 3 年 WORM 保管） / `IMP-DX-SCAF-039`（Scaffold author hash transform） / `IMP-DX-TFC-042`（TFC new_joiner_hash 個人特定排除） / `IMP-DX-EMR-059`（EM レポート hash 化済データ流入 CI 検証）
- 間接: `IMP-SEC-REV-050〜059`（退職 revoke Runbook、Security 側 HIGH インシデント） / `IMP-SEC-OBO-040〜048`（OpenBao の Sev1 root token 取扱 / Auto-unseal 監視も 統合監査の対象）

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

Protobuf から生成される言語別コード（Rust / Go / TS / C#）の物理配置の論理面。OpenAPI 経路（HTTP 例外系）の生成コード配置も同 DS-SW-COMP-122 が論理面を持つ。

- 直接: `IMP-CODEGEN-POL-001〜007`（コード生成方針 7 件） / `IMP-CODEGEN-BUF-010〜017`（buf 生成 8 ID） / `IMP-CODEGEN-OAS-020〜027`（OpenAPI 3 ジェネレータ 8 ID） / `IMP-CODEGEN-GLD-040〜047`（生成器挙動 pin 8 ID） / `IMP-BUILD-POL-007`（生成物 commit と隔離）
- 間接: `IMP-BUILD-CW-010〜017`（Rust 生成コードの workspace 配置） / `IMP-BUILD-GM-020〜027`（Go 生成コードの module 配置）

### DS-SW-COMP-123（tier1 Observability 契約）

tier1 の観測性（トレース / メトリクス / ログ）用の Protobuf 契約。

- 直接: 本章では未結合（00 章 IMP-DIR-T1-022 で受ける）
- 間接: `IMP-OBS-OTEL-010〜019`（Collector が契約に沿って propagate）

### DS-SW-COMP-124（tier1 Go Dapr ファサード）

tier1 の Dapr ファサード層の Go 実装物。stable Dapr Go SDK を使用（ADR-TIER1-001）。Dapr ファサードは Istio Ambient データプレーン上で動作し、その mTLS 証明書は cert-manager + istio-csr 経由で SPIRE SVID から供給される。

- 直接: `IMP-BUILD-GM-020〜027`（Go module 8 ID）
- 間接: `IMP-OBS-POL-001`（Dapr 経由のトレース propagation） / `IMP-SEC-POL-006`（Istio Ambient で mTLS 強制） / `IMP-SEC-SP-020〜035`（SPIRE Workload API 統合） / `IMP-SEC-CRT-065`（istio-csr 経由 Ambient mTLS 証明書供給） / `IMP-SEC-CRT-066`（SVID 1h ローテーション）

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

- 直接: `IMP-DEV-POL-002`（Scaffold 経由） / `IMP-DEV-GP-020〜026`（Golden Path examples が SDK のみ使用） / `IMP-CODEGEN-SCF-030〜037`（Scaffold CLI が SDK 入りテンプレートを生成） / `IMP-DEV-SO-035`（Scaffold 4 出力に SDK 利用テンプレ含む） / `IMP-DEV-ONB-052`（Day 1 環境構築で SDK ローカル動作確認） / `IMP-DX-DORA-010〜020`（DORA 11 ID = platform レベルの計測） / `IMP-DX-SPC-021〜029`（SPACE 9 ID） / `IMP-DX-SCAF-030〜039`（Scaffold 利用率 10 ID） / `IMP-DX-TFC-040〜049`（TFC 10 ID） / `IMP-DX-EMR-050〜059`（EM レポート 10 ID） / `IMP-TRACE-CI-010〜019`（整合性 CI 10 ID = `tools/trace-check/` 物理配置 platform 化） / `IMP-TRACE-CAT-020〜029`（catalog-info 検証 10 ID = `tools/catalog-check/` 物理配置 platform 化）
- 間接: `IMP-BUILD-CW-010`（SDK Rust workspace 分離） / `IMP-BUILD-GM-027`（SDK Go module name 固定） / `IMP-DEV-ONB-056`（Week 1 学習リストで SDK 概念導入）

### DS-SW-COMP-133（SDK generators）

SDK 4 言語の生成器群（tonic-build / protoc-gen-go / ts-proto / Grpc.Tools）。

- 直接: 本章では未結合（IMP-CODEGEN-BUF-010〜017 が buf 生成として受けるが、リリース時点 で CODEGEN サブ接頭辞 SDK が拡張）
- 間接: `IMP-CODEGEN-BUF-011, 012`（SDK 生成先分離 / internal 除外）

### DS-SW-COMP-134（tier1 operator）

tier1 のカスタム Operator（CRD 定義と controller 実装）。

- 直接: 本章では未結合（`IMP-DIR-T1-024` で受ける）
- 間接: `IMP-REL-ARG-010〜017`（Operator の ArgoCD Application 定義）

### DS-SW-COMP-135（配信系インフラ = Harbor / ArgoCD / Backstage / Scaffold / cosign / Kyverno）

Harbor / ArgoCD / Backstage / Scaffold CLI / OTel Collector / cosign / Kyverno などの配信・admission・運用基盤を束ねた論理面。本章で最も結合密度の高い DS-SW-COMP。

- 直接: `IMP-CI-POL-001〜007`（CI/CD 方針 7 件） / `IMP-CI-RWF-010〜021`（reusable workflow 12 ID） / `IMP-CI-PF-030〜037`（path-filter 8 ID） / `IMP-CI-HAR-040〜051`（Harbor 運用 12 ID） / `IMP-CI-QG-060〜067`（quality gate 8 ID） / `IMP-CI-BP-070〜077`（branch protection 8 ID） / `IMP-REL-POL-001〜007`（リリース方針 7 件） / `IMP-REL-ARG-010〜017`（ArgoCD App 8 ID） / `IMP-REL-PD-020〜028`（Argo Rollouts 9 ID） / `IMP-REL-FFD-030〜039`（flagd 配布・SDK・評価ログ 10 ID） / `IMP-REL-AT-040〜049`（AnalysisTemplate 共通 5 本 + 継承 10 ID） / `IMP-REL-RB-050〜059`（rollback runbook 10 ID） / `IMP-SUP-POL-001〜004`（サプライチェーン方針 4 件 = SLSA / cosign / SBOM / Kyverno） / `IMP-SUP-COS-010〜018`（cosign 9 ID） / `IMP-SUP-SBM-020〜023`（CycloneDX SBOM 生成 + cosign attest + Kyverno 必須化） / `IMP-SUP-SLSA-030〜033`（hosted runner + Provenance v1 + Rekor + Kyverno verifyAttestations） / `IMP-SUP-SLSA-035`（虚偽申告検知 admission reject） / `IMP-SUP-FOR-041`（cosign download sbom Forensics 起点） / `IMP-SUP-FLG-050〜052`（flag cosign sign-blob + OCI Artifact + Kyverno verify-blob） / `IMP-DEV-SO-030〜037`（Scaffold 運用 8 ID） / `IMP-DEV-BSN-040〜048`（Backstage 連携 9 ID） / `IMP-DX-SPC-027`（SPACE Scorecards 表示） / `IMP-DX-SCAF-035`（Scaffold Adoption Rate Scorecards） / `IMP-DX-TFC-041`（onboardingTimeFactRetriever TechInsights） / `IMP-DX-EMR-050`（EM Generator backend job） / `IMP-DX-EMR-053〜055`（Slack / Confluence / Backstage Catalog 3 配信）
- 間接: `IMP-DEV-POL-002`（Scaffold 経由） / `IMP-DEV-POL-007`（Backstage 真実源化） / `IMP-CODEGEN-SCF-030〜037`（Scaffold CLI engine 8 ID） / `IMP-DEV-ONB-051`（Day 0 Backstage Group 登録経路） / `IMP-DEV-ONB-055`（onboarding SLI を TechInsights で計測） / `IMP-TRACE-CAT-020〜029`（catalog-info.yaml スキーマ検証 10 ID = Backstage 同期前の事前検証として配信系基盤に統合）

### DS-SW-COMP-136〜140（リリース時点 以降）

採用初期 で追加される SDK / 配信系コンポーネント。リリース時点 本章では未結合。

## DS-SW-COMP 対応カバレッジ

本章が直接/間接で結びつく DS-SW-COMP は 17 種（001 / 002 / 003 / 006 / 019 / 085 / 119 / 120 / 121 / 122 / 123 / 124 / 125 / 126 / 127 / 128 / 129 / 132 / 133 / 134 / 135 / 141）。リリース時点 確定の DS-SW-COMP が 120〜135 + 141 + tier1 俯瞰 019 前後 = 約 35 件であるため、本章のカバレッジは **約 50%**（残りは 00 章 `IMP-DIR-*` および リリース時点 以降の実装節で受ける）。

最も結合密度が高いのは **DS-SW-COMP-135（配信系インフラ）** で、12 章のうち 30 / 50 / 70 / 80 / 85 の 5 章（CI/CD・開発者体験・リリース・サプライチェーン・Identity）から 160 超の ID が結合する。これは「配信経路と開発者体験が全章横断で効く」という実装ドキュメントの自然な構造であり、DS-SW-COMP 改訂時の影響範囲が最大になる領域であることを示す。次点は **DS-SW-COMP-141（多層防御統括）** で、Forensics / 棚卸し / SBOM 監査保管 / Incident Taxonomy が集中する。

## 追加 IMP-* 対応一覧（孤立 ID 解消）

本節は `tools/trace-check/check-orphan.sh` で「ADR/DS-SW-COMP/NFR マトリクス全てで未参照」と検出された ID を、tier・配信系などのコンポーネント観点で DS-SW-COMP へ間接対応として紐付けた追補リストである。

| IMP-ID | 対応 DS-SW-COMP | 対応形式 | 紐付け根拠（要約） |
|---|---|---|---|
| IMP-BUILD-CS-060 | DS-SW-COMP-135 | 間接 | コンテナスキャン = 配信系インフラ品質ゲート |
| IMP-BUILD-CS-068 | DS-SW-COMP-135 | 間接 | コンテナスキャン追加設定 = Harbor 連携 |
| IMP-BUILD-CW-011 | DS-SW-COMP-003 | 間接 | workspace.dependencies = tier1 Rust workspace 設計 |
| IMP-BUILD-CW-015 | DS-SW-COMP-003 | 間接 | clippy -D warnings = tier1 Rust 品質ゲート |
| IMP-BUILD-CW-016 | DS-SW-COMP-003 | 間接 | rustfmt 強制 = tier1 Rust 統一フォーマット |
| IMP-BUILD-CW-017 | DS-SW-COMP-003 | 間接 | deny.toml ライセンス = tier1 依存管理 |
| IMP-BUILD-CW-018 | DS-SW-COMP-003 | 間接 | cargo-audit = tier1 セキュリティゲート |
| IMP-BUILD-DS-040 | DS-SW-COMP-135 | 間接 | .NET sidecar ビルド = 配信系インフラ対応 |
| IMP-BUILD-DS-048 | DS-SW-COMP-135 | 間接 | .NET sidecar 追加設定 |
| IMP-BUILD-GM-021 | DS-SW-COMP-003 | 間接 | Go module 命名規約 = tier1 Go+Rust 構成 |
| IMP-BUILD-GM-022 | DS-SW-COMP-003 | 間接 | Go module replace 管理 |
| IMP-BUILD-GM-023 | DS-SW-COMP-003 | 間接 | Go toolchain pin = tier1 再現性 |
| IMP-BUILD-GM-024 | DS-SW-COMP-003 | 間接 | Go vet + staticcheck = tier1 品質ゲート |
| IMP-BUILD-GM-026 | DS-SW-COMP-003 | 間接 | golangci-lint = tier1 品質標準 |
| IMP-BUILD-GM-028 | DS-SW-COMP-003 | 間接 | Go module proxy = tier1 依存再現性 |
| IMP-BUILD-PF-050 | DS-SW-COMP-135 | 間接 | platform CI profile = 配信系 CI 設定 |
| IMP-BUILD-PF-058 | DS-SW-COMP-135 | 間接 | platform CI profile 追加設定 |
| IMP-BUILD-TP-030 | DS-SW-COMP-135 | 間接 | test profile = 配信系テスト設定 |
| IMP-BUILD-TP-038 | DS-SW-COMP-135 | 間接 | test profile 追加設定 |
| IMP-CI-BP-071 | DS-SW-COMP-135 | 間接 | branch protection ルール追加 = 配信系セキュリティ |
| IMP-CI-BP-072 | DS-SW-COMP-135 | 間接 | branch protection CODEOWNERS |
| IMP-CI-BP-073 | DS-SW-COMP-135 | 間接 | branch protection required checks |
| IMP-CI-BP-075 | DS-SW-COMP-135 | 間接 | branch protection stale dismiss |
| IMP-CI-BP-077 | DS-SW-COMP-135 | 間接 | branch protection conversation resolution |
| IMP-CI-BP-078 | DS-SW-COMP-135 | 間接 | branch protection 署名コミット必須 |
| IMP-CI-HAR-042 | DS-SW-COMP-135 | 間接 | Harbor ロボットアカウント = Harbor 配信基盤 |
| IMP-CI-HAR-043 | DS-SW-COMP-135 | 間接 | Harbor quota |
| IMP-CI-HAR-044 | DS-SW-COMP-135 | 間接 | Harbor GC |
| IMP-CI-HAR-045 | DS-SW-COMP-135 | 間接 | Harbor 脆弱性スキャン |
| IMP-CI-HAR-046 | DS-SW-COMP-135 | 間接 | Harbor webhook |
| IMP-CI-HAR-048 | DS-SW-COMP-135 | 間接 | Harbor レプリケーション |
| IMP-CI-HAR-049 | DS-SW-COMP-135 | 間接 | Harbor 監査ログ |
| IMP-CI-HAR-050 | DS-SW-COMP-135 | 間接 | Harbor OIDC 連携 |
| IMP-CI-HAR-051 | DS-SW-COMP-135 | 間接 | Harbor チャート管理 |
| IMP-CI-HAR-052 | DS-SW-COMP-135 | 間接 | Harbor イメージ署名検証 |
| IMP-CI-LCDT-080 | DS-SW-COMP-135 | 間接 | lifecycle drift 検知 = 配信系陳腐化防止 |
| IMP-CI-LCDT-081 | DS-SW-COMP-135 | 間接 | lifecycle drift 通知 |
| IMP-CI-LCDT-082 | DS-SW-COMP-135 | 間接 | lifecycle drift 自動 PR |
| IMP-CI-LCDT-083 | DS-SW-COMP-135 | 間接 | lifecycle drift EOL 判定 |
| IMP-CI-LCDT-084 | DS-SW-COMP-135 | 間接 | lifecycle drift 週次スキャン |
| IMP-CI-PF-032 | DS-SW-COMP-135 | 間接 | path-filter 追加 = ArgoCD パス管理 |
| IMP-CI-PF-034 | DS-SW-COMP-135 | 間接 | path-filter infra |
| IMP-CI-PF-035 | DS-SW-COMP-135 | 間接 | path-filter deploy |
| IMP-CI-PF-036 | DS-SW-COMP-135 | 間接 | path-filter docs |
| IMP-CI-PF-037 | DS-SW-COMP-135 | 間接 | path-filter tools |
| IMP-CI-PF-038 | DS-SW-COMP-135 | 間接 | path-filter tests |
| IMP-CI-QG-061 | DS-SW-COMP-135 | 間接 | QG Go coverage = 配信系品質ゲート |
| IMP-CI-QG-062 | DS-SW-COMP-135 | 間接 | QG Rust coverage |
| IMP-CI-QG-063 | DS-SW-COMP-135 | 間接 | QG TypeScript coverage |
| IMP-CI-QG-064 | DS-SW-COMP-135 | 間接 | QG Python coverage |
| IMP-CI-QG-065 | DS-SW-COMP-135 | 間接 | QG mutation score |
| IMP-CI-QG-066 | DS-SW-COMP-135 | 間接 | QG DAST |
| IMP-CI-QG-067 | DS-SW-COMP-135 | 間接 | QG SCA license |
| IMP-CI-QG-068 | DS-SW-COMP-135 | 間接 | QG secret scan |
| IMP-CI-RWF-011 | DS-SW-COMP-135 | 間接 | reusable workflow 追加 = ArgoCD CI 標準 |
| IMP-CI-RWF-014 | DS-SW-COMP-135 | 間接 | reusable workflow matrix |
| IMP-CI-RWF-015 | DS-SW-COMP-135 | 間接 | reusable workflow concurrency |
| IMP-CI-RWF-017 | DS-SW-COMP-135 | 間接 | reusable workflow permissions 最小化 |
| IMP-CI-RWF-019 | DS-SW-COMP-135 | 間接 | reusable workflow cache |
| IMP-CI-RWF-020 | DS-SW-COMP-135 | 間接 | reusable workflow artifact |
| IMP-CI-RWF-021 | DS-SW-COMP-135 | 間接 | reusable workflow timeout |
| IMP-CI-RWF-022 | DS-SW-COMP-135 | 間接 | reusable workflow retry |
| IMP-CODEGEN-BUF-014 | DS-SW-COMP-003 | 間接 | buf generate 追加 = tier1 gRPC 生成 |
| IMP-CODEGEN-BUF-015 | DS-SW-COMP-003 | 間接 | buf lint 追加ルール |
| IMP-CODEGEN-BUF-016 | DS-SW-COMP-003 | 間接 | buf breaking 検知 |
| IMP-CODEGEN-BUF-017 | DS-SW-COMP-003 | 間接 | buf BSR remote plugin |
| IMP-CODEGEN-BUF-018 | DS-SW-COMP-003 | 間接 | buf managed mode |
| IMP-CODEGEN-GLD-041 | DS-SW-COMP-003 | 間接 | golden file 追加 = tier1 codegen 回帰検証 |
| IMP-CODEGEN-GLD-042 | DS-SW-COMP-003 | 間接 | golden file Go pin |
| IMP-CODEGEN-GLD-043 | DS-SW-COMP-003 | 間接 | golden file Rust pin |
| IMP-CODEGEN-GLD-044 | DS-SW-COMP-003 | 間接 | golden file TypeScript pin |
| IMP-CODEGEN-GLD-045 | DS-SW-COMP-003 | 間接 | golden file Python pin |
| IMP-CODEGEN-GLD-046 | DS-SW-COMP-003 | 間接 | golden file diff 自動 PR |
| IMP-CODEGEN-GLD-047 | DS-SW-COMP-003 | 間接 | golden file CI 強制チェック |
| IMP-CODEGEN-GLD-048 | DS-SW-COMP-003 | 間接 | golden file snapshot 更新フロー |
| IMP-CODEGEN-OAS-021 | DS-SW-COMP-003 | 間接 | OpenAPI spec 追加 = tier1 gRPC-HTTP ゲートウェイ |
| IMP-CODEGEN-OAS-022 | DS-SW-COMP-003 | 間接 | OpenAPI バリデーション |
| IMP-CODEGEN-OAS-025 | DS-SW-COMP-003 | 間接 | OpenAPI バージョン管理 |
| IMP-CODEGEN-OAS-026 | DS-SW-COMP-003 | 間接 | OpenAPI 差分レポート |
| IMP-CODEGEN-OAS-027 | DS-SW-COMP-135 | 間接 | OpenAPI Redoc 公開 = 配信系公開基盤 |
| IMP-CODEGEN-OAS-028 | DS-SW-COMP-003 | 間接 | OpenAPI mock サーバ |
| IMP-CODEGEN-SCF-032 | DS-SW-COMP-135 | 間接 | Scaffold template = Scaffold 配信基盤 |
| IMP-CODEGEN-SCF-033 | DS-SW-COMP-135 | 間接 | Scaffold Go 雛形 |
| IMP-CODEGEN-SCF-034 | DS-SW-COMP-135 | 間接 | Scaffold Rust 雛形 |
| IMP-CODEGEN-SCF-035 | DS-SW-COMP-135 | 間接 | Scaffold Backstage 登録 |
| IMP-CODEGEN-SCF-036 | DS-SW-COMP-135 | 間接 | Scaffold テスト雛形 |
| IMP-CODEGEN-SCF-037 | DS-SW-COMP-135 | 間接 | Scaffold catalog-info.yaml |
| IMP-CODEGEN-SCF-038 | DS-SW-COMP-135 | 間接 | Scaffold CI workflow |
| IMP-CODEGEN-POL-008 | DS-SW-COMP-003 | 間接 | codegen ポリシー追加 = tier1 codegen ポリシー |
| IMP-DEP-LIC-030 | DS-SW-COMP-135 | 間接 | ライセンス検査 = 配信系ライセンス管理 |
| IMP-DEP-REN-010 | DS-SW-COMP-135 | 間接 | Renovate = 配信系依存更新 |
| IMP-DEP-SBM-020 | DS-SW-COMP-135 | 間接 | SBOM = 配信系サプライチェーン |
| IMP-DEV-BSN-041 | DS-SW-COMP-135 | 間接 | Backstage プラグイン = Backstage 配信基盤 |
| IMP-DEV-BSN-043 | DS-SW-COMP-135 | 間接 | Backstage TechDocs |
| IMP-DEV-BSN-044 | DS-SW-COMP-135 | 間接 | Backstage Catalog 同期 |
| IMP-DEV-BSN-047 | DS-SW-COMP-135 | 間接 | Backstage GitHub Actions 統合 |
| IMP-DEV-BSN-049 | DS-SW-COMP-135 | 間接 | Backstage Kubernetes プラグイン |
| IMP-DEV-DC-013 | DS-SW-COMP-135 | 間接 | Dev Container 追加 = 配信系開発環境 |
| IMP-DEV-DC-016 | DS-SW-COMP-135 | 間接 | Dev Container GPU 対応 |
| IMP-DEV-DC-017 | DS-SW-COMP-135 | 間接 | Dev Container port forwarding |
| IMP-DEV-DC-018 | DS-SW-COMP-135 | 間接 | Dev Container lifecycle scripts |
| IMP-DEV-GP-023 | DS-SW-COMP-135 | 間接 | GitHub Pages SDK 例 = 配信系サンプル |
| IMP-DEV-GP-024 | DS-SW-COMP-135 | 間接 | GitHub Pages TypeScript 例 |
| IMP-DEV-GP-026 | DS-SW-COMP-135 | 間接 | GitHub Pages Python 例 |
| IMP-DEV-GP-027 | DS-SW-COMP-135 | 間接 | GitHub Pages Rust 例 |
| IMP-DEV-ONB-053 | DS-SW-COMP-135 | 間接 | onboarding チェックリスト = 配信系オンボード |
| IMP-DEV-ONB-054 | DS-SW-COMP-135 | 間接 | onboarding 自動セットアップ |
| IMP-DEV-ONB-057 | DS-SW-COMP-135 | 間接 | onboarding SLI 計測 |
| IMP-DEV-SO-032 | DS-SW-COMP-135 | 間接 | Scaffold 操作ガイド = Scaffold 配信基盤 |
| IMP-DEV-SO-033 | DS-SW-COMP-135 | 間接 | Scaffold カスタムテンプレート |
| IMP-DEV-SO-034 | DS-SW-COMP-135 | 間接 | Scaffold パラメータバリデーション |
| IMP-DEV-SO-036 | DS-SW-COMP-135 | 間接 | Scaffold dry-run モード |
| IMP-DEV-SO-038 | DS-SW-COMP-135 | 間接 | Scaffold 生成ログ保存 |
| IMP-DX-DORA-021 | DS-SW-COMP-135 | 間接 | DORA 4 keys 追加 = 配信系 DX 計測 |
| IMP-DX-SCAF-033 | DS-SW-COMP-135 | 間接 | Scaffold Adoption Rate = Scaffold 計測 |
| IMP-OBS-EB-052 | DS-SW-COMP-135 | 間接 | Error Budget 追加 = 配信系 SLO 管理 |
| IMP-OBS-EB-055 | DS-SW-COMP-135 | 間接 | Error Budget Slack 通知 |
| IMP-OBS-EB-056 | DS-SW-COMP-135 | 間接 | Error Budget 自動 incident 起票 |
| IMP-OBS-EB-057 | DS-SW-COMP-135 | 間接 | Error Budget 週次レポート |
| IMP-OBS-INC-072 | DS-SW-COMP-135 | 間接 | incident 対応追加 = 配信系 incident 運用 |
| IMP-OBS-LGTM-021 | DS-SW-COMP-135 | 間接 | LGTM 追加 = 観測性配信基盤 |
| IMP-OBS-LGTM-023 | DS-SW-COMP-135 | 間接 | Grafana dashboard 追加 |
| IMP-OBS-LGTM-024 | DS-SW-COMP-135 | 間接 | Mimir retention |
| IMP-OBS-LGTM-025 | DS-SW-COMP-135 | 間接 | Tempo sampling |
| IMP-OBS-LGTM-027 | DS-SW-COMP-135 | 間接 | Loki pipeline |
| IMP-OBS-LGTM-029 | DS-SW-COMP-135 | 間接 | alertmanager routing |
| IMP-OBS-PYR-031 | DS-SW-COMP-135 | 間接 | Pyroscope 追加 = 配信系継続プロファイリング |
| IMP-OBS-PYR-032 | DS-SW-COMP-135 | 間接 | Pyroscope Go SDK |
| IMP-OBS-PYR-034 | DS-SW-COMP-135 | 間接 | Pyroscope Rust SDK |
| IMP-OBS-PYR-036 | DS-SW-COMP-135 | 間接 | Pyroscope サンプリング間隔 |
| IMP-OBS-PYR-037 | DS-SW-COMP-135 | 間接 | Pyroscope label 戦略 |
| IMP-OBS-PYR-038 | DS-SW-COMP-135 | 間接 | Pyroscope retention |
| IMP-OBS-PYR-039 | DS-SW-COMP-135 | 間接 | Pyroscope alert ルール |
| IMP-OBS-RB-081 | DS-SW-COMP-135 | 間接 | 観測性 runbook 追加 = 配信系 runbook |
| IMP-OBS-RB-082 | DS-SW-COMP-135 | 間接 | alert → runbook リンク |
| IMP-OBS-RB-083 | DS-SW-COMP-135 | 間接 | 自動 PD 起票 |
| IMP-OBS-RB-084 | DS-SW-COMP-135 | 間接 | escalation |
| IMP-OBS-RB-085 | DS-SW-COMP-135 | 間接 | DR 手順 |
| IMP-OBS-RB-086 | DS-SW-COMP-135 | 間接 | rollback 手順 |
| IMP-OBS-RB-087 | DS-SW-COMP-135 | 間接 | post-mortem テンプレート |
| IMP-OBS-RB-088 | DS-SW-COMP-135 | 間接 | SLO violation 対応 |
| IMP-OBS-RB-089 | DS-SW-COMP-135 | 間接 | on-call ハンドオフ |
| IMP-OBS-SLO-048 | DS-SW-COMP-135 | 間接 | SLO 追加 = 配信系 SLO 管理 |
| IMP-REL-ARG-018 | DS-SW-COMP-135 | 間接 | ArgoCD Application 追加 = ArgoCD 配信基盤 |
| IMP-REL-PD-029 | DS-SW-COMP-135 | 間接 | Argo Rollouts ProgressDeadline = Argo Rollouts 配信 |
| IMP-SEC-CRT-070 | DS-SW-COMP-135 | 間接 | cert-manager 証明書 = 配信系 PKI |
| IMP-SEC-KC-023 | DS-SW-COMP-135 | 間接 | Keycloak 追加 = 配信系 IdP |
| IMP-SEC-KEY-001 | DS-SW-COMP-135 | 間接 | Key 管理 = 配信系鍵管理 |
| IMP-SEC-OBO-050 | DS-SW-COMP-135 | 間接 | OpenBao 追加 = 配信系シークレット管理 |
| IMP-SEC-SP-036 | DS-SW-COMP-135 | 間接 | SPIFFE/SPIRE 追加 = 配信系 workload identity |
| IMP-SUP-COS-019 | DS-SW-COMP-135 | 間接 | cosign 追加 = 配信系署名 |
| IMP-SUP-FLG-058 | DS-SW-COMP-135 | 間接 | feature flag cosign 追加 = 配信系 flag 署名 |
| IMP-SUP-FOR-049 | DS-SW-COMP-135 | 間接 | Forensics 追加 = 配信系 SBOM 監査 |
| IMP-TRACE-CAT-021 | DS-SW-COMP-135 | 間接 | catalog-info.yaml 追加 = Backstage 同期基盤 |
| IMP-TRACE-CAT-022 | DS-SW-COMP-135 | 間接 | catalog-info.yaml 必須フィールド |
| IMP-TRACE-CAT-024 | DS-SW-COMP-135 | 間接 | catalog-info.yaml カスタムアノテーション |
| IMP-TRACE-CAT-027 | DS-SW-COMP-135 | 間接 | catalog-info.yaml CI バリデーション |
| IMP-TRACE-CAT-028 | DS-SW-COMP-135 | 間接 | catalog-info.yaml 同期確認 |
| IMP-TRACE-CAT-030 | DS-SW-COMP-135 | 間接 | catalog-info.yaml 差分検知 |
| IMP-TRACE-CI-011 | DS-SW-COMP-135 | 間接 | trace check CI 追加 = 配信系品質ゲート |
| IMP-TRACE-CI-012 | DS-SW-COMP-135 | 間接 | trace check orphan 検知 |
| IMP-TRACE-CI-013 | DS-SW-COMP-135 | 間接 | trace check cross-ref |
| IMP-TRACE-CI-014 | DS-SW-COMP-135 | 間接 | trace check grand-total |
| IMP-TRACE-CI-015 | DS-SW-COMP-135 | 間接 | trace check PR ブロック |
| IMP-TRACE-CI-016 | DS-SW-COMP-135 | 間接 | trace check Slack 通知 |
| IMP-TRACE-CI-017 | DS-SW-COMP-135 | 間接 | trace check contracts 検証 |
| IMP-TRACE-CI-019 | DS-SW-COMP-135 | 間接 | trace check スケジュール実行 |
| IMP-BUILD-POL-008 | DS-SW-COMP-003 | 間接 | contracts 昇格ポリシー追加 = tier1 contracts |
| IMP-CI-POL-008 | DS-SW-COMP-135 | 間接 | CI ポリシー追加 = 配信系 CI ポリシー |
| IMP-TRACE-POL-005 | DS-SW-COMP-135 | 間接 | trace check ポリシー追加 = 双方向リンク |
| IMP-TRACE-POL-006 | DS-SW-COMP-135 | 間接 | trace check ポリシー（孤立 ID 通知） |

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- IMP-ID 台帳: [`../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- ADR 対応: [`../10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../10_ADR対応表/01_ADR-IMP対応マトリクス.md)
- 上流の DS-SW-COMP 管理: [`../../../04_概要設計/80_トレーサビリティ/`](../../../04_概要設計/80_トレーサビリティ/)
- 並列索引の DS-SW-COMP 対応: [`../../00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md`](../../00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md)
