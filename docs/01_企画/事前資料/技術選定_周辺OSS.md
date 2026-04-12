# 技術選定比較表 (周辺 OSS 編)

## 目的

`技術選定比較表.md` は実行基盤中核 (k8s / Istio / Envoy Gateway / Kafka / Dapr 等) に焦点を絞っている。
本資料はその **周辺で必須になる OSS** (認証・CI/CD・レジストリ・脆弱性スキャン・キャッシュ) の選定根拠を補完する。

対象カテゴリは以下の 5 つ。

- A. ID / 認証基盤
- B. GitOps / 継続的デリバリ
- C. CI / パイプライン
- D. コンテナレジストリ / 脆弱性スキャン
- E. キャッシュ / KV ストア

## 選定の前提条件

`技術選定比較表.md` と同じ。以下を再掲する。

- オンプレミス / クラウド VM で動作すること (マネージドサービス非依存)
- ベンダーロックインを回避できること
- **OSI 承認された OSS ライセンスであること** (RSALv2 / SSPL / BSL はライセンス的に OSS ではないため対象外)
- コミュニティが活発で長期運用に耐えること
- JTC の情報システム部門が扱える習得難易度であること

---

## A. ID / 認証基盤

### 候補一覧

| 候補 | 採用可否 | 評価 |
|---|---|---|
| **Keycloak** | ◎ 採用 | Red Hat 発の OSS。OIDC / SAML / LDAP / ソーシャルログイン / 2FA を標準装備。Admin UI が成熟している。 |
| Zitadel | ○ 次点 | Go 製でモダン。マルチテナント / イベントソーシング設計。日本語情報が少ない。 |
| Authentik | △ 次点 | Python 製。管理 UI が洗練されているが Keycloak ほどの実績はない。 |
| Dex | △ 却下 | OIDC のフロントに特化。ユーザー DB を持たないため MVP 単独採用は不可。 |
| ORY Hydra + Kratos | △ 却下 | 柔軟性は高いが複数コンポーネントの組み合わせが必要で JTC 運用に不向き。 |

### 採用理由 (Keycloak)

1. **ユーザー DB を自前で持てる** — MVP では AD 連携なしで Keycloak ローカル DB を認証源とする
2. **将来の AD / LDAP フェデレーションを追加設定のみで実現可能** — Phase 2 以降で JTC 既存 AD を取り込める退路を確保
3. **Envoy Gateway の ext_authz 相当として oauth2-proxy 経由で連携可能** — 既知のパターンで運用手順を組み立てやすい
4. **OSS で商用版との機能差がない** — Red Hat Build of Keycloak は商用サポート契約のみで機能差異なし
5. **日本語情報 / 事例が豊富** — JTC 情シス部門での導入ハードルが低い

### MVP スコープ

- Keycloak を **Realm = k1s0** で 1 つ立ち上げる
- ユーザーは Keycloak ローカル DB で直接管理 (AD 連携なし)
- OIDC クライアントを tier1 / アプリ配信ポータル / Backstage / Argo CD / Harbor の各コンポーネント向けに登録
- 将来の AD フェデレーションは Realm 設定の追加のみで対応可能にしておく

### トレードオフ

- **ローカル DB 運用の煩雑さ** — Phase 2 以降で AD 連携に切替える前提で、MVP 中はユーザー数を絞って運用負荷を抑える
- **高可用構成の難しさ** — Keycloak は PostgreSQL バックエンド + 複数レプリカ構成が推奨。CloudNativePG との組み合わせで HA を確保する

---

## B. GitOps / 継続的デリバリ

### 候補一覧

| 候補 | 採用可否 | 評価 |
|---|---|---|
| **Argo CD** | ◎ 採用 | CNCF Graduated。Web UI が成熟し可視性が高い。Backstage 連携プラグイン公式提供あり。 |
| Flux CD | ○ 次点 | CNCF Graduated。CLI / GitOps 純度重視。UI 標準装備なし。 |
| Argo CD + Argo Rollouts | ◎ 併用候補 | カナリア / Blue-Green リリース。Phase 2 以降で追加検討。 |
| Spinnaker | △ 却下 | 機能は豊富だが重量級。JTC 情シスでの運用負荷が大きい。 |
| Jenkins X | △ 却下 | Jenkins 依存。k1s0 の軽量指向と整合しない。 |

### 採用理由 (Argo CD)

1. **Web UI によるデプロイ可視化** — JTC 情シスにとって GitOps の学習曲線を緩和できる
2. **Backstage Argo CD プラグインが公式提供** — Software Catalog からデプロイ状態を直接確認可能
3. **ApplicationSet による tier 単位の一括管理** — tier1 / tier2 / tier3 を別々の ApplicationSet で管理できる
4. **マルチクラスタ対応** — 将来の本番 / ステージング分離に対応可能
5. **Keycloak との OIDC 連携が標準装備** — SSO 統合が設定のみで完結

### MVP スコープ

- Argo CD を `operation` 名前空間にデプロイ
- tier1 / tier2 / tier3 それぞれに ApplicationSet を 1 つずつ
- Git リポジトリは GitHub.com の単一リポジトリを想定 (monorepo / polyrepo の選定は別途)
- Keycloak OIDC で SSO、Argo CD の RBAC は Keycloak グループでマッピング
- Backstage の Argo CD プラグインを有効化

### トレードオフ

- **Argo CD 自身も GitOps で管理したい** — app-of-apps パターンで自己管理する。Bootstrap の初回だけ手作業 apply
- **Secret の扱い** — Argo CD が Git に置いた Secret を復号するために SealedSecrets / External Secrets Operator のいずれかが必要 (別途選定)

---

## C. CI / パイプライン

### 候補一覧

| 候補 | 採用可否 | 評価 |
|---|---|---|
| **GitHub Actions (self-hosted runner)** | ◎ 採用 (主) | **基本の CI エンジン**。PR / テスト / ビルド / スキャン / デプロイまでを 1 本のワークフローで完結させる。 |
| **Tekton** | ○ 採用 (代替) | **GHA が使えない環境向けのフォールバック**。完全エアギャップ / GitHub.com 到達不可 / ポリシー上 GHA 禁止の環境で利用。 |
| Jenkins | △ 却下 | 実績は豊富だが YAML / Groovy / プラグイン管理が属人化しやすい。 |
| Drone / Woodpecker | △ 却下 | 軽量だが GitHub Actions ほどのワークフロー資産がない。 |
| GitLab CI | × 対象外 | GitHub.com 前提のため不適。 |
| Argo Workflows | △ 却下 | Tekton と競合。Tekton の方が CI 特化で扱いやすい。 |

### 採用理由 (GHA を主、Tekton を代替)

1. **原則 GHA に一本化** — PR / Issue / Checks が GitHub 上で完結し、開発者の認知負荷が最小
2. **self-hosted runner を k8s 上で起動** — オンプレ k8s クラスタのリソースで GHA ジョブを実行。`actions-runner-controller` で runner Pod を宣言的に管理
3. **ビルド / スキャン / デプロイも GHA 側で完結** — Kaniko / Trivy / `crane` / `argocd` CLI を GHA の step から呼び出し、Tekton なしで全ステージを実行
4. **Tekton は同等のパイプラインを k8s ネイティブで提供する退路** — 別環境 / 別拠点で GHA が使えないとき、同じロジックを Tekton Pipeline で再現する。MVP では **インストールしない**
5. **どちらも OSS** — GHA は GitHub.com 依存だが runner は Apache 2.0。Tekton は CNCF Graduated

### 役割分担の基本方針

| ステージ | 主 (基本) | 代替 (GHA 不可時) |
|---|---|---|
| PR 時のユニットテスト / Lint / 型チェック | **GHA** | Tekton |
| コンテナイメージビルド (Kaniko) | **GHA** | Tekton |
| Trivy スキャン | **GHA** (step) | Tekton (Task) |
| Harbor への push | **GHA** (step) | Tekton (Task) |
| GitOps リポ更新 (image tag 書き換え) | **GHA** (step) | Tekton (Task) |
| リリースタグ付け / チェンジログ生成 | **GHA** | — |

> **MVP では GHA のみを構築する**。Tekton は Phase 2 以降、GHA を利用できない拠点や顧客環境が登場した時点で追加する。

詳細なフロー図は `CI_CD_パイプライン構成.md` を参照。

### トレードオフ

- **GitHub.com への依存** — GitHub.com が停止 / 到達不可になると CI が動かない。重要な緊急リリース経路としてローカルでの `docker build` + 手動 push の手順書を用意
- **self-hosted runner の k8s 運用負荷** — `actions-runner-controller` を使って宣言的に管理。Phase 1 では最小 2 ノード構成
- **Tekton 代替を追加する時の二重管理** — 同じパイプラインロジックを 2 箇所で保守することになる。両方を **雛形生成 CLI で生成する前提**にして、開発者は個別に書かない運用とする

---

## D. コンテナレジストリ / 脆弱性スキャン

### 候補一覧

| 候補 | 採用可否 | 評価 |
|---|---|---|
| **Harbor** | ◎ 採用 | CNCF Graduated。イメージ管理 / 脆弱性スキャン / 署名 / レプリケーションを統合。 |
| Zot | ○ 次点 | 軽量 OCI ネイティブ。UI がシンプルで機能は Harbor より絞られる。 |
| Nexus Repository | △ 却下 | 汎用アーティファクトリポジトリ。OSS 版は機能制限あり。 |
| Quay | △ 却下 | Red Hat 製。OSS 版 (Project Quay) は存在するがコミュニティが小さい。 |

### 脆弱性スキャン

| 候補 | 採用可否 | 評価 |
|---|---|---|
| **Trivy (Harbor 内蔵)** | ◎ 採用 | Harbor が標準スキャナとして同梱。Aqua Security 提供の OSS。 |
| Clair | △ 次点 | Harbor で選択可能だが Trivy の方がコミュニティが活発。 |
| Grype | △ 却下 | 単体での実績はあるが Harbor との統合が Trivy ほど深くない。 |

### 採用理由 (Harbor + 内蔵 Trivy)

1. **レジストリとスキャンが 1 つの製品に統合されている** — 別途スキャナを運用する必要がない
2. **Keycloak OIDC 連携が標準装備** — SSO 統合が設定のみで完結
3. **RBAC とプロジェクト分離** — tier1 / tier2 / tier3 をプロジェクト単位で権限分離可能
4. **CVE 検知時の push 拒否ポリシー** — 重大度に応じた自動ブロックを設定可能
5. **イメージ署名 (Cosign / Notation) のサポート** — サプライチェーンセキュリティの基盤として拡張可能

### MVP スコープ

- Harbor を `infra` 名前空間にデプロイ
- プロジェクトは `tier1` / `tier2` / `tier3` / `infra` の 4 つ
- Trivy スキャンを push 時に自動実行、Critical 以上を検出したら push 拒否
- Keycloak OIDC で SSO
- イメージ署名は Phase 2 以降 (Cosign 導入時に追加)

### トレードオフ

- **Harbor のストレージ需要** — イメージ蓄積で数百 GB 規模になるため、永続ストレージ (Longhorn / Rook-Ceph) の設計が前提。MVP では単一 PV で開始し Phase 2 で分散化
- **Trivy DB 更新のオフライン対応** — JTC 環境によってはインターネット接続制限がある。DB を定期的にミラーする運用を Phase 2 で設計

---

## E. キャッシュ / KV ストア

### 候補一覧

| 候補 | 採用可否 | 評価 |
|---|---|---|
| **Valkey** | ◎ 採用 | Linux Foundation 傘下、BSD-3 ライセンス。Redis 7.2.4 からのフォークで完全互換。 |
| Redis 7.4+ | × 却下 | RSALv2 / SSPL デュアルライセンス。**OSI 承認 OSS ではない** ため本資料の前提条件に反する。 |
| KeyDB | △ 却下 | Redis フォークだが Snap Inc. 買収後にコミュニティ活動が鈍化。 |
| DragonflyDB | △ 却下 | BSL ライセンス。OSS ではない。 |
| etcd | △ 別用途 | k8s の内部状態管理が主。汎用キャッシュ用途には不向き。 |

### 採用理由 (Valkey)

1. **ライセンスが OSI 承認の BSD-3** — k1s0 の「OSS 積み上げ」「ベンダーロックイン回避」原則と完全整合
2. **Redis 7.2.4 からのフォークで wire protocol / コマンド / クライアント完全互換** — Dapr State Store の Redis Component をそのまま利用可能
3. **Linux Foundation 傘下でコミュニティが活発** — AWS / Google / Oracle / Ericsson が支援。AWS ElastiCache もデフォルトを Valkey に切替済み
4. **既存 Redis 利用ノウハウがそのまま活かせる** — 学習コストが発生しない

### MVP スコープ

- Valkey を tier1 の Dapr State Store / Cache のバックエンドとして採用
- tier2 / tier3 は **Valkey / Redis のどちらも直接意識しない** — tier1 公開 API (`k1s0.State` / `k1s0.Cache`) 経由でのみアクセス
- これにより将来の差し替え (Valkey → 別実装) が tier1 内部で閉じる

### トレードオフ

- **Dapr Component の `redis.v1` 名称は変わらない** — 設定値や Component 定義は `redis` のままで、バックエンドだけ Valkey に向ける。tier1 チームが Component YAML を管理するため tier2 / tier3 は影響を受けない
- **新しいプロジェクトで実績が浅い** — フォークは 2024 年。ただし Redis からの完全互換のためリスクは限定的

---

## 結論

| カテゴリ | 採用 OSS | ライセンス |
|---|---|---|
| ID / 認証 | **Keycloak** | Apache 2.0 |
| GitOps CD | **Argo CD** | Apache 2.0 |
| CI (主) | **GitHub Actions (self-hosted runner)** | Apache 2.0 (runner) |
| CI (代替 / GHA 不可環境) | **Tekton** | Apache 2.0 |
| レジストリ | **Harbor** | Apache 2.0 |
| 脆弱性スキャン | **Trivy (Harbor 内蔵)** | Apache 2.0 |
| キャッシュ / KV | **Valkey** | BSD-3-Clause |

- いずれも **OSI 承認された OSS ライセンス**で、RSALv2 / SSPL / BSL のような制限ライセンスを含まない
- すべて **Keycloak OIDC** を中心とした SSO で統合可能。利用者は 1 アカウントで全ポータルにアクセスできる
- **Harbor + Trivy + Tekton + Argo CD** の連鎖で、PR → ビルド → スキャン → レジストリ → デプロイまで GitOps で完結する
- 詳細なパイプラインフローは `CI_CD_パイプライン構成.md` を参照
