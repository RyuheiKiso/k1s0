# SEC-SEC: セキュリティ基本要件

本ファイルは、k1s0 プラットフォームの**ネットワーク境界・Pod 実行境界・脆弱性管理**の 3 層を横断する最低線を定義する。IAM（`01_IAM.md`）、鍵管理（`02_kms_crypto.md`）、監査（`03_audit.md`）など個別領域の前提となる土台であり、ここが崩れると上位のあらゆる統制が空転する。

JTC 情シス・監査部門が稟議で最初に問うのは「**誰が誰と通信でき、誰が何のコードを実行でき、脆弱性を何日で塞ぐのか**」の 3 点である。本ファイルは ADR-0001 で採択した Istio Ambient Mesh による透過的 mTLS を前提に、Pod Security Standard Restricted、デフォルト deny の NetworkPolicy、CI の CVE スキャンゲート、Secret 平文禁止、脆弱性対応 SLA、侵入検知、ログ集約の 8 領域を要件化する。

本カテゴリの制約に従い、全要件で受け入れ基準を**検知可能性 / 防御可能性 / 復旧可能性**の 3 軸で組み立てる。漏洩・侵害は事後復旧が極端に難しいため、「起きないこと」の保証ではなく「起きたら気づけること」「起きても拡がらないこと」を優先する。

---

## 前提

- [`../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md`](../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md) Istio Ambient Mesh 採用
- [`../00_共通/03_constraint.md`](../00_共通/03_constraint.md) COM-CON-002 / COM-CON-005
- [`../00_共通/04_risk.md`](../00_共通/04_risk.md) COM-RSK-006（OSS EOL）
- [`../10_アーキテクチャ/00_infra.md`](../10_アーキテクチャ/00_infra.md) Kubernetes 基盤
- [`01_IAM.md`](./01_IAM.md) 認証・認可（上位統制）
- [`08_artifact_integrity.md`](./08_artifact_integrity.md) 供給チェーン（CVE スキャンの入口）

---

## 要件本体

### SEC-SEC-001: クラスタ内通信は mTLS 必須（Istio Ambient ztunnel）

- 優先度: MUST（未暗号化の内部通信は改正個人情報保護法 第 23 条「安全管理措置」違反となり稟議通過不可）
- Phase: Phase 1a
- 関連: ADR-0001、`COM-CON-002`、`SEC-IAM-001`、`QUA-AVL-*`

現状、アプリ本体に mTLS を実装する方式では各言語の TLS ライブラリ差異・証明書期限切れ事故・CA 管理の属人化が発生する。Dapr の mTLS と Istio の mTLS が二重に走ると ADR-0001 で整理した 5 つの構造的衝突を引き起こす。

本要件が満たされた世界では、ztunnel (DaemonSet) が HBONE トンネルで Pod-to-Pod の L4 mTLS を透過的に担い、アプリ開発者は TLS を意識せずに平文のように書ける。証明書ローテは Istio のコントロールプレーンが担当し、アプリ起動時に証明書期限切れが発覚する事故が構造的に起こらない。

崩れた場合、内部通信の一部が平文になると tcpdump レベルの侵入で業務データ・認証トークン・監査イベントが抜き取られる。改正個人情報保護法第 26 条の漏えい等報告義務（速やかに個人情報保護委員会に報告）が発動し、JTC の信用失墜と顧客通知コストが直接発生する。

**受け入れ基準**

- 検知: Istio Telemetry で mTLS 適用率 100% が Grafana ダッシュボードに日次表示される
- 防御: `PeerAuthentication` が全 namespace で `mtls.mode: STRICT` を強制し、平文を受理しない
- 防御: ztunnel の証明書有効期限は 24 時間以下、自動ローテが 6 時間前に起動する
- 復旧: 証明書失効事故時、手動再発行手順（30 分以内の復旧）が Runbook 化されている

**検証方法**

- Kyverno ポリシーで `PeerAuthentication` の存在と `STRICT` 設定を CI 拒否ゲート化
- Phase 1a POC 時に平文 HTTP で `curl` を打ち、通信が拒否されることを記録
- Istio `istioctl authn tls-check` を週次で全サービスに対して実行し監査ログに記録

---

### SEC-SEC-002: Pod Security Standard Restricted を全ワークロード namespace に強制

- 優先度: MUST（特権コンテナ経由のクラスタ脱出は一撃で全テナントデータが抜ける）
- Phase: Phase 1a
- 関連: `COM-CON-007`、`SEC-ART-003`

現状、`privileged: true` や `hostPath` マウントを許容する namespace があると、単一 Pod の脆弱性がクラスタ全体へ横展開する。Kubernetes 1.25 以降の Pod Security Admission (PSA) は宣言的に強制できるが、デフォルトは `privileged`（何でも許可）である。

本要件が満たされた世界では、アプリ稼働 namespace はすべて `pod-security.kubernetes.io/enforce: restricted` が付与され、root 実行禁止・全 capability 剥奪・RuntimeDefault Seccomp 強制が admission で弾かれる。tier3 の野良 Deployment が混入しても、PSA が最後の砦として機能する。

崩れた場合、1 つの Pod が root 化された瞬間に Linux capability 経由で他 Pod のファイルディスクリプタが覗けてしまい、テナント分離（`QUA-TNT-*`）が成立しない。

**受け入れ基準**

- 検知: `kubectl get ns -o json` で全ワークロード namespace に `restricted` ラベルがあることを日次確認
- 防御: `restricted` 違反の Pod は admission で CreateContainer が拒否される
- 防御: 例外は `privileged-system` のような明示的 namespace に限定し、その namespace へのデプロイは Keycloak の特権ロール承認を要する
- 復旧: 違反が検知されたら当該 Pod を即時停止し、Slack ChatOps で通知

**検証方法**

- Kyverno / Gatekeeper で PSA 違反を block し、テストケース（privileged Pod）の作成拒否を CI で検証
- Falco で `runAsUser=0` の実行ログを収集し週次レポート

---

### SEC-SEC-003: CVE スキャンを CI 必須ゲートにする（High / Critical をブロック）

- 優先度: MUST（既知脆弱性の放置は FISC 安全対策基準 7.6 系に抵触）
- Phase: Phase 1a
- 関連: `SEC-ART-001`、`COM-RSK-006`、`OPS-CICD-*`

現状、JTC の多くのプロジェクトは月次 CVE レビューを運用しているが、ビルド時点で検知できないため、既に本番で数週間稼働した後で脆弱性が発覚する。対応期間が 1 ヶ月を超えることも珍しくない。

本要件が満たされた世界では、全コンテナイメージが Trivy と Grype の両方でスキャンされ、High / Critical の新規検出があれば PR マージがブロックされる。Fixable な CVE は自動 PR で Dependabot / Renovate が差分を出す。

崩れた場合、Log4Shell クラスの 0-day に対して SLA（72 時間以内パッチ）を守れず、個人情報保護委員会への事故報告義務に発展する。

**受け入れ基準**

- 検知: 全イメージの SBOM・CVE 結果が Harbor に自動保存され、検索可能
- 防御: Critical (CVSS ≥ 9.0) は PR マージ絶対拒否、High (7.0〜8.9) はレビュアー 2 名の承認を要する
- 防御: Renovate が Fixable CVE を 24 時間以内に PR 起票
- 復旧: Critical 検知から本番パッチ適用までの SLA は 72 時間（業界平均 30 日を短縮、FISC 安対基準 準則 7.6「情報セキュリティに関するシステムの整備」の趣旨に沿う）

**検証方法**

- GHA ワークフローで意図的に脆弱性含みのイメージを push し、block されることを確認
- 月次で CVE 検知→パッチの平均所要時間を計測し 72 時間を超えた案件を棚卸し

---

### SEC-SEC-004: NetworkPolicy は default-deny、必要経路を明示 allowlist

- 優先度: MUST（テナント越境通信の構造的防止）
- Phase: Phase 1a
- 関連: `QUA-TNT-*`、`SEC-RES-*`

現状、Kubernetes の NetworkPolicy はデフォルトで全通信許可であり、`tier3-mynumber` のような機微 namespace に対する意図せぬ通信が発生しうる。

本要件が満たされた世界では、全 namespace で `default-deny-ingress` と `default-deny-egress` の 2 枚が自動付与され、必要経路のみが NetworkPolicy で個別許可される。Istio AuthorizationPolicy が L7 の許可制御を重ねる二層防御となる。

崩れた場合、tier3 のアプリが誤って tier1 control plane に直接通信する、あるいはマイナンバー namespace へ一般業務 Pod からアクセスが発生するなど、ゼロトラストの前提が根底から崩れる。

**受け入れ基準**

- 検知: NetworkPolicy 欠落 namespace の日次スキャンレポート
- 防御: 新規 namespace 作成時に default-deny 2 枚が admission で自動注入される
- 防御: Istio `AuthorizationPolicy` と NetworkPolicy の両方を設けることで L4/L7 二層防御
- 復旧: ポリシー誤設定で業務停止した場合、最後の正常 YAML へのロールバック（5 分以内）が Runbook 化

**検証方法**

- Phase 1a で疎通テスト matrix（from-ns × to-ns）を自動実行し、許可経路以外がすべて拒否されることを確認

---

### SEC-SEC-005: Secret の平文 ConfigMap / 環境変数 / git 格納の禁止

- 優先度: MUST（git 履歴への漏洩は事実上回収不可能）
- Phase: Phase 1a
- 関連: `SEC-KMS-*`、`SEC-IAM-003`

現状、多くの JTC プロジェクトで API キーやパスワードが ConfigMap・環境変数・`.env` として git に紛れ込む事故が後を絶たない。git 履歴からの除去は事実上不可能で、鍵ローテが唯一の対処となる。

本要件が満たされた世界では、全 Secret は OpenBao から External Secrets Operator (ESO) 経由で注入され、git には参照 ID のみが置かれる。gitleaks / trufflehog が pre-commit と CI で秘密値混入を検知する。

崩れた場合、git push 後に平文鍵が外部 Fork された瞬間、鍵ローテと影響範囲調査で 1 週間級のインシデント対応が発生する。

**受け入れ基準**

- 検知: gitleaks が PR 単位で全コミットをスキャン、検知時は PR ブロック
- 防御: `kubectl get configmap --all-namespaces -o yaml | grep -iE "password|secret|token"` が常に空集合
- 防御: 環境変数への直接埋め込みは Kyverno で拒否、全 Secret は ESO 経由
- 復旧: 万一漏洩した場合、該当鍵の OpenBao ローテを 15 分以内に完了する手順が Runbook 化

**検証方法**

- gitleaks を pre-commit と GHA の両段で実行
- 四半期ごとに全 git 履歴を trufflehog で fullscan

---

### SEC-SEC-006: 脆弱性対応 SLA を重大度別に定義

- 優先度: MUST（対応遅延は監査指摘の頻出項目）
- Phase: Phase 1a
- 関連: `SEC-SEC-003`、`COM-RSK-006`

現状、脆弱性対応の期限が曖昧だと、優先度判断が属人化し、Critical と Low が同じ扱いを受ける運用に陥る。

本要件が満たされた世界では、CVSS スコアに応じた対応 SLA が明文化され、逸脱はインシデントとして記録される。ISMAP / FISC 安対基準のセキュリティ監査に対して即時回答できる。

崩れた場合、監査人から「脆弱性管理体制が整備されていない」と指摘を受け、ISMAP 登録に影響する。

**受け入れ基準**

- 検知: 全 CVE の検知日時と対応完了日時が Harbor + Backstage に記録される
- 防御: Critical=72 時間 / High=7 日 / Medium=30 日 / Low=90 日 の SLA（業界平均 Critical=30 日を大幅短縮、FISC 安対基準 7.6 準則の趣旨）
- 復旧: SLA 逸脱時はインシデント起票、月次で経営報告

**検証方法**

- Backstage Software Catalog と CVE DB を連携し SLA 逸脱を dashboard 化

---

### SEC-SEC-007: 侵入検知（IDS/Runtime Security）を全 Node に導入

- 優先度: SHOULD（Phase 1a はログ検知のみ、Phase 1b で Falco 本格稼働）
- Phase: Phase 1b
- 関連: `SEC-AUD-*`、`QUA-OBS-*`

現状、Pod 内で execve された怪しいコマンド（`curl` → `sh` パイプなど）は検知されず、横展開の兆候に気づけない。

本要件が満たされた世界では、Falco が eBPF 経由で syscall を監視し、Falco Rules に基づき「shell 起動」「機微ファイルアクセス」「network 外向き異常」などを検知・通知する。

崩れた場合、Supply chain 攻撃のポストエクスプロイトが数週間検知されず、ラテラルムーブメントで全テナント侵害に至る。

**受け入れ基準**

- 検知: Falco のアラートが Alertmanager 経由で SOC 通知される
- 防御: Falco sidekick と AuditSink を統合し監査ログ（`SEC-AUD-*`）と突き合わせ可能
- 復旧: 検知から自動隔離（NetworkPolicy 強化）まで 10 分以内

**検証方法**

- Atomic Red Team のシナリオを Phase 1b 終了時に実行し検知率を計測

---

### SEC-SEC-008: セキュリティログの集約と相関分析

- 優先度: SHOULD（Phase 1a は kubectl logs / journald、Phase 1b で統合）
- Phase: Phase 1b
- 関連: `SEC-AUD-*`、`QUA-OBS-*`

現状、kube-apiserver audit log、Istio アクセスログ、Falco アラート、Keycloak イベントが分散保管され、インシデント対応時の手動つき合わせで初動が遅れる。

本要件が満たされた世界では、Loki / Grafana で統一クエリでき、Keycloak の不審ログイン・Istio の異常通信・Falco の syscall 異常を時系列で相関分析できる。

崩れた場合、インシデント対応の初動が数時間遅れ、攻撃者の滞留時間（dwell time）が伸びて被害範囲が拡大する。

**受け入れ基準**

- 検知: 4 種類のログが Loki の同一 tenant に集約される
- 防御: 保管期間は監査ログと同等（7 年）、WORM 格納
- 復旧: インシデント時の単一ダッシュボード（Grafana）で相関可能

**検証方法**

- Phase 1b でテーブルトップ演習を実施、ログ相関で初動時間 30 分以内を計測

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| SEC-SEC-001 | mTLS 必須（Istio Ambient） | MUST | 1a |
| SEC-SEC-002 | Pod Security Standard Restricted | MUST | 1a |
| SEC-SEC-003 | CVE スキャン CI ゲート | MUST | 1a |
| SEC-SEC-004 | NetworkPolicy default-deny | MUST | 1a |
| SEC-SEC-005 | Secret 平文禁止 | MUST | 1a |
| SEC-SEC-006 | 脆弱性対応 SLA | MUST | 1a |
| SEC-SEC-007 | 侵入検知（Falco） | SHOULD | 1b |
| SEC-SEC-008 | セキュリティログ集約 | SHOULD | 1b |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 6 | SEC-SEC-001, 002, 003, 004, 005, 006 |
| SHOULD | 2 | SEC-SEC-007, 008 |

### Phase 達成度

| Phase | 必達件数 | 未達影響 |
|---|---|---|
| 1a | 6 | 稟議通過不可。改正個人情報保護法 第 23 条違反 |
| 1b | 2 | 侵入検知なしで Phase 2 本番運用は FISC 安対基準に抵触 |
