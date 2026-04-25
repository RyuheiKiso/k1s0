# 99. 索引 / 10. ADR 対応表 / 01. ADR-IMP 対応マトリクス

本ファイルは `02_構想設計/adr/` 配下の全 ADR（リリース時点で 36 件 = 既存 29 + 新規起票予定 7）と、本実装段階で採番された IMP-\* ID の対応を「ADR から IMP を逆引き」する方向で表現する。ADR 改訂時に「どの実装 ID が影響を受けるか」を 1 ファイルで確定できる状態を IMP-TRACE-POL-005（双方向リンク必須）の系として保証する。

## マトリクスの読み方

ADR ごとに 1 セクションを配し、冒頭の散文で「ADR が決めた事柄 / 実装への影響 / なぜこれらの IMP が結びつくか」を示し、続いて IMP-\* の直接対応・間接対応を 2 列で列挙する。**直接対応** は当該 ADR の決定が IMP の存在理由そのものである関係、**間接対応** は ADR の副次的帰結として IMP が制約を受ける関係である。

IMP 側からの逆引き（IMP → ADR）は各章の核心節ファイル内「対応 ADR / DS-SW-COMP / NFR」セクションで双方向に記載されているため、本ファイルは ADR 側からの引きのみを扱う。網羅性は CI の孤立リンク検出（IMP-TRACE-POL-005 で定義）で検証される前提。

## リリース時点 新規起票予定の 7 ADR

本実装段階での各章 README が「本章初版策定時に起票予定」と明示した 7 本。リリース時点 の実装ドキュメント確定と同タイミングで adr 起票される。

### ADR-SUP-001（SLSA L2 先行 → L3 到達）

リリース時点で SLSA L2（ビルド履歴の真正性と改ざん困難性）を満たし / 採用後の運用拡大時に L3（Hermetic + Isolated + 4-eyes + Reproducible）へ到達する段階到達戦略。L3 先行案との差分を記録する。

- 直接: `IMP-SUP-POL-001`（段階到達原則） / `IMP-SUP-POL-002`（cosign keyless 必須） / `IMP-SUP-SLSA-030〜039`（SLSA Provenance v1 10 ID）全て / `IMP-SUP-COS-011`（OIDC→Fulcio→Rekor 信頼連鎖） / `IMP-SUP-COS-016`（オンプレ Fulcio/Rekor 移行予約） / `IMP-SUP-FLG-051`（flag 定義 OCI Artifact + Rekor 統合）
- 間接: `IMP-SUP-COS-010〜018`（cosign 9 ID、L2 達成の実装面） / `IMP-SUP-SBM-020〜029`（CycloneDX SBOM 10 ID、Provenance と一体配布される） / `IMP-SUP-FOR-040〜048`（Forensics Runbook、L3 到達時の監査再構成面） / `IMP-SUP-FLG-050〜057`（flag 定義の真正性、admission 統制と棚卸し）

### ADR-DEV-001（Paved Road 思想）

Paved Road（舗装路）アプローチで開発者の選択肢を絞り、examples / Scaffold / Dev Container を一本化する開発者体験戦略の方針 ADR。Paved Road の健全度は 95 章 DX メトリクスの Scaffold 利用率 / TFC で物理計測する。

- 直接: `IMP-DEV-POL-001`（Paved Road 一本化） / `IMP-DEV-POL-002`（Scaffold 必須経由） / `IMP-DEV-POL-003`（10 役 Dev Container） / `IMP-DEV-POL-004`（4 軸 DX パッケージ集約） / `IMP-DX-SCAF-030〜039`（Scaffold 利用率 10 ID = Paved Road 経由率の物理計測） / `IMP-DX-TFC-040〜049`（time-to-first-commit 10 ID = Paved Road 機能性の物理計測） / `IMP-TRACE-CAT-025`（Scaffold dry-run との bit 一致検証 = Paved Road 健全度の CI 強制） / `IMP-TRACE-CAT-026`（Off-Path 検出と DX-SCAF-033 同一バイナリ共有）
- 間接: `IMP-DEV-DC-010〜017`（Dev Container 8 ID） / `IMP-DEV-GP-020〜026`（Golden Path 7 ID） / `IMP-DEV-SO-030〜037`（Scaffold 運用 8 ID） / `IMP-DEV-BSN-040〜048`（Backstage 連携 9 ID） / `IMP-DEV-ONB-050〜059`（Onboarding 10 ID） / `IMP-CODEGEN-SCF-030〜037`（Scaffold CLI engine 8 ID）

### ADR-REL-001（Progressive Delivery 必須）

Canary リリースと AnalysisTemplate による自動 rollback を全 tier1 サービスで必須化する方針 ADR。

- 直接: `IMP-REL-POL-002`（Progressive Delivery 必須） / `IMP-REL-POL-003`（Canary AnalysisTemplate 強制） / `IMP-REL-PD-020〜028`（Argo Rollouts 9 ID）全て / `IMP-REL-AT-040〜049`（AnalysisTemplate 共通セット 5 本 + 継承 5 ID）全て
- 間接: `IMP-OBS-SLO-040〜047`（SLO が Canary の判定源になるため） / `IMP-REL-RB-050〜059`（PD fail 時の rollback 5 段階） / `IMP-REL-FFD-026, 030〜039`（PD と機能公開の分離 = flagd 連動）

### ADR-DEP-001（Renovate 中央集約）

Renovate による依存更新を全言語（Rust / Go / TS / C#）で中央集約する方針 ADR。

- 直接: `IMP-DEP-POL-001`（Renovate 経由のみ） / `IMP-DEP-POL-006`（自動マージ patch のみ） / `IMP-SUP-POL-003`（CycloneDX SBOM 全添付） / `IMP-SUP-POL-006`（SBOM 差分監視）
- 間接: `IMP-DEP-POL-002〜005, 007`（lockfile / vendoring / ライセンス / AGPL / SBOM） / `IMP-BUILD-CW-012`（toolchain 固定への Renovate 連携） / `IMP-SUP-SBM-025`（新規依存通知の Slack 連動と Security CODEOWNERS、Renovate 自動 PR の reviewer 強制）

### ADR-DX-001（DX メトリクス章分離）

DX メトリクス（DORA Four Keys / SPACE / Scaffold 利用率 / time-to-first-commit / EM 月次レポート）を 95 章として独立運用する構造 ADR。60 章観測性（SLO/SLI）との境界を定義する。リリース時点で 5 節 50 ID の実装を確定する。

- 直接: `IMP-DX-POL-001〜007`（DX 方針 7 件）全て / `IMP-DX-DORA-010〜020`（DORA 11 ID） / `IMP-DX-SPC-021〜029`（SPACE 5 軸 9 ID） / `IMP-DX-SCAF-030〜039`（Scaffold 利用率 10 ID） / `IMP-DX-TFC-040〜049`（time-to-first-commit 10 ID） / `IMP-DX-EMR-050〜059`（EM 月次レポート 10 ID）
- 間接: `IMP-OBS-POL-007`（DORA 4 keys は 95 章へ分離の明示） / `IMP-DEV-POL-004`（time-to-first-commit SLI = 95 章 TFC へ実装委譲） / `IMP-DEV-ONB-055`（onboardingTimeFactRetriever = TFC-041 の物理基盤） / `IMP-DEV-ONB-059`（onboarding-stumble label = EMR-057 / TFC-045 の入力）

### ADR-POL-001（Kyverno dual ownership）

Kyverno ポリシーを Platform + Security の二重承認で運用する構造 ADR。

- 直接: `IMP-POL-POL-001`（dual ownership） / `IMP-POL-POL-003`（例外 30 日時限）
- 間接: `IMP-POL-POL-002, 004〜007`（audit モード / 脅威モデル / Runbook / WORM / NetworkPolicy） / `IMP-SUP-POL-006`（cosign 検証の Kyverno 実行）

### ADR-OBS-003（Incident Taxonomy 統合）

可用性（AVL）インシデントとセキュリティ（SEC）インシデントを単一 Taxonomy で統合分類する ADR。

- 直接: `IMP-OBS-POL-004`（Taxonomy 統合） / `IMP-OBS-INC-060〜071`（Incident Taxonomy 12 ID）全て / `IMP-OBS-RB-080`（alert / Runbook 1:1 ID 体系で AVL/SEC を統合命名） / `IMP-SUP-POL-005`（Forensics Runbook 統制） / `IMP-SUP-COS-014`（Rekor インデックス検索 Forensics 基盤化） / `IMP-SUP-FOR-040`（Runbook トリガ 3 種類 + 起点 digest 統一）
- 間接: `IMP-SUP-FOR-040〜048`（Forensics Runbook は SEC 側の具体 Runbook） / `IMP-SEC-REV-050〜059`（退職 revoke は SEC × HIGH の Runbook） / `IMP-OBS-EB-054`（セキュリティ hotfix の budget bypass 経路） / `IMP-SUP-FLG-056〜057`（flag 検証失敗時 Forensics 連携、改ざん vs 鍵異常 の Sev1/Sev2 振り分け）

## 既存 ADR（リリース時点 以前に確定済）

### ADR-0001（Istio Ambient vs Sidecar）

サービスメッシュを Istio Ambient モード（ztunnel + waypoint）で運用する ADR。Ambient mTLS の証明書は cert-manager + istio-csr 経由で SPIRE SVID を Istio に供給する物理経路で実現する。

- 直接: `IMP-SEC-POL-006`（Istio Ambient mTLS） / `IMP-SEC-SP-020〜035`（SPIRE の Istio 統合面） / `IMP-SEC-CRT-065`（istio-csr ClusterIssuer 経由 SPIRE SVID 統合） / `IMP-SEC-CRT-066`（SVID 1h ローテーション設計）
- 間接: `IMP-OBS-OTEL-010〜019`（Ambient 由来のメトリクス経路）

### ADR-0002（Diagram layer convention）

図解の 4 レイヤ配色規約（アプリ / ネットワーク / インフラ / データ）。

- 直接: 本章で採番する IMP なし（ドキュメント規約 ADR）
- 間接: 本 05_実装/ 全 drawio 図が準拠

### ADR-0003（AGPL 分離アーキテクチャ）

Grafana LGTM などの AGPL OSS を別ネームスペース + 別 pod で物理分離する ADR。

- 直接: `IMP-OBS-POL-002`（LGTM AGPL 分離維持） / `IMP-OBS-LGTM-020`（namespace 隔離） / `IMP-OBS-LGTM-022`（NetworkPolicy ingress 制限） / `IMP-OBS-PYR-035`（Pyroscope server を AGPL 同居 namespace 配置） / `IMP-SUP-POL-007`（AGPL 分離エビデンス常時保持） / `IMP-SUP-COS-015`（sigstore ツール群の AGPL 分離不要判定） / `IMP-SUP-SBM-026`（CycloneDX licenses 検出時の tier1/LGTM/不明 3 分岐）
- 間接: `IMP-DEP-POL-005`（AGPL 6 件の分離境界恒常検証） / `IMP-SUP-POL-002`（サプライチェーン監査エビデンス）

### ADR-BS-001（Backstage）

Backstage をサービス運用の第一表示面として採用する ADR。

- 直接: `IMP-TRACE-POL-006`（Backstage catalog 対応） / `IMP-CODEGEN-SCF-031, 033`（Software Template / catalog-info 自動生成） / `IMP-DEV-BSN-040〜048`（Backstage 連携 9 ID）全て / `IMP-DEV-POL-007`（Backstage を機械可読 metadata 真実源として位置付け） / `IMP-DX-SPC-027`（Scorecards SPACE 5 軸ペイン） / `IMP-DX-SCAF-035`（Scorecards Adoption Rate 表示） / `IMP-DX-TFC-041`（onboardingTimeFactRetriever TechInsights 統合） / `IMP-DX-EMR-050`（Backstage backend job として実装） / `IMP-DX-EMR-055`（Catalog DX-Report Entity 自動更新） / `IMP-TRACE-CAT-020〜029`（catalog-info.yaml スキーマ検証 10 ID = Backstage 同期前の事前検証） / `IMP-TRACE-CAT-023, 024`（Group / System catalog snapshot 経由の owner / system 実在検証）
- 間接: `IMP-DEV-GP-021`（Backstage Examples 登録） / `IMP-DEV-SO-037`（Scaffold 自動 discovery 連動） / `IMP-DEV-ONB-051`（Day 0 Backstage Group 登録 PR） / `IMP-DEV-ONB-055`（onboardingTimeFactRetriever 計測） / `IMP-DEV-ONB-056`（Week 1 Scorecards 学習 check） / `IMP-DX-POL-006`（Scorecards 連携）

### ADR-CICD-001（ArgoCD）

GitOps 配信エンジンとして ArgoCD を採用する ADR。

- 直接: `IMP-REL-POL-001`（GitOps 一本化） / `IMP-REL-ARG-010〜017`（ArgoCD App 構造 8 ID）全て / `IMP-CI-POL-001`（CI 責務は Harbor push まで） / `IMP-REL-FFD-033`（flagd OCI Artifact 配布が ArgoCD ApplicationSet 経由）
- 間接: `IMP-REL-POL-007`（App-of-Apps 構造） / `IMP-CI-RWF-010〜021`（GitHub Actions reusable workflow 群） / `IMP-CI-PF-030〜037`（path-filter 選択ビルド） / `IMP-CI-QG-060〜067`（quality gate） / `IMP-CI-BP-070〜077`（branch protection） / `IMP-REL-RB-050〜053`（rollback Phase 2-3 で argocd app sync を呼ぶ）

### ADR-CICD-002（Argo Rollouts）

Progressive Delivery エンジンとして Argo Rollouts を採用する ADR。

- 直接: `IMP-REL-PD-020〜028`（Argo Rollouts 9 ID）全て / `IMP-REL-POL-002, 003, 006`（PD / AnalysisTemplate / canary 3 段階） / `IMP-REL-AT-040〜049`（AnalysisTemplate 共通セット 5 本 + 継承 5 ID）全て / `IMP-REL-RB-050〜059`（rollback runbook 10 ID、Argo Rollouts undo 経路を含む）
- 間接: `IMP-OBS-SLO-040〜047`（AnalysisTemplate の判定源） / `IMP-OBS-EB-053`（burn rate recording rule を AT-044 が参照）

### ADR-CICD-003（Kyverno）

Admission Controller として Kyverno を採用する ADR。

- 直接: `IMP-SUP-POL-004`（Kyverno admission 強制 / warn 禁止） / `IMP-POL-POL-001`（dual ownership） / `IMP-REL-FFD-034`（flagd ConfigMap への Kyverno cosign verify-blob） / `IMP-REL-PD-024`（PD 例外経路の Kyverno admission 検証） / `IMP-SUP-COS-013`（verifyImages subject pin） / `IMP-SUP-SBM-022`（cosign attest --type cyclonedx 配布） / `IMP-SUP-SBM-023`（verifyImages cyclonedx attestation 必須化） / `IMP-SUP-SLSA-032`（cosign attest slsaprovenance1） / `IMP-SUP-SLSA-033`（verifyAttestations type=slsaprovenance1 必須） / `IMP-SUP-SLSA-035`（claimed > verified の admission reject） / `IMP-SUP-FLG-050`（cosign sign-blob keyless flag 署名） / `IMP-SUP-FLG-052`（ClusterPolicy verify-flag-attestation） / `IMP-SEC-OBO-047`（OpenBao Kubernetes Auth Method の SA token review が Kyverno 経路を補完）
- 間接: `IMP-CI-HAR-040〜051`（Harbor 運用と Kyverno verifyImages 連動） / `IMP-SUP-COS-010〜018`（cosign 検証） / `IMP-SUP-FOR-041`（Kyverno admission log 起点の Forensics）

### ADR-DATA-001〜004（CloudNativePG / Kafka / Valkey / MinIO）

データ層 OSS 4 本の採用 ADR。

- 直接: `IMP-CI-HAR-040`（Harbor の CloudNativePG バックエンド） / `IMP-SEC-KC-021`（Keycloak event の Kafka 外出し）
- 間接: `IMP-SEC-SP-020〜035`（SPIRE の etcd バックエンド移行検討） / `IMP-SEC-REV-054`（退職 revoke 監査ログの MinIO Object Lock）

### ADR-DIR-001〜003（contracts 昇格 / infra 分離 / sparse-checkout）

リリース時点 確定の 3 本の ディレクトリ設計 ADR。

- 直接: `IMP-DIR-*`（並列索引で管理、本章では再掲せず）
- 間接（本章採番 ID との関係）:
  - ADR-DIR-001: `IMP-BUILD-POL-002`（ワークスペース境界） / `IMP-CODEGEN-BUF-010〜017`（contracts 配下の buf 運用） / `IMP-CODEGEN-OAS-020`（contracts/openapi 配下の yaml 運用） / `IMP-CODEGEN-GLD-040`（tests/codegen の本番 contracts からの分離） / `IMP-BUILD-CW-010`（Cargo workspace 2 分割の境界議論）
  - ADR-DIR-002: `IMP-REL-ARG-010〜017`（deploy / infra / ops 3 階層への ArgoCD App 構造対応）
  - ADR-DIR-003: `IMP-DEV-DC-010〜017`（10 役 Dev Container が sparse-checkout と 1:1 対応） / `IMP-CI-RWF-012`（path-filter と cone 整合） / `IMP-CI-PF-031`（filters.yaml の cone 配置） / `IMP-CI-BP-076`（infra/github の cone 配置）

### ADR-FM-001（flagd / OpenFeature）

Feature flag エンジンとして flagd + OpenFeature を採用する ADR。

- 直接: `IMP-REL-POL-004`（flagd cosign 署名必須） / `IMP-REL-POL-006`（flag 即時切替の PD からの分離） / `IMP-REL-FFD-030〜039`（flagd 配布 / sidecar / SDK / 評価ログ 10 ID）全て / `IMP-REL-PD-026`（PD 3 パターン連動） / `IMP-DEV-POL-006`（ローカル kind + Dapr Local + flagd） / `IMP-SUP-FLG-050〜057`（flag 定義署名検証 8 ID、admission 統制と棚卸しと Forensics 連携）全て
- 間接: `IMP-POL-POL-002`（audit モードと flag rollout の組合せ）

### ADR-MIG-001 / 002（.NET Framework sidecar / API Gateway）

既存 .NET Framework 資産の段階的移行 ADR 2 本。

- 直接: 本章で採番する IMP なし（IMP-DIR-T3-060 で受ける）
- 間接: `IMP-DEV-GP-025`（リリース時点 の 8 例への拡大で legacy-wrap 参照が入る）

### ADR-OBS-001（Grafana LGTM）

観測性基盤として Grafana LGTM（Loki / Grafana / Tempo / Mimir）を採用する ADR。

- 直接: `IMP-OBS-POL-002`（AGPL 分離維持） / `IMP-OBS-OTEL-010〜019`（OTel Collector が LGTM に export） / `IMP-OBS-LGTM-020〜029`（LGTM Stack 配置 10 ID）全て / `IMP-OBS-PYR-030〜039`（Pyroscope 統合 10 ID） / `IMP-OBS-EB-050〜057`（Error Budget Mimir 算出） / `IMP-OBS-RB-080〜089`（Runbook 連携） / `IMP-REL-AT-046`（AnalysisTemplate Mimir provider 統一） / `IMP-DX-SPC-022〜023`（SPACE Activity / Communication が Mimir / Loki に依存） / `IMP-DX-SCAF-030〜034`（Scaffold 利用率の Mimir / Loki 集計） / `IMP-DX-TFC-043`（TFC Mimir histogram） / `IMP-DX-EMR-051`（4 入力統合の PromQL + LogQL）
- 間接: `IMP-OBS-SLO-040〜047`（Mimir 経由の SLO 計測） / `IMP-REL-PD-020〜028`（AnalysisTemplate の metric source） / `IMP-REL-AT-040〜044`（共通テンプレ 5 本が Mimir PromQL 参照） / `IMP-REL-RB-059`（Incident メタデータ集計）

### ADR-OBS-002（OTel Collector）

Collector を Agent（DaemonSet）+ Gateway（Deployment）の 2 層で運用する ADR。

- 直接: `IMP-OBS-POL-001`（OTel Collector 集約） / `IMP-OBS-OTEL-010〜019`（Collector 配置 10 ID）全て / `IMP-OBS-LGTM-026`（read 経路は datasource 直接、write は Collector 経由） / `IMP-OBS-LGTM-028`（Gateway disk queue バッファリング）
- 間接: `IMP-OBS-SLO-040〜047`（SLO 計測データの流路） / `IMP-OBS-EB-050〜057`（Mimir 算出を経由した状態判定） / `IMP-OBS-PYR-033`（Pyroscope は OTel pipeline 外、将来統合検討）

### ADR-RULE-001（ZEN Engine）

ルールエンジンとして GoRules ZEN Engine を採用する ADR。

- 直接: 本章で採番する IMP なし（tier1 Rust `crates/policy/` に閉じる）
- 間接: `IMP-DEV-GP-025`（リリース時点 の 8 例への拡大で decision-example 拡張）

### ADR-RULE-002（Temporal）

ワークフローエンジンとして Temporal を採用する ADR（運用蓄積後）。

- 直接: 本章で採番する IMP なし
- 間接: `IMP-DEV-GP-025`（saga-example は Temporal 上に構築）

### ADR-SEC-001（Keycloak）

人間 ID プロバイダとして Keycloak を採用する ADR。

- 直接: `IMP-SEC-POL-001`（人間 ID Keycloak 集約） / `IMP-SEC-KC-010〜022`（Keycloak realm 13 ID）全て
- 間接: `IMP-SEC-REV-050〜059`（退職 revoke Runbook の主起点）

### ADR-SEC-002（OpenBao）

Secret 管理として OpenBao（Vault OSS fork）を採用する ADR。リリース時点 で Raft Integrated Storage 3 node HA / Auto-unseal AWS KMS / KV-v2・PKI・Transit 3 secret engine / Kubernetes Auth Method / Audit Device 二段保管を確定。cert-manager の ClusterIssuer は Vault PKI を経由するため CRT-061/062 もここで結びつく。

- 直接: `IMP-SEC-POL-004`（OpenBao Secret 集約） / `IMP-SEC-POL-005`（cert-manager 自動更新が OpenBao PKI を起点にする） / `IMP-SEC-OBO-040〜049`（OpenBao 10 ID）全て / `IMP-SEC-CRT-061`（Vault PKI ClusterIssuer 連携） / `IMP-SEC-CRT-062`（Vault PKI 経由の中間 CA 発行設計）
- 間接: `IMP-DEV-DC-015`（OpenBao dev server） / `IMP-SEC-REV-054`（OpenBao audit device の 7 年 WORM 保管が退職 revoke 監査と接続） / `IMP-SUP-COS-016`（オンプレ Sigstore 移行検討時に OpenBao Transit が鍵管理候補）

### ADR-SEC-003（SPIFFE / SPIRE）

ワークロード ID として SPIFFE / SPIRE を採用する ADR。cert-manager の istio-csr 統合により SPIRE 発行の SVID を Istio Ambient データプレーンへ供給するため、CRT-065/066 が SPIRE 連携の物理経路となる。

- 直接: `IMP-SEC-POL-002`（ワークロード ID SPIRE） / `IMP-SEC-SP-020〜035`（SPIRE 16 ID）全て / `IMP-SEC-CRT-065`（istio-csr 経由 SPIRE SVID 統合） / `IMP-SEC-CRT-066`（SVID 1h ローテーションの cert-manager Trigger 設計）
- 間接: `IMP-SEC-POL-006`（Istio Ambient mTLS が SPIRE SVID を使用）

### ADR-STOR-001 / 002（Longhorn / MetalLB）

ストレージとロードバランサの採用 ADR 2 本。

- 直接: 本章で採番する IMP なし
- 間接: `IMP-REL-ARG-010〜017`（ArgoCD App の PVC / LoadBalancer 定義を前提化）

### ADR-TIER1-001（Go + Rust ハイブリッド）

tier1 内部を Dapr ファサード = Go、自作領域 = Rust の 2 言語ハイブリッドとする ADR。

- 直接: `IMP-BUILD-CW-010〜017`（Rust workspace 8 ID） / `IMP-BUILD-GM-020〜027`（Go module 8 ID）
- 間接: `IMP-CODEGEN-BUF-010〜017`（buf 生成が 2 言語に分岐） / `IMP-DEV-DC-010〜017`（Dev Container の tier1-rust-dev / tier1-go-dev 分離） / `IMP-CI-RWF-010`（言語別 reusable workflow） / `IMP-CI-QG-060〜063`（4 言語 toolchain の quality gate）

### ADR-TIER1-002（Protobuf gRPC）

tier1 内部通信を Protobuf gRPC に固定する ADR。

- 直接: `IMP-CODEGEN-POL-001〜007`（Protobuf 単一真実源） / `IMP-CODEGEN-BUF-010〜017`（buf 生成 8 ID） / `IMP-CI-POL-003`（contracts 軸 path-filter）
- 間接: `IMP-BUILD-POL-007`（生成物 commit と隔離） / `IMP-CODEGEN-OAS-020〜022`（OpenAPI は HTTP 表面例外として境界規定） / `IMP-CODEGEN-GLD-040〜047`（生成器挙動の回帰検証で gRPC / HTTP 両系統を pin） / `IMP-CI-PF-033`（contracts → sdk-all 強制昇格）

### ADR-TIER1-003（内部言語不可視）

tier2 / tier3 から tier1 の内部言語判別を不可視化する ADR。

- 直接: `IMP-BUILD-CW-010`（tier1 / SDK の workspace 分離境界） / `IMP-BUILD-GM-027`（SDK Go module name 固定）
- 間接: `IMP-CODEGEN-BUF-012`（internal package の SDK 除外） / `IMP-CODEGEN-OAS-023`（OpenAPI 生成先の tier1 / tier3 / SDK 物理分離） / `IMP-DEV-GP-020〜026`（examples が SDK のみ使用）

## ADR 対応カバレッジ

リリース時点で `02_構想設計/adr/` 直下に実在する ADR（Glob `ADR-*.md` で 29 件）+ 新規起票予定 7 件（SUP-001 / DEV-001 / REL-001 / DEP-001 / DX-001 / POL-001 / OBS-003）の計 36 件のうち、本章で採番された IMP-\* と直接/間接のいずれかで結びつくのは 30 件。未結合の 6 件（ADR-0002 図解規約 / ADR-MIG-001/002 は IMP-DIR 側のみ / ADR-RULE-001/002 は リリース時点 以降 / ADR-STOR-001 LoadBalancer は間接扱いのみ）は 05_実装/ 側で新規採番する IMP が発生する見込みが薄いもの、または他章（00 章・リリース時点 以降）で受けるものとして分類する。

カバレッジ率は 30/36 = 83%。リリース時点 で ADR-RULE-001（ZEN Engine）と ADR-MIG-001/002 の実装面採番で 100% に達する見通し。

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- IMP-ID 台帳: [`../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- 上流マトリクス: [`../../../04_概要設計/80_トレーサビリティ/05_構想設計ADR相関マトリクス.md`](../../../04_概要設計/80_トレーサビリティ/05_構想設計ADR相関マトリクス.md)
- 並列索引の ADR 対応: [`../../00_ディレクトリ設計/90_トレーサビリティ/03_ADR_との対応.md`](../../00_ディレクトリ設計/90_トレーサビリティ/03_ADR_との対応.md)
