# 99. 索引 / 30. NFR 対応表 / 01. NFR-IMP 対応マトリクス

本ファイルは `03_要件定義/30_非機能要件/` 配下の NFR-\* ID（非機能要件 ID）と、本実装段階で採番された IMP-\* ID の対応を「NFR から IMP を逆引き」する方向で表現する。監査対応や受け入れ基準レビュー時に「この NFR を満たす実装 ID は何か」を 1 ファイルで特定できる状態を IMP-TRACE-POL-005（双方向リンク）の系として提供する。

## NFR カテゴリ体系

NFR は `03_要件定義/30_非機能要件/` で 9 カテゴリ（A〜I）× サブカテゴリ構造で定義される。本マトリクスは同じ構造に従い、サブカテゴリ単位で節を配する。サブカテゴリ略号は以下に固定（カテゴリ A〜I 全てが本章 IMP-\* から参照されるわけではない点に注意）。

- **A 可用性**: CONT（継続性）/ FT（耐障害性）/ DR（災害復旧）/ REC（復旧性）
- **B 性能拡張性**: WL（ワークロード）/ PERF（性能）/ RES（拡張性）/ QA（品質保証）
- **C 運用保守性**: NOP（ノーマル運用）/ MNT（保守）/ IR（インシデント対応）/ ENV（環境）/ SUP（支援体制）/ MGMT（運用管理）
- **D 移行性**: TIM（時期）/ MTH（手法）/ OBJ（対象）/ PLN（計画）
- **E セキュリティ**: AC（アクセス制御）/ ENC（暗号）/ MON（監視）/ NW（ネットワーク）/ SIR（インシデント対応）/ PRE（予防）/ RSK（リスク）/ AV（マルウェア）/ WEB（Web）
- **F システム環境エコロジー**: SYS（システム）/ CHR（時系）/ STD（標準）
- **G データ保護とプライバシー**: CLS（分類）/ ENC（暗号）/ AC（アクセス制御）
- **H アーティファクト完整性とコンプライアンス**: INT（完整性）/ KEY（鍵）/ COMP（コンプライアンス）
- **I SLI/SLO**: SLI / SLO

重要: 旧ドラフトに存在した NFR-E-DXP / NFR-G-DES 等は現行 NFR 体系に存在せず、本マトリクスからは除外する。

## マトリクスの読み方

各節の冒頭で、該当カテゴリが本実装段階で何を要求し、実装面ではどの章・どの IMP 群で受けるかを散文で示す。続いて NFR ID ごとに「直接対応 IMP」と「間接対応 IMP」の 2 列を表で列挙する。NFR 側の「受け入れ基準」は `03_要件定義/30_非機能要件/` の各カテゴリファイルで確定しているため、本ファイルは ID 間の対応のみを示す。

## A 可用性（Availability）

可用性は SLA / SLO / Runbook / 自動復旧の 4 本柱で実装される。本章では主に 60 章観測性（SLO/SLI）・70 章リリース（自動 rollback）・60 章 Incident Taxonomy から結合する。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-A-CONT-001（SLA 99%） | `IMP-OBS-POL-003` / `IMP-OBS-SLO-040〜047`（内部 SLO 99.9% を 99% SLA の上位に設定） / `IMP-OBS-EB-050〜057`（Error Budget による SLO 違反の自動制御） / `IMP-REL-POL-001〜007`（リリース 7 軸原則） / `IMP-REL-ARG-010〜017`（ArgoCD App 構造） / `IMP-REL-PD-020〜028`（Argo Rollouts PD） / `IMP-REL-AT-040〜049`（AnalysisTemplate 共通 5 本 + 継承） / `IMP-REL-RB-050〜059`（rollback runbook 10 ID） | `IMP-OBS-RB-080〜089`（Runbook で MTTR 短縮） |
| NFR-A-FT-001（自動復旧 15 分以内） | `IMP-REL-POL-005`（手動 rollback 15 分以内） / `IMP-REL-PD-027`（手動 rollback 1 コマンド化） / `IMP-REL-RB-050〜059`（5 段階タイムライン + 1 コマンド + 4-eyes + 演習 + Postmortem 自動化） / `IMP-SEC-OBO-040`（Raft Integrated Storage 3 node HA で Vault 単一障害点排除） / `IMP-SEC-OBO-046`（Auto-unseal による Pod 再起動時の自動復旧） / `IMP-SEC-CRT-067`（5 分 Reconciliation Loop による証明書失効自動検知） / `IMP-SEC-CRT-068`（Prometheus Alert 4 段階エスカレーション 7d Sev3 → 24h Sev2 → 1h Sev1 → renewal fail Sev2） | `IMP-OBS-INC-064`（Sev1 Runbook） / `IMP-REL-AT-040〜049`（AT failureLimit 経由の自動 abort） |
| NFR-A-REC-002（Runbook 15 本） | `IMP-OBS-INC-064`（AVL/SEC × Sev1/Sev2 の 4 セル 15 Runbook） / `IMP-OBS-SLO-045`（Runbook 一対一対応） / `IMP-OBS-RB-080〜089`（SLI ↔ Alert ↔ Runbook 1:1 結合と CI 強制） | `IMP-SEC-REV-050〜059`（退職 revoke Runbook） |
| NFR-A-DR-\* | 運用蓄積後で対応（本章未結合） | `IMP-REL-ARG-010〜017`（ArgoCD による GitOps DR 基盤） |
| NFR-A-FT-002〜004（多重化 / 障害注入 / 縮退） | リリース時点 で対応（本章未結合） | `IMP-REL-PD-026〜028`（Canary / Blue-Green の多重化経路） |

## B 性能拡張性（Performance / Scalability）

性能系は SLI 計測を前提とし、ビルド時間・ランタイム p99・Scale の 3 層で実装される。本章では 10 章ビルド設計（ビルド時間 SLI）と 60 章観測性（ランタイム SLI）に結合が集中する。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-B-PERF-001（tier1 API p99 < 500ms） | `IMP-OBS-SLO-040〜047`（API 種別 p99 SLO 階層） / `IMP-OBS-POL-003`（SRE Book 準拠 SLO） / `IMP-OBS-PYR-030〜039`（Pyroscope 連続プロファイリングで hot-spot 5 分以内特定） | `IMP-BUILD-POL-001〜007`（性能基盤としてのビルド方針） / `IMP-OBS-OTEL-010〜019`（サンプリング 100% 維持） |
| NFR-B-PERF-002〜007（State Get / Decision / PubSub / Log / Flag の p99） | `IMP-OBS-SLO-040〜047`（各 API の SLO 細目） | `IMP-REL-PD-023`（AnalysisTemplate の判定源） |
| NFR-B-WL-\*（ワークロード規模） | リリース時点 で対応（本章未結合） | `IMP-OBS-OTEL-010〜019`（規模計測基盤） |
| NFR-B-RES-\*（水平・垂直拡張） | リリース時点 で対応（本章未結合） | `IMP-CI-RWF-013`（runner 自動スケール） |
| NFR-B-QA-\*（性能試験） | リリース時点 で対応（本章未結合） | `IMP-CI-RWF-018`（coverage 閾値段階導入の先に性能試験） |

## C 運用保守性（Operability / Maintainability）

採用側の小規模運用前提の NFR が 12 章全体に伸びる最重要カテゴリ。本章では 10 章ビルド・30 章 CI/CD・40 章依存管理・50 章 DX・90 章ガバナンス全てに結合する。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-C-NOP-001（採用側の小規模運用） | `IMP-DEV-POL-003`（10 役 Dev Container） / `IMP-DEV-POL-004`（time-to-first-commit SLI） / `IMP-DEV-ONB-050`（Day 1 4 時間 SLI 化） / `IMP-DEV-ONB-052〜053`（Day 1 4 step / Hello World 5 step） / `IMP-DEV-GP-020〜024`（Golden Path examples） / `IMP-DX-TFC-040〜049`（TFC 10 ID = 採用側 Onboarding 動線の物理計測） / `IMP-DX-EMR-050〜057`（EM 月次レポート = 計測値が読まれる物理経路） | `IMP-BUILD-POL-006`（ビルド時間 SLI） / `IMP-DX-POL-001〜007`（DORA 4 keys） / `IMP-DX-SPC-028`（個人ランキング化禁止 = 心理的安全性） / `IMP-DX-SCAF-039`（Scaffold PII transform） |
| NFR-C-NOP-002（可視性） | `IMP-TRACE-POL-001〜007`（索引 7 原則） / `IMP-TRACE-CI-010〜019`（整合性 CI 10 ID = 索引整合状態の常時可視化） / `IMP-TRACE-CAT-020〜029`（catalog-info 検証 10 ID = Backstage 同期前検証で可視化品質を担保） / `IMP-TRACE-CAT-029`（月次 Off-Path Sev3 通知 + EM レポート転載） / `IMP-OBS-POL-001〜007`（観測性方針） / `IMP-DEV-POL-007`（Backstage を機械可読 metadata 真実源化） / `IMP-DEV-BSN-040〜048`（Backstage 連携 9 ID） / `IMP-DEV-ONB-055`（onboarding SLI 計測経路） / `IMP-DEV-ONB-058`（Month 1 自走判定可視化） / `IMP-DX-SPC-027`（SPACE 5 軸 Scorecards 表示） / `IMP-DX-SCAF-035`（Scaffold Adoption Rate Scorecards） / `IMP-DX-TFC-041`（TFC TechInsights 表示） / `IMP-DX-EMR-053〜055`（EM 月次レポート 3 配信先） | 全 IMP-\*（可視性は全実装に効く） |
| NFR-C-NOP-004（ビルド所要時間） | `IMP-BUILD-POL-006`（ビルド時間 SLI 計測） / `IMP-CI-POL-002`（quality gate 統制） / `IMP-CI-RWF-016`（cache キー規約） / `IMP-CI-PF-030〜037`（path-filter 選択ビルド） / `IMP-CI-QG-060〜067`（quality gate 並列実行） | `IMP-BUILD-CW-013`（sccache） / `IMP-BUILD-GM-025`（GOCACHE リモート） / `IMP-CI-RWF-013`（runner 自動スケール） |
| NFR-C-MNT-003（API 互換方針 12 か月） | `IMP-CODEGEN-POL-003`（buf breaking 検知） / `IMP-CODEGEN-BUF-013, 016`（FILE level / v1-v2 分岐） / `IMP-CODEGEN-OAS-024, 027`（oasdiff `--fail-on ERR` / OpenAPI v1-v2 分岐） / `IMP-CI-QG-060〜067`（quality gate 機械化） | `IMP-REL-ARG-010〜017`（段階 rollout） |
| NFR-C-IR-001（Severity 別応答） | `IMP-OBS-INC-060〜071`（Incident Taxonomy） / `IMP-SEC-REV-050`（退職 revoke 15 分 SLA） / `IMP-OBS-RB-080〜089`（Runbook 連携で MTTR 短縮） / `IMP-OBS-PYR-030〜039`（Pyroscope で root cause 5 分以内特定） / `IMP-REL-RB-050〜059`（rollback runbook 5 段階 + 第二/第三経路） / `IMP-REL-FFD-039`（kill switch Sev2 自動連動） / `IMP-SEC-OBO-048`（OpenBao root token 利用 = Sev1） / `IMP-SEC-CRT-068`（cert-manager Prometheus Alert 4 段階エスカレーション 7d Sev3 → 24h Sev2 → 1h Sev1 → renewal fail Sev2） | `IMP-DX-POL-002`（Severity 別 DORA 分離） |
| NFR-C-IR-002（Circuit Breaker） | `IMP-REL-POL-003`（AnalysisTemplate 強制） / `IMP-REL-POL-006`（canary 3 段階） / `IMP-REL-PD-020〜021`（canary 既定 + tier1 10 段階細分化） / `IMP-REL-AT-040〜049`（AT 共通 5 本 + 継承の failureLimit による自動 abort） / `IMP-REL-RB-050〜055`（rollback 5 段階 + 第二/第三経路） / `IMP-OBS-EB-051〜052`（Error Budget 連動の Canary 自動下方修正/Sync 停止） | `IMP-REL-PD-023〜025`（failureLimit） |
| NFR-C-MGMT-001（設定 Git 管理） | `IMP-REL-POL-001`（GitOps 一本化） / `IMP-CI-POL-007`（Renovate PR） / `IMP-CI-PF-031`（filters.yaml 単一真実源） / `IMP-CI-BP-076`（branch protection の terraform IaC 化） / `IMP-DEV-BSN-042`（catalog-info.yaml 必須 5 属性） / `IMP-DEV-BSN-048`（catalog-info.yaml を真実源化） / `IMP-DEV-SO-031, 035`（Scaffold 4 出力の Git 管理） / `IMP-TRACE-CI-010〜019`（整合性 CI = 索引の Git 管理状態を CI 段で強制） / `IMP-TRACE-CAT-020〜029`（catalog-info 検証 = Backstage 同期前の Git 管理状態強制） | `IMP-BUILD-POL-007`（生成物 commit） |
| NFR-C-MGMT-002（Flag / Decision 変更監査） | `IMP-POL-POL-004`（脅威モデル ADR 化） / `IMP-POL-POL-006`（WORM 監査） / `IMP-DEV-POL-006`（動線詰まりの月次帰着フィードバック） / `IMP-DEV-SO-037`（template 更新影響範囲可視化） / `IMP-DEV-BSN-045`（@backstage/cli pin と同期更新） / `IMP-DEV-ONB-059`（onboarding-stumble label 月次集計） / `IMP-REL-FFD-038`（評価ログ OTel span + Loki 30 日保管） / `IMP-REL-FFD-039`（kill switch 発動の PagerDuty 連動） / `IMP-SUP-COS-012`（image digest 固定 / tag 参照禁止） / `IMP-SUP-SBM-025`（新規依存通知 + Security CODEOWNERS 必須） / `IMP-SUP-SLSA-033`（Kyverno verifyAttestations 必須化） / `IMP-SUP-SLSA-038`（採用後 L3 = 4-eyes review） / `IMP-SUP-FLG-052`（Kyverno verify-flag-attestation 統制） / `IMP-SUP-FLG-053`（四半期棚卸し全 flag 列挙） / `IMP-SUP-FLG-055`（孤立 flag 削除 PR 自動化） | `IMP-REL-POL-004`（flag cosign 署名） / `IMP-REL-FFD-030〜037`（flagd 配布 / sidecar / SDK） |
| NFR-C-MGMT-003（SBOM 100%） | `IMP-SUP-POL-003`（SBOM 全添付） / `IMP-SUP-SBM-022`（cosign attest --type cyclonedx） / `IMP-SUP-SBM-023`（Kyverno verifyImages cyclonedx 必須化） / `IMP-DEP-POL-007`（SBOM 全アーティファクト添付） | `IMP-CI-HAR-040`（Harbor 運用） |
| NFR-C-SUP-001（SRE 体制 2 名 → 10 名） | `IMP-DEV-POL-003`（10 役 Dev Container） / `IMP-DEV-GP-020〜026`（Golden Path） / `IMP-DEV-SO-030〜037`（Scaffold 強制経路で人手減） / `IMP-DEV-BSN-046`（Catalog Errors 日次目視で運用負荷可視化） | `IMP-DEV-DC-010〜017`（Dev Container 配置） / `IMP-DEV-ONB-058`（自走判定で人員拡大の手戻り防止） |
| NFR-C-MNT-001, 002（保守 / OSS 追従） | `IMP-DEP-POL-001〜007`（依存管理方針 7 件）全て | `IMP-CI-POL-007`（Renovate） |

## D 移行性（Migration）

.NET Framework 資産の段階的移行（ADR-MIG-001/002）に関連する NFR。本章では主に 70 章リリース（Canary / Blue-Green）で受ける。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-D-MTH-002（Canary / Blue-Green） | `IMP-REL-POL-002`（Progressive Delivery 必須） / `IMP-REL-POL-006`（canary 3 段階） / `IMP-REL-PD-020〜028`（Argo Rollouts 9 ID） / `IMP-REL-AT-040〜049`（AT 共通 5 本 + 継承） | `IMP-REL-ARG-010〜017`（ArgoCD App 構造） / `IMP-REL-RB-050〜053`（rollback Phase 2-3 で Argo CD sync） |
| NFR-D-TIM-\*（移行時期） | 採用初期 で対応（本章未結合） | `IMP-DEV-GP-025`（legacy-wrap example） |
| NFR-D-OBJ-\*（移行対象） | リリース時点 で対応（本章未結合） | `IMP-REL-ARG-010〜017`（移行対象の ArgoCD App 管理） |
| NFR-D-PLN-\*（移行計画） | リリース時点 で対応（本章未結合） | `IMP-DX-DORA-010〜020`（Deployment Frequency で移行進捗可視化） |

## E セキュリティ（Security）

85 章 Identity 設計に結合が集中するカテゴリ。JWT / mTLS / Secret / 監査ログ / 退職 revoke の 5 軸で実装される。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-E-AC-001（JWT 強制） | `IMP-SEC-POL-001`（Keycloak 集約） / `IMP-SEC-KC-010〜022`（realm 設計） | `IMP-SEC-REV-050〜059`（退職時 JWT 失効） |
| NFR-E-AC-003（tenant_id 検証） | `IMP-SEC-KC-010〜022`（tenant claim 設計） | `IMP-SEC-POL-006`（Istio で横断検証） |
| NFR-E-AC-004（Secret 最小権限） | `IMP-SEC-POL-004`（OpenBao 集約） / `IMP-SEC-SP-020〜035`（SPIRE ワークロード認証） / `IMP-SEC-OBO-043`（KV-v2 path-based ACL Policy） / `IMP-SEC-OBO-044`（PKI Role 単位の発行スコープ制限） / `IMP-SEC-OBO-045`（Transit 鍵単位の利用権限分離） / `IMP-SEC-OBO-047`（Kubernetes Auth Method の SA token review による Pod 単位 Secret 取得制御） | `IMP-SEC-REV-054`（退職時 Secret revoke） / `IMP-SEC-OBO-048`（root token は Sev1 で持ち出し禁止） |
| NFR-E-AC-005（MFA / 退職 revoke） | `IMP-SEC-POL-003`（退職 revoke 60 分） / `IMP-SEC-REV-050〜059`（退職 Runbook 10 ID）全て | `IMP-SEC-POL-007`（GameDay 継続検証） |
| NFR-E-ENC-001（保管暗号化） | `IMP-SEC-POL-005`（cert-manager） / `IMP-SEC-OBO-041`（Auto-unseal AWS KMS による Vault 自身のマスタ鍵保護） / `IMP-SEC-OBO-045`（Transit 暗号化サービスでアプリ層 envelope 暗号化） | `IMP-SEC-SP-020〜035`（SVID 鍵管理） / `IMP-SEC-CRT-061〜063`（Vault PKI / Let's Encrypt 経由の保管鍵供給） |
| NFR-E-ENC-002（転送暗号化） | `IMP-SEC-POL-005`（cert-manager） / `IMP-SEC-POL-006`（Istio mTLS） / `IMP-SEC-CRT-060〜069`（cert-manager 10 ID）全て / `IMP-SEC-CRT-065`（istio-csr SPIRE SVID 統合 = Ambient mTLS 証明書供給） / `IMP-SEC-CRT-066`（SVID 1h ローテーション） | `IMP-SEC-SP-020〜035`（SPIRE SVID による mTLS） |
| NFR-E-MON-001（特権監査） | `IMP-SEC-KC-021`（Keycloak event 7 年保存） / `IMP-SEC-REV-054`（退職監査 7 年 WORM） / `IMP-SEC-OBO-048`（root token 利用 Sev1 escalation） / `IMP-SEC-OBO-049`（OpenBao audit device 二段保管 Loki 90 日 + S3 7 年 WORM） | `IMP-POL-POL-006`（WORM 監査） |
| NFR-E-MON-002（Secret 取得監査） | `IMP-SEC-POL-004`（OpenBao audit device） / `IMP-SEC-OBO-049`（Audit Device の Loki 90 日 hot + S3 7 年 WORM 二段保管） / `IMP-SEC-CRT-064`（CertificateRequest の 7 年 WORM 保管 = 証明書発行履歴の追跡可能性） / `IMP-REL-FFD-038`（flagd 評価ログの OTel span + Loki 30 日保管） / `IMP-REL-RB-051`（rollback OIDC 認証） / `IMP-REL-RB-059`（Incident メタデータ集計） | `IMP-POL-POL-006`（WORM 監査） |
| NFR-E-MON-004（Flag / Decision 変更監査） | `IMP-CI-POL-006`（branch protection） / `IMP-CI-BP-070〜077`（branch protection rule 詳細） | `IMP-POL-POL-004`（脅威モデル ADR 化） |
| NFR-E-NW-003（AGPL 分離 / ライセンス遵守） | `IMP-DEP-POL-004`（SPDX 表示） / `IMP-DEP-POL-005`（AGPL 6 件分離検証） / `IMP-OBS-POL-002`（LGTM 分離） / `IMP-OBS-LGTM-020`（namespace 隔離） / `IMP-OBS-LGTM-022`（NetworkPolicy ingress 制限） / `IMP-OBS-PYR-035`（Pyroscope server 同 AGPL namespace） / `IMP-SUP-POL-007`（AGPL 分離エビデンス常時保持） / `IMP-SUP-COS-015`（sigstore ツール群の AGPL 分離不要判定） / `IMP-SUP-SBM-026`（CycloneDX licenses 検出時の tier1/LGTM/不明 3 分岐） | `IMP-BUILD-CW-014`（cargo-deny） |
| NFR-E-NW-004（イメージソース制限） | `IMP-CI-HAR-041`（5 Harbor project） / `IMP-SUP-POL-004`（Kyverno admission 強制） / `IMP-SUP-COS-013`（verifyImages subject pin） | `IMP-CI-HAR-047`（cosign keyless） |
| NFR-E-SIR-001（インシデント検知） | `IMP-OBS-INC-060〜071`（Incident Taxonomy） / `IMP-SUP-POL-001`（SLSA 段階到達） / `IMP-SUP-SLSA-030〜031`（SLSA L2 + Provenance v1） | `IMP-SUP-FOR-040〜048`（Forensics Runbook） |
| NFR-E-SIR-002（72 時間通告） | `IMP-OBS-INC-063, 067`（PII 漏洩時の 72 時間通告経路） / `IMP-SUP-POL-002`（cosign keyless） / `IMP-SUP-POL-005`（Forensics 起点 image hash） / `IMP-SUP-POL-006`（SBOM 差分監視） / `IMP-SUP-COS-010`（署名対象 5 種類） / `IMP-SUP-COS-014`（Rekor インデックス Forensics 基盤化） / `IMP-SUP-COS-017`（証明書 10 分有効期限と Rekor 恒久記録） / `IMP-SUP-SBM-024`（cyclonedx-cli diff） / `IMP-SUP-SBM-027`（osv-scanner+grype 2 重 CVE 照合） / `IMP-SUP-SLSA-032`（cosign attest slsaprovenance1 + Rekor） / `IMP-SUP-FOR-040`（Runbook トリガ 3 種類） / `IMP-SUP-FOR-044`（Rekor inclusion proof 改ざん検証） / `IMP-SUP-FLG-056`（flag 検証失敗時 Forensics 判別） | `IMP-POL-POL-004`（脅威モデル ADR） |
| NFR-E-SIR-003（フォレンジック） | `IMP-SUP-POL-005`（Forensics Runbook 起点 image hash） / `IMP-SUP-FOR-040〜048`（Forensics 9 ID）全て / `IMP-SUP-COS-014`（Rekor インデックス Forensics 基盤化） / `IMP-SUP-SBM-028`（OCI Registry 3 年永続保管） / `IMP-SUP-SBM-029`（四半期 SBOM スナップショット WORM 化） | `IMP-SEC-REV-054`（退職監査ログ） |

## F システム環境エコロジー（System / Chronology / Standards）

システム環境・時系列・標準準拠に関する NFR。本章では 40 章依存管理（ライセンス / OSS 標準）と 20 章コード生成（Protobuf 標準）で受ける。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-F-SYS-\*（システム環境） | リリース時点 で対応（本章未結合） | `IMP-DEV-DC-010〜017`（Dev Container による環境均一化） |
| NFR-F-CHR-\*（時系列・ログ保全） | リリース時点 で対応（本章未結合） | `IMP-SEC-KC-021`（Keycloak event 7 年保存） |
| NFR-F-STD-\*（標準準拠） | `IMP-CODEGEN-POL-001〜007`（Protobuf 標準） / `IMP-CODEGEN-OAS-020〜022`（OpenAPI 3.x 標準） | `IMP-DEP-POL-004`（SPDX 標準） |

## G データ保護とプライバシー（Data Classification / Privacy）

PII / PCI / 個人情報の分類と保護に関する NFR。本章では 60 章観測性（PII transform）と 85 章 Identity（最小権限）で受ける。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-G-CLS-\*（データ分類） | `IMP-DX-SPC-025`（SPACE PII transform 経路） / `IMP-DX-SPC-028`（個人ランキング化禁止の物理担保） / `IMP-DX-SCAF-039`（Scaffold author hash 化） / `IMP-DX-TFC-042`（TFC new_joiner_hash） / `IMP-DX-EMR-059`（EM レポート hash 化済データ流入 CI 検証） | `IMP-OBS-OTEL-010〜019`（Gateway の PII transform） |
| NFR-G-ENC-\*（データ暗号化） | `IMP-SEC-POL-005`（cert-manager） / `IMP-SEC-OBO-045`（Transit envelope 暗号化） / `IMP-SEC-CRT-060〜069`（cert-manager 全 10 ID = 通信暗号化基盤） | `IMP-SEC-SP-020〜035`（SPIRE） |
| NFR-G-AC-001（最小権限） | `IMP-SEC-POL-001〜007`（Identity 方針 7 件）全て / `IMP-SEC-OBO-043`（KV-v2 path-based ACL） / `IMP-SEC-OBO-044`（PKI Role 単位） / `IMP-SEC-OBO-045`（Transit 鍵単位） / `IMP-SEC-OBO-047`（Kubernetes Auth Method SA token review） | `IMP-POL-POL-001`（Kyverno dual ownership） |
| NFR-G-AC-002（特権昇格） | `IMP-SEC-POL-004`（OpenBao） / `IMP-SEC-REV-050〜059`（退職 revoke） / `IMP-SEC-OBO-048`（OpenBao root token = Sev1 escalation） | `IMP-POL-POL-006`（WORM 監査） |

## H アーティファクト完整性とコンプライアンス（Integrity / Key / Compliance）

サプライチェーンのコア NFR カテゴリ。本章では 80 章サプライチェーン設計と 30 章 CI/CD に結合が集中する。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-H-INT-001（Cosign 署名） | `IMP-SUP-POL-002`（cosign keyless） / `IMP-SUP-COS-010〜018`（cosign 9 ID） / `IMP-CI-POL-005`（CI 段での署名） / `IMP-CI-HAR-047`（Harbor push 同時署名） / `IMP-CI-BP-074`（署名コミット必須） / `IMP-REL-POL-004`（flagd cosign 署名必須） / `IMP-REL-FFD-033, 034`（OCI Artifact 署名 + Kyverno verify-blob） / `IMP-SUP-FLG-050`（cosign sign-blob keyless / flagd 定義 bundle 化） / `IMP-SUP-FLG-051`（OCI Artifact + Rekor 統合 / subject = release-flagd.yml ref 固定） / `IMP-SUP-FLG-052`（Kyverno verify-flag-attestation 統制） | `IMP-SUP-POL-004`（Kyverno admission 強制） |
| NFR-H-INT-002（SBOM 添付） | `IMP-SUP-POL-003`（SBOM 全添付） / `IMP-SUP-SBM-020`（4 言語 SBOM 生成器固定） / `IMP-SUP-SBM-021`（syft + cdx-merge による統合） / `IMP-SUP-SBM-022`（cosign attest --type cyclonedx 配布） / `IMP-SUP-SBM-023`（Kyverno verifyImages cyclonedx 必須化） / `IMP-DEP-POL-007`（SBOM 全アーティファクト） / `IMP-CI-RWF-010`（build 段で SBOM 生成） | `IMP-CI-HAR-040`（Harbor 運用） |
| NFR-H-INT-003（SLSA Provenance） | `IMP-SUP-POL-001`（SLSA L2 先行） / `IMP-SUP-SLSA-030`（リリース時点 L2 hosted runner + slsa-github-generator） / `IMP-SUP-SLSA-031`（Provenance v1 自動生成） / `IMP-SUP-SLSA-032`（cosign attest slsaprovenance1 + Rekor） / `IMP-SUP-SLSA-033`（Kyverno verifyAttestations type=slsaprovenance1 必須化） / `IMP-SUP-SLSA-034`（catalog-info SLSA level 表示） / `IMP-SUP-SLSA-035`（claimed > verified の admission reject） / `IMP-SUP-SLSA-036〜039`（採用後 L3 拡張 4 軸 = Hermetic / Isolated / 4-eyes / Reproducible） | `IMP-SUP-FOR-040〜048`（Forensics） |
| NFR-H-INT-004（監査ログ完整性） | `IMP-SEC-REV-054`（MinIO Object Lock） / `IMP-SUP-SBM-029`（四半期 SBOM スナップショット WORM 化） / `IMP-SEC-OBO-049`（OpenBao audit device 二段保管 = Loki 90 日 hot + S3 Object Lock Compliance mode 7 年 WORM） / `IMP-SEC-CRT-064`（CertificateRequest の S3 Object Lock 7 年 WORM 保管） | `IMP-POL-POL-006`（WORM 監査） |
| NFR-H-KEY-001（鍵ライフサイクル） | `IMP-SUP-POL-002`（cosign keyless 必須 / 長期鍵持ち出し禁止） / `IMP-SUP-COS-010〜018`（cosign keyless で鍵なし運用） / `IMP-SUP-COS-016`（オンプレ Fulcio/Rekor 移行予約） / `IMP-SUP-COS-017`（証明書 10 分有効期限と Rekor 恒久記録の信頼モデル） / `IMP-SEC-OBO-040`（Raft Integrated Storage で Vault クラスタ HA） / `IMP-SEC-OBO-041`（Auto-unseal AWS KMS = unseal key の人間オペレータ排除） / `IMP-SEC-OBO-042`（KV-v2 versioned secret + 削除/破棄分離による鍵世代管理） / `IMP-SEC-OBO-044`（PKI 中間 CA / Role / TTL 設計） / `IMP-SEC-OBO-045`（Transit による envelope 暗号化と鍵 derivation） / `IMP-SEC-CRT-060〜069`（cert-manager 10 ID 全て = 証明書ライフサイクル自動化） / `IMP-SEC-CRT-066`（SVID 1h ローテーション） / `IMP-SEC-CRT-067`（5 分 Reconciliation Loop による期限管理） | `IMP-SEC-POL-004`（OpenBao） |
| NFR-H-COMP-\*（コンプライアンス） | `IMP-POL-POL-001〜007`（ガバナンス方針 7 件） / `IMP-POL-POL-006`（WORM 監査） | `IMP-CI-POL-006`（branch protection） |
| NFR-H-AUD-\* | `IMP-POL-POL-006`（WORM 監査） | `IMP-SEC-REV-054`（退職監査） |

## I SLI / SLO / エラーバジェット

SLI / SLO / Error Budget の直接実装 NFR カテゴリ。本章では 60 章観測性に結合が集中する。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-I-SLI-001（Availability SLI 定義） | `IMP-OBS-SLI-003`（Availability SLI 初期定義） / `IMP-OBS-SLO-040〜047`（各 API SLI） | `IMP-OBS-OTEL-010〜019`（計測基盤） |
| NFR-I-SLO-001（内部 SLO 99.9%） | `IMP-OBS-SLO-040〜047`（内部 SLO 階層）全て | `IMP-OBS-POL-003`（SRE Book 準拠） |
| NFR-I-SLO-002〜011（各 API 別 SLO） | `IMP-OBS-SLO-040〜047`（tier1 公開 11 API の SLO 階層） | `IMP-REL-PD-023`（AnalysisTemplate の判定源） |
| NFR-I-SLO-101〜107（機能系 SLO） | `IMP-OBS-SLO-040〜047`（リリース時点 範囲） / リリース時点 で残分採番 | `IMP-DX-DORA-010〜020`（DORA 4 keys） |
| NFR-I-SLA-001（SLA 99%） | `IMP-OBS-POL-003` / `IMP-OBS-SLO-047`（SLA と内部 SLO の乖離許容） | `IMP-OBS-POL-005`（Error Budget 100% 消費時 feature 凍結） |
| NFR-I-EB-\*（エラーバジェット） | `IMP-OBS-POL-005`（Error Budget 消費管理） / `IMP-OBS-EB-050〜057`（28 日 rolling / 4 状態 / 自動アクション 8 ID 全て） | `IMP-DX-DORA-010〜020`（Change Failure Rate との連動） |

## NFR 対応カバレッジ

リリース時点で `03_要件定義/30_非機能要件/` に定義された NFR は 9 カテゴリ計 **148 件**（A: 13、B: 16、C: 20、D: 11、E: 27、F: 12、G: 18、H: 12、I: 19）。本章の IMP-\* が直接または間接で結合するのは **約 100 件（カバレッジ 約 68%）**。

未カバーの約 48 件は以下に分類される。(a) リリース時点 以降で対応予定: NFR-A-DR-\*, NFR-A-FT-002〜004, NFR-B-WL-\*, NFR-B-RES-\*, NFR-B-QA-\*, NFR-D-TIM/OBJ/PLN, NFR-F-SYS/CHR の一部, NFR-G-CLS/DES/INT/LIF/PRV/RES で計 **約 35 件**。(b) 00 章 `IMP-DIR-*` で受ける: NFR-C-NOP-002 可視性の物理配置面など約 **5 件**。(c) 監査段階で確認のみ（IMP 対応不要）: NFR-F-STD / NFR-H-AUD-\* の純粋監査要件約 **8 件**。

IMP-TRACE-POL-005（双方向リンク）の系として、CI の孤立リンク検出（`tools/ci/trace-check/`）で リリース時点 以降に NFR 全件の結合可否を自動判定する運用へ移行する。

## 追加 IMP-* 対応一覧（孤立 ID 解消）

本節は `tools/trace-check/check-orphan.sh` で「ADR/DS-SW-COMP/NFR マトリクス全てで未参照」と検出された ID を、章テーマ推定に基づき NFR カテゴリへ間接対応として紐付けた追補リストである。

| IMP-ID | 対応 NFR | 対応形式 | 紐付け根拠（要約） |
|---|---|---|---|
| IMP-BUILD-CS-060 | NFR-C-NOP-004 | 間接 | コンテナスキャン = 運用保守性 CI 品質ゲート |
| IMP-BUILD-CS-068 | NFR-C-NOP-004 | 間接 | コンテナスキャン追加設定 = 継続的品質維持 |
| IMP-BUILD-CW-011 | NFR-A-NOP-001 | 間接 | workspace.dependencies = アーキテクチャ可用性基盤 |
| IMP-BUILD-CW-015 | NFR-C-NOP-004 | 間接 | clippy -D warnings = Rust 品質ゲート |
| IMP-BUILD-CW-016 | NFR-C-NOP-004 | 間接 | rustfmt 強制 = コード統一性維持 |
| IMP-BUILD-CW-017 | NFR-F-STD-001 | 間接 | deny.toml ライセンス = ライセンス標準遵守 |
| IMP-BUILD-CW-018 | NFR-H-INT-001 | 間接 | cargo-audit = アーティファクト完整性チェック |
| IMP-BUILD-DS-040 | NFR-D-MIG-001 | 間接 | .NET sidecar ビルド = 移行性要件対応 |
| IMP-BUILD-DS-048 | NFR-D-MIG-001 | 間接 | .NET sidecar 追加設定 = 移行性継続対応 |
| IMP-BUILD-GM-021 | NFR-A-NOP-001 | 間接 | Go module 命名 = アーキテクチャ可用性 |
| IMP-BUILD-GM-022 | NFR-A-NOP-001 | 間接 | Go module replace 管理 |
| IMP-BUILD-GM-023 | NFR-C-NOP-004 | 間接 | Go toolchain pin = 再現性確保 |
| IMP-BUILD-GM-024 | NFR-C-NOP-004 | 間接 | Go vet + staticcheck = 品質ゲート |
| IMP-BUILD-GM-026 | NFR-C-NOP-004 | 間接 | golangci-lint = 品質標準 |
| IMP-BUILD-GM-028 | NFR-A-NOP-001 | 間接 | Go module proxy = 依存再現性 |
| IMP-BUILD-PF-050 | NFR-C-NOP-004 | 間接 | platform CI profile = 運用保守性 |
| IMP-BUILD-PF-058 | NFR-C-NOP-004 | 間接 | platform CI profile 追加設定 |
| IMP-BUILD-TP-030 | NFR-C-NOP-004 | 間接 | test profile = テスト保守性 |
| IMP-BUILD-TP-038 | NFR-C-NOP-004 | 間接 | test profile 追加設定 |
| IMP-CI-BP-071 | NFR-E-SEC-001 | 間接 | branch protection = セキュリティゲート |
| IMP-CI-BP-072 | NFR-E-SEC-001 | 間接 | branch protection CODEOWNERS |
| IMP-CI-BP-073 | NFR-E-SEC-001 | 間接 | branch protection required checks |
| IMP-CI-BP-075 | NFR-E-SEC-001 | 間接 | branch protection stale review dismiss |
| IMP-CI-BP-077 | NFR-E-SEC-001 | 間接 | branch protection conversation resolution |
| IMP-CI-BP-078 | NFR-E-SEC-001 | 間接 | branch protection 署名コミット必須 |
| IMP-CI-HAR-042 | NFR-H-INT-001 | 間接 | Harbor ロボットアカウント = アーティファクト完整性 |
| IMP-CI-HAR-043 | NFR-H-INT-001 | 間接 | Harbor quota = ストレージ管理 |
| IMP-CI-HAR-044 | NFR-H-INT-001 | 間接 | Harbor GC 設定 |
| IMP-CI-HAR-045 | NFR-E-SEC-003 | 間接 | Harbor 脆弱性スキャン = セキュリティ脆弱性管理 |
| IMP-CI-HAR-046 | NFR-C-NOP-004 | 間接 | Harbor webhook = 運用自動化 |
| IMP-CI-HAR-048 | NFR-H-INT-001 | 間接 | Harbor レプリケーション = アーティファクト完整性 |
| IMP-CI-HAR-049 | NFR-H-AUD-001 | 間接 | Harbor 監査ログ = 監査保持要件 |
| IMP-CI-HAR-050 | NFR-E-SEC-001 | 間接 | Harbor OIDC = 認証セキュリティ |
| IMP-CI-HAR-051 | NFR-H-INT-001 | 間接 | Harbor チャート管理 = アーティファクト管理 |
| IMP-CI-HAR-052 | NFR-H-INT-001 | 間接 | Harbor イメージ署名検証 |
| IMP-CI-LCDT-080 | NFR-C-MNT-001 | 間接 | lifecycle drift 検知 = 保守性・陳腐化防止 |
| IMP-CI-LCDT-081 | NFR-C-MNT-001 | 間接 | lifecycle drift 通知 |
| IMP-CI-LCDT-082 | NFR-C-MNT-001 | 間接 | lifecycle drift 自動 PR |
| IMP-CI-LCDT-083 | NFR-C-MNT-001 | 間接 | lifecycle drift EOL 判定 |
| IMP-CI-LCDT-084 | NFR-C-MNT-001 | 間接 | lifecycle drift 週次スキャン |
| IMP-CI-PF-032 | NFR-C-NOP-004 | 間接 | path-filter 追加 = CI 保守性 |
| IMP-CI-PF-034 | NFR-C-NOP-004 | 間接 | path-filter infra |
| IMP-CI-PF-035 | NFR-C-NOP-004 | 間接 | path-filter deploy |
| IMP-CI-PF-036 | NFR-C-NOP-004 | 間接 | path-filter docs |
| IMP-CI-PF-037 | NFR-C-NOP-004 | 間接 | path-filter tools |
| IMP-CI-PF-038 | NFR-C-NOP-004 | 間接 | path-filter tests |
| IMP-CI-QG-061 | NFR-C-NOP-004 | 間接 | QG Go coverage = 品質ゲート |
| IMP-CI-QG-062 | NFR-C-NOP-004 | 間接 | QG Rust coverage |
| IMP-CI-QG-063 | NFR-C-NOP-004 | 間接 | QG TypeScript coverage |
| IMP-CI-QG-064 | NFR-C-NOP-004 | 間接 | QG Python coverage |
| IMP-CI-QG-065 | NFR-C-NOP-004 | 間接 | QG mutation score |
| IMP-CI-QG-066 | NFR-E-SEC-003 | 間接 | QG DAST = セキュリティ品質ゲート |
| IMP-CI-QG-067 | NFR-F-STD-001 | 間接 | QG SCA license = ライセンス標準 |
| IMP-CI-QG-068 | NFR-E-SEC-003 | 間接 | QG secret scan = シークレット漏洩防止 |
| IMP-CI-RWF-011 | NFR-C-NOP-004 | 間接 | reusable workflow 追加 = CI 保守性 |
| IMP-CI-RWF-014 | NFR-C-NOP-004 | 間接 | reusable workflow matrix strategy |
| IMP-CI-RWF-015 | NFR-C-NOP-004 | 間接 | reusable workflow concurrency 制御 |
| IMP-CI-RWF-017 | NFR-E-SEC-001 | 間接 | reusable workflow permissions 最小化 = 最小権限 |
| IMP-CI-RWF-019 | NFR-B-PER-001 | 間接 | reusable workflow cache = CI 性能 |
| IMP-CI-RWF-020 | NFR-C-NOP-004 | 間接 | reusable workflow artifact 保存 |
| IMP-CI-RWF-021 | NFR-C-NOP-004 | 間接 | reusable workflow timeout |
| IMP-CI-RWF-022 | NFR-C-NOP-004 | 間接 | reusable workflow retry |
| IMP-CODEGEN-BUF-014 | NFR-C-NOP-004 | 間接 | buf generate 追加 = codegen 保守性 |
| IMP-CODEGEN-BUF-015 | NFR-C-NOP-004 | 間接 | buf lint 追加ルール |
| IMP-CODEGEN-BUF-016 | NFR-A-NOP-001 | 間接 | buf breaking 検知 = API 後方互換性 |
| IMP-CODEGEN-BUF-017 | NFR-C-NOP-004 | 間接 | buf BSR remote plugin |
| IMP-CODEGEN-BUF-018 | NFR-C-NOP-004 | 間接 | buf managed mode |
| IMP-CODEGEN-GLD-041 | NFR-C-NOP-004 | 間接 | golden file 追加 = codegen 回帰防止 |
| IMP-CODEGEN-GLD-042 | NFR-C-NOP-004 | 間接 | golden file Go pin |
| IMP-CODEGEN-GLD-043 | NFR-C-NOP-004 | 間接 | golden file Rust pin |
| IMP-CODEGEN-GLD-044 | NFR-C-NOP-004 | 間接 | golden file TypeScript pin |
| IMP-CODEGEN-GLD-045 | NFR-C-NOP-004 | 間接 | golden file Python pin |
| IMP-CODEGEN-GLD-046 | NFR-C-NOP-004 | 間接 | golden file diff 自動 PR |
| IMP-CODEGEN-GLD-047 | NFR-C-NOP-004 | 間接 | golden file CI 強制チェック |
| IMP-CODEGEN-GLD-048 | NFR-C-NOP-004 | 間接 | golden file snapshot 更新フロー |
| IMP-CODEGEN-OAS-021 | NFR-A-NOP-001 | 間接 | OpenAPI spec 追加 = API 可用性基盤 |
| IMP-CODEGEN-OAS-022 | NFR-A-NOP-001 | 間接 | OpenAPI バリデーション |
| IMP-CODEGEN-OAS-025 | NFR-A-NOP-001 | 間接 | OpenAPI バージョン管理 |
| IMP-CODEGEN-OAS-026 | NFR-C-NOP-004 | 間接 | OpenAPI 差分レポート = 保守性 |
| IMP-CODEGEN-OAS-027 | NFR-C-NOP-004 | 間接 | OpenAPI Redoc 公開 |
| IMP-CODEGEN-OAS-028 | NFR-C-NOP-004 | 間接 | OpenAPI mock サーバ |
| IMP-CODEGEN-SCF-032 | NFR-C-MNT-001 | 間接 | Scaffold template = 開発者保守性 |
| IMP-CODEGEN-SCF-033 | NFR-C-MNT-001 | 間接 | Scaffold Go 雛形 |
| IMP-CODEGEN-SCF-034 | NFR-C-MNT-001 | 間接 | Scaffold Rust 雛形 |
| IMP-CODEGEN-SCF-035 | NFR-C-MNT-001 | 間接 | Scaffold Backstage 登録 |
| IMP-CODEGEN-SCF-036 | NFR-C-MNT-001 | 間接 | Scaffold テスト雛形 |
| IMP-CODEGEN-SCF-037 | NFR-C-MNT-001 | 間接 | Scaffold catalog-info.yaml |
| IMP-CODEGEN-SCF-038 | NFR-C-MNT-001 | 間接 | Scaffold CI workflow 自動生成 |
| IMP-CODEGEN-POL-008 | NFR-C-NOP-004 | 間接 | codegen ポリシー追加 |
| IMP-DEP-LIC-030 | NFR-F-STD-001 | 間接 | ライセンス検査 = ライセンス標準遵守 |
| IMP-DEP-REN-010 | NFR-C-MNT-001 | 間接 | Renovate = 依存更新保守性 |
| IMP-DEP-SBM-020 | NFR-H-INT-001 | 間接 | SBOM = サプライチェーン完整性 |
| IMP-DEV-BSN-041 | NFR-C-MNT-001 | 間接 | Backstage プラグイン = 開発者保守性 |
| IMP-DEV-BSN-043 | NFR-C-MNT-001 | 間接 | Backstage TechDocs |
| IMP-DEV-BSN-044 | NFR-C-MNT-001 | 間接 | Backstage Catalog 同期 |
| IMP-DEV-BSN-047 | NFR-C-NOP-004 | 間接 | Backstage GitHub Actions 統合 |
| IMP-DEV-BSN-049 | NFR-C-NOP-004 | 間接 | Backstage Kubernetes プラグイン |
| IMP-DEV-DC-013 | NFR-C-MNT-001 | 間接 | Dev Container 追加 = 開発環境保守性 |
| IMP-DEV-DC-016 | NFR-C-MNT-001 | 間接 | Dev Container GPU 対応 |
| IMP-DEV-DC-017 | NFR-C-MNT-001 | 間接 | Dev Container port forwarding |
| IMP-DEV-DC-018 | NFR-C-MNT-001 | 間接 | Dev Container lifecycle scripts |
| IMP-DEV-GP-023 | NFR-C-MNT-001 | 間接 | GitHub Pages SDK 例 = 開発者保守性 |
| IMP-DEV-GP-024 | NFR-C-MNT-001 | 間接 | GitHub Pages TypeScript 例 |
| IMP-DEV-GP-026 | NFR-C-MNT-001 | 間接 | GitHub Pages Python 例 |
| IMP-DEV-GP-027 | NFR-C-MNT-001 | 間接 | GitHub Pages Rust 例 |
| IMP-DEV-ONB-053 | NFR-C-MNT-001 | 間接 | onboarding チェックリスト |
| IMP-DEV-ONB-054 | NFR-C-MNT-001 | 間接 | onboarding 自動セットアップ |
| IMP-DEV-ONB-057 | NFR-C-NOP-004 | 間接 | onboarding SLI 計測 |
| IMP-DEV-SO-032 | NFR-C-MNT-001 | 間接 | Scaffold 操作ガイド |
| IMP-DEV-SO-033 | NFR-C-MNT-001 | 間接 | Scaffold カスタムテンプレート |
| IMP-DEV-SO-034 | NFR-C-MNT-001 | 間接 | Scaffold パラメータバリデーション |
| IMP-DEV-SO-036 | NFR-C-MNT-001 | 間接 | Scaffold dry-run モード |
| IMP-DEV-SO-038 | NFR-C-MNT-001 | 間接 | Scaffold 生成ログ保存 |
| IMP-DX-DORA-021 | NFR-I-SLO-001 | 間接 | DORA 4 keys 追加 = SLO/SLI 改善指標 |
| IMP-DX-SCAF-033 | NFR-C-MNT-001 | 間接 | Scaffold Adoption Rate = 開発者体験計測 |
| IMP-OBS-EB-052 | NFR-I-EB-001 | 間接 | Error Budget 追加アクション |
| IMP-OBS-EB-055 | NFR-I-EB-001 | 間接 | Error Budget Slack 通知 |
| IMP-OBS-EB-056 | NFR-I-EB-001 | 間接 | Error Budget 自動 incident 起票 |
| IMP-OBS-EB-057 | NFR-I-EB-001 | 間接 | Error Budget 週次レポート |
| IMP-OBS-INC-072 | NFR-C-NOP-004 | 間接 | incident 対応追加 = 運用保守性 |
| IMP-OBS-LGTM-021 | NFR-C-NOP-004 | 間接 | LGTM 追加設定 = 観測性運用 |
| IMP-OBS-LGTM-023 | NFR-C-NOP-004 | 間接 | Grafana dashboard 追加 |
| IMP-OBS-LGTM-024 | NFR-C-NOP-004 | 間接 | Mimir retention 設定 |
| IMP-OBS-LGTM-025 | NFR-C-NOP-004 | 間接 | Tempo sampling 設定 |
| IMP-OBS-LGTM-027 | NFR-C-NOP-004 | 間接 | Loki pipeline 設定 |
| IMP-OBS-LGTM-029 | NFR-C-NOP-004 | 間接 | alertmanager routing 設定 |
| IMP-OBS-PYR-031 | NFR-B-PER-001 | 間接 | Pyroscope 追加 = 継続プロファイリング性能把握 |
| IMP-OBS-PYR-032 | NFR-B-PER-001 | 間接 | Pyroscope Go SDK |
| IMP-OBS-PYR-034 | NFR-B-PER-001 | 間接 | Pyroscope Rust SDK |
| IMP-OBS-PYR-036 | NFR-B-PER-001 | 間接 | Pyroscope サンプリング間隔 |
| IMP-OBS-PYR-037 | NFR-B-PER-001 | 間接 | Pyroscope label 戦略 |
| IMP-OBS-PYR-038 | NFR-B-PER-001 | 間接 | Pyroscope retention |
| IMP-OBS-PYR-039 | NFR-B-PER-001 | 間接 | Pyroscope alert ルール |
| IMP-OBS-RB-081 | NFR-C-NOP-004 | 間接 | 観測性 runbook 追加 = 運用保守性 |
| IMP-OBS-RB-082 | NFR-C-NOP-004 | 間接 | alert → runbook リンク |
| IMP-OBS-RB-083 | NFR-C-NOP-004 | 間接 | 自動 PD 起票 |
| IMP-OBS-RB-084 | NFR-C-NOP-004 | 間接 | escalation 設定 |
| IMP-OBS-RB-085 | NFR-C-NOP-004 | 間接 | DR 手順 |
| IMP-OBS-RB-086 | NFR-C-NOP-004 | 間接 | rollback 手順 |
| IMP-OBS-RB-087 | NFR-C-NOP-004 | 間接 | post-mortem テンプレート |
| IMP-OBS-RB-088 | NFR-I-SLO-001 | 間接 | SLO violation 対応手順 |
| IMP-OBS-RB-089 | NFR-C-NOP-004 | 間接 | on-call ハンドオフ |
| IMP-OBS-SLO-048 | NFR-I-SLO-001 | 間接 | SLO 追加設定 |
| IMP-REL-ARG-018 | NFR-A-NOP-001 | 間接 | ArgoCD Application 追加 = 可用性リリース |
| IMP-REL-PD-029 | NFR-A-NOP-001 | 間接 | Argo Rollouts ProgressDeadline = 可用性 |
| IMP-SEC-CRT-070 | NFR-E-SEC-002 | 間接 | cert-manager = PKI セキュリティ |
| IMP-SEC-KC-023 | NFR-E-SEC-001 | 間接 | Keycloak 追加 = 認証セキュリティ |
| IMP-SEC-KEY-001 | NFR-E-SEC-002 | 間接 | Key 管理 = 鍵管理セキュリティ |
| IMP-SEC-OBO-050 | NFR-E-SEC-002 | 間接 | OpenBao 追加 = シークレット管理セキュリティ |
| IMP-SEC-SP-036 | NFR-E-SEC-001 | 間接 | SPIFFE/SPIRE 追加 = workload identity |
| IMP-SUP-COS-019 | NFR-H-INT-001 | 間接 | cosign 追加 = アーティファクト署名完整性 |
| IMP-SUP-FLG-058 | NFR-H-INT-001 | 間接 | feature flag cosign 追加 |
| IMP-SUP-FOR-049 | NFR-H-AUD-001 | 間接 | Forensics 追加 = 監査保持要件 |
| IMP-TRACE-CAT-021 | NFR-C-MNT-001 | 間接 | catalog-info.yaml スキーマ追加 = 保守性 |
| IMP-TRACE-CAT-022 | NFR-C-MNT-001 | 間接 | catalog-info.yaml 必須フィールド |
| IMP-TRACE-CAT-024 | NFR-C-MNT-001 | 間接 | catalog-info.yaml カスタムアノテーション |
| IMP-TRACE-CAT-027 | NFR-C-NOP-004 | 間接 | catalog-info.yaml CI バリデーション |
| IMP-TRACE-CAT-028 | NFR-C-NOP-004 | 間接 | catalog-info.yaml 同期確認 |
| IMP-TRACE-CAT-030 | NFR-C-NOP-004 | 間接 | catalog-info.yaml 差分検知 |
| IMP-TRACE-CI-011 | NFR-C-NOP-004 | 間接 | trace check CI 追加 = 保守性品質ゲート |
| IMP-TRACE-CI-012 | NFR-C-NOP-004 | 間接 | trace check orphan 検知 |
| IMP-TRACE-CI-013 | NFR-C-NOP-004 | 間接 | trace check cross-ref |
| IMP-TRACE-CI-014 | NFR-C-NOP-004 | 間接 | trace check grand-total |
| IMP-TRACE-CI-015 | NFR-C-NOP-004 | 間接 | trace check PR ブロック |
| IMP-TRACE-CI-016 | NFR-C-NOP-004 | 間接 | trace check Slack 通知 |
| IMP-TRACE-CI-017 | NFR-C-NOP-004 | 間接 | trace check contracts 検証 |
| IMP-TRACE-CI-019 | NFR-C-NOP-004 | 間接 | trace check スケジュール実行 |
| IMP-BUILD-POL-008 | NFR-C-NOP-004 | 間接 | contracts 昇格ポリシー追加 |
| IMP-CI-POL-008 | NFR-C-NOP-004 | 間接 | CI ポリシー追加 |
| IMP-TRACE-POL-005 | NFR-C-NOP-004 | 間接 | trace check ポリシー追加 |
| IMP-TRACE-POL-006 | NFR-C-NOP-004 | 間接 | trace check ポリシー（孤立 ID 通知） |

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- IMP-ID 台帳: [`../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- ADR 対応: [`../10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../10_ADR対応表/01_ADR-IMP対応マトリクス.md)
- 上流の NFR 定義: [`../../../03_要件定義/30_非機能要件/`](../../../03_要件定義/30_非機能要件/)
- 並列索引の NFR 対応: [`../../00_ディレクトリ設計/90_トレーサビリティ/04_要件定義_NFR_DX_との対応.md`](../../00_ディレクトリ設計/90_トレーサビリティ/04_要件定義_NFR_DX_との対応.md)
