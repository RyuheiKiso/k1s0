# 周辺 OSS の選定

## 目的

実行基盤中核 ([`01_実行基盤中核OSS.md`](../02_中核OSS/01_実行基盤中核OSS.md)) の周辺で必須となる OSS の選定根拠を整理する。対象カテゴリは以下のとおり。

- A. ID / 認証基盤
- B. GitOps / 継続的デリバリ
- C. CI / パイプライン
- D. コンテナレジストリ / 脆弱性スキャン
- E. キャッシュ / KV ストア
- F. RDBMS
- G. ポリシーエンジン / Admission Controller
- H. 証明書管理
- I. オブジェクトストレージ
- J. Feature Flag
- K. Chaos Engineering
- L. 分散ストレージ
- M. 接続プーリング
- N. 依存パッケージ自動更新
- O. イベント駆動自動化

ルールエンジン (BRE) は [`03_ルールエンジン.md`](../02_中核OSS/03_ルールエンジン.md)、イベントスキーマレジストリは [`06_イベントスキーマレジストリ.md`](06_イベントスキーマレジストリ.md)、ローカル開発ツールは [`../../05_CICDと配信/04_ローカル開発環境.md`](../../05_CICDと配信/04_ローカル開発環境.md) でそれぞれ別扱いとする。

追加採用 OSS (ストレージ / 接続プーリング / 依存管理 / イベント自動化) は [`07_ストレージと運用補助.md`](07_ストレージと運用補助.md) で別扱いとする。

---

## A. ID / 認証基盤

| 候補 | 採否 | 評価 |
|---|---|---|
| **Keycloak** | 採用 | Red Hat 発の OSS。OIDC / SAML / LDAP / ソーシャルログイン / 2FA を標準装備。Admin UI が成熟 |
| Zitadel | 次点 | Go 製でモダン。マルチテナント / イベントソーシング設計。日本語情報が少ない |
| Authentik | 次点 | Python 製。管理 UI は洗練されているが Keycloak ほどの実績はない |
| Dex | 却下 | OIDC のフロントに特化。ユーザー DB を持たないため MVP 単独採用は不可 |
| ORY Hydra + Kratos | 却下 | 柔軟性は高いが複数コンポーネントの組み合わせが必要で JTC 運用に不向き |

### 採用理由 (Keycloak)

1. **ユーザー DB を自前で持てる** — MVP では AD 連携なしで Keycloak ローカル DB を認証源とする
2. **将来の AD / LDAP フェデレーションを追加設定のみで実現可能** — Phase 2 以降で JTC 既存 AD を取り込む退路を確保
3. **Envoy Gateway の ext_authz 相当として oauth2-proxy 経由で連携可能**
4. **OSS 版と商用版の機能差がない** — Red Hat Build of Keycloak は商用サポート契約のみで機能差異なし
5. **日本語情報 / 事例が豊富**

### MVP スコープ

- Keycloak を Realm = `k1s0` で 1 つ立ち上げる
- ユーザーは Keycloak ローカル DB で直接管理 (AD 連携なし)
- OIDC クライアントを tier1 / アプリ配信ポータル / Backstage / Argo CD / Harbor の各コンポーネント向けに登録

### トレードオフ

- ローカル DB 運用の煩雑さ → Phase 2 以降で AD 連携に切替える前提で、MVP 中はユーザー数を絞る
- 高可用構成の難しさ → Keycloak は PostgreSQL バックエンド + 複数レプリカ構成が推奨。`infra` 層の CloudNativePG 共有クラスタ上に `keycloak` DB を作成し HA を確保 (セクション F 参照)

---

## B. GitOps / 継続的デリバリ

| 候補 | 採否 | 評価 |
|---|---|---|
| **Argo CD** | 採用 | CNCF Graduated。Web UI が成熟。Backstage 連携プラグイン公式提供あり |
| Flux CD | 次点 | CNCF Graduated。CLI / GitOps 純度重視。UI 標準装備なし |
| Argo CD + Argo Rollouts | 併用候補 | カナリア / Blue-Green リリース。Phase 2 以降で追加検討 |
| Spinnaker | 却下 | 機能豊富だが重量級 |
| Jenkins X | 却下 | Jenkins 依存。k1s0 の軽量指向と整合しない |

### 採用理由 (Argo CD)

1. **Web UI によるデプロイ可視化** — JTC 情シスの GitOps 学習曲線を緩和
2. **Backstage Argo CD プラグインが公式提供** — Software Catalog からデプロイ状態を直接確認可能
3. **ApplicationSet による tier 単位の一括管理** — tier1 / tier2 / tier3 を別々の ApplicationSet で管理
4. **マルチクラスタ対応** — 将来の本番 / ステージング分離に対応
5. **Keycloak OIDC 連携が標準装備**

### MVP スコープ

- Argo CD を `operation` namespace にデプロイ
- tier1 / tier2 / tier3 それぞれに ApplicationSet を 1 つずつ
- Git リポジトリは GitHub.com の単一リポジトリ (monorepo) で開始
- Backstage の Argo CD プラグインを有効化

---

## C. CI / パイプライン

| 候補 | 採否 | 評価 |
|---|---|---|
| **GitHub Actions (self-hosted runner)** | 採用 (主) | 基本の CI エンジン。PR / テスト / ビルド / スキャン / デプロイまで 1 本のワークフローで完結 |
| **Tekton** | 採用 (代替) | GHA が使えない環境向けのフォールバック。完全エアギャップ / GitHub.com 到達不可時に利用 |
| Jenkins | 却下 | 実績は豊富だが YAML / Groovy / プラグイン管理が属人化 |
| Drone / Woodpecker | 却下 | 軽量だが GitHub Actions ほどのワークフロー資産がない |
| GitLab CI | 対象外 | GitHub.com 前提のため不適 |
| Argo Workflows | 却下 | Tekton と競合。Tekton の方が CI 特化で扱いやすい |

### 採用理由 (GHA を主、Tekton を代替)

1. **原則 GHA に一本化** — PR / Issue / Checks が GitHub 上で完結し、開発者の認知負荷が最小
2. **self-hosted runner を k8s 上で起動** — オンプレ k8s クラスタで GHA ジョブを実行 (`actions-runner-controller` で宣言的管理)
3. **ビルド / スキャン / デプロイも GHA で完結** — Kaniko / Trivy / `crane` / `argocd` CLI を GHA step から呼び出し
4. **Tekton は同等パイプラインを k8s ネイティブで提供する退路** — MVP ではインストールしない
5. **どちらも OSS** — runner は Apache 2.0、Tekton は CNCF Graduated

詳細なパイプライン設計は [`../../05_CICDと配信/00_CICDパイプライン.md`](../../05_CICDと配信/00_CICDパイプライン.md) を参照。

---

## D. コンテナレジストリ / 脆弱性スキャン

### レジストリ

| 候補 | 採否 | 評価 |
|---|---|---|
| **Harbor** | 採用 | CNCF Graduated。イメージ管理 / 脆弱性スキャン / 署名 / レプリケーションを統合 |
| Zot | 次点 | 軽量 OCI ネイティブ。機能は Harbor より絞られる |
| Nexus Repository | 却下 | 汎用アーティファクトリポジトリ。OSS 版は機能制限あり |
| Quay | 却下 | Red Hat 製。Project Quay はコミュニティが小さい |

### 脆弱性スキャン

| 候補 | 採否 | 評価 |
|---|---|---|
| **Trivy (Harbor 内蔵)** | 採用 | Harbor が標準スキャナとして同梱 |
| Clair | 次点 | Harbor で選択可だが Trivy のコミュニティがより活発 |
| Grype | 却下 | 単体実績はあるが Harbor との統合が Trivy ほど深くない |

### 採用理由 (Harbor + 内蔵 Trivy)

1. レジストリとスキャンが 1 つの製品に統合されている
2. Keycloak OIDC 連携が標準装備
3. RBAC とプロジェクト分離 (`tier1` / `tier2` / `tier3` / `infra`)
4. CVE 検知時の push 拒否ポリシーを設定可能
5. イメージ署名 (Cosign / Notation) のサポート

### MVP スコープ

- Harbor を `infra` namespace にデプロイ
- プロジェクトは `tier1` / `tier2` / `tier3` / `infra` の 4 つ
- Trivy スキャンを push 時に自動実行、Critical 以上を検出したら push 拒否
- イメージ署名は Phase 2 以降 (Cosign 導入時に追加)

---

## E. キャッシュ / KV ストア

| 候補 | 採否 | 評価 |
|---|---|---|
| **Valkey** | 採用 | Linux Foundation 傘下、BSD-3 ライセンス。Redis 7.2.4 からのフォークで完全互換 |
| Redis 7.4+ | 却下 | RSALv2 / SSPL デュアルライセンス。OSI 承認 OSS ではない |
| KeyDB | 却下 | Redis フォークだが Snap Inc. 買収後にコミュニティ活動が鈍化 |
| DragonflyDB | 却下 | BSL ライセンス。OSS ではない |
| etcd | 別用途 | k8s の内部状態管理が主。汎用キャッシュ用途には不向き |

### 採用理由 (Valkey)

1. **ライセンスが OSI 承認の BSD-3** — OSS 積み上げ原則と完全整合
2. **Redis 7.2.4 からのフォークで wire protocol / コマンド / クライアント完全互換** — Dapr State Store の Redis Component をそのまま利用可能
3. **Linux Foundation 傘下でコミュニティが活発** — AWS / Google / Oracle / Ericsson が支援
4. **既存 Redis 利用ノウハウがそのまま活かせる**

### MVP スコープ

- Valkey を tier1 の Dapr State Store / Cache のバックエンドとして採用
- tier2 / tier3 は Valkey / Redis のどちらも直接意識しない (`k1s0.State` / `k1s0.Cache` 経由のみ)
- 将来の差し替えが tier1 内部で閉じる

---

## F. RDBMS

| 候補 | 採否 | 評価 |
|---|---|---|
| **CloudNativePG + PostgreSQL** | 採用 | CNCF Sandbox の k8s ネイティブ PostgreSQL Operator。HA / 自動フェイルオーバー / バックアップを宣言的に管理 |
| Percona Operator for PostgreSQL | 次点 | 機能豊富だが CloudNativePG よりコミュニティ規模が小さい |
| Zalando postgres-operator | 次点 | 実績あるが CNCF 外。CloudNativePG の方がエコシステム整合性が高い |
| CrunchyData PGO | 次点 | 商用版との機能差がやや不透明 |
| MySQL (各種 Operator) | 却下 | PostgreSQL の方が Keycloak / Backstage / Dapr Component との親和性が高い |

### 採用理由 (CloudNativePG + PostgreSQL)

1. **Keycloak / Backstage / Argo CD が PostgreSQL を推奨バックエンド** — 統一すれば運用が 1 系統で済む
2. **Dapr State Store / Configuration の PostgreSQL Component** — Valkey 障害時のフォールバック先として機能する
3. **tier2 業務サービスの RDBMS ニーズ** — 業務ドメインロジックが RDBMS を要求するケースは JTC で極めて多い。tier1 が共有クラスタを提供し、tier2 / tier3 がスキーマを分離して利用する
4. **CloudNativePG は CNCF Sandbox (Apache 2.0)** — k1s0 の OSS ライセンス方針に完全適合
5. **宣言的 HA** — プライマリ障害時に自動フェイルオーバー。JTC 情シスの夜間対応負荷を軽減
6. **バックアップを CRD で宣言** — `ScheduledBackup` リソースで定期バックアップを自動化。オンプレ NFS / S3 互換 (MinIO) にバックアップ可能

### MVP スコープ

- CloudNativePG Operator を `infra` namespace にデプロイ
- PostgreSQL クラスタを 1 つ作成 (プライマリ 1 + レプリカ 1 の最小 HA 構成)
- データベースを論理分離: `keycloak` / `backstage` / `argocd` / `harbor`
- 定期バックアップ (ローカル PV へのベースバックアップ + WAL アーカイブ)
- tier2 / tier3 サービス向けのデータベースプロビジョニングは Phase 2 以降

### トレードオフ

- 共有 PostgreSQL クラスタの障害は複数コンポーネントに波及する → レプリカ + 自動フェイルオーバーで緩和。Phase 3 でクリティカル度に応じたクラスタ分離を検討
- ストレージ I/O がボトルネックになりやすい → SSD を必須とし、Longhorn / ローカル PV で低レイテンシを確保

---

## G. ポリシーエンジン / Admission Controller

| 候補 | 採否 | 評価 |
|---|---|---|
| **Kyverno** | 採用 | CNCF Graduated。YAML ネイティブでポリシーを記述。学習コストが低い |
| OPA / Gatekeeper | 次点 | CNCF Graduated。Rego 言語によるポリシー記述。表現力は高いが Rego の習得コストが JTC 向きではない |
| Kubewarden | 却下 | CNCF Sandbox。WASM ベースで柔軟だがエコシステムが発展途上 |
| Polaris | 却下 | ベストプラクティス検証に特化。Admission 制御の汎用性が不足 |

### 採用理由 (Kyverno)

1. **ポリシーが k8s ネイティブ YAML** — Rego のような独自言語を覚える必要がない。JTC 情シスの学習曲線を最小化
2. **Validate / Mutate / Generate の 3 機能** — リソース検証だけでなく、デフォルト値の自動付与やリソースの自動生成が可能
3. **PSS (PodSecurityStandards) を完全に代替可能** — PSA (PodSecurityAdmission) より柔軟な例外制御。namespace / ワークロード単位で例外定義が可能
4. **cosign 署名検証を Admission で実行** — Phase 2 の署名済みイメージのみデプロイ許可を実現
5. **ClusterPolicy / Policy の分離** — クラスタ全体ポリシーと namespace 個別ポリシーを明確に分離管理
6. **CNCF Graduated (Apache 2.0)** — k1s0 の OSS ライセンス方針に完全適合

### k1s0 における役割

Kyverno は API 設計原則の「規律はツールで強制する」を k8s クラスタレベルに拡張する。

| ポリシーカテゴリ | 具体例 |
|---|---|
| Pod セキュリティ | privileged 禁止、root 実行禁止、hostNetwork 禁止 (PSS Restricted 相当) |
| イメージ制御 | `harbor.k1s0.internal/*` 以外の pull 禁止、`:latest` タグ禁止 |
| ラベル強制 | `app.kubernetes.io/part-of`、`k1s0.io/tier` 等の必須ラベル付与 |
| リソース制限 | `resources.requests` / `resources.limits` の必須化 |
| Dapr 隠蔽保護 | scaffold CLI が生成した annotation パターン以外の `dapr.io/*` を拒否 |
| サプライチェーン (Phase 2) | cosign 未署名イメージの deploy 拒否 |

### MVP スコープ

- MVP-1b で Kyverno を `infra` namespace にデプロイ (HA 3 replicas)
- PSS Restricted 相当の ClusterPolicy を適用し、PodSecurityAdmission を無効化
- Harbor レジストリ制限ポリシーと必須ラベルポリシーを適用
- Phase 2 で cosign 署名検証ポリシーを追加

### トレードオフ

- Admission Webhook の可用性 — Kyverno が停止すると Pod 作成が止まる → HA 構成 (3 replicas) + `failurePolicy: Fail` で安全側に倒す
- ポリシー数の増大による管理コスト → ポリシーを Git 管理し、Argo CD で GitOps 適用

---

## H. 証明書管理

| 候補 | 採否 | 評価 |
|---|---|---|
| **cert-manager** | 採用 | CNCF Graduated。k8s ネイティブの証明書ライフサイクル管理 |
| 手動証明書管理 | 却下 | 証明書の期限切れによる障害リスクが高い |
| Istio CA のみ | 不十分 | Istio mTLS は Istio CA が管理するが、Istio 外 (Envoy Gateway / Harbor / Keycloak) の TLS 証明書は管理できない |

### 採用理由 (cert-manager)

1. **証明書の自動発行・自動更新** — 証明書期限切れによるサービス停止を根本的に排除
2. **Istio 外の TLS 証明書を管理** — Envoy Gateway (外部向け TLS 終端)、Harbor (HTTPS)、Keycloak (HTTPS)、Backstage (HTTPS) の証明書を自動管理
3. **内部 CA / 外部 CA に対応** — オンプレ環境では自己署名 CA (SelfSigned / CA Issuer)、将来は Let's Encrypt や企業 CA との連携が可能
4. **Certificate / Issuer が k8s CRD** — 証明書の状態を `kubectl` で宣言的に管理・監視できる
5. **CNCF Graduated (Apache 2.0)** — k1s0 の OSS ライセンス方針に完全適合

### k1s0 における役割

| 管理対象 | Issuer 種別 | 備考 |
|---|---|---|
| Envoy Gateway の外部向け TLS | CA Issuer (内部 CA) | クライアントが信頼する証明書を自動発行 |
| Harbor の HTTPS | CA Issuer (内部 CA) | レジストリアクセスの暗号化 |
| Keycloak の HTTPS | CA Issuer (内部 CA) | OIDC エンドポイントの暗号化 |
| Backstage の HTTPS | CA Issuer (内部 CA) | 開発者ポータルの暗号化 |
| Argo CD の HTTPS | CA Issuer (内部 CA) | GitOps ダッシュボードの暗号化 |
| Istio メッシュ内 mTLS | (対象外) | Istio CA が管理。cert-manager は関与しない |

### MVP スコープ

- MVP-1a で cert-manager を `infra` namespace にデプロイ
- SelfSigned Issuer → CA Issuer のチェーンで内部 CA を構築
- Envoy Gateway / Harbor / Keycloak / Backstage / Argo CD の TLS 証明書を自動発行
- 証明書の自動更新を有効化 (デフォルト: 期限 30 日前に更新)

### トレードオフ

- 自己署名 CA の信頼配布 — クライアント端末に CA 証明書を信頼させる必要がある → JTC では Active Directory グループポリシーでの配布が一般的
- cert-manager 自体の可用性 — 停止しても既存証明書は有効期限まで動作する。新規発行・更新のみ止まる → HA 構成と証明書期限の監視で緩和

---

## I. オブジェクトストレージ

| 候補 | 採否 | 評価 |
|---|---|---|
| **MinIO** | 採用 | S3 互換のオブジェクトストレージ。単一バイナリで軽量。k8s Operator あり |
| Ceph (RADOS Gateway) | 却下 | 機能豊富だがオペレーター規模が大きく 2〜3 名体制では運用困難 |
| SeaweedFS | 却下 | 軽量だがコミュニティ規模が MinIO より小さい |

### 採用理由 (MinIO)

1. **Harbor のストレージバックエンド** — Harbor は S3 互換ストレージをバックエンドに使用可能。ノード障害時のイメージ消失を防ぐ
2. **CloudNativePG のバックアップ先** — barman-cloud-wal-archive が S3 API に WAL を書き出す。MinIO がなければバックアップがノードローカルに閉じ、ノード全壊時にバックアップも消失する
3. **OpenTofu State の保存先** — S3 バックエンドで State を共有し、複数人での `tofu apply` 時の State 競合を防ぐ
4. **将来のアーカイブ先** — 監査ログの長期保存 (7 年) や SBOM の保管先として統一的に利用できる
5. **AGPL-3.0 だが社内利用は問題なし** — Grafana / Loki と同じ位置付け

### k1s0 における役割

| 利用者 | 用途 |
|---|---|
| Harbor | コンテナイメージの保存先 |
| CloudNativePG (barman-cloud) | PostgreSQL のバックアップ / WAL アーカイブ |
| OpenTofu | State ファイルの共有保存 |
| tier1 Audit (Phase 3) | 監査ログのアーカイブ先 |
| Loki (Phase 3) | ログの長期保存先 |

### MVP スコープ

- MVP-1b で MinIO を `infra` namespace にデプロイ (単一インスタンス、レプリカなし)
- Harbor のバックエンドとして構成
- CloudNativePG のバックアップ先として構成
- OpenTofu State の S3 バックエンドとして構成
- Phase 2 以降で HA 構成 (Erasure Coding) に拡張

### トレードオフ

- AGPL-3.0 ライセンス → 社内利用のため改変配布義務は発生しない。Grafana / Loki と同じ扱い
- 単一インスタンスの MVP 構成ではストレージ冗長性がない → ノードをまたぐ PV (Longhorn 等) との組み合わせ、または Phase 2 での HA 化で対応

---

## J. Feature Flag

| 候補 | 採否 | 評価 |
|---|---|---|
| **OpenFeature + flagd** | 採用 | CNCF Incubating。ベンダー中立の Feature Flag 標準 API + 軽量評価エンジン |
| LaunchDarkly | 却下 | 商用 SaaS。オンプレ完結の原則に反する |
| Unleash | 次点 | OSS 版あり。機能は十分だが OpenFeature との統合で flagd の方が軽量 |
| Flipt | 次点 | Go 製で軽量。OpenFeature Provider もあるが、コミュニティ規模は flagd が上回る |

### 採用理由 (OpenFeature + flagd)

1. **CNCF Incubating** — k1s0 の CNCF 採用方針と整合
2. **ベンダー中立** — OpenFeature は標準 API 仕様であり、将来の評価エンジン差し替え (flagd → LaunchDarkly 等) が容易
3. **flagd は単一バイナリ・軽量** — メモリ 64〜128 MB で動作。JTC の限られた VM リソースに適合
4. **フラグ定義が JSON/YAML ファイル** — Git 管理可能。GitOps と一貫する
5. **C# / Go / TS の SDK が公式提供** — tier1 クライアントライブラリの対象言語と一致

### k1s0 における役割

Feature Flag は Dapr 隠蔽と同じ原則で tier1 が隠蔽する。tier2 / tier3 は flagd の存在を意識しない。

| tier1 公開 API | 内部実装 | 用途 |
|---|---|---|
| `k1s0.Feature.IsEnabled(key, context)` | Go ファサード → flagd gRPC | フラグの ON/OFF 評価 |
| `k1s0.Feature.GetVariant(key, context)` | Go ファサード → flagd gRPC | 多値バリアント評価 (A/B テスト等) |

主な利用シーン:

- **段階的ロールアウト** — 新機能を部署単位 / ユーザー割合で段階的に有効化
- **レガシー共存の制御弁** — .NET Framework 旧サービスと新 tier2 サービスの切替を部署単位で制御
- **ZEN Engine ルール変更の安全弁** — 業務担当が編集した JDM の適用範囲をフラグで限定
- **障害時の機能制限** — 問題のある機能をデプロイなしで即座に無効化

### MVP スコープ

- Phase 2 で flagd を `tier1` namespace にデプロイ
- tier1 公開 API に `k1s0.Feature` を追加
- フラグ定義は Git リポジトリの `flags/` ディレクトリに JSON で配置し、Argo CD で同期
- Phase 3 以降でユーザーセグメント / 段階的ロールアウト (パーセンテージベース) を本格活用

### トレードオフ

- フラグの乱立リスク → フラグの命名規則 / 有効期限 / 棚卸しルールを定め、不要になったフラグを定期的に削除する運用ルールが必要
- flagd の可用性 → 障害時は参照系 API と同じ cache-fallback パターンを適用。最後に評価成功した結果をキャッシュで返す

---

## K. Chaos Engineering

| 候補 | 採否 | 評価 |
|---|---|---|
| **Litmus** | 採用 | CNCF Incubating。k8s ネイティブの Chaos Engineering プラットフォーム。Web UI あり |
| Chaos Mesh | 次点 | CNCF Incubating。機能は同等だが Litmus の方が Web UI / ワークフロー管理が成熟 |
| Gremlin | 却下 | 商用 SaaS。オンプレ完結の原則に反する |
| PowerfulSeal | 却下 | 軽量だがコミュニティが縮小傾向 |
| Chaos Toolkit | 却下 | Python ベース。k8s ネイティブではなく操作が煩雑 |

### 採用理由 (Litmus)

1. **CNCF Incubating (Apache 2.0)** — k1s0 の CNCF 採用方針と整合
2. **ChaosExperiment が k8s CRD** — 実験定義を YAML で Git 管理し、Argo CD で GitOps 適用できる
3. **Web UI (Litmus Portal)** — 実験の実行状況・結果を可視化。JTC 運用チームの学習曲線を緩和
4. **定期実行 (CronChaosEngine)** — 週次 / 日次で自動的に障害注入テストを実行し、縮退動作を継続的に検証
5. **Grafana 連携** — 実験結果をダッシュボードで表示。既存の Grafana スタックに統合

### k1s0 における役割

グレースフルデグラデーション設計 ([`../../02_アーキテクチャ/02_可用性と信頼性/03_グレースフルデグラデーション.md`](../../02_アーキテクチャ/02_可用性と信頼性/03_グレースフルデグラデーション.md)) で定義した縮退動作が実装と一致することを自動検証する。

| 検証対象 | 実験内容 | 期待する縮退動作 |
|---|---|---|
| PostgreSQL 障害 | CloudNativePG プライマリの Pod delete | フェイルオーバー 30 秒以内完了。Auth API は JWKS キャッシュで継続 |
| Valkey 障害 | Valkey Pod の delete | State API が fail-fast。Settings API がキャッシュで継続 |
| Dapr Sidecar 障害 | 特定 Pod の daprd コンテナ kill | 該当 Pod のみ影響。他 Pod は正常 |
| ネットワーク遅延 | tier1 ↔ infra 間に 500ms 遅延注入 | tier1 API の p99 が SLA 内に収まるか検証 |
| tier1 Go サービス障害 | tier1-go-facade の 1 Pod を kill | ReplicaSet で自動復旧。業務への影響時間を計測 |

### MVP スコープ

- Phase 2 で Litmus を `operation` namespace にデプロイ
- 初期実験セット: PostgreSQL フェイルオーバー / Valkey 障害 / Dapr Sidecar 障害の 3 パターン
- CronChaosEngine で週次自動実行
- Grafana ダッシュボードで結果を可視化
- Phase 3 以降でネットワーク障害注入・カスケード障害テストに拡張

### トレードオフ

- 本番環境での障害注入リスク → MVP / Phase 2 では専用の test namespace で実施。本番 namespace への適用は Phase 3 のマルチクラスタ (staging / prod 分離) 後
- Litmus Portal のリソース消費 → Portal なしでも CRD ベースで実験可能。リソースが逼迫する場合は CLI + CRD のみで運用し、Portal は後から追加

---

## 結論

| カテゴリ | 採用 OSS | ライセンス |
|---|---|---|
| ID / 認証 | Keycloak | Apache 2.0 |
| GitOps CD | Argo CD | Apache 2.0 |
| CI (主) | GitHub Actions (self-hosted runner) | Apache 2.0 (runner) |
| CI (代替 / GHA 不可環境) | Tekton | Apache 2.0 |
| レジストリ | Harbor | Apache 2.0 |
| 脆弱性スキャン | Trivy (Harbor 内蔵) | Apache 2.0 |
| キャッシュ / KV | Valkey | BSD-3-Clause |
| RDBMS | CloudNativePG + PostgreSQL | Apache 2.0 |
| ポリシーエンジン | Kyverno | Apache 2.0 |
| 証明書管理 | cert-manager | Apache 2.0 |
| イベントスキーマレジストリ | Apicurio Registry | Apache 2.0 |
| ローカル開発ツール | Tilt | Apache 2.0 |
| オブジェクトストレージ | MinIO | AGPL-3.0 |
| Feature Flag | OpenFeature + flagd | Apache 2.0 |
| Chaos Engineering | Litmus | Apache 2.0 |

すべて OSI 承認された OSS ライセンスで、RSALv2 / SSPL / BSL のような制限ライセンスを含まない。Keycloak OIDC を中心とした SSO で統合され、利用者は 1 アカウントで全ポータルにアクセスできる。

---

## 関連ドキュメント

- [`00_選定方針.md`](../01_俯瞰/00_選定方針.md) — 前提条件と判断軸
- [`01_実行基盤中核OSS.md`](../02_中核OSS/01_実行基盤中核OSS.md) — k8s / Istio / Kafka / Dapr 等
- [`03_ルールエンジン.md`](../02_中核OSS/03_ルールエンジン.md) — ZEN Engine
- [`04_選定一覧.md`](../01_俯瞰/04_選定一覧.md) — 採用 OSS の全体一覧
- [`05_IaC.md`](05_IaC.md) — OpenTofu (IaC) の採用根拠
- [`06_イベントスキーマレジストリ.md`](06_イベントスキーマレジストリ.md) — Apicurio Registry の採用根拠
- [`../../05_CICDと配信/00_CICDパイプライン.md`](../../05_CICDと配信/00_CICDパイプライン.md) — GHA / Argo CD / Harbor の統合フロー
- [`../../05_CICDと配信/04_ローカル開発環境.md`](../../05_CICDと配信/04_ローカル開発環境.md) — Tilt によるローカル開発環境
- [`../../02_アーキテクチャ/02_可用性と信頼性/03_グレースフルデグラデーション.md`](../../02_アーキテクチャ/02_可用性と信頼性/03_グレースフルデグラデーション.md) — Litmus で検証する縮退動作の定義
- [`../../02_アーキテクチャ/02_可用性と信頼性/01_障害復旧とバックアップ.md`](../../02_アーキテクチャ/02_可用性と信頼性/01_障害復旧とバックアップ.md) — MinIO をバックアップ先として利用
