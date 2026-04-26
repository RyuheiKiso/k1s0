# `.github/repo-settings.md` — GitHub リポジトリ設定の正典

本ファイルは **GitHub UI / API でのリポジトリ設定値** を Git 管理下の正典として記録する。`.github/CODEOWNERS` / `.github/labels.yml` / `infra/github/*.tf`（terraform-provider-github、後段）と並んで、リポジトリの「設定 IaC 化」の一翼を担う。

## 関連設計

- [`docs/05_実装/30_CI_CD設計/50_branch_protection/01_branch_protection.md`](../docs/05_実装/30_CI_CD設計/50_branch_protection/01_branch_protection.md) — branch protection 5 rule
- [`plan/02_開発環境整備/06_GitHubリポジトリ設定.md`](../plan/02_開発環境整備/06_GitHubリポジトリ設定.md) — plan
- [`plan/16_OSS公開準備/01_GitHub_repo_settings.md`](../plan/16_OSS公開準備/01_GitHub_repo_settings.md) — リリース直前の最終確認

## 基本情報（About）

| 項目 | 値 | 備考 |
|---|---|---|
| name | `k1s0` | リポジトリ識別子 |
| description | `Dapr を tier1 に閉じ込め、Istio Ambient と共存させる polyglot プラットフォーム基盤。` | README TL;DR の 1 行目を採用 |
| homepage | （リリース後に Backstage TechDocs URL or null） | リリース時点では未設定 |
| visibility | public | リリース後 |
| topics | `kubernetes` `kubernetes-platform` `dapr` `istio-ambient` `polyglot` `monorepo` `decision-as-data` `zen-engine` `apache-2-license` `oss` | 10 件 |
| social-preview-image | （drawio で 1280×640 を作成、リリース直前） | 未設定 |

## 機能の有効化 / 無効化

| 機能 | 設定 | 理由 |
|---|---|---|
| Issues | 有効 | bug / feature 受付 |
| Discussions | 有効 | Q&A / Ideas / Show-and-Tell の 3 カテゴリ |
| Wiki | 無効 | README + docs/ + Backstage TechDocs で代替 |
| Projects | 任意（個人 OSS では当面無効） | リリース後に判断 |
| Sponsors | 無効（リリース時点） | 起案者が会社員のため境界曖昧、リリース時点+ で再判定 |
| Pages | 無効 | TechDocs に集約 |
| Releases | 有効 | 必須 |
| Packages | 有効 | GHCR で image 配布 |
| Security advisories | 有効 | 脆弱性 private 報告 |
| Code scanning (CodeQL) | 有効 | リリース時点 から開始 |
| Dependency graph | 有効 | Renovate との併用 |
| Dependabot alerts | 有効 | Renovate と二重防御 |
| Secret scanning | 有効 | gitleaks との二重 |

## ブランチ保護（main）

詳細仕様は [`docs/05_実装/30_CI_CD設計/50_branch_protection/01_branch_protection.md`](../docs/05_実装/30_CI_CD設計/50_branch_protection/01_branch_protection.md) を正典とする。terraform-provider-github 構成を `infra/github/` に配置（後段）。

| 項目 | 値 |
|---|---|
| 必須 status check | `ci-overall` 1 本（集約 job） |
| strict mode | 有効（main 最新を含む状態でのみ merge 可） |
| 必須レビュー数 | 1（個人 OSS）/ 2（採用拡大期、ADR で切替） |
| dismiss stale review | 有効 |
| require code owner review | 有効 |
| 許可する merge 方式 | squash merge のみ |
| linear history | 必須 |
| require signed commits | 有効 |
| enforce_admins | 有効（管理者にも適用） |
| direct push to main | 禁止（管理者含む） |
| allow deletions | 無効 |
| allow force pushes | 無効 |
| required conversation resolution | 必須 |
| merge queue | リリース時点 では無効。PR 量 > 50/day で ADR を経て有効化 |

`release/*` ブランチも main と同一設定。

## Actions 設定

| 項目 | 値 | 理由 |
|---|---|---|
| Actions の有効範囲 | `Allow OWNER and select non-OWNER actions` | 第三者 actions を whitelist 制で許可 |
| Workflow permissions | `Read repository contents and packages permissions` | 既定読み取りのみ、書込みは per-job で `permissions:` ブロックで明示 |
| Allow GitHub Actions to create and approve pull requests | 有効 | Renovate / 自動更新 PR で必要 |
| Fork pull request workflows | `Require approval for first-time contributors` | 初回 fork PR の actions 実行に承認必須 |
| Artifact retention days | 30（main）/ 7（PR） | デフォルト |

## Secrets / Variables（リリース時点 想定）

リリース時点 で必要となる secrets / variables の最低集合。実値は GitHub UI で設定し、本ファイルでは **キー名のみ** を記録する（実値の管理経路は `docs/05_実装/85_Identity設計/` の secrets-matrix と整合）。

| キー | スコープ | 用途 |
|---|---|---|
| `RENOVATE_TOKEN` | repository | Renovate self-hosted 用 GitHub PAT |
| `NUGET_API_KEY` | environment: release | NuGet 公開（K1s0.Sdk） |
| `NPM_TOKEN` | environment: release | npm 公開（@k1s0/sdk） |
| `CRATES_IO_TOKEN` | environment: release | crates.io 公開（k1s0-sdk） |
| `COSIGN_*` | — | OIDC keyless のため secret 不要 |

GH Actions の `GITHUB_TOKEN` は OIDC + GHCR push に再利用。

## Environments

| 環境 | 用途 | 保護 |
|---|---|---|
| `release` | リリース pipeline 実行用 | required reviewer 1 名（manual approval） |
| `staging`（任意） | staging 配信 | wait timer 0 |
| `production`（任意） | prod 配信 | required reviewer 2 名 + wait timer 5 分 |

リリース時点 では `release` のみ作成、`staging` / `production` は採用側 cluster 設置に応じて追加。

## terraform-provider-github 適用フロー

設定変更は `infra/github/*.tf` に対する PR で行い、`tools/ci/jobs/tf-apply.sh`（後段）が CI から `terraform plan` / `apply` を実行する。Web UI での直接設定変更は禁止し、変更履歴をすべて Git に集約する。

## 適用ステータス

| 項目 | 状態 | 備考 |
|---|---|---|
| description / topics / homepage | 未適用 | 本ファイル確定後に GitHub UI で適用 |
| ブランチ保護 main | 未適用 | terraform-provider-github 構成 (`infra/github/`) は後段で作成 |
| CODEOWNERS | ✅ 配置済 | `.github/CODEOWNERS` |
| labels | 未適用 | `.github/labels.yml` 配置済、sync スクリプトは後段 |
| Security advisories | 未適用 | UI で有効化必要 |
| CodeQL | 未適用 | `.github/workflows/codeql.yml`（後段） |

リリース直前（フェーズ 16-01 / 16-02）に本ファイルの全項目が適用済みであることを確認する。

## 関連

- [`.github/CODEOWNERS`](CODEOWNERS) — レビュー担当
- [`.github/labels.yml`](labels.yml) — ラベル定義
- [`docs/05_実装/30_CI_CD設計/50_branch_protection/01_branch_protection.md`](../docs/05_実装/30_CI_CD設計/50_branch_protection/01_branch_protection.md)
- [`plan/16_OSS公開準備/01_GitHub_repo_settings.md`](../plan/16_OSS公開準備/01_GitHub_repo_settings.md)
