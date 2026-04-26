# 01. 対応 IMP-SUP 索引

本ファイルは k1s0 サプライチェーン章（80_サプライチェーン設計）配下の全実装 ID（IMP-SUP-*）を 1 箇所にカタログ化する索引である。各 ID の実装位置・対応 ADR / DS-SW-COMP / NFR を逆引き可能とし、Forensics や監査時に「どの ID がどこに実装され、どの要件を満たすか」を 1 ファイルで把握できる構造を提供する。99_索引（モノレポ全体）との関係は、99_索引が全 12 接頭辞横断、本ファイルは IMP-SUP-* に限定した深掘りカタログ、と分担する。

## サブ接頭辞別の集計

サプライチェーン章は 6 つのサブ接頭辞で構成される。それぞれが独立したセクションを持ち、合計 53 ID を採番する。

| サブ接頭辞 | セクション | ID 範囲 | ID 数 | 主担当責務 |
|------------|------------|---------|-------|-----------|
| POL | 00_方針 | 001〜007 | 7 | サプライチェーン原則の提示 |
| COS | 10_cosign署名 | 010〜018 | 9 | keyless 署名と Kyverno 検証 |
| SBM | 20_CycloneDX_SBOM | 020〜029 | 10 | SBOM 生成・配布・差分監視 |
| SLSA | 30_SLSA_プロビナンス | 030〜039 | 10 | Provenance v1 と Kyverno 強制 |
| FOR | 40_Forensics_Runbook | 040〜048 | 9 | image hash 逆引き Runbook |
| FLG | 50_flag_定義署名検証 | 050〜057 | 8 | flag 定義の真正性と棚卸し |

合計 53 IDで、80 章は実装 IDの数においてもモノレポ最大級のセクションとなる。これは「採用側組織のセキュリティ部門と直接対峙する章」であり、admission 強制・透明性ログ・棚卸し・Forensics・棚卸しレポート保管といった監査統制が幅広く要求されるためである。

## POL（00_方針 / 7 ID）

[00_方針/01_サプライチェーン原則.md](../00_方針/01_サプライチェーン原則.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SUP-POL-001 | SLSA v1.1 段階到達（リリース時点 L2 / 採用後 L3） | ADR-SUP-001 / DS-SW-COMP-141 / NFR-E-SIR-001 |
| IMP-SUP-POL-002 | cosign keyless 署名必須（長期鍵持ち出し禁止） | ADR-SUP-001 / NFR-E-SIR-002 |
| IMP-SUP-POL-003 | CycloneDX SBOM 全 image 添付 | ADR-DEP-001 / NFR-H-INT-002 / NFR-C-MGMT-003 |
| IMP-SUP-POL-004 | Kyverno による admission 強制（warn 禁止） | ADR-CICD-003 / NFR-C-IR-002 |
| IMP-SUP-POL-005 | Forensics Runbook で image hash → tier1 API 逆引き | ADR-OBS-003 / NFR-E-SIR-002 |
| IMP-SUP-POL-006 | SBOM 差分監視と新規依存自動検知 | ADR-DEP-001 / NFR-E-SIR-002 |
| IMP-SUP-POL-007 | AGPL 分離エビデンスの常時保持 | ADR-0003 / NFR-E-NW-003 |

POL は方針層であり、後続の 5 セクション（COS/SBM/SLSA/FOR/FLG）が方針の物理化を担う。POL 単独ではプロセス記述に留まり、admission 拒否や棚卸しの自動化は他セクションで実現する。

## COS（10_cosign署名 / 9 ID）

[10_cosign署名/01_cosign_keyless署名.md](../10_cosign署名/01_cosign_keyless署名.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SUP-COS-010 | 署名対象 5 種類（image/SBOM/Provenance/flagd/Helm）固定 | ADR-CICD-003 / NFR-E-SIR-002 |
| IMP-SUP-COS-011 | GitHub OIDC → Fulcio → Rekor の信頼連鎖 4 段 | ADR-SUP-001 |
| IMP-SUP-COS-012 | image 参照は digest 固定、tag 参照禁止 | NFR-C-MGMT-002 |
| IMP-SUP-COS-013 | Kyverno verifyImages で subject に release.yml ref を固定 | ADR-CICD-003 |
| IMP-SUP-COS-014 | Rekor インデックス検索を Forensics 基盤化 | ADR-OBS-003 |
| IMP-SUP-COS-015 | sigstore ツール群の AGPL 分離不要判定 | ADR-0003 |
| IMP-SUP-COS-016 | リリース時点オンプレ Fulcio / Rekor 移行手順の予約 | ADR-SUP-001 |
| IMP-SUP-COS-017 | 証明書 10 分有効期限と Rekor 恒久記録の信頼モデル | NFR-E-SIR-002 |
| IMP-SUP-COS-018 | 月次 cluster 全 Pod の署名 cross-check 監査 | NFR-A-RTO-001 |

COS-010 は署名対象、COS-013 は admission 検証 subject、COS-018 は遡及検証、と 3 軸で署名統制を成立させる。COS-016 は リリース時点 のオフサイト依存リスクへの予約措置である。

## SBM（20_CycloneDX_SBOM / 10 ID）

[20_CycloneDX_SBOM/01_CycloneDX_SBOM設計.md](../20_CycloneDX_SBOM/01_CycloneDX_SBOM設計.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SUP-SBM-020 | 4 言語 SBOM 生成器固定（cargo-cyclonedx / cyclonedx-gomod / cdxgen / dotnet-CycloneDX） | DS-SW-COMP-135 / NFR-H-INT-002 |
| IMP-SUP-SBM-021 | syft コンテナ階層スキャンと cdx-merge による統合 | DS-SW-COMP-135 |
| IMP-SUP-SBM-022 | `cosign attach sbom` + `cosign attest --type cyclonedx` の 2 段配布 | ADR-CICD-003 / NFR-C-MGMT-003 |
| IMP-SUP-SBM-023 | Kyverno verifyImages による cyclonedx attestation 必須化 | ADR-CICD-003 / NFR-C-MGMT-003 |
| IMP-SUP-SBM-024 | `cyclonedx-cli diff` による前回 release との component 差分検出 | NFR-E-SIR-002 |
| IMP-SUP-SBM-025 | 新規依存通知 + Slack `#k1s0-supply-chain` + Security CODEOWNERS 必須 | ADR-DEP-001 / NFR-C-MGMT-002 |
| IMP-SUP-SBM-026 | AGPL/GPL/SSPL 検出時の判定フロー（tier1/LGTM/不明 3 分岐） | ADR-0003 / NFR-E-NW-003 |
| IMP-SUP-SBM-027 | osv-scanner + grype 2 重 CVE 照合と重複削除 | NFR-E-SIR-002 |
| IMP-SUP-SBM-028 | OCI Registry での 3 年永続保管（GC 後も SBOM 残存） | DS-SW-COMP-141 / NFR-A-RTO-001 |
| IMP-SUP-SBM-029 | 四半期 SBOM スナップショットの WORM 化監査保管 | DS-SW-COMP-141 |

SBM-022/023 で SBOM 添付率 100% を admission レベルで保証し、SBM-024/025 で差分監視、SBM-027 で CVE 連動、SBM-028/029 で長期監査を成立させる。SBM-020 の 4 言語固定は、生成器の選択肢を増やさないことで再現性と監査性を優先する設計判断である。

## SLSA（30_SLSA_プロビナンス / 10 ID）

[30_SLSA_プロビナンス/01_SLSA_プロビナンス設計.md](../30_SLSA_プロビナンス/01_SLSA_プロビナンス設計.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SUP-SLSA-030 | リリース時点 SLSA L2 到達（hosted runner + slsa-github-generator） | ADR-SUP-001 / NFR-E-SIR-001 |
| IMP-SUP-SLSA-031 | Provenance v1 自動生成（in-toto v1 形式） | ADR-SUP-001 |
| IMP-SUP-SLSA-032 | `cosign attest --type slsaprovenance1` + Rekor 透明性ログ | ADR-CICD-003 / NFR-E-SIR-002 |
| IMP-SUP-SLSA-033 | Kyverno verifyAttestations で type=slsaprovenance1 必須化 | ADR-CICD-003 / NFR-C-MGMT-002 |
| IMP-SUP-SLSA-034 | catalog-info.yaml `k1s0.io/slsa-level` annotation 表示 | DS-SW-COMP-141 |
| IMP-SUP-SLSA-035 | TechInsights Scorecard で claimed > verified を admission reject | ADR-CICD-003 |
| IMP-SUP-SLSA-036 | 採用後 L3 拡張 1: Hermetic build（egress block + vendoring） | ADR-SUP-001 |
| IMP-SUP-SLSA-037 | 採用後 L3 拡張 2: Isolated builder（Self-hosted ARC + single-use VM） | ADR-SUP-001 |
| IMP-SUP-SLSA-038 | 採用後 L3 拡張 3: Two-person review（CODEOWNERS + 4-eyes） | NFR-C-MGMT-002 |
| IMP-SUP-SLSA-039 | 採用後 L3 拡張 4: Reproducible build（同 commit から同 hash） | ADR-SUP-001 |

SLSA-030〜035 でリリース時点 L2 を成立させ、SLSA-036〜039 で採用後 L3 への拡張パスを予約する。SLSA-035 の「虚偽申告検知」が claim と verify の乖離を構造的に防ぐ要石である。

## FOR（40_Forensics_Runbook / 9 ID）

[40_Forensics_Runbook/01_image_hash逆引き_Forensics.md](../40_Forensics_Runbook/01_image_hash逆引き_Forensics.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SUP-FOR-040 | Runbook トリガ 3 種類と起点 digest 統一 | ADR-OBS-003 / NFR-E-SIR-002 |
| IMP-SUP-FOR-041 | cosign download sbom によるパッケージ展開 | ADR-CICD-003 |
| IMP-SUP-FOR-042 | image digest → Deployment 逆引きスクリプト | NFR-A-RTO-001 |
| IMP-SUP-FOR-043 | Deployment → Service → VirtualService → tier1 API mapping | DS-SW-COMP-141 |
| IMP-SUP-FOR-044 | Rekor inclusion proof による改ざん検証 | NFR-E-SIR-002 |
| IMP-SUP-FOR-045 | impact-report.md 自動生成と招集 | NFR-A-RTO-001 |
| IMP-SUP-FOR-046 | 15 分 / 48 時間の 2 段階 SLI | NFR-A-RTO-001 / NFR-A-CONT-001 |
| IMP-SUP-FOR-047 | 四半期 GameDay 演習（LitmusChaos 注入） | NFR-A-FT-001 |
| IMP-SUP-FOR-048 | Runbook 改訂 / 実行権限の分離と 3 チーム承認 | NFR-C-IR-001 |

FOR-040〜045 が逆引き本体、FOR-046 が時間 SLI、FOR-047 が形骸化防止の演習、FOR-048 が運用統制、と 4 軸で Forensics の継続性を担保する。

## FLG（50_flag_定義署名検証 / 8 ID）

[50_flag_定義署名検証/01_flag_定義署名検証.md](../50_flag_定義署名検証/01_flag_定義署名検証.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SUP-FLG-050 | cosign sign-blob keyless 署名（flagd 定義 bundle 化） | ADR-FM-001 / ADR-CICD-003 |
| IMP-SUP-FLG-051 | OCI Artifact 配布 + Rekor 統合（subject = release-flagd.yml ref 固定） | ADR-SUP-001 |
| IMP-SUP-FLG-052 | Kyverno ClusterPolicy `verify-flag-attestation` 統制 | ADR-CICD-003 / NFR-C-MGMT-002 |
| IMP-SUP-FLG-053 | 四半期棚卸し Step 1（全 flag 定義列挙、attestation 必須） | NFR-C-MGMT-002 |
| IMP-SUP-FLG-054 | 四半期棚卸し Step 2（Loki 評価ログ cross-check、90 日閾値） | DS-SW-COMP-141 |
| IMP-SUP-FLG-055 | 四半期棚卸し Step 3（孤立 flag 通知 + 30 日後自動削除 PR） | NFR-C-MGMT-002 / NFR-E-CONF-001 |
| IMP-SUP-FLG-056 | 検証失敗時 Forensics 判別（改ざん vs 鍵異常 の 2 分岐） | NFR-E-SIR-002 |
| IMP-SUP-FLG-057 | PagerDuty Sev1/Sev2 自動エスカレーション（CISO + CTO 経路） | NFR-A-FT-001 |

FLG は 70 章 30 節（flagd 運用）の真正性レイヤとして配置される。FLG-052 の admission 統制、FLG-053〜055 の棚卸し、FLG-056〜057 の Forensics 連携で flag ライフサイクル全体を監査統制下に置く。

## ADR 別逆引き

主要 ADR が IMP-SUP-* のどの ID 群を呼び出すかを示す。ADR が 1 章で完結せず、複数セクションに散る構造を可視化する。

| ADR | 呼出 IMP-SUP ID |
|-----|-----------------|
| ADR-CICD-003（Kyverno） | POL-004, COS-010, COS-013, SBM-022, SBM-023, SLSA-032, SLSA-033, SLSA-035, FOR-041, FLG-050, FLG-052 |
| ADR-SUP-001（SLSA L2→L3） | POL-001, POL-002, COS-011, COS-016, SLSA-030, SLSA-031, SLSA-036, SLSA-037, SLSA-039, FLG-051 |
| ADR-DEP-001（Renovate） | POL-003, POL-006, SBM-025 |
| ADR-0003（AGPL 分離） | POL-007, COS-015, SBM-026 |
| ADR-OBS-003（Forensics SLI 分離） | POL-005, COS-014, FOR-040 |
| ADR-FM-001（flagd） | FLG-050 |

ADR-CICD-003 と ADR-SUP-001 が 80 章の中核 ADR で、両者で 21 ID（53 中 39%）をカバーする。ADR-OBS-003 が 60 章観測性と接続し、ADR-FM-001 が 70 章 flagd と接続する経路を示している。

## NFR 別逆引き

| NFR | 呼出 IMP-SUP ID |
|-----|-----------------|
| NFR-E-SIR-002（脆弱性検知） | POL-002, POL-005, POL-006, COS-010, COS-014, COS-017, SBM-024, SBM-027, SLSA-032, FOR-040, FOR-044, FLG-056 |
| NFR-C-MGMT-002（リリース統制） | COS-012, SBM-025, SLSA-033, SLSA-038, FLG-052, FLG-053, FLG-055 |
| NFR-A-RTO-001（復旧時間） | COS-018, SBM-028, FOR-042, FOR-045, FOR-046 |
| NFR-C-MGMT-003（SBOM 100%） | POL-003, SBM-022, SBM-023 |
| NFR-E-NW-003（AGPL 分離維持） | POL-007, SBM-026 |
| NFR-E-SIR-001（インシデント分類） | POL-001, SLSA-030 |
| NFR-A-FT-001（フォルトトレランス演習） | FOR-047, FLG-057 |
| NFR-A-CONT-001（事業継続） | FOR-046 |
| NFR-H-INT-002（SBOM インテグリティ） | POL-003, SBM-020 |
| NFR-C-IR-001（インシデント対応） | FOR-048 |
| NFR-C-IR-002（admission） | POL-004 |
| NFR-E-CONF-001（構成変更追跡） | FLG-055 |
| NFR-C-MGMT-002 別軸（リリース統制 / flagd） | FLG-052, FLG-053, FLG-055 |

NFR-E-SIR-002（脆弱性検知）が 12 ID で最頻出。POL/COS/SBM/SLSA/FOR/FLG 全セクションに分散しており、80 章全体が脆弱性検知 NFR 達成のために配置されている構造を示す。

## DS-SW-COMP 別逆引き

| DS-SW-COMP | 呼出 IMP-SUP ID |
|------------|-----------------|
| DS-SW-COMP-135（配信系インフラ Harbor / Kyverno / cosign） | POL-002〜004, COS-010〜018 全部分, SBM-020〜023, SLSA-030〜033, SLSA-035, FOR-041, FLG-050〜052 |
| DS-SW-COMP-141（多層防御統括 / Observability + Security 統合監査） | POL-001, POL-005〜007, COS-014, COS-018, SBM-024〜029, SLSA-034, SLSA-036〜039, FOR-040, FOR-042〜048, FLG-053〜057 |
| DS-SW-COMP-085（OTel Gateway / Mimir、評価ログ経路） | FLG-054 |

DS-SW-COMP-135（配信系）と DS-SW-COMP-141（多層防御統括）の 2 軸で 80 章を完全カバーする。135 は admission / 署名 / 配布の縦経路、141 は監査 / Forensics / 棚卸しの横断的統制を担当する分担構造である。085 は 70 章中心の DS-SW-COMP だが、本章 FLG-054 で flag 評価ログ cross-check の経路として接続する。

## 集計結果

- 全 IMP-SUP ID: 53（POL 7 + COS 9 + SBM 10 + SLSA 10 + FOR 9 + FLG 8）
- 関連 ADR: 6 件（ADR-CICD-003 / ADR-SUP-001 / ADR-DEP-001 / ADR-0003 / ADR-OBS-003 / ADR-FM-001）
- 関連 DS-SW-COMP: 3 件（135 / 141 / 085）
- 関連 NFR: 13 件以上（E-SIR-002 が最頻出）

## 関連索引

- [`../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md): モノレポ全 12 接頭辞の集計
- [`../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md): 全 ADR ↔ IMP の対応
- [`../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md`](../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md): DS-SW-COMP ↔ IMP の対応
- [`../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md`](../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md): NFR ↔ IMP の対応
