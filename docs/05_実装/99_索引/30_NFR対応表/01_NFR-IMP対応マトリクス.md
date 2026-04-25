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
| NFR-A-CONT-001（SLA 99%） | `IMP-OBS-POL-003` / `IMP-OBS-SLO-040〜047`（内部 SLO 99.9% を 99% SLA の上位に設定） | `IMP-REL-POL-001`（GitOps） / `IMP-REL-PD-020〜028`（Progressive Delivery） |
| NFR-A-FT-001（自動復旧 15 分以内） | `IMP-REL-POL-004`（手動 rollback 15 分以内） / `IMP-REL-PD-023〜025`（AnalysisTemplate / failureLimit） | `IMP-OBS-INC-064`（Sev1 Runbook） |
| NFR-A-REC-002（Runbook 15 本） | `IMP-OBS-INC-064`（AVL/SEC × Sev1/Sev2 の 4 セル 15 Runbook） / `IMP-OBS-SLO-045`（Runbook 一対一対応） | `IMP-SEC-REV-050〜059`（退職 revoke Runbook） |
| NFR-A-DR-\* | 運用蓄積後で対応（本章未結合） | `IMP-REL-ARG-010〜017`（ArgoCD による GitOps DR 基盤） |
| NFR-A-FT-002〜004（多重化 / 障害注入 / 縮退） | リリース時点 で対応（本章未結合） | `IMP-REL-PD-026〜028`（Canary / Blue-Green の多重化経路） |

## B 性能拡張性（Performance / Scalability）

性能系は SLI 計測を前提とし、ビルド時間・ランタイム p99・Scale の 3 層で実装される。本章では 10 章ビルド設計（ビルド時間 SLI）と 60 章観測性（ランタイム SLI）に結合が集中する。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-B-PERF-001（tier1 API p99 < 500ms） | `IMP-OBS-SLO-040〜047`（API 種別 p99 SLO 階層） / `IMP-OBS-POL-003`（SRE Book 準拠 SLO） | `IMP-BUILD-POL-001〜007`（性能基盤としてのビルド方針） / `IMP-OBS-OTEL-010〜019`（サンプリング 100% 維持） |
| NFR-B-PERF-002〜007（State Get / Decision / PubSub / Log / Flag の p99） | `IMP-OBS-SLO-040〜047`（各 API の SLO 細目） | `IMP-REL-PD-023`（AnalysisTemplate の判定源） |
| NFR-B-WL-\*（ワークロード規模） | リリース時点 で対応（本章未結合） | `IMP-OBS-OTEL-010〜019`（規模計測基盤） |
| NFR-B-RES-\*（水平・垂直拡張） | リリース時点 で対応（本章未結合） | `IMP-CI-RWF-013`（runner 自動スケール） |
| NFR-B-QA-\*（性能試験） | リリース時点 で対応（本章未結合） | `IMP-CI-RWF-018`（coverage 閾値段階導入の先に性能試験） |

## C 運用保守性（Operability / Maintainability）

採用側の小規模運用前提の NFR が 12 章全体に伸びる最重要カテゴリ。本章では 10 章ビルド・30 章 CI/CD・40 章依存管理・50 章 DX・90 章ガバナンス全てに結合する。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-C-NOP-001（採用側の小規模運用） | `IMP-DEV-POL-003`（10 役 Dev Container） / `IMP-DEV-POL-004`（time-to-first-commit SLI） | `IMP-BUILD-POL-006`（ビルド時間 SLI） / `IMP-DX-POL-001〜007`（DORA 4 keys） |
| NFR-C-NOP-002（可視性） | `IMP-TRACE-POL-001〜007`（索引 7 原則） / `IMP-OBS-POL-001〜007`（観測性方針） | 全 IMP-\*（可視性は全実装に効く） |
| NFR-C-NOP-004（ビルド所要時間） | `IMP-BUILD-POL-006`（ビルド時間 SLI 計測） / `IMP-CI-POL-002`（quality gate 統制） / `IMP-CI-RWF-016`（cache キー規約） | `IMP-BUILD-CW-013`（sccache） / `IMP-BUILD-GM-025`（GOCACHE リモート） |
| NFR-C-MNT-003（API 互換方針 12 か月） | `IMP-CODEGEN-POL-003`（buf breaking 検知） / `IMP-CODEGEN-BUF-013, 016`（FILE level / v1-v2 分岐） | `IMP-REL-ARG-010〜017`（段階 rollout） |
| NFR-C-IR-001（Severity 別応答） | `IMP-OBS-INC-060〜071`（Incident Taxonomy） / `IMP-SEC-REV-050`（退職 revoke 15 分 SLA） | `IMP-DX-POL-002`（Severity 別 DORA 分離） |
| NFR-C-IR-002（Circuit Breaker） | `IMP-REL-POL-003`（AnalysisTemplate 強制） | `IMP-REL-PD-023〜025`（failureLimit） |
| NFR-C-MGMT-001（設定 Git 管理） | `IMP-REL-POL-001`（GitOps 一本化） / `IMP-CI-POL-007`（Renovate PR） | `IMP-BUILD-POL-007`（生成物 commit） |
| NFR-C-MGMT-002（Flag / Decision 変更監査） | `IMP-POL-POL-004`（脅威モデル ADR 化） / `IMP-POL-POL-006`（WORM 監査） | `IMP-REL-POL-006`（flag 即時切替） |
| NFR-C-MGMT-003（SBOM 100%） | `IMP-SUP-POL-003`（SBOM 全添付） / `IMP-DEP-POL-007`（SBOM 全アーティファクト添付） | `IMP-CI-HAR-040`（Harbor 運用） |
| NFR-C-SUP-001（SRE 体制 2 名 → 10 名） | `IMP-DEV-POL-003`（10 役 Dev Container） / `IMP-DEV-GP-020〜026`（Golden Path） | `IMP-DEV-DC-010〜017`（Dev Container 配置） |
| NFR-C-MNT-001, 002（保守 / OSS 追従） | `IMP-DEP-POL-001〜007`（依存管理方針 7 件）全て | `IMP-CI-POL-007`（Renovate） |

## D 移行性（Migration）

.NET Framework 資産の段階的移行（ADR-MIG-001/002）に関連する NFR。本章では主に 70 章リリース（Canary / Blue-Green）で受ける。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-D-MTH-002（Canary / Blue-Green） | `IMP-REL-POL-002`（Progressive Delivery 必須） / `IMP-REL-PD-020〜028`（Argo Rollouts 9 ID） | `IMP-REL-ARG-010〜017`（ArgoCD App 構造） |
| NFR-D-TIM-\*（移行時期） | 採用初期 で対応（本章未結合） | `IMP-DEV-GP-025`（legacy-wrap example） |
| NFR-D-OBJ-\*（移行対象） | リリース時点 で対応（本章未結合） | `IMP-REL-ARG-010〜017`（移行対象の ArgoCD App 管理） |
| NFR-D-PLN-\*（移行計画） | リリース時点 で対応（本章未結合） | `IMP-DX-DORA-010〜020`（Deployment Frequency で移行進捗可視化） |

## E セキュリティ（Security）

85 章 Identity 設計に結合が集中するカテゴリ。JWT / mTLS / Secret / 監査ログ / 退職 revoke の 5 軸で実装される。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-E-AC-001（JWT 強制） | `IMP-SEC-POL-001`（Keycloak 集約） / `IMP-SEC-KC-010〜022`（realm 設計） | `IMP-SEC-REV-050〜059`（退職時 JWT 失効） |
| NFR-E-AC-003（tenant_id 検証） | `IMP-SEC-KC-010〜022`（tenant claim 設計） | `IMP-SEC-POL-006`（Istio で横断検証） |
| NFR-E-AC-004（Secret 最小権限） | `IMP-SEC-POL-004`（OpenBao 集約） / `IMP-SEC-SP-020〜035`（SPIRE ワークロード認証） | `IMP-SEC-REV-054`（退職時 Secret revoke） |
| NFR-E-AC-005（MFA / 退職 revoke） | `IMP-SEC-POL-003`（退職 revoke 60 分） / `IMP-SEC-REV-050〜059`（退職 Runbook 10 ID）全て | `IMP-SEC-POL-007`（GameDay 継続検証） |
| NFR-E-ENC-001（保管暗号化） | `IMP-SEC-POL-005`（cert-manager） | `IMP-SEC-SP-020〜035`（SVID 鍵管理） |
| NFR-E-ENC-002（転送暗号化） | `IMP-SEC-POL-005`（cert-manager） / `IMP-SEC-POL-006`（Istio mTLS） | `IMP-SEC-SP-020〜035`（SPIRE SVID による mTLS） |
| NFR-E-MON-001（特権監査） | `IMP-SEC-KC-021`（Keycloak event 7 年保存） / `IMP-SEC-REV-054`（退職監査 7 年 WORM） | `IMP-POL-POL-006`（WORM 監査） |
| NFR-E-MON-002（Secret 取得監査） | `IMP-SEC-POL-004`（OpenBao audit device） | `IMP-POL-POL-006`（WORM 監査） |
| NFR-E-MON-004（Flag / Decision 変更監査） | `IMP-CI-POL-006`（branch protection） | `IMP-POL-POL-004`（脅威モデル ADR 化） |
| NFR-E-NW-003（AGPL 分離 / ライセンス遵守） | `IMP-DEP-POL-004`（SPDX 表示） / `IMP-DEP-POL-005`（AGPL 6 件分離検証） / `IMP-OBS-POL-002`（LGTM 分離） | `IMP-BUILD-CW-014`（cargo-deny） |
| NFR-E-NW-004（イメージソース制限） | `IMP-CI-HAR-041`（5 Harbor project） / `IMP-SUP-POL-006`（Kyverno verifyImages） | `IMP-CI-HAR-047`（cosign keyless） |
| NFR-E-SIR-001（インシデント検知） | `IMP-OBS-INC-060〜071`（Incident Taxonomy） | `IMP-SUP-FOR-040〜048`（Forensics Runbook） |
| NFR-E-SIR-002（72 時間通告） | `IMP-OBS-INC-063, 067`（PII 漏洩時の 72 時間通告経路） | `IMP-POL-POL-004`（脅威モデル ADR） |
| NFR-E-SIR-003（フォレンジック） | `IMP-SUP-POL-005`（Forensics Runbook 起点 image hash） / `IMP-SUP-FOR-040〜048`（Forensics 9 ID）全て | `IMP-SEC-REV-054`（退職監査ログ） |

## F システム環境エコロジー（System / Chronology / Standards）

システム環境・時系列・標準準拠に関する NFR。本章では 40 章依存管理（ライセンス / OSS 標準）と 20 章コード生成（Protobuf 標準）で受ける。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-F-SYS-\*（システム環境） | リリース時点 で対応（本章未結合） | `IMP-DEV-DC-010〜017`（Dev Container による環境均一化） |
| NFR-F-CHR-\*（時系列・ログ保全） | リリース時点 で対応（本章未結合） | `IMP-SEC-KC-021`（Keycloak event 7 年保存） |
| NFR-F-STD-\*（標準準拠） | `IMP-CODEGEN-POL-001〜007`（Protobuf 標準） | `IMP-DEP-POL-004`（SPDX 標準） |

## G データ保護とプライバシー（Data Classification / Privacy）

PII / PCI / 個人情報の分類と保護に関する NFR。本章では 60 章観測性（PII transform）と 85 章 Identity（最小権限）で受ける。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-G-CLS-\*（データ分類） | リリース時点 で対応（本章未結合） | `IMP-OBS-OTEL-010〜019`（Gateway の PII transform） |
| NFR-G-ENC-\*（データ暗号化） | `IMP-SEC-POL-005`（cert-manager） | `IMP-SEC-SP-020〜035`（SPIRE） |
| NFR-G-AC-001（最小権限） | `IMP-SEC-POL-001〜007`（Identity 方針 7 件）全て | `IMP-POL-POL-001`（Kyverno dual ownership） |
| NFR-G-AC-002（特権昇格） | `IMP-SEC-POL-004`（OpenBao） / `IMP-SEC-REV-050〜059`（退職 revoke） | `IMP-POL-POL-006`（WORM 監査） |

## H アーティファクト完整性とコンプライアンス（Integrity / Key / Compliance）

サプライチェーンのコア NFR カテゴリ。本章では 80 章サプライチェーン設計と 30 章 CI/CD に結合が集中する。

| NFR | 直接対応 IMP | 間接対応 IMP |
|---|---|---|
| NFR-H-INT-001（Cosign 署名） | `IMP-SUP-POL-002`（cosign keyless） / `IMP-SUP-COS-010〜018`（cosign 9 ID） / `IMP-CI-POL-005`（CI 段での署名） / `IMP-CI-HAR-047`（Harbor push 同時署名） | `IMP-SUP-POL-006`（Kyverno 検証） |
| NFR-H-INT-002（SBOM 添付） | `IMP-SUP-POL-003`（SBOM 全添付） / `IMP-DEP-POL-007`（SBOM 全アーティファクト） / `IMP-CI-RWF-010`（build 段で SBOM 生成） | `IMP-CI-HAR-040`（Harbor 運用） |
| NFR-H-INT-003（SLSA Provenance） | `IMP-SUP-POL-001`（SLSA L2 先行） | `IMP-SUP-FOR-040〜048`（Forensics） |
| NFR-H-INT-004（監査ログ完整性） | `IMP-SEC-REV-054`（MinIO Object Lock） | `IMP-POL-POL-006`（WORM 監査） |
| NFR-H-KEY-001（鍵ライフサイクル） | `IMP-SUP-POL-007`（鍵管理） / `IMP-SEC-KEY-001, 002`（鍵管理初期採番） / `IMP-SUP-COS-010〜018`（cosign keyless で鍵なし運用） | `IMP-SEC-POL-004`（OpenBao） |
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
| NFR-I-EB-\*（エラーバジェット） | `IMP-OBS-POL-005`（Error Budget 消費管理） | `IMP-DX-DORA-010〜020`（Change Failure Rate との連動） |

## NFR 対応カバレッジ

リリース時点で `03_要件定義/30_非機能要件/` に定義された NFR は 9 カテゴリ計 **148 件**（A: 13、B: 16、C: 20、D: 11、E: 27、F: 12、G: 18、H: 12、I: 19）。本章の IMP-\* が直接または間接で結合するのは **約 100 件（カバレッジ 約 68%）**。

未カバーの約 48 件は以下に分類される。(a) リリース時点 以降で対応予定: NFR-A-DR-\*, NFR-A-FT-002〜004, NFR-B-WL-\*, NFR-B-RES-\*, NFR-B-QA-\*, NFR-D-TIM/OBJ/PLN, NFR-F-SYS/CHR の一部, NFR-G-CLS/DES/INT/LIF/PRV/RES で計 **約 35 件**。(b) 00 章 `IMP-DIR-*` で受ける: NFR-C-NOP-002 可視性の物理配置面など約 **5 件**。(c) 監査段階で確認のみ（IMP 対応不要）: NFR-F-STD / NFR-H-AUD-\* の純粋監査要件約 **8 件**。

IMP-TRACE-POL-005（双方向リンク）の系として、CI の孤立リンク検出（`tools/ci/trace-check/`）で リリース時点 以降に NFR 全件の結合可否を自動判定する運用へ移行する。

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- IMP-ID 台帳: [`../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- ADR 対応: [`../10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../10_ADR対応表/01_ADR-IMP対応マトリクス.md)
- 上流の NFR 定義: [`../../../03_要件定義/30_非機能要件/`](../../../03_要件定義/30_非機能要件/)
- 並列索引の NFR 対応: [`../../00_ディレクトリ設計/90_トレーサビリティ/04_要件定義_NFR_DX_との対応.md`](../../00_ディレクトリ設計/90_トレーサビリティ/04_要件定義_NFR_DX_との対応.md)
