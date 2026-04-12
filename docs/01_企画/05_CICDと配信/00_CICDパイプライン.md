# CI / CD パイプライン

## 目的

[`../04_技術選定/02_周辺OSS.md`](../04_技術選定/02_周辺OSS.md) で採用決定した GitHub Actions / Harbor / Trivy / Argo CD を、**どう組み合わせて 1 本のパイプラインにするか** を整理する。個別の OSS 選定根拠は参照先を見ること。

---

## 1. 設計原則

1. **GHA を基本 CI エンジンとする** — PR / テスト / ビルド / スキャン / デプロイ連携まで GHA のワークフロー 1 本で完結
2. **Tekton は GHA が使えない環境向けのフォールバック** — MVP では構築しない。別拠点 / 顧客環境で GHA 不可の場合に同等の Pipeline を提供
3. **開発者は CI 設定を手書きしない** — 雛形生成 CLI が GHA ワークフロー (と将来的な Tekton Pipeline) をセットで生成
4. **デプロイはすべて GitOps** — CI は Git にマニフェストを commit するだけ。実デプロイは Argo CD が担う
5. **Harbor / Trivy でイメージの門番を担う** — CVE Critical 以上が検出されたイメージは push 拒否
6. **Backstage からすべての状態が見える** — GHA / Argo CD のプラグインで統合表示

---

## 2. 全体フロー (基本: GHA パス)

![CICD フロー](./img/CICDフロー.svg)

**ポイント**:

- GHA の self-hosted runner は **k8s 上の Pod** として稼働 (`actions-runner-controller`)
- 1 本のワークフロー内でビルド / スキャン / push / GitOps 更新までを step として実行
- マニフェスト更新は同一リポジトリ内の `deploy/` ディレクトリ、または別リポジトリ (GitOps リポ) のどちらも可能。MVP では単一リポジトリで開始
- Argo CD は GitOps リポの変更を検知して同期。開発者は Argo CD を直接操作しない

---

## 3. 代替フロー (Tekton / GHA 不可環境)

GHA が利用できない環境 (GitHub.com 到達不可、完全エアギャップ、ポリシー上の禁止等) では、**同じステージを Tekton Pipeline で再現**する。MVP では構築しないが、設計の退路として残す。

| 項目 | 内容 |
|---|---|
| ソース管理 | GitHub.com ではなく、オンプレ Git (Forgejo / Gitea 等) |
| トリガ | Tekton Triggers の `EventListener` が Git Webhook を受ける |
| ステージ | GHA ワークフローと **同じロジックを Tekton Task として並行管理** |
| 着手時期 | Phase 2 以降。MVP ではインストールしない |

---

## 4. GHA ワークフローのステージ構成

GHA self-hosted runner 上で以下を **1 本のワークフローの step** として順次実行する。

| ステージ | 使用ツール | 備考 |
|---|---|---|
| PR トリガ | GHA `on: pull_request` | GitHub Checks / PR コメント統合 |
| Lint / フォーマット | clippy / rustfmt / eslint / prettier | 失敗時に PR をブロック |
| ユニットテスト | `cargo test` / `dotnet test` / `go test` | テスト結果を GitHub Checks に出力 |
| 型チェック | `tsc` / `cargo check` | PR ブロック条件 |
| イベントスキーマ互換性チェック (Phase 2 以降) | Apicurio Registry REST API | `schemas/` 内の JSON Schema を Apicurio に送信し BACKWARD 互換性を検証。違反時は PR ブロック |
| コンテナビルド | Kaniko (in Pod) | Docker デーモン不要、self-hosted runner Pod 内で実行 |
| ソースコード脆弱性スキャン | Trivy FS | ビルド前後の早期検知 |
| Harbor への push | `crane` / `skopeo` | クラスタ内通信で高速 |
| イメージ脆弱性スキャン | Harbor 内蔵 Trivy | push 後に Harbor 側で自動実行 |
| イメージ署名 (Phase 2 以降) | Cosign | サプライチェーンセキュリティ |
| マニフェスト更新 | `yq` + `git commit` | GitOps リポへの image tag 書き換え |
| リリースタグ付け | `actions/github-script` | GitHub Releases との統合 |
| チェンジログ生成 | `git-cliff` / `release-please` | コミット履歴からの自動生成 |

### self-hosted runner の構成

- `actions-runner-controller` で runner Pod を k8s 上に宣言的に管理
- runner Pod は **Kaniko / Trivy / crane / argocd CLI を同梱したカスタムイメージ**を使用
- runner Pod は Harbor / Argo CD / GitOps リポに **クラスタ内通信** で到達
- Phase 1 では最小 2 Pod 構成、負荷に応じてスケール

---

## 5. Argo CD が担うステージ

| ステージ | 動作 | 備考 |
|---|---|---|
| Git リポ監視 | 3 分間隔でポーリング (既定) | Webhook で即時化も可能 |
| マニフェスト差分検知 | Argo CD Controller | 自動同期 / 手動同期を環境ごとに設定 |
| k8s リソース適用 | `kubectl apply` 相当 | ApplicationSet で tier 単位に分割 |
| ヘルスチェック | Argo CD Health Assessment | カスタムヘルスチェックも追加可能 |
| ロールバック | Git revert → Argo CD 同期 | 運用は Git 主導で統一 |

---

## 6. イメージ品質ゲート

| チェ���ク点 | ツール | 検出時の動作 | 導入時期 |
|---|---|---|---|
| ���ルド前 (FS スキャン) | Trivy FS (GHA step) | `HIGH` 以上で警告、`CRITICAL` でパイプライン失敗 | MVP-1b |
| push 後 (イメージスキャン) | Harbor 内蔵 Trivy | `CRITICAL` を含むイメージ��� `pull` を拒否 (Harbor プロジェクトポ���シー) | MVP-1b |
| 開発ブランチ例外 | — | `dev` ブランチは警告���み、`main` の���厳格適用 | MVP-1b |
| イメージソース制限 | Kyverno | `harbor.k1s0.internal/*` 以外のイメージを含む Pod を Admission で拒否 | MVP-1b |
| `:latest` タグ禁止 | Kyverno | `:latest` タグ指定を Admission で拒否 | MVP-1b |
| イメージ署名検証 | Cosign + Kyverno | 未署名イメージの deploy を k8s Admission Webhook で拒否 | Phase 2 |

---

## 7. 認証の一元化 (Keycloak OIDC)

| コンポーネント | OIDC クライアント種別 | 用途 |
|---|---|---|
| GitHub.com | (対象外) | GitHub 側の認証は GitHub アカウントのまま |
| self-hosted runner | (対象外) | runner は GitHub からトークン受領 |
| Harbor | Confidential | Web UI / Registry 操作 |
| Argo CD | Confidential | Web UI / CLI (SSO) |
| Backstage | Confidential | 開発者ポータル |
| Tekton Dashboard (Phase 2 以降) | Confidential | 代替フロー採用時のみ |

GitHub.com を除くすべての Web UI を **Keycloak ローカル DB のユーザー** で SSO する。MVP ではユーザー数十名を想定し、AD 連携は Phase 2 以降で追加する。

---

## 8. Backstage との統合

Backstage Software Catalog の各コンポーネントに、以下のアノテーションを付与して可視化する。

| アノテーション | 対象 | 表示内容 |
|---|---|---|
| `github.com/project-slug` | 全コンポーネント | GitHub リポジトリ情報 / PR 状態 / Actions ワークフロー実行状況 |
| `backstage.io/techdocs-ref` | 全コンポーネント | TechDocs (サービス設計書) |
| `argocd/app-name` | tier1 / tier2 / tier3 | Argo CD Application 同期状態 |
| `harbor/project-name` | tier1 / tier2 / tier3 | Harbor 内の該当プロジェクト |
| `tektoncd.dev/pipeline` (Phase 2 以降) | 代替フロー採用時 | Tekton Pipeline 実行状態 |

これにより開発者は **Backstage の 1 画面から PR / Pipeline / デプロイ / レジストリ状態** をまとめて確認できる。

---

## 9. 雛形生成 CLI が生成するファイル

開発者が tier2 / tier3 サービスを新規に立ち上げるとき、雛形生成 CLI (tier1 提供) は以下を同時に生成する。**開発者がこれらを手書きすることは想定しない**。

| 生成ファイル | 内容 |
|---|---|
| `.github/workflows/ci.yml` | GHA の PR 時ビルド / テスト / Lint 定義 |
| `.github/workflows/release.yml` | main ブランチ push 時のビルド / スキャン / push / GitOps 更新 |
| `deploy/base/*.yaml` | k8s マニフェスト (Deployment / Service / Dapr Component) |
| `deploy/overlays/*/kustomization.yaml` | 環境別 overlay (dev / staging / prod) |
| `argocd/application.yaml` | Argo CD Application 定義 |
| `catalog-info.yaml` | Backstage Software Catalog エントリ |
| `Tiltfile` | ローカル開発環境定義 (tier1 依存サービス起動 + 差分ビルド) |
| `tilt/tier1-deps.yaml` | tier1 サービス群のローカル構成マニフェスト |
| `tilt/dapr-components-local.yaml` | ローカル用 Dapr Component (in-memory) |
| `schemas/events/` (Phase 2 以降) | イベント JSON Schema (Apicurio Registry 連携) |
| `tekton/pipeline.yaml` (Phase 2 以降) | GHA 不可環境向けの同等 Pipeline 定義 |
| `tekton/task-*.yaml` (Phase 2 以降) | 共通 Task の参照 (catalog から) |

CI / CD の進化 (新しいスキャンステップ追加等) は **雛形更新 + 既存サービスへの一括反映** で行い、個別チームに手作業で追従させない。Tekton 代替フロー導入以降は、同じ変更を GHA ワークフローと Tekton Pipeline の両方に同時反映する。

---

## 10. MVP スコープと Phase 分離

### MVP-1a

- GHA self-hosted runner セットアップ (`actions-runner-controller`)
- runner イメージに Kaniko / Trivy / crane / argocd CLI を同梱
- Argo CD インストール + tier1 向け ApplicationSet
- Keycloak OIDC で Argo CD / Backstage を SSO 統合
- cert-manager デプロイ + 内部 CA 構築 (SelfSigned → CA Issuer チェーン)
- Envoy Gateway / Keycloak / Argo CD の TLS 証明書を cert-manager で自動発行
- **Tekton はインストールしない**

### MVP-1b

- Harbor + 内蔵 Trivy 起動、プロジェクト作成 (`tier1` / `tier2` / `tier3` / `infra`)
- Harbor / Backstage の TLS 証明書を cert-manager で自動発行
- Kyverno デプロイ (HA 3 replicas) + 基本 ClusterPolicy 適用
  - PSS Restricted 相当 (privileged / root / hostNetwork 禁止)
  - イメージソース制限 (`harbor.k1s0.internal/*` 以外を拒否)
  - `:latest` タグ禁止
  - 必須ラベル / `resources.requests` / `resources.limits` の強制
- Keycloak OIDC で Harbor を SSO 統合
- 1 本のサンプルサービス (tier1 リファレンス実装) で GHA フロー疎通

### Phase 2

- Apicurio Registry 導入 + CI にイベントスキーマ互換性チェックを追加
- Cosign 署名と Kyverno の未署名ブロック (署名検証ポリシー追加)
- Argo Rollouts によるカナリア / Blue-Green
- Trivy DB のオフラインミラー運用 (JTC 環境向け)
- Backstage Software Templates から雛形生成 CLI を呼び出し
- cert-manager の外部 CA 連携検討 (企業 CA との統合)
- **Tekton 代替フローの検討開始** — 別拠点 / 顧客環境で GHA 不可のケースが出た時点で着手

### Phase 3 以降

- マルチクラスタ (staging / prod 分離)
- External Secrets Operator + Keycloak バックアップ
- Argo Image Updater による自動タグ更新の検討
- Tekton Chains によるサプライチェーン証跡 (代替フロー採用時)

---

## 11. 運用上の未決事項

以下は本資料では扱わず、ADR として個別に決定する。

- GitOps リポジトリを単一リポ / 別リポのどちらにするか (MVP は単一、Phase 2 で再検討)
- Secret 管理方式 (SealedSecrets vs External Secrets Operator)
- 環境別 overlay 設計 (kustomize / helm のどちらを主軸にするか)
- Trivy DB の更新経路 (インターネット接続制限環境での定期同期手順)
- Harbor のストレージバックエンド (Longhorn / Rook-Ceph / MinIO)

---

## 関連ドキュメント

- [`../04_技術選定/02_周辺OSS.md`](../04_技術選定/02_周辺OSS.md) — GHA / Harbor / Argo CD / Keycloak / Valkey の個別選定根拠
- [`../04_技術選定/04_選定一覧.md`](../04_技術選定/04_選定一覧.md) — 採用 OSS 一覧
- [`01_開発者ポータル_Backstage.md`](./01_開発者ポータル_Backstage.md) — Backstage 側の統合方針
- [`02_アプリ配信ポータル.md`](./02_アプリ配信ポータル.md) — tier3 アプリのエンドユーザー配信
- [`../03_tier1設計/03_API設計原則.md`](../03_tier1設計/03_API設計原則.md) — 雛形生成 CLI の設計原則
