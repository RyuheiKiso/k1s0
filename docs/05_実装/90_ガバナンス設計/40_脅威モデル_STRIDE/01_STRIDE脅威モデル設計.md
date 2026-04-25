# 01. STRIDE 脅威モデル設計

本ファイルは k1s0 における STRIDE 脅威モデルの物理配置・作成粒度・更新プロセス・緩和策リンクを実装段階確定版として確定する。90 章方針の IMP-POL-POL-004（tier1 公開 11 API / 外部連携ごとの STRIDE）を、`docs/05_実装/90_ガバナンス設計/40_脅威モデル_STRIDE/` 配下の API 別ファイル、STRIDE 6 区分テンプレート、API 追加時の PR チェックリスト、85 章 / 80 章 / 60 章との双方向リンクで具体化する。

![STRIDE 6 区分 × tier1 公開 11 API × 緩和策 IMP-* 対応マトリクス](img/40_STRIDE_11API脅威マトリクス.svg)

脅威モデルを「全体で 1 枚」作ると粒度が荒れて実装とリンクしない。tier1 の 11 API それぞれに、外部連携（SAP / SaaS / オンプレ SaaS / 顧客 IdP）それぞれに、STRIDE 6 区分で脅威を洗い出し、緩和策を対応 IMP-* ID として記録することで、「この脅威はここで緩和されている」という双方向参照可能な台帳を構築する。本節はその作成 / 更新 / 審査プロセスを固定する。

崩れると、脅威モデルが PDF で 1 度作られて棚に仕舞い込まれ、API 追加時のセキュリティ評価が抜ける。結果、「この新 API には tenant_id 検証が入っていない」「この外部連携は replay 攻撃耐性が無い」といった穴が稼働後に発覚する。脅威モデルと実装の双方向リンクが維持される限り、API 追加 PR の段階で脅威網羅性が検証される構造が守られる。

## OSS リリース時点での確定範囲

- リリース時点: tier1 公開 11 API 分の STRIDE 脅威モデル初版作成、85 章 / 80 章 / 60 章との双方向リンク
- リリース時点: 外部連携面（SAP / SaaS 等）5-8 件分の追加、年次レビュー定着
- 採用後の運用拡大時: 脅威モデルの自動検証（CI で「この API に tenant_id 検証 IMP が無い」を検出）

## 作成単位と粒度

STRIDE 脅威モデルは以下の単位で作成する（IMP-POL-STR-040）。

- **tier1 公開 11 API**: `40_脅威モデル_STRIDE/11_tier1_<api>.md` で 11 ファイル（Service Invoke / State / PubSub / Secrets / Binding / Workflow / Log / Telemetry / Decision / Audit+PII / Feature）
- **外部連携面**: `40_脅威モデル_STRIDE/20_external_<連携先>.md`（SAP / Salesforce / Workday / 顧客 IdP 等、採用決定時点で個別起票）
- **tier 境界**: `40_脅威モデル_STRIDE/30_tier_boundary_<tier>.md`（tier1↔tier2 / tier2↔tier3 / tier3↔client の 3 境界）
- **データ層**: `40_脅威モデル_STRIDE/40_data_<ストア>.md`（CloudNativePG / Kafka / Valkey / MinIO 各 1 ファイル）

tier1 公開 11 API 別の粒度にする根拠は、API 種別によって STRIDE の該当脅威が異なるためである。Secrets API は Information Disclosure が主軸、Workflow API は Tampering（状態改竄）と Elevation of Privilege（ワークフロー権限昇格）が主軸となる。API 単位に砕くことで、各実装節（85 章 / 80 章 / 60 章）から脅威エントリを双方向参照できる。

## STRIDE 6 区分テンプレート

各ファイルは STRIDE 6 区分で章分けし、各区分で以下 5 項目を記述する（IMP-POL-STR-041）。

- **脅威 ID**: `THR-<API>-<区分>-<連番>`（例: `THR-SECRETS-I-001` = Secrets API の Information Disclosure 1 番）
- **悪用シナリオ**: 攻撃者視点の具体記述（「攻撃者が ServiceAccount token を漏洩させ、Secrets API で他 tenant の secret を取得する」等）
- **影響資産**: tier1 API / データ / 認証情報 / 監査ログのいずれか
- **緩和策**: 対応 IMP-* ID（85 章 IMP-SEC-KC-* / 80 章 IMP-SUP-COS-* / 本章 IMP-POL-KYV-* 等）
- **残余リスクと受容状態**: 緩和後も残る risk の受容レベル（`Accepted` / `Monitored` / `Under mitigation`）

STRIDE 6 区分の定義は Microsoft STRIDE の原典に準拠する（IMP-POL-STR-042）。Spoofing（なりすまし）/ Tampering（改竄）/ Repudiation（否認）/ Information Disclosure（情報漏洩）/ Denial of Service（サービス不能）/ Elevation of Privilege（権限昇格）。各 API で全 6 区分を記述し、該当脅威ゼロの区分は「該当なし / 理由」を明記（空白禁止）する。

## tier1 API 別の主要脅威例

STRIDE の主要脅威を API ごとに事前に予測しておくと、初版作成の出発点になる（IMP-POL-STR-043）。以下は リリース時点 初版の想定脅威一覧で、実装レビューで精緻化する。

- **Service Invoke**: Spoofing（JWT 偽造）→ 85 章 IMP-SEC-KC-020 JWT 5 項目検証、DoS（無制限 RPS）→ NFR-E-NW-004 レート制限
- **State**: Tampering（state 値改竄）→ Valkey TLS + AUTH、Information Disclosure（tenant 越し参照）→ NFR-E-AC-003 tenant_id 強制
- **PubSub**: Spoofing（偽 publish）→ publish ACL、Repudiation（誰が publish したか不明）→ NFR-E-MON-001 特権操作 Audit
- **Secrets**: Information Disclosure（secret 取得越境）→ NFR-E-AC-004 Secret 最小権限、Elevation of Privilege（policy 書換）→ ADR-SEC-002 OpenBao
- **Binding**: Spoofing（偽外部 endpoint）→ allowlist（NFR-E-NW-001）、Tampering（response 改竄）→ mTLS + 完整性検証
- **Workflow**: Tampering（状態改竄）→ Saga イベントソーシング、Elevation of Privilege（補償動作権限）→ policy 分離
- **Log**: Tampering（log 改竄）→ WORM 保存（IMP-POL-POL-005）、Repudiation（ログ削除否認）→ 監査証跡
- **Telemetry**: Information Disclosure（PII 誤送信）→ NFR-E-ENC-003 マスキング、DoS（metric 洪水）→ cardinality 制限
- **Decision**: Tampering（ルール改竄）→ NFR-C-MGMT-002 Git 管理、Information Disclosure（ルール漏洩）→ OpenBao
- **Audit / PII**: Tampering（改竄）→ NFR-H-AUD-001 完整性、Information Disclosure（PII 漏洩）→ NFR-G-ENC-003 アプリ層暗号
- **Feature**: Tampering（flag 書換）→ NFR-C-MGMT-002 Git 管理、Elevation of Privilege（feature 有効化）→ RBAC

これらは出発点で、実装レビューで リリース時点 時点の精緻化を行う。各脅威は対応する NFR / IMP-* ID へ双方向リンクを張り、緩和策の実装状況をトレーサビリティマトリクスで追跡する。

## API 追加時の PR チェックリスト

tier1 公開 API の追加 / 仕様変更 PR には、STRIDE 脅威モデル更新を必須のチェック項目として組み込む（IMP-POL-STR-044）。PR template に以下の 6 個のチェックボックスを置き、全て満たさない PR は CI で block する。

- [ ] Spoofing 脅威を洗い出した / 該当なしの場合は理由を記載
- [ ] Tampering 脅威を洗い出した / 該当なしの場合は理由を記載
- [ ] Repudiation 脅威を洗い出した / 該当なしの場合は理由を記載
- [ ] Information Disclosure 脅威を洗い出した / 該当なしの場合は理由を記載
- [ ] Denial of Service 脅威を洗い出した / 該当なしの場合は理由を記載
- [ ] Elevation of Privilege 脅威を洗い出した / 該当なしの場合は理由を記載

対応する STRIDE ファイル（`11_tier1_<api>.md`）を同 PR 内で更新し、新規脅威エントリに対応する IMP-* ID を 85 章 / 80 章 / 60 章にも追加する。「脅威は洗い出したが緩和策 IMP を作らない」ケースは `受容` 状態として明示記述する（IMP-POL-STR-045）。

## 年次全件レビュー

STRIDE 脅威モデルは一度作って終わりではない。年次で全件レビューを実施し、以下を再確認する（IMP-POL-STR-046）。

- 環境変化（新たな攻撃手法 / 新規コンポーネント / 外部連携追加）による新規脅威の追加
- 緩和策の実装実態確認（IMP-* ID が実装済 / 稼働中であることの証拠）
- 残余リスクの受容状態再確認（`Under mitigation` が 1 年放置なら Accepted/Monitored に遷移判断）
- 対応 NFR の実測値確認（SLO 達成状況 / インシデント履歴との突合）

年次レビューは Security（D）主導、architecture-team + Tier Lead 合議で実施し、diff を ADR として起票する（IMP-POL-POL-004 継承）。全件レビューに 1 週間を確保し、`ops/audit/stride-annual-YYYY.md` に結果を WORM 保管する。レビュー後の改訂は PR 1 本で全 STRIDE ファイルに反映する。

## 外部連携面の脅威モデル

外部連携（SAP / Salesforce / 顧客 IdP 等）は tier1 とは脅威プロファイルが異なる。外部の完整性を内部の完整性と同程度に信頼できないため、以下の追加脅威を必ず考慮する（IMP-POL-STR-047）。

- 接続先 endpoint のなりすまし: NFR-E-NW-001 allowlist + TLS pinning
- 外部側からの replay 攻撃: timestamp + nonce + HMAC の 3 点セット
- 外部側での改竄（顧客 IdP の乗っ取り）: k1s0-tenants realm の Brokering で境界維持（85 章 IMP-SEC-KC-011）
- 外部側での情報漏洩: 送信データの最小化（NFR-G-DES-001）、外部送信 log の WORM 保管
- 外部側の DoS 転嫁: Circuit Breaker（NFR-C-IR-002）、quota 管理

外部連携は採用決定時点で個別 STRIDE ファイルを起票し、ADR-<領域>-XXX と対を成す。外部連携廃止時は STRIDE ファイルも `Deprecated` とし、削除はしない（過去の判断記録として保持）。

## 受け入れ基準

- tier1 公開 11 API 分の STRIDE ファイル 11 個が リリース時点点で存在
- 各ファイルに STRIDE 6 区分が網羅され、該当なしには理由明記
- 全脅威エントリに対応 IMP-* ID（85 章 / 80 章 / 60 章等）がリンクされている
- API 追加 / 仕様変更 PR に STRIDE 更新チェックリストが必須化、未更新で block
- 年次全件レビューが稼働、diff ADR が年 1 本以上起票
- 外部連携面の STRIDE が採用決定時点で個別起票
- 全脅威エントリの残余リスク状態（`Accepted` / `Monitored` / `Under mitigation`）が明示、`Under mitigation` の放置検出が年次レビューで実施

## RACI

| 役割 | 責務 |
|---|---|
| Security（主担当 / D） | STRIDE 初版作成、年次レビュー主導、脅威エントリ承認 |
| SRE（共担当 / B） | DoS / Availability 脅威の緩和策実装、Runbook 連動 |
| Architecture team（共担当 / A） | API 追加時の STRIDE 更新レビュー、残余リスク受容判断 |
| Tier Lead（I） | tier 境界脅威の実装責任、緩和策 IMP の実装 |

## 対応 IMP-POL-STR ID

| ID | 主題 | 適用段階 |
|---|---|---|
| IMP-POL-STR-040 | 作成単位（tier1 11 API / 外部連携 / tier 境界 / データ層） | 採用初期 |
| IMP-POL-STR-041 | STRIDE 6 区分テンプレートと必須 5 項目 | 採用初期 |
| IMP-POL-STR-042 | Microsoft STRIDE 原典準拠と全 6 区分網羅要求 | 採用初期 |
| IMP-POL-STR-043 | tier1 API 別主要脅威例と出発点の定義 | 採用初期 |
| IMP-POL-STR-044 | API 追加 / 変更 PR の STRIDE 更新必須チェックリスト | 採用初期 |
| IMP-POL-STR-045 | 緩和策なし脅威の `受容` 明示記述 | 採用初期 |
| IMP-POL-STR-046 | 年次全件レビューと diff ADR 起票 | 採用後の運用拡大時 |
| IMP-POL-STR-047 | 外部連携面の追加脅威 5 種（なりすまし / replay / 改竄 / 漏洩 / DoS 転嫁） | 採用後の運用拡大時 |

## 対応 ADR / DS-SW-COMP / NFR

- ADR-POL-001（Kyverno 二分所有モデル、本章初版策定時に起票予定）/ [ADR-SEC-001](../../../02_構想設計/adr/ADR-SEC-001-keycloak.md)（Keycloak）/ [ADR-CICD-003](../../../02_構想設計/adr/ADR-CICD-003-kyverno.md)（Kyverno）
- DS-SW-COMP-141（多層防御統括）/ DS-SW-COMP-001 / DS-SW-COMP-002（tier1 公開 API 基盤）
- NFR-E-RSK-001（STRIDE 脅威モデリング）/ NFR-E-SIR-002（漏洩報告）/ NFR-E-AC-001（JWT 強制）/ NFR-E-AC-003（tenant_id 検証）/ NFR-E-NW-001（外部 URL allowlist）/ NFR-E-NW-002（NetworkPolicy テナント隔離）/ NFR-E-NW-004（レート制限）/ NFR-H-AUD-001（監査ログ完整性）/ NFR-G-AC-001（最小権限）

## 関連章

- `10_Kyverno_Policy/` — 緩和策としての validate policy マッピング
- `20_ADR_プロセス/` — STRIDE 改訂時の ADR 起票ルール
- `../85_Identity設計/` — 認証認可系緩和策（Spoofing / Elevation of Privilege）
- `../80_サプライチェーン設計/` — サプライチェーン改竄緩和策（Tampering）
- `../60_観測性設計/` — Repudiation / DoS 検知の observability
