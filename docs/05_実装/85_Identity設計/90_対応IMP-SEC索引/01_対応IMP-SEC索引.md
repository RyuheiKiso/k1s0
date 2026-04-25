# 01. 対応 IMP-SEC 索引

本ファイルは k1s0 Identity 章（85_Identity設計）配下の全実装 ID（IMP-SEC-*）を 1 箇所にカタログ化する索引である。各 ID の実装位置・対応 ADR / DS-SW-COMP / NFR を逆引き可能とし、退職時 revoke の漏れチェックや GameDay 演習設計時に「どの ID がどこに実装され、どの要件を満たすか」を 1 ファイルで把握できる構造を提供する。99_索引（モノレポ全体）との関係は、99_索引が全 12 接頭辞横断、本ファイルは IMP-SEC-* に限定した深掘りカタログ、と分担する。

## サブ接頭辞別の集計

Identity 章は 5 つのサブ接頭辞で構成される。それぞれが独立したセクションを持ち、合計 66 ID を採番する。Identity は人間 ID とワークロード ID の 2 系統を抱え、さらに secret / 証明書 / 退職 revoke の 3 領域を含むため、IMP 数は 12 章の中でも 80_サプライチェーン に次ぐ 2 番目の大きさとなる。なお、IMP-SEC-KEY-* サブ接頭辞は採用初期の OpenBao 鍵ライフサイクル詳細採番のために予約状態であり、本リリース時点では POL/KC/SP/OBO/CRT/REV の 5 サブ接頭辞のみ採番済。

| サブ接頭辞 | セクション | ID 範囲 | ID 数 | 主担当責務 |
|------------|------------|---------|-------|-----------|
| POL | 00_方針 | 001〜007 | 7 | Identity 6 原則の提示 |
| KC | 10_Keycloak_realm | 010〜022 | 13 | 人間 ID プロバイダ realm 設計 |
| SP | 20_SPIRE_SPIFFE | 020〜035 | 16 | ワークロード ID SVID 発行 |
| OBO | 30_OpenBao | 040〜049 | 10 | Secret 管理（KV-v2/PKI/Transit） |
| CRT | 40_cert-manager | 060〜069 | 10 | 証明書自動更新と istio-csr 統合 |
| REV | 50_退職時revoke手順 | 050〜059 | 10 | 退職時 60 分 SLA + GameDay 演習 |

合計 66 ID（POL 7 + KC 13 + SP 16 + OBO 10 + CRT 10 + REV 10）で、Identity 章は人間 ID（KC）+ ワークロード ID（SP）+ Secret（OBO）+ 証明書（CRT）+ 退職運用（REV）の 5 領域を一体的に統制する。番号レンジは sub-prefix 単位で運用され、KC（010-022）と SP（020-035）の重複は ID prefix で区別される。IMP-SEC-KEY-001〜009 は予約のみで本リリース時点では未採番（採用初期で OpenBao 鍵ライフサイクルを切り出す際に採番予定）。

## POL（00_方針 / 7 ID）

[00_方針/01_Identity原則.md](../00_方針/01_Identity原則.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SEC-POL-001 | 人間 ID Keycloak 集約 | ADR-SEC-001 / NFR-E-AC-001 |
| IMP-SEC-POL-002 | ワークロード ID SPIRE | ADR-SEC-003 / NFR-E-AC-004 |
| IMP-SEC-POL-003 | 退職 revoke 60 分以内 | NFR-E-AC-005 |
| IMP-SEC-POL-004 | OpenBao Secret 集約 | ADR-SEC-002 / NFR-E-AC-004 |
| IMP-SEC-POL-005 | cert-manager 証明書自動更新 | NFR-E-ENC-001 / NFR-E-ENC-002 |
| IMP-SEC-POL-006 | Istio Ambient mTLS | ADR-0001 / NFR-E-ENC-002 |
| IMP-SEC-POL-007 | GameDay 継続検証 | NFR-A-FT-001 |

POL は方針層であり、後続の 5 セクション（KC/SP/OBO/CRT/REV）が方針の物理化を担う。POL 単独では原則記述に留まり、realm 設計や SVID rotation や退職 revoke 運用は他セクションで実現する。

## KC（10_Keycloak_realm / 13 ID）

[10_Keycloak_realm/01_Keycloak_realm設計.md](../10_Keycloak_realm/01_Keycloak_realm設計.md) で採番される。

| ID | 内容 | 主対応 |
|----|------|--------|
| IMP-SEC-KC-010〜015 | realm 階層 / tenant claim / JWT scope / login flow / MFA / Trusted Web Origins | ADR-SEC-001 / NFR-E-AC-001/003 |
| IMP-SEC-KC-016〜022 | admin event 監査 / Postgres backend / 7 年 audit 保存 / federation / SCIM 連携 / Recovery flow / break-glass account | NFR-E-MON-001 / NFR-H-COMP-* |

詳細は所属ファイル参照。KC 13 ID は人間 ID の login / token 発行 / 監査の 3 軸で構成される。

## SP（20_SPIRE_SPIFFE / 16 ID）

[20_SPIRE_SPIFFE/01_SPIRE_SPIFFE設計.md](../20_SPIRE_SPIFFE/01_SPIRE_SPIFFE設計.md) で採番される。

| ID | 内容 | 主対応 |
|----|------|--------|
| IMP-SEC-SP-020〜025 | SPIRE Server HA 3 replica / DataStore / TrustDomain / Bundle Endpoint / Federation | ADR-SEC-003 |
| IMP-SEC-SP-026〜030 | SPIRE Agent DaemonSet / PSAT 認証 / Workload Attestor / SVID 1h rotation / Workload API socket | NFR-E-AC-004 |
| IMP-SEC-SP-031〜035 | Dapr 統合 / Istio mTLS 統合 / Observability metrics / GameDay 演習 / Disaster Recovery | ADR-0001 / NFR-A-FT-001 |

詳細は所属ファイル参照。SP 16 ID は ワークロード ID 発行 → 配布 → 統合 → 監視 → 演習 の 5 軸で構成される。

## OBO（30_OpenBao / 10 ID）

[30_OpenBao/01_OpenBao設計.md](../30_OpenBao/01_OpenBao設計.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SEC-OBO-040 | `infra/identity/openbao/` Helm chart 配置と段階 HA（single → 3 node → DR） | ADR-SEC-002 / DS-SW-COMP-006 |
| IMP-SEC-OBO-041 | Raft Integrated Storage + 日次 snapshot + S3 Object Lock 30 日 | ADR-SEC-002 / NFR-A-DR-* |
| IMP-SEC-OBO-042 | 3 secret engine（KV-v2 / PKI / Transit）の責務分離と cross-reference 禁止 | ADR-SEC-002 |
| IMP-SEC-OBO-043 | KV-v2 path 設計（`secret/k1s0/<ns>/<app>/`）+ SDK 経由必須化 | NFR-E-AC-004 |
| IMP-SEC-OBO-044 | PKI Secret Engine 配置と root CA cold storage 分離 | NFR-H-KEY-001 |
| IMP-SEC-OBO-045 | Transit Secret Engine による PII 鍵 / JWT signing key の Encryption-as-a-Service | NFR-G-ENC-* / NFR-H-KEY-001 |
| IMP-SEC-OBO-046 | Kubernetes Auth Method（SA token review API 経由） | NFR-E-AC-004 |
| IMP-SEC-OBO-047 | role IaC 管理（Terraform）+ Security CODEOWNERS 必須 | NFR-C-MGMT-001 |
| IMP-SEC-OBO-048 | Auto-unseal（AWS KMS / GCP KMS / Azure KeyVault）+ KMS CMK 年次 rotation | NFR-A-FT-001 / NFR-H-KEY-001 |
| IMP-SEC-OBO-049 | Audit Device 二段保管（Loki 90 日 + S3 7 年）+ root token 利用 Sev1 通知 | NFR-E-MON-002 / NFR-H-INT-004 |

OBO は Secret 管理の中核で、KV-v2 / PKI / Transit の 3 engine 分離が「engine ごとの監査ログ独立」と「Forensics 時の事故域分離」を成立させる設計判断である。Auto-unseal の KMS 依存は可用性の前提として明示的に受容する。

## CRT（40_cert-manager / 10 ID）

[40_cert-manager/01_cert-manager設計.md](../40_cert-manager/01_cert-manager設計.md) で採番される。

| ID | 内容 | 主対応 ADR / DS-SW-COMP / NFR |
|----|------|-------------------------------|
| IMP-SEC-CRT-060 | controller Reconciliation Loop 5 min cycle と renewBefore 自動再発行 | NFR-A-FT-001 |
| IMP-SEC-CRT-061 | 3 ClusterIssuer 責務分離（Let's Encrypt / Vault PKI / SelfSigned bootstrap） | ADR-SEC-002 |
| IMP-SEC-CRT-062 | public 90 日 / private 30 日の duration 二段設定 + OpenBao PKI engine 連携 | NFR-E-ENC-001 |
| IMP-SEC-CRT-063 | `privateKey.rotationPolicy: Always` による renewal 時 key rotate 必須化 | NFR-H-KEY-001 |
| IMP-SEC-CRT-064 | CertificateRequest 7 年 audit log 保管（S3 Object Lock Compliance mode） | NFR-H-INT-004 |
| IMP-SEC-CRT-065 | istio-csr による SPIRE SVID → Istio mTLS 統合と 1h rotation 自動化 | ADR-0001 / ADR-SEC-003 |
| IMP-SEC-CRT-066 | cert-manager と SPIRE-SPIFFE の境界（ingress cert vs workload mTLS SVID） | ADR-0001 |
| IMP-SEC-CRT-067 | controller metrics 3 系列の OTel 収集 | NFR-E-MON-002 |
| IMP-SEC-CRT-068 | Prometheus Alert 4 段階エスカレーション（7d Sev3 / 24h Sev2 / 1h Sev1 / renewal fail Sev2） | NFR-A-FT-001 |
| IMP-SEC-CRT-069 | renewal 失敗時 4 step Runbook（status 確認 / Issuer 到達性 / 強制 renew / 手動 emergency） | NFR-C-IR-001 |

CRT は cert 自動更新の中核で、3 Issuer 分離（CRT-061）と 4 段階アラート（CRT-068）が「手動運用ゼロ」と「期限切れ事故ゼロ」を構造的に保証する。istio-csr による SPIRE 統合（CRT-065/066）が Istio Ambient mTLS の自動化を担う。

## REV（50_退職時revoke手順 / 10 ID）

[50_退職時revoke手順/01_退職時revoke手順.md](../50_退職時revoke手順/01_退職時revoke手順.md) で採番される。

| ID | 内容 | 主対応 |
|----|------|--------|
| IMP-SEC-REV-050〜054 | 起点通知 / 60 分 SLA / Keycloak 無効化 / OpenBao token revoke / 監査ログ 7 年 WORM | NFR-E-AC-005 / NFR-H-INT-004 |
| IMP-SEC-REV-055〜059 | Service Account 最小権限 / cert revoke / 漏れ検出 / 四半期 GameDay / 第三者監査 | NFR-A-FT-001 / NFR-H-COMP-* |

詳細は所属ファイル参照。REV 10 ID は退職通知 → 60 分 SLA → 各 ID 系統の revoke → 漏れ検出 → 演習 の 5 軸で構成される。

## ADR 別逆引き

主要 ADR が IMP-SEC-* のどの ID 群を呼び出すかを示す。

| ADR | 呼出 IMP-SEC ID |
|-----|-----------------|
| ADR-SEC-001（Keycloak） | POL-001, KC-010〜022（全 13） |
| ADR-SEC-002（OpenBao） | POL-004, OBO-040〜049（全 10）, CRT-061, CRT-062 |
| ADR-SEC-003（SPIRE-SPIFFE） | POL-002, SP-020〜035（全 16）, CRT-065, CRT-066 |
| ADR-0001（Istio Ambient） | POL-006, SP-031〜032（Istio 統合）, CRT-065, CRT-066 |
| ADR-CICD-003（Kyverno） | OBO-047（policy 強制経路） |
| ADR-0003（AGPL/BUSL 分離） | OBO-040（OpenBao = MPL-2.0、Vault BUSL からの fork 採用判断） |

ADR-SEC-001/002/003 が Identity の 3 本柱 ADR で、合計 47 ID（66 中 71%）を直接的に覆う（ADR-SEC-001 = POL-001 + KC 13 / ADR-SEC-002 = POL-004 + OBO 10 + CRT 061-062 / ADR-SEC-003 = POL-002 + SP 16 + CRT 065-066）。ADR-0001 は Istio Ambient 経路で SP / CRT 両方に効く境界 ADR。

## NFR 別逆引き

| NFR | 呼出 IMP-SEC ID |
|-----|-----------------|
| NFR-E-AC-001（JWT 強制） | POL-001, KC-010〜015 |
| NFR-E-AC-003（tenant_id 検証） | KC-011〜015 |
| NFR-E-AC-004（Secret 最小権限） | POL-002, POL-004, OBO-043, OBO-046, SP-026〜030 |
| NFR-E-AC-005（MFA / 退職 revoke） | POL-003, KC-014, REV-050〜059 |
| NFR-E-ENC-001（保管暗号化） | POL-005, OBO-045, CRT-062 |
| NFR-E-ENC-002（転送暗号化） | POL-005, POL-006, CRT-061〜065 |
| NFR-E-MON-001（特権監査） | KC-016〜018, REV-054 |
| NFR-E-MON-002（Secret 取得監査） | OBO-049, CRT-067 |
| NFR-H-INT-004（監査ログ完整性） | OBO-049, CRT-064, REV-054 |
| NFR-H-KEY-001（鍵ライフサイクル） | POL-005, OBO-044, OBO-045, OBO-048, CRT-063 |
| NFR-A-FT-001（自動復旧 15 分以内） | POL-007, OBO-048, CRT-060, CRT-068, REV-058, SP-035 |
| NFR-C-IR-001（Severity 別応答） | CRT-068, CRT-069, REV-050 |
| NFR-G-AC-001（最小権限） | POL-001〜007（Identity 方針 7 件全て） |

NFR-E-AC 系（4 件）と NFR-E-MON 系（2 件）と NFR-H-KEY-001（1 件）が Identity 章の中核 NFR で、合計 7 NFR が 66 ID 中 50+ ID を呼び出す密度高い結合となっている。

## DS-SW-COMP 別逆引き

| DS-SW-COMP | 呼出 IMP-SEC ID |
|------------|-----------------|
| DS-SW-COMP-006（Secret Store） | POL-004, OBO-040〜049（全 10） |
| DS-SW-COMP-124（tier1 Go Dapr ファサード / SVID 統合） | POL-002, SP-031〜032 |
| DS-SW-COMP-141（多層防御統括） | POL-001〜007（方針 7 全て）, KC-018, OBO-049, CRT-064, REV-054 |

DS-SW-COMP-006（Secret Store）は OpenBao の物理面、DS-SW-COMP-141（多層防御統括）は Identity 全体の統合監査面、と分担する構造。

## 集計結果

- 全 IMP-SEC ID: 66（POL 7 + KC 13 + SP 16 + OBO 10 + CRT 10 + REV 10）
- 関連 ADR: 6 件（ADR-SEC-001/002/003 / ADR-0001 / ADR-CICD-003 / ADR-0003）
- 関連 DS-SW-COMP: 3 件（006 / 124 / 141）
- 関連 NFR: 13 件以上（NFR-E-AC/MON/H-KEY 系 が中核）

## 関連索引

- [`../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../../99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md): モノレポ全 12 接頭辞の集計
- [`../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../../99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md): 全 ADR ↔ IMP の対応
- [`../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md`](../../99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md): DS-SW-COMP ↔ IMP の対応
- [`../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md`](../../99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md): NFR ↔ IMP の対応
