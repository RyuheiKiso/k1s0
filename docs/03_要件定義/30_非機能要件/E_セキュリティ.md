# E. セキュリティ

本書は IPA 非機能要求グレード 2018 の「E. セキュリティ」に準拠し、k1s0 の前提制約、リスク分析、アクセス制御、データ秘匿、不正監視、ネットワーク対策、マルウェア対策、Web 対策、インシデント対応の要件を定義する。採用側組織の情報セキュリティ方針と個人情報保護法（2022 年改正）を満たすことが採用検討通過の前提である。

## 本章の位置付け

セキュリティ要件は、監査対応と日常運用の両面で外部に対する約束を規定する。採用 OSS の組み合わせで組織的に実現可能な水準を示し、全社セキュリティ規程との整合を担保する。

## E1. 前提と制約

### NFR-E-PRE-001: 準拠規格

**現状**: 準拠規格が曖昧だと監査対応が都度個別対応となる。

**要件達成後**: 以下の規格・法令への準拠を明示する。

- **個人情報保護法**（令和 4 年施行、令和 2 年改正）: 仮名加工情報、漏えい等報告義務
- **J-SOX**（金融商品取引法）: IT 全般統制（アクセス・変更・運用）
- **ISO/IEC 27001 相当**: 情報セキュリティマネジメントの参考フレームワーク（認証取得は 採用側の全社ロールアウト 以降判定）
- **NIST SP 800-53 相当**: セキュリティ統制の参考フレームワーク
- **採用側組織の社内情報セキュリティ方針**: 全具体条項

**崩れた時**: 監査対応の都度、規格の解釈で紛糾する。

**受け入れ基準**:

- 90_付録/02_非機能要求グレード判定.md で各規格への対応表を保持
- 法令改正時は 3 か月以内に本書を改訂

**優先度**: MUST

## E2. リスク分析

### NFR-E-RSK-001: 脅威モデリング

**現状**: 脅威モデリングなしに設計を進めると、想定外の攻撃ベクタが残る。

**要件達成後**: STRIDE（Spoofing / Tampering / Repudiation / Information Disclosure / Denial of Service / Elevation of Privilege）に基づく脅威モデリングを リリース時点 で実施し、ADR として残す。リリース時点 / 2 のスコープ変更時に再評価。

**崩れた時**: 設計後期または本番で脅威が発覚し、手戻りが発生する。

**受け入れ基準**:

- STRIDE ベースの脅威モデリング ADR を公開
- 主要 OSS（Dapr / Istio / Keycloak / OpenBao）の CVE 履歴を年次レビュー

**優先度**: MUST

### NFR-E-RSK-002: ペネトレーションテスト

**現状**: 外部目線での検証が無いと、内部バイアスで盲点が残る。

**要件達成後**: リリース時点 で外部委託または内部 Red Team によるペネトレーションテストを実施する。指摘事項は優先度別に是正し、重大指摘は本番稼働前に解消。

**崩れた時**: 本番稼働後に脆弱性が発覚し、インシデント対応で信頼失墜。

**受け入れ基準**:

- リリース時点 で年 1 回のペネトレーションテスト
- 指摘事項の是正ログを残す

**優先度**: MUST

## E3. アクセス制御

### NFR-E-AC-001: JWT 認証強制

**現状**: tier1 API への匿名アクセスは、内部ネットワークでも許容できない（ゼロトラスト原則）。

**要件達成後**: 全 tier1 API 呼び出しで Keycloak 発行の JWT を強制する。JWT 未添付・署名不正・有効期限切れは `K1s0Error.Unauthorized` で拒否。Istio AuthorizationPolicy で L7 検証。

**崩れた時**: 内部からの権限昇格攻撃で越境アクセスが発生する。

**受け入れ基準**:

- 全 tier1 API で JWT 必須
- Istio AuthorizationPolicy のデフォルト拒否
- 匿名アクセス試行を Grafana で可視化、Severity 3 アラート

**優先度**: MUST

### NFR-E-AC-002: ロールベース認可（RBAC）

**現状**: 操作権限の細粒度制御が無いと、最小権限原則違反となる。

**要件達成後**: Keycloak Realm Role（プラットフォーム全体）、Client Role（個別コンポーネント）、アプリケーションロール（業務サービス別）の 3 階層でロール管理する。tier1 API は Keycloak Role に基づき認可判定。ZEN Engine 決定表で業務ポリシー評価。

**崩れた時**: 過剰権限で内部不正リスクが増す。

**受け入れ基準**:

- ロール定義を Backstage で一覧表示
- 最小権限の原則で初期ロール定義
- 四半期ごとにロール棚卸し

**優先度**: MUST

### NFR-E-AC-003: tenant_id クレーム検証

**現状**: マルチテナント隔離が機能要件レベルで担保されても、運用側の徹底で崩れうる。

**要件達成後**: 全 tier1 API で JWT の `tenant_id` クレームを検証し、リソースの tenant_id と突き合わせる。不一致は `K1s0Error.Forbidden` で拒否。Kyverno で tenant_id ラベル必須化。Litmus Chaos で月次の越境試行検証（採用後の運用拡大時）。

**崩れた時**: テナント越境でデータ漏えいが発生し、監査対応で重大インシデント扱い。

**受け入れ基準**:

- JWT クレーム検証の SDK 実装
- Kyverno ポリシーで tenant_id ラベル必須
- 越境試行を Grafana で可視化

**優先度**: MUST

### NFR-E-AC-004: Secret 取得の最小権限

**現状**: 広範な Secret 取得権限で漏えい時の影響範囲が拡大する。

**要件達成後**: 各サービスには必要な Secret のみへのアクセスを許可する。OpenBao の Policy で `<tenant_id>/<service_name>/*` のみアクセス可能とする。

**崩れた時**: 1 サービス侵害で他サービスの Secret まで漏えいする。

**受け入れ基準**:

- OpenBao Policy テンプレート整備
- Policy 適用を CI で検証

**優先度**: MUST

### NFR-E-AC-005: 多要素認証（MFA）

**現状**: パスワードのみではフィッシング・リスト型攻撃に脆弱。

**要件達成後**: 採用側のマルチクラスタ移行時、特権操作（Backstage 管理機能、Argo CD 本番同期、OpenBao 秘密更新）で MFA を必須化する。Keycloak の TOTP または FIDO2 / WebAuthn。

**崩れた時**: 特権アカウント漏えいで重大インシデント発生。

**受け入れ基準**:

- 採用側のマルチクラスタ移行時 で MFA 必須化の運用開始
- Keycloak MFA 設定の標準化

**優先度**: MUST（採用側のマルチクラスタ移行時）、SHOULD（リリース時点）

## E4. データ秘匿

### NFR-E-ENC-001: 通信暗号化

**現状**: 内部ネットワーク通信が平文だと、内部侵害時の影響が拡大する。

**要件達成後**: tier1 内部通信は Istio Ambient ztunnel の mTLS（HBONE）で自動暗号化。外部通信（Envoy Gateway、Keycloak）は TLS 必須。証明書は cert-manager で自動発行・更新。

**崩れた時**: 内部通信盗聴、MITM 攻撃のリスクが顕在化する。

**受け入れ基準**:

- Istio AuthorizationPolicy で mTLS 強制
- cert-manager の証明書有効期限を Prometheus で監視
- 有効期限 30 日前にアラート

**優先度**: MUST

### NFR-E-ENC-002: データ保管暗号化

**現状**: ディスク暗号化が無いと、ディスク廃棄時のデータ漏えいリスクが残る。

**要件達成後**: Longhorn のボリューム暗号化、PostgreSQL の Transparent Data Encryption、MinIO Server-Side Encryption を有効化する。暗号化鍵は OpenBao Transit で一元管理。

**崩れた時**: ディスク盗難時の業務データ漏えい、廃棄時のサニタイズ不備でコンプライアンス違反。

**受け入れ基準**:

- 全ストレージで暗号化有効化
- 鍵ローテーション（年 1 回）を自動化
- リリース時点で 基本 / 運用成熟化

**優先度**: MUST

### NFR-E-ENC-003: PII マスキング・仮名化

**現状**: ログ・分析データに PII が直接含まれ、漏えい時の影響範囲が大きい。

**要件達成後**: Pii API（FR-T1-PII-001、002）でマスキング・仮名化を強制する。Log API で `pii:true` 属性のフィールドは自動マスキング。

**崩れた時**: PII 漏えいで個人情報保護法違反、漏えい等報告義務発生。

**受け入れ基準**:

- Log / Audit ログに PII 生値が残らない
- 定期監査（四半期）で実態検証

**優先度**: MUST

## E5. 不正追跡・監視

### NFR-E-MON-001: 全特権操作の Audit 記録

**現状**: 特権操作の証跡が散在すると、不正検知が困難。

**要件達成後**: 以下を全て Audit API に記録する。

- tier1 API 呼び出し（認可成功・失敗）
- Secret 取得・ローテーション
- Decision 評価
- Workflow 実行
- Binding 操作（外部送信）
- Feature Flag 変更
- Backstage 管理機能実行
- Argo CD 同期（特に本番 prod）
- OpenBao 秘密更新
- k8s RBAC 変更

**崩れた時**: 不正検知が属人的となり、内部不正が長期間検知されない。

**受け入れ基準**:

- Audit ログの網羅率 100%（リリース時点）
- ハッシュチェーンで改ざん検知

**優先度**: MUST

### NFR-E-MON-002: Secret 取得の監査

**現状**: Secret 取得のログが薄いと、漏えい発生時の影響範囲特定が困難。

**要件達成後**: OpenBao Audit Device を有効化、全 Secret 取得を Audit API に転送する。異常パターン（短時間の大量取得、通常時間外の取得）を Prometheus で検知、Severity 2 アラート。

**崩れた時**: Secret 漏えい時の経路特定ができず、影響範囲が拡大する。

**受け入れ基準**:

- OpenBao Audit Device 有効化
- 異常パターン検知ルールを リリース時点 で整備

**優先度**: MUST

### NFR-E-MON-003: 操作ログの可視化

**現状**: Audit ログが記録されていても、可視化がなければ運用に使えない。

**要件達成後**: Grafana で「誰がいつ何をしたか」を可視化するダッシュボードを提供する。tenant_id / user_id / action 別フィルタ、異常パターンのアラート設定を含む。

**崩れた時**: 監査対応で都度ログ抽出の手作業が発生し、対応リードタイムが長期化する。

**受け入れ基準**:

- Grafana ダッシュボード 2 本（監査者用、運用者用）
- Audit API のクエリ API（FR-T1-AUDIT-002 連携）

**優先度**: MUST

### NFR-E-MON-004: Feature Flag / 決定表変更の監査

**現状**: ランタイム設定の変更が監査されないと、不正な業務ルール変更を検知できない。

**要件達成後**: Feature Flag と Decision 決定表の変更は Git commit + Audit 記録の両方で残す。Backstage で変更履歴を可視化。

**崩れた時**: 業務ルールの不正改ざんで採用検討が無効化される。

**受け入れ基準**:

- 全変更が Git と Audit の両方に残る
- 変更者と承認者が記録される

**優先度**: MUST

## E6. ネットワーク対策

### NFR-E-NW-001: 外部 URL allowlist

**現状**: tier2 アプリが自由に外部 URL にアクセスできると、SSRF 攻撃や不正なデータ流出のリスクがある。

**要件達成後**: Binding API（FR-T1-BINDING-003）で外部 URL の allowlist を tenant 単位で管理、allowlist 外は `K1s0Error.Forbidden` で拒否。Istio AuthorizationPolicy で egress 制御。

**崩れた時**: SSRF 攻撃の踏み台、意図しない外部データ送信が発生する。

**受け入れ基準**:

- allowlist Component YAML の整備
- egress トラフィックの Prometheus 可視化

**優先度**: MUST

### NFR-E-NW-002: NetworkPolicy によるテナント隔離

**現状**: k8s のフラットネットワークで tenant 間通信が自由に行えると、越境リスクが残る。

**要件達成後**: Kyverno で tenant-id ラベル必須化、NetworkPolicy で既定拒否 + tier1 公開 API のみ明示許可。Istio AuthorizationPolicy で L7 SPIFFE ID 検証。

**崩れた時**: tenant 越境通信で情報漏えい、コンプライアンス違反。

**受け入れ基準**:

- Kyverno ポリシーで未ラベル Pod 作成を拒否
- NetworkPolicy のデフォルト拒否
- リリース時点で 実装 / Chaos Engineering 検証

**優先度**: MUST

### NFR-E-NW-004: レート制限と接続数上限（DoS 対策）

**現状**: 呼出側の暴走（再試行嵐、バグループ、外部 DDoS）が tier1 API を飽和させると、正常トラフィックまで巻き込まれる。

**要件達成後**: 以下 3 層でレート制限と接続数上限を強制する。

- 外部境界（Envoy Gateway + WAF）: テナント単位の RPS 上限、同一 IP 同時接続数、Slowloris 検知
- サービス間（Istio Ambient）: 呼出元 SPIFFE ID 単位の RPS クォータ、Circuit Breaker
- バックエンド（Valkey / PostgreSQL）: Connection Pool 上限、Slow Query キル

超過時は `K1s0Error.ResourceExhausted` を返す。テナント単位クォータは 60_事業契約/06_課金メータリング で定義された契約プラン値と連動する。

**崩れた時**: 1 テナントの暴走で全テナントがダウン（ノイジーネイバー）、または外部 DDoS で SLA を毀損する。STRIDE DoS 脅威（T0→T1、T1→T2、T2→T3）の緩和策が不在となり、脅威モデリング（NFR-E-RSK-001）の想定と乖離する。

**受け入れ基準**:

- テナント RPS 上限を Backstage で確認可能
- Chaos 試験（リリース時点）で 10 倍負荷時に 1 テナントのみ 429 を返し他テナント正常稼働を確認
- Circuit Breaker 発動を Grafana で可視化、5 分超の継続発動は Sev3 起票

**優先度**: MUST（リリース時点、STRIDE DoS 脅威の唯一の緩和策）

### NFR-E-NW-003: 外部境界遮断（AGPL OSS 対応）

**現状**: AGPL-3.0 OSS（Grafana / Loki / Tempo / Pyroscope / MinIO / Renovate）はネットワーク経由の提供がソース開示義務を発生させる。

**要件達成後**: AGPL OSS は社内限定で提供し、外部ネットワーク遮断とプロセス分離を維持する。Istio Gateway で外部アクセスを拒否、内部のみ allowlist で許可。これにより「無改変利用 + 内部限定」として義務発生を回避（法務サマリ [../../01_企画/05_法務サマリ/OSSライセンス適合.md](../../01_企画/05_法務サマリ/01_OSSライセンス適合.md) に従う）。

**崩れた時**: AGPL 義務違反でソース開示要求が発生、採用検討で約束した法務適合が崩れる。

**受け入れ基準**:

- AGPL OSS が外部ネットワークからアクセス不可
- プロセス分離（k1s0 本体と別プロセス）を維持
- 年次で法務部門レビュー

**優先度**: MUST

## E7. マルウェア対策

### NFR-E-AV-001: コンテナイメージスキャン

**現状**: 脆弱性を含むイメージのデプロイが検知されないと、既知脆弱性の放置リスク。

**要件達成後**: CI で Trivy によるイメージスキャンを必須化（リリース時点）。Critical CVE は PR ブロック。採用後の運用拡大時、Cosign 署名と Kyverno 強制（署名なしイメージのデプロイ拒否）を追加。

**崩れた時**: 脆弱性を含むイメージが本番稼働し、侵害の起点となる。

**受け入れ基準**:

- 全 PR で Trivy スキャン実行
- Critical / High は原則 PR ブロック（例外は記録）
- 採用後の運用拡大時 で Cosign + Kyverno

**優先度**: MUST

### NFR-E-AV-002: SBOM と依存関係追跡

**現状**: SBOM なしでは脆弱性発覚時の影響範囲特定が困難。

**要件達成後**: リリース時点 で SBOM 生成率 100%（NFR-C-MGMT-003 と整合）。CycloneDX / SPDX 形式で CI 自動生成、Backstage で一覧表示。

**崩れた時**: CVE 発覚時の対応リードタイムが長期化、監査指摘。

**受け入れ基準**:

- CI の SBOM 生成必須
- Backstage で SBOM 検索可能

**優先度**: MUST

## E8. Web 対策

### NFR-E-WEB-001: OWASP Top 10 対策

**現状**: tier3 アプリで標準的な Web 脆弱性対策が徹底されないと、XSS / SQLi / CSRF 等のリスク。

**要件達成後**: tier3 アプリ向けのセキュリティガイドラインを tier1 が提供する。OWASP Top 10 対策（入力検証、出力エスケープ、SQL パラメタライズ、CSRF トークン、Content Security Policy）を SDK ヘルパまたはドキュメントとして提供。

**崩れた時**: tier3 アプリが個別に Web 脆弱性を抱え、エンドユーザー体験と社内データが危険に晒される。

**受け入れ基準**:

- tier3 向けセキュリティガイドライン文書（リリース時点）
- SDK ヘルパ（CSRF トークン、CSP ヘッダ設定等）

**優先度**: SHOULD

### NFR-E-WEB-002: WAF（Web Application Firewall）

**現状**: 既知攻撃パターンへの防御が無いと、bot 攻撃で負荷増大。

**要件達成後**: Envoy Gateway + WAF プラグイン（ModSecurity 等）で OWASP Core Rule Set を適用する。採用側のマルチクラスタ移行時で検討。

**崩れた時**: bot 攻撃で業務影響、DDoS 耐性不足。

**受け入れ基準**:

- 採用側のマルチクラスタ移行時 で WAF 導入判定
- 優先度 SHOULD

**優先度**: SHOULD（採用側のマルチクラスタ移行時+）

## E9. セキュリティインシデント対応・復旧

### NFR-E-SIR-001: インシデント対応 Runbook

**現状**: セキュリティインシデント対応が属人的で、対応遅延と情報混乱が発生する。

**要件達成後**: 以下のインシデント分類別に Runbook を整備する（NFR-A-REC-002 の 15 本に含む）。

- Severity 1: テナント越境検知、秘密情報漏えい、DDoS
- Severity 2: 不正ログイン試行、異常な Secret 取得パターン
- Severity 3: 設定ミス、軽微な認可違反

**崩れた時**: 対応リードタイム長期化、対応品質のばらつき、ポストモーテムで教訓が蓄積しない。

**受け入れ基準**:

- Runbook 15 本のうち 5 本以上がセキュリティ関連
- 訓練（年 2 回）で Runbook 実行確認

**優先度**: MUST

### NFR-E-SIR-002: 漏えい等報告義務対応

**現状**: 個人情報保護法（2022 年改正）の漏えい等報告義務（3 日以内速報、30 日以内確報）への対応フローが曖昧。

**要件達成後**: 漏えい検知から 24 時間以内に情シスマネージャと監査担当に通知、72 時間以内に個人情報保護委員会への速報、30 日以内に確報を提出するフローを Runbook 化する。対象データの特定は Audit API + tenant_id 検索で支援。

**崩れた時**: 報告期限超過で法令違反、監督官庁からの指導。

**受け入れ基準**:

- 漏えい対応 Runbook（Runbook-SEC-LEAK-001）
- 影響範囲特定の標準クエリテンプレート

**優先度**: MUST

### NFR-E-SIR-003: フォレンジック証跡保全

**現状**: インシデント発生時に証跡が上書きされ、原因特定と法的対応が困難。

**要件達成後**: Severity 1 インシデント発生時は Audit ログ、Pod ログ、メモリダンプを 90 日以上保全する。MinIO Object Lock で改ざん拒否。フォレンジック手順を Runbook 化。

**崩れた時**: 法的対応・監査対応で証跡不足、信頼失墜。

**受け入れ基準**:

- 証跡保全の Runbook
- MinIO Object Lock 設定
- 優先度 MUST、リリース時点 で整備

**優先度**: MUST

## サマリ

| ID | タイトル | 適用段階 | 優先度 |
|---|---|---|---|
| NFR-E-PRE-001 | 準拠規格明示 | 採用初期 | MUST |
| NFR-E-RSK-001 | STRIDE 脅威モデリング | 採用初期 | MUST |
| NFR-E-RSK-002 | ペネトレーションテスト | 採用後の運用拡大時 | MUST |
| NFR-E-AC-001 | JWT 認証強制 | 採用初期 | MUST |
| NFR-E-AC-002 | RBAC | 採用初期 | MUST |
| NFR-E-AC-003 | tenant_id 検証 | 採用初期 | MUST |
| NFR-E-AC-004 | Secret 最小権限 | 採用初期 | MUST |
| NFR-E-AC-005 | MFA | 3 | MUST |
| NFR-E-ENC-001 | 通信暗号化 | 採用初期 | MUST |
| NFR-E-ENC-002 | データ保管暗号化 | 採用初期 | MUST |
| NFR-E-ENC-003 | PII マスキング | 採用初期 | MUST |
| NFR-E-MON-001 | 特権操作 Audit | 1a/1c | MUST |
| NFR-E-MON-002 | Secret 取得監査 | 採用初期 | MUST |
| NFR-E-MON-003 | Audit 可視化 | 採用初期 | MUST |
| NFR-E-MON-004 | Flag/Decision 変更監査 | 採用初期 | MUST |
| NFR-E-NW-001 | 外部 URL allowlist | 採用初期 | MUST |
| NFR-E-NW-002 | NetworkPolicy 隔離 | 採用初期 | MUST |
| NFR-E-NW-003 | AGPL 外部境界遮断 | 採用初期 | MUST |
| NFR-E-NW-004 | レート制限・接続数上限（DoS 対策） | 採用初期 | MUST |
| NFR-E-AV-001 | イメージスキャン | 1a/2 | MUST |
| NFR-E-AV-002 | SBOM | 採用後の運用拡大時 | MUST |
| NFR-E-WEB-001 | OWASP Top 10 | 採用初期 | SHOULD |
| NFR-E-WEB-002 | WAF | 3 | SHOULD |
| NFR-E-SIR-001 | インシデント Runbook | 1b/1c | MUST |
| NFR-E-SIR-002 | 漏えい報告義務 | 採用後の運用拡大時 | MUST |
| NFR-E-SIR-003 | フォレンジック | 採用後の運用拡大時 | MUST |
