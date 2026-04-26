# Contributing to k1s0

k1s0 への貢献を歓迎します。本書は PR を出す前に必ず一読してください。

## はじめに

- **個人 OSS** のため、メンテナのレビュー帯域は限られます。Issue で議論を尽くしてから PR を出すことを推奨します。
- 大きな変更（新 API / 新 tier / アーキテクチャ変更）は事前に **ADR**（[`docs/02_構想設計/adr/`](docs/02_構想設計/adr/)）を起票してください（[`/adr` slash command](.claude/commands/adr.md)）。
- 最初のコミットを送る前に [Code of Conduct](CODE_OF_CONDUCT.md) と [LICENSE](LICENSE) に同意したものとみなします。

## 開発環境

### Dev Container 推奨

VS Code + Dev Container プロファイル（`tools/devcontainer/profiles/`）を使うと、役割別の toolchain（Rust / Go / .NET / Node / buf / kubectl / dapr CLI / drawio CLI 等）が自動で揃います。

```bash
# 役割別 sparse-checkout を有効化
./tools/sparse/checkout-role.sh tier1-go-dev   # Go ファサード開発
# 利用可能な役割: tier1-rust-dev / tier1-go-dev / tier2-dev / tier3-web-dev /
#                tier3-native-dev / platform-cli-dev / sdk-dev / infra-ops /
#                docs-writer / full
```

詳細は [`tools/devcontainer/README.md`](tools/devcontainer/README.md) と [`tools/sparse/README.md`](tools/sparse/README.md) を参照。

### ローカルセットアップ（Dev Container を使わない場合）

`make doctor` で必要な toolchain がそろっているか診断できます。

```bash
make doctor          # 必要なら role を自動検出
make pre-commit      # pre-commit hooks を全ファイルに走らせる
make codegen         # contracts → 4 言語 SDK 生成（要: buf CLI）
make lint            # proto lint + pre-commit
```

## コミット規約

### Conventional Commits 必須

[Conventional Commits 1.0](https://www.conventionalcommits.org/) を強制します（commitlint で検証）。

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

- **type**: `feat` / `fix` / `docs` / `style` / `refactor` / `perf` / `test` / `build` / `ci` / `chore` / `revert`
- **scope**: `contracts` / `sdk-{dotnet,go,rust,typescript}` / `tier1-{go,rust}` / `tier2` / `tier3-{web,native,bff,legacy}` / `platform` / `infra` / `deploy` / `ops` / `tools` / `docs` / `tests` / `security` / `deps` / `release`
- **subject**: 50 文字以内、lowercase か日本語で開始、句点なし

詳細と実例は [`commitlint.config.js`](commitlint.config.js) と [`plan/02_開発環境整備/12_コミットメッセージ規約.md`](plan/02_開発環境整備/12_コミットメッセージ規約.md) を参照。

### DCO Sign-off 必須（リリース後）

[Developer Certificate of Origin](https://developercertificate.org/) の sign-off をリリース後の commit に求めます。

```bash
git commit -s -m "fix(tier1-go): correct retry budget"
# → "Signed-off-by: Your Name <your@email>" が footer に追加される
```

## ブランチ運用

- **main**: 保護ブランチ。直接 push 禁止、PR 経由必須。
- **feature ブランチ**: `feat/<short-description>` / `fix/<short-description>` / `docs/<short-description>` 等。
- **release ブランチ**: `release/v0.x.y` で stabilize。

## PR ガイドライン

### 必須チェック

PR を出す前に以下が緑であることを確認してください（CI で自動検証されます）:

- [ ] `pre-commit run --all-files`（japanese header / 500 行制限 / link check 等）
- [ ] `buf lint && buf format -d --exit-code`（proto 変更時）
- [ ] `make codegen-check`（contracts 変更時、生成物 diff なし）
- [ ] 言語別 lint / test（変更箇所のみ自動検出、`pr.yml` 参照）
- [ ] commitlint（PR title + 全 commit）
- [ ] `ci-overall` 集約 job（branch protection の必須 status check）

### PR 説明

- **What**: 何を変えたか（実装の概要）
- **Why**: なぜ変えたか（issue 番号 / ADR ID / 要件 ID 紐付け）
- **How tested**: どう検証したか（unit / integration / 手動 / e2e）
- **Risk**: 後方互換性 / breaking change / migration 要否

### PR レビュー基準

- [Review Checklist](.claude/skills/docs-review-checklist/SKILL.md) と [Self-review checklist](docs/00_format/review_checklist.md) を参照。
- 大型 PR（500 行超）は分割を推奨。レビュー帯域を圧迫します。

## コーディング規約

### 共通

- **日本語コメント必須**: コードの各行 1 行上に日本語コメント、ファイル冒頭に説明。詳細は [`CLAUDE.md`](CLAUDE.md) と [`src/CLAUDE.md`](src/CLAUDE.md) を参照。
- **1 ファイル 500 行以内**（docs 例外、コードは厳格）。超えたら分割。
- **Secret コミット禁止**: `.env` / credentials / token は `.gitignore` 経由で除外。Renovate / OpenBao で管理。
- **生成コード手動編集禁止**: `*/generated/` 配下は `make codegen` で再生成する。

### Go（tier1 / tier2 / SDK / tools）

- Go 1.22+、`gofmt` + `goimports` 強制、`golangci-lint` v2 で 0 issue。
- `forbidigo`: tier1 では Dapr Go SDK の直接 import を `internal/adapter/dapr/` 配下に限定（ADR-TIER1-003）。
- module path は monorepo path-style: `github.com/k1s0/k1s0/src/<dir>/`。

### Rust（tier1 core / setup CLI / platform）

- Edition 2024、`#![forbid(unsafe_code)]` を crate root に置く（FFI 境界を除く）。
- panic は `catch_unwind` で FFI 境界を超えない。

### .NET（tier2 / tier3-native）

- .NET 8.0+、`dotnet format` 緑（warning も含む）。

### TypeScript（tier3-web）

- pnpm 9+ + Node 20、`pnpm lint` + `pnpm tsc --noEmit` 緑。

### Proto（contracts）

- buf v2、proto3、`STANDARD` lint。`google.api.http` annotation で REST 互換。
- `package` 命名: `k1s0.<module>.<api>.v<n>`。

## ドキュメント

- **docs 編集**: [`docs/CLAUDE.md`](docs/CLAUDE.md) と関連 Skill を必ず読む（`docs-delivery-principles` / `docs-design-spec` / `docs-adr-authoring` 等）。
- **ADR**: 構造に触る変更は ADR 必須（[`/adr` command](.claude/commands/adr.md)）。
- **drawio**: 図の編集には [`drawio-authoring` Skill](.claude/skills/drawio-authoring/SKILL.md) を必ず参照。

## テスト

- **Unit**: 各言語の native test framework（`go test` / `cargo test` / `dotnet test` / `pnpm test`）。
- **Integration**: Testcontainers で本物の依存（Postgres / Kafka / Keycloak）。Mock 禁止。
- **Contract**: Pact（消費者駆動）+ OpenAPI comparator。
- **Fuzz**: `cargo-fuzz`（Rust）/ `go-fuzz`（Go）を nightly で。
- **Chaos**: LitmusChaos で縮退動作を仕様化。

## セキュリティ

セキュリティ脆弱性の報告は [SECURITY.md](SECURITY.md) を参照。**公開 issue / PR で報告しないこと**。

## ライセンス

貢献されたコードは [Apache License 2.0](LICENSE) で配布されます。コントリビュータは自分のコントリビューションを Apache 2.0 でライセンスする権利を持つことを保証する必要があります（DCO 適用後はこれを sign-off で明示）。

## 質問 / 議論

- **General questions**: GitHub Discussions（リリース後に有効化）。
- **Bug reports**: GitHub Issues + Bug template。
- **Feature requests**: GitHub Issues + Feature template + （大型なら）ADR 起票。
- **Security**: SECURITY.md の手順に従う。

ご貢献をお待ちしています。
