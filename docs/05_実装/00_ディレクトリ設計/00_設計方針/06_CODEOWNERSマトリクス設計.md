# 06. CODEOWNERS マトリクス設計

本ファイルは `.github/CODEOWNERS` による path-pattern ベースの所有権定義を固定する。モノレポにおいて責務境界がディレクトリ階層で表現されている（IMP-DIR-ROOT-001）ことを前提に、CODEOWNERS がこの境界を機械的に強制する仕組みを整える。

## CODEOWNERS が必要な理由

モノレポは単一リポジトリに全資産が集約されるため、PR レビュー時に「誰の承認が必要か」を自動判定する仕組みがないと、以下の問題が発生する。

- tier1 所有の Go コードへの PR が tier3 メンバーだけのレビューでマージされ、tier1 チームが後から気付く
- 契約変更（`src/contracts/`）が契約レビュー担当を経由せずにマージされる
- infra の重要変更が SRE を通らずにマージされ、本番障害を引き起こす
- ドキュメント修正が無関係な全員にレビュー依頼が飛ぶ

GitHub の CODEOWNERS は path-pattern と GitHub team / user の対応を定義するだけのファイルだが、Branch Protection Rule と組み合わせることで「CODEOWNERS で指定された owner の approval が必須」を強制できる。

## チーム構成前提

採用検討時点では実在の人員は 2 名だが、運用蓄積後のチーム拡大を見据えて以下の GitHub team を予約する。GitHub org は `k1s0`（運用蓄積後で本設立）を想定。

- `@k1s0/arch-council` : アーキテクチャ評議会（技術 Lead + Product Owner）
- `@k1s0/contract-reviewers` : 契約レビュー担当
- `@k1s0/tier1-rust` : tier1 Rust 実装者
- `@k1s0/tier1-go` : tier1 Go 実装者
- `@k1s0/tier2-dev` : tier2 実装者
- `@k1s0/tier3-web` : tier3 Web 実装者
- `@k1s0/tier3-native` : tier3 Native（MAUI）実装者
- `@k1s0/sdk-team` : SDK 4 言語実装者
- `@k1s0/platform-team` : 雛形 CLI / analyzer / Backstage 実装者
- `@k1s0/sre-ops` : SRE + 運用担当
- `@k1s0/gitops-team` : GitOps 配信担当
- `@k1s0/docs-team` : ドキュメント執筆担当
- `@k1s0/security-team` : セキュリティレビュー担当
- `@k1s0/compliance-team` : コンプライアンス監査担当
- `@k1s0/release-managers` : リリース管理担当

リリース時点では全チームが同一の 2 名に割り当てられる状態で開始する。リリース時点 以降の採用進捗に応じて段階的に分離する。

## マトリクス定義

`.github/CODEOWNERS` の path-pattern は以下の方針で定義する。順序は GitHub CODEOWNERS の評価順（後勝ち）に従い、粒度の細かいルールを下に配置する。

### 全体デフォルト

```text
# 全体のフォールバック - アーキテクチャ評議会が確認
*                                               @k1s0/arch-council
```

### ルート直下ファイル

```text
# ルート直下の重要ファイル
/CLAUDE.md                                      @k1s0/arch-council
/README.md                                      @k1s0/arch-council @k1s0/docs-team
/LICENSE                                        @k1s0/compliance-team
/NOTICE                                         @k1s0/compliance-team
/SECURITY.md                                    @k1s0/security-team
/GOVERNANCE.md                                  @k1s0/arch-council
/CODE_OF_CONDUCT.md                             @k1s0/arch-council
/CONTRIBUTING.md                                @k1s0/arch-council @k1s0/docs-team
/CHANGELOG.md                                   @k1s0/release-managers
/Makefile                                       @k1s0/platform-team @k1s0/sre-ops
/.gitattributes                                 @k1s0/arch-council @k1s0/sre-ops
/.gitignore                                     @k1s0/arch-council
/.git-blame-ignore-revs                         @k1s0/arch-council
```

### GitHub ワークフロー

```text
# CI/CD 定義は SRE と arch-council の両方の approval が必要
/.github/                                       @k1s0/sre-ops @k1s0/arch-council
/.github/workflows/                             @k1s0/sre-ops @k1s0/arch-council
/.github/CODEOWNERS                             @k1s0/arch-council
/.github/PULL_REQUEST_TEMPLATE.md               @k1s0/arch-council @k1s0/docs-team
/.github/ISSUE_TEMPLATE/                        @k1s0/arch-council @k1s0/docs-team
```

### スパースチェックアウト cone 定義

```text
# cone 定義の変更は arch-council + sre-ops の approval が必要
/.sparse-checkout/                              @k1s0/arch-council @k1s0/sre-ops
```

### 契約（最優先）

```text
# Protobuf 契約は契約レビュー担当が必須
/src/contracts/                                 @k1s0/contract-reviewers
/src/contracts/buf.yaml                         @k1s0/contract-reviewers @k1s0/arch-council
/src/contracts/buf.gen.*.yaml                   @k1s0/contract-reviewers @k1s0/platform-team
/src/contracts/tier1/                           @k1s0/contract-reviewers @k1s0/tier1-rust @k1s0/tier1-go
/src/contracts/internal/                        @k1s0/contract-reviewers @k1s0/tier1-rust @k1s0/tier1-go
```

### tier1

```text
# tier1 実装は各言語担当
/src/tier1/                                     @k1s0/tier1-rust @k1s0/tier1-go
/src/tier1/go/                                  @k1s0/tier1-go
/src/tier1/rust/                                @k1s0/tier1-rust
```

### SDK

```text
# SDK は SDK チーム + 契約レビュー（契約変更に追随する場合）
/src/sdk/                                       @k1s0/sdk-team
/src/sdk/dotnet/                                @k1s0/sdk-team @k1s0/tier3-native
/src/sdk/go/                                    @k1s0/sdk-team @k1s0/tier2-dev
/src/sdk/typescript/                            @k1s0/sdk-team @k1s0/tier3-web
/src/sdk/rust/                                  @k1s0/sdk-team @k1s0/tier1-rust
```

### tier2

```text
/src/tier2/                                     @k1s0/tier2-dev
/src/tier2/dotnet/                              @k1s0/tier2-dev
/src/tier2/go/                                  @k1s0/tier2-dev
/src/tier2/templates/                           @k1s0/tier2-dev @k1s0/platform-team
```

### tier3

```text
/src/tier3/                                     @k1s0/tier3-web @k1s0/tier3-native
/src/tier3/web/                                 @k1s0/tier3-web
/src/tier3/native/                              @k1s0/tier3-native
/src/tier3/bff/                                 @k1s0/tier3-web @k1s0/tier2-dev
/src/tier3/legacy-wrap/                         @k1s0/tier3-native @k1s0/platform-team
```

### platform

```text
/src/platform/                                  @k1s0/platform-team
/src/platform/cli/                              @k1s0/platform-team @k1s0/tier1-rust
/src/platform/analyzer/                         @k1s0/platform-team
/src/platform/backstage-plugins/                @k1s0/platform-team
```

### infra

```text
/infra/                                         @k1s0/sre-ops
/infra/k8s/                                     @k1s0/sre-ops
/infra/mesh/                                    @k1s0/sre-ops @k1s0/security-team
/infra/dapr/                                    @k1s0/sre-ops @k1s0/tier1-go
/infra/data/                                    @k1s0/sre-ops @k1s0/tier1-rust
/infra/security/                                @k1s0/security-team @k1s0/sre-ops
/infra/observability/                           @k1s0/sre-ops
/infra/feature-management/                      @k1s0/sre-ops @k1s0/tier1-go
/infra/scaling/                                 @k1s0/sre-ops
/infra/environments/                            @k1s0/sre-ops @k1s0/arch-council
```

### deploy

```text
/deploy/                                        @k1s0/gitops-team @k1s0/sre-ops
/deploy/apps/                                   @k1s0/gitops-team
/deploy/charts/                                 @k1s0/gitops-team @k1s0/sre-ops
/deploy/kustomize/                              @k1s0/gitops-team
/deploy/rollouts/                               @k1s0/gitops-team @k1s0/sre-ops
/deploy/opentofu/                               @k1s0/sre-ops @k1s0/arch-council
/deploy/image-updater/                          @k1s0/gitops-team
```

### ops

```text
/ops/                                           @k1s0/sre-ops
/ops/runbooks/                                  @k1s0/sre-ops @k1s0/docs-team
/ops/chaos/                                     @k1s0/sre-ops
/ops/dr/                                        @k1s0/sre-ops @k1s0/arch-council
/ops/oncall/                                    @k1s0/sre-ops
/ops/load/                                      @k1s0/sre-ops
/ops/scripts/                                   @k1s0/sre-ops
```

### tools

```text
/tools/                                         @k1s0/platform-team
/tools/devcontainer/                            @k1s0/platform-team @k1s0/arch-council
/tools/local-stack/                             @k1s0/platform-team @k1s0/sre-ops
/tools/codegen/                                 @k1s0/platform-team @k1s0/contract-reviewers
/tools/sparse/                                  @k1s0/platform-team @k1s0/sre-ops
/tools/ci/                                      @k1s0/sre-ops @k1s0/platform-team
/tools/git-hooks/                               @k1s0/arch-council @k1s0/platform-team
/tools/migration/                               @k1s0/sre-ops @k1s0/arch-council
```

### tests / examples / third_party

```text
/tests/                                         @k1s0/sre-ops @k1s0/platform-team
/tests/e2e/                                     @k1s0/sre-ops
/tests/contract/                                @k1s0/contract-reviewers @k1s0/platform-team
/tests/integration/                             @k1s0/platform-team

/examples/                                      @k1s0/platform-team @k1s0/tier2-dev @k1s0/tier3-web

/third_party/                                   @k1s0/arch-council @k1s0/compliance-team @k1s0/security-team
```

### docs

```text
/docs/                                          @k1s0/docs-team
/docs/00_format/                                @k1s0/docs-team @k1s0/arch-council
/docs/01_企画/                                  @k1s0/arch-council
/docs/02_構想設計/                              @k1s0/arch-council
/docs/02_構想設計/adr/                          @k1s0/arch-council
/docs/03_要件定義/                              @k1s0/arch-council
/docs/04_概要設計/                              @k1s0/arch-council
/docs/05_実装/                                  @k1s0/arch-council @k1s0/platform-team
```

## 承認必須数の設定

Branch Protection Rule と組み合わせ、以下の必須承認数を設定する。

- `main` ブランチへの PR: CODEOWNERS に列挙されるすべての owner から 1 名以上の承認
- 契約変更（`src/contracts/` 配下）: `@k1s0/contract-reviewers` から 複数名以上の承認（運用蓄積後、チーム 3 名以上時）
- セキュリティ関連（`infra/security/` / `src/tier1/rust/crates/audit/` / `src/tier1/rust/crates/pii/`）: `@k1s0/security-team` から 1 名以上の承認

## CODEOWNERS 変更時の運用

CODEOWNERS 自体の変更は `@k1s0/arch-council` の承認が必須。変更は以下の場合に発生する。

- 新規 GitHub team の追加
- 既存 team の解散 / 統合
- ディレクトリ構造変更に伴う path-pattern 追従
- 段階進展に伴う責任分離の細分化

変更時は対応 IMP-DIR を更新し、[../90_トレーサビリティ/03_ADR_との対応.md](../90_トレーサビリティ/03_ADR_との対応.md) で関連 ADR を参照する。

## 対応 IMP-DIR ID

本ファイルは CODEOWNERS 方針そのものであり、直接の IMP-DIR 採番はしない。各実装レイアウト章で CODEOWNERS との対応を明記する。

## 対応 ADR

- ADR-DIR-001 / ADR-DIR-002 / ADR-DIR-003

## 対応要件

- NFR-C-NOP-001（採用側の小規模運用）/ NFR-H-AUD-\*（監査）/ DX-CICD-\* / 制約 8
