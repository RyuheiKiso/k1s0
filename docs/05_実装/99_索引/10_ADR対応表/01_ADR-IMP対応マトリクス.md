# 99. 索引 / 10. ADR 対応表 / 01. ADR-IMP 対応マトリクス

本ファイルは `02_構想設計/adr/` 配下の全 ADR（Phase 0 時点で 36 件 = 既存 29 + 新規起票予定 7）と、本実装フェーズで採番された IMP-\* ID の対応を「ADR から IMP を逆引き」する方向で表現する。ADR 改訂時に「どの実装 ID が影響を受けるか」を 1 ファイルで確定できる状態を IMP-TRACE-POL-005（双方向リンク必須）の系として保証する。

## マトリクスの読み方

ADR ごとに 1 セクションを配し、冒頭の散文で「ADR が決めた事柄 / 実装への影響 / なぜこれらの IMP が結びつくか」を示し、続いて IMP-\* の直接対応・間接対応を 2 列で列挙する。**直接対応** は当該 ADR の決定が IMP の存在理由そのものである関係、**間接対応** は ADR の副次的帰結として IMP が制約を受ける関係である。

IMP 側からの逆引き（IMP → ADR）は各章の核心節ファイル内「対応 ADR / DS-SW-COMP / NFR」セクションで双方向に記載されているため、本ファイルは ADR 側からの引きのみを扱う。網羅性は CI の孤立リンク検出（IMP-TRACE-POL-005 で定義）で検証される前提。

## Phase 0 新規起票予定の 7 ADR

本実装フェーズでの各章 README が「本章初版策定時に起票予定」と明示した 7 本。Phase 0 の実装ドキュメント確定と同タイミングで adr 起票される。

### ADR-SUP-001（SLSA L2 先行 → L3 到達）

Phase 0 で SLSA L2（ビルド履歴の真正性と改ざん困難性）を満たし、Phase 1b で L3（ハーメティックビルド）を目指す段階到達戦略。L3 先行案との差分を記録する。

- 直接: `IMP-SUP-POL-001`（段階到達原則） / `IMP-SUP-POL-003`（SBOM 全添付） / `IMP-SUP-POL-006`（Kyverno 検証）
- 間接: `IMP-SUP-COS-010〜018`（cosign 9 ID、L2 達成の実装面） / `IMP-SUP-FOR-040〜048`（Forensics Runbook、L3 到達時の監査再構成面）

### ADR-DEV-001（Paved Road 思想）

Paved Road（舗装路）アプローチで開発者の選択肢を絞り、examples / Scaffold / Dev Container を一本化する開発者体験戦略の方針 ADR。

- 直接: `IMP-DEV-POL-001`（Paved Road 一本化） / `IMP-DEV-POL-002`（Scaffold 必須経由） / `IMP-DEV-POL-003`（10 役 Dev Container）
- 間接: `IMP-DEV-DC-010〜017`（Dev Container 8 ID） / `IMP-DEV-GP-020〜026`（Golden Path 7 ID） / `IMP-CODEGEN-SCF-030〜037`（Scaffold CLI 8 ID）

### ADR-REL-001（Progressive Delivery 必須）

Canary リリースと AnalysisTemplate による自動 rollback を全 tier1 サービスで必須化する方針 ADR。

- 直接: `IMP-REL-POL-002`（Progressive Delivery 必須） / `IMP-REL-POL-003`（Canary AnalysisTemplate 強制）
- 間接: `IMP-REL-PD-020〜028`（Argo Rollouts 9 ID）全て / `IMP-OBS-SLO-040〜047`（SLO が Canary の判定源になるため）

### ADR-DEP-001（Renovate 中央集約）

Renovate による依存更新を全言語（Rust / Go / TS / C#）で中央集約する方針 ADR。

- 直接: `IMP-DEP-POL-001`（Renovate 経由のみ） / `IMP-DEP-POL-006`（自動マージ patch のみ）
- 間接: `IMP-DEP-POL-002〜005, 007`（lockfile / vendoring / ライセンス / AGPL / SBOM） / `IMP-BUILD-CW-012`（toolchain 固定への Renovate 連携）

### ADR-DX-001（DX メトリクス章分離）

DX メトリクス（DORA Four Keys / time-to-first-commit）を 95 章として独立運用する構造 ADR。60 章観測性（SLO/SLI）との境界を定義する。

- 直接: `IMP-DX-POL-001〜007`（DX 方針 7 件）全て
- 間接: `IMP-DX-DORA-010〜020`（DORA 11 ID） / `IMP-OBS-POL-007`（DORA 4 keys は 95 章へ分離の明示）

### ADR-POL-001（Kyverno dual ownership）

Kyverno ポリシーを Platform + Security の二重承認で運用する構造 ADR。

- 直接: `IMP-POL-POL-001`（dual ownership） / `IMP-POL-POL-003`（例外 30 日時限）
- 間接: `IMP-POL-POL-002, 004〜007`（audit モード / 脅威モデル / Runbook / WORM / NetworkPolicy） / `IMP-SUP-POL-006`（cosign 検証の Kyverno 実行）

### ADR-OBS-003（Incident Taxonomy 統合）

可用性（AVL）インシデントとセキュリティ（SEC）インシデントを単一 Taxonomy で統合分類する ADR。

- 直接: `IMP-OBS-POL-004`（Taxonomy 統合） / `IMP-OBS-INC-060〜071`（Incident Taxonomy 12 ID）全て
- 間接: `IMP-SUP-FOR-040〜048`（Forensics Runbook は SEC 側の具体 Runbook） / `IMP-SEC-REV-050〜059`（退職 revoke は SEC × HIGH の Runbook）

## 既存 ADR（Phase 0 以前に確定済）

### ADR-0001（Istio Ambient vs Sidecar）

サービスメッシュを Istio Ambient モード（ztunnel + waypoint）で運用する ADR。

- 直接: `IMP-SEC-POL-006`（Istio Ambient mTLS） / `IMP-SEC-SP-020〜035`（SPIRE の Istio 統合面）
- 間接: `IMP-OBS-OTEL-010〜019`（Ambient 由来のメトリクス経路）

### ADR-0002（Diagram layer convention）

図解の 4 レイヤ配色規約（アプリ / ネットワーク / インフラ / データ）。

- 直接: 本章で採番する IMP なし（ドキュメント規約 ADR）
- 間接: 本 05_実装/ 全 drawio 図が準拠

### ADR-0003（AGPL 分離アーキテクチャ）

Grafana LGTM などの AGPL OSS を別ネームスペース + 別 pod で物理分離する ADR。

- 直接: `IMP-OBS-POL-002`（LGTM AGPL 分離維持）
- 間接: `IMP-DEP-POL-005`（AGPL 6 件の分離境界恒常検証） / `IMP-SUP-POL-002`（サプライチェーン監査エビデンス）

### ADR-BS-001（Backstage）

Backstage をサービス運用の第一表示面として採用する ADR。

- 直接: `IMP-TRACE-POL-006`（Backstage catalog 対応） / `IMP-CODEGEN-SCF-031, 033`（Software Template / catalog-info 自動生成）
- 間接: `IMP-DEV-GP-021`（Backstage Examples 登録） / `IMP-DX-POL-006`（Scorecards 連携）

### ADR-CICD-001（ArgoCD）

GitOps 配信エンジンとして ArgoCD を採用する ADR。

- 直接: `IMP-REL-POL-001`（GitOps 一本化） / `IMP-REL-ARG-010〜017`（ArgoCD App 構造 8 ID）全て
- 間接: `IMP-REL-POL-007`（App-of-Apps 構造） / `IMP-CI-RWF-010`（CI → Harbor 完結、ArgoCD が pull）

### ADR-CICD-002（Argo Rollouts）

Progressive Delivery エンジンとして Argo Rollouts を採用する ADR。

- 直接: `IMP-REL-PD-020〜028`（Argo Rollouts 9 ID）全て / `IMP-REL-POL-002, 003`（PD / AnalysisTemplate）
- 間接: `IMP-OBS-SLO-040〜047`（AnalysisTemplate の判定源）

### ADR-CICD-003（Kyverno）

Admission Controller として Kyverno を採用する ADR。

- 直接: `IMP-SUP-POL-006`（Kyverno 署名検証） / `IMP-POL-POL-001`（dual ownership）
- 間接: `IMP-CI-HAR-040〜051`（Harbor 運用と Kyverno verifyImages 連動） / `IMP-SUP-COS-010〜018`（cosign 検証）

### ADR-DATA-001〜004（CloudNativePG / Kafka / Valkey / MinIO）

データ層 OSS 4 本の採用 ADR。

- 直接: `IMP-CI-HAR-040`（Harbor の CloudNativePG バックエンド） / `IMP-SEC-KC-021`（Keycloak event の Kafka 外出し）
- 間接: `IMP-SEC-SP-020〜035`（SPIRE の etcd バックエンド移行検討） / `IMP-SEC-REV-054`（退職 revoke 監査ログの MinIO Object Lock）

### ADR-DIR-001〜003（contracts 昇格 / infra 分離 / sparse-checkout）

Phase 0 確定の 3 本の ディレクトリ設計 ADR。

- 直接: `IMP-DIR-*`（並列索引で管理、本章では再掲せず）
- 間接（本章採番 ID との関係）:
  - ADR-DIR-001: `IMP-BUILD-POL-002`（ワークスペース境界） / `IMP-CODEGEN-BUF-010〜017`（contracts 配下の buf 運用） / `IMP-BUILD-CW-010`（Cargo workspace 2 分割の境界議論）
  - ADR-DIR-002: `IMP-REL-ARG-010〜017`（deploy / infra / ops 3 階層への ArgoCD App 構造対応）
  - ADR-DIR-003: `IMP-DEV-DC-010〜017`（10 役 Dev Container が sparse-checkout と 1:1 対応） / `IMP-CI-RWF-012`（path-filter と cone 整合）

### ADR-FM-001（flagd / OpenFeature）

Feature flag エンジンとして flagd + OpenFeature を採用する ADR。

- 直接: `IMP-REL-POL-006`（flag 即時切替の PD からの分離） / `IMP-DEV-POL-006`（ローカル kind + Dapr Local + flagd）
- 間接: `IMP-POL-POL-002`（audit モードと flag rollout の組合せ）

### ADR-MIG-001 / 002（.NET Framework sidecar / API Gateway）

既存 .NET Framework 資産の段階的移行 ADR 2 本。

- 直接: 本章で採番する IMP なし（IMP-DIR-T3-060 で受ける）
- 間接: `IMP-DEV-GP-025`（Phase 1a の 8 例への拡大で legacy-wrap 参照が入る）

### ADR-OBS-001（Grafana LGTM）

観測性基盤として Grafana LGTM（Loki / Grafana / Tempo / Mimir）を採用する ADR。

- 直接: `IMP-OBS-POL-002`（AGPL 分離維持） / `IMP-OBS-OTEL-010〜019`（OTel Collector が LGTM に export）
- 間接: `IMP-OBS-SLO-040〜047`（Mimir 経由の SLO 計測） / `IMP-REL-PD-020〜028`（AnalysisTemplate の metric source）

### ADR-OBS-002（OTel Collector）

Collector を Agent（DaemonSet）+ Gateway（Deployment）の 2 層で運用する ADR。

- 直接: `IMP-OBS-POL-001`（OTel Collector 集約） / `IMP-OBS-OTEL-010〜019`（Collector 配置 10 ID）全て
- 間接: `IMP-OBS-SLO-040〜047`（SLO 計測データの流路）

### ADR-RULE-001（ZEN Engine）

ルールエンジンとして GoRules ZEN Engine を採用する ADR。

- 直接: 本章で採番する IMP なし（tier1 Rust `crates/policy/` に閉じる）
- 間接: `IMP-DEV-GP-025`（Phase 1a の 8 例への拡大で decision-example 拡張）

### ADR-RULE-002（Temporal）

ワークフローエンジンとして Temporal を採用する ADR（Phase 1b 以降）。

- 直接: 本章で採番する IMP なし
- 間接: `IMP-DEV-GP-025`（saga-example は Temporal 上に構築）

### ADR-SEC-001（Keycloak）

人間 ID プロバイダとして Keycloak を採用する ADR。

- 直接: `IMP-SEC-POL-001`（人間 ID Keycloak 集約） / `IMP-SEC-KC-010〜022`（Keycloak realm 13 ID）全て
- 間接: `IMP-SEC-REV-050〜059`（退職 revoke Runbook の主起点）

### ADR-SEC-002（OpenBao）

Secret 管理として OpenBao（Vault OSS fork）を採用する ADR。

- 直接: `IMP-SEC-POL-004`（OpenBao Secret 集約） / `IMP-SEC-KEY-001, 002`（鍵管理）
- 間接: `IMP-DEV-DC-015`（OpenBao dev server）

### ADR-SEC-003（SPIFFE / SPIRE）

ワークロード ID として SPIFFE / SPIRE を採用する ADR。

- 直接: `IMP-SEC-POL-002`（ワークロード ID SPIRE） / `IMP-SEC-SP-020〜035`（SPIRE 16 ID）全て
- 間接: `IMP-SEC-POL-006`（Istio Ambient mTLS が SPIRE SVID を使用）

### ADR-STOR-001 / 002（Longhorn / MetalLB）

ストレージとロードバランサの採用 ADR 2 本。

- 直接: 本章で採番する IMP なし
- 間接: `IMP-REL-ARG-010〜017`（ArgoCD App の PVC / LoadBalancer 定義を前提化）

### ADR-TIER1-001（Go + Rust ハイブリッド）

tier1 内部を Dapr ファサード = Go、自作領域 = Rust の 2 言語ハイブリッドとする ADR。

- 直接: `IMP-BUILD-CW-010〜017`（Rust workspace 8 ID） / `IMP-BUILD-GM-020〜027`（Go module 8 ID）
- 間接: `IMP-CODEGEN-BUF-010〜017`（buf 生成が 2 言語に分岐） / `IMP-DEV-DC-010〜017`（Dev Container の tier1-rust-dev / tier1-go-dev 分離）

### ADR-TIER1-002（Protobuf gRPC）

tier1 内部通信を Protobuf gRPC に固定する ADR。

- 直接: `IMP-CODEGEN-POL-001〜007`（Protobuf 単一真実源） / `IMP-CODEGEN-BUF-010〜017`（buf 生成 8 ID）
- 間接: `IMP-BUILD-POL-007`（生成物 commit と隔離）

### ADR-TIER1-003（内部言語不可視）

tier2 / tier3 から tier1 の内部言語判別を不可視化する ADR。

- 直接: `IMP-BUILD-CW-010`（tier1 / SDK の workspace 分離境界） / `IMP-BUILD-GM-027`（SDK Go module name 固定）
- 間接: `IMP-CODEGEN-BUF-012`（internal package の SDK 除外） / `IMP-DEV-GP-020〜026`（examples が SDK のみ使用）

## ADR 対応カバレッジ

Phase 0 時点で `02_構想設計/adr/` 直下に実在する ADR（Glob `ADR-*.md` で 29 件）+ 新規起票予定 7 件（SUP-001 / DEV-001 / REL-001 / DEP-001 / DX-001 / POL-001 / OBS-003）の計 36 件のうち、本章で採番された IMP-\* と直接/間接のいずれかで結びつくのは 30 件。未結合の 6 件（ADR-0002 図解規約 / ADR-MIG-001/002 は IMP-DIR 側のみ / ADR-RULE-001/002 は Phase 1a 以降 / ADR-STOR-001 LoadBalancer は間接扱いのみ）は 05_実装/ 側で新規採番する IMP が発生する見込みが薄いもの、または他章（00 章・Phase 1a 以降）で受けるものとして分類する。

カバレッジ率は 30/36 = 83%。Phase 1a で ADR-RULE-001（ZEN Engine）と ADR-MIG-001/002 の実装面採番で 100% に達する見通し。

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- IMP-ID 台帳: [`../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- 上流マトリクス: [`../../../04_概要設計/80_トレーサビリティ/05_構想設計ADR相関マトリクス.md`](../../../04_概要設計/80_トレーサビリティ/05_構想設計ADR相関マトリクス.md)
- 並列索引の ADR 対応: [`../../00_ディレクトリ設計/90_トレーサビリティ/03_ADR_との対応.md`](../../00_ディレクトリ設計/90_トレーサビリティ/03_ADR_との対応.md)
