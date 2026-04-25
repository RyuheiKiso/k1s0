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

- 直接: `IMP-OBS-OTEL-010〜019`（OTel Collector 配置 10 ID、Gateway 部分） / `IMP-OBS-LGTM-022`（NetworkPolicy で Gateway のみ ingress 許可） / `IMP-OBS-LGTM-028`（Gateway disk queue バッファリング） / `IMP-REL-AT-040〜044`（共通テンプレ 5 本が Mimir PromQL 経由で SLI 参照） / `IMP-REL-AT-046`（Mimir provider 統一） / `IMP-SUP-FLG-054`（flag 評価ログ cross-check が Loki 30 日保管に依存、四半期棚卸し Step 2） / `IMP-DX-SPC-022`（SPACE Activity span 集約） / `IMP-DX-SPC-025`（SPACE PII transform） / `IMP-DX-SCAF-030`（Scaffold CLI span 集約） / `IMP-DX-SCAF-039`（Scaffold author hash transform） / `IMP-DX-TFC-040`（TFC 5 ステージ span 集約） / `IMP-DX-TFC-042`（TFC new_joiner_hash transform）
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

- 直接: `IMP-DEV-POL-002`（Scaffold 経由） / `IMP-DEV-GP-020〜026`（Golden Path examples が SDK のみ使用） / `IMP-CODEGEN-SCF-030〜037`（Scaffold CLI が SDK 入りテンプレートを生成） / `IMP-DEV-SO-035`（Scaffold 4 出力に SDK 利用テンプレ含む） / `IMP-DEV-ONB-052`（Day 1 環境構築で SDK ローカル動作確認） / `IMP-DX-DORA-010〜020`（DORA 11 ID = platform レベルの計測） / `IMP-DX-SPC-021〜029`（SPACE 9 ID） / `IMP-DX-SCAF-030〜039`（Scaffold 利用率 10 ID） / `IMP-DX-TFC-040〜049`（TFC 10 ID） / `IMP-DX-EMR-050〜059`（EM レポート 10 ID）
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
- 間接: `IMP-DEV-POL-002`（Scaffold 経由） / `IMP-DEV-POL-007`（Backstage 真実源化） / `IMP-CODEGEN-SCF-030〜037`（Scaffold CLI engine 8 ID） / `IMP-DEV-ONB-051`（Day 0 Backstage Group 登録経路） / `IMP-DEV-ONB-055`（onboarding SLI を TechInsights で計測）

### DS-SW-COMP-136〜140（リリース時点 以降）

採用初期 で追加される SDK / 配信系コンポーネント。リリース時点 本章では未結合。

## DS-SW-COMP 対応カバレッジ

本章が直接/間接で結びつく DS-SW-COMP は 17 種（001 / 002 / 003 / 006 / 019 / 085 / 119 / 120 / 121 / 122 / 123 / 124 / 125 / 126 / 127 / 128 / 129 / 132 / 133 / 134 / 135 / 141）。リリース時点 確定の DS-SW-COMP が 120〜135 + 141 + tier1 俯瞰 019 前後 = 約 35 件であるため、本章のカバレッジは **約 50%**（残りは 00 章 `IMP-DIR-*` および リリース時点 以降の実装節で受ける）。

最も結合密度が高いのは **DS-SW-COMP-135（配信系インフラ）** で、12 章のうち 30 / 50 / 70 / 80 / 85 の 5 章（CI/CD・開発者体験・リリース・サプライチェーン・Identity）から 160 超の ID が結合する。これは「配信経路と開発者体験が全章横断で効く」という実装ドキュメントの自然な構造であり、DS-SW-COMP 改訂時の影響範囲が最大になる領域であることを示す。次点は **DS-SW-COMP-141（多層防御統括）** で、Forensics / 棚卸し / SBOM 監査保管 / Incident Taxonomy が集中する。

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- IMP-ID 台帳: [`../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- ADR 対応: [`../10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../10_ADR対応表/01_ADR-IMP対応マトリクス.md)
- 上流の DS-SW-COMP 管理: [`../../../04_概要設計/80_トレーサビリティ/`](../../../04_概要設計/80_トレーサビリティ/)
- 並列索引の DS-SW-COMP 対応: [`../../00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md`](../../00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md)
