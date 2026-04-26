# 15. monorepo CI orchestration（命名規約 / cache 階層 / matrix scaling / artifact passing）

本ファイルは k1s0 モノレポ全体の **GitHub Actions ワークフロー** を体系的に管理するための規約を確定する。10 章 reusable workflow 設計（4 本構成）と 20 章 path-filter（11 軸）を前提とし、本ファイルは「`.github/workflows/` 配下が将来 30+ ファイルに育っても破綻しない」体系を担う。

## 関連設計

- [10_reusable_workflow/01_reusable_workflow設計.md](10_reusable_workflow/01_reusable_workflow設計.md)（4 本構成 / IMP-CI-RWF-010 / 011 / 021）
- [20_path_filter選択ビルド/](20_path_filter選択ビルド/)（11 軸の path-filter）
- [30_quality_gate/](30_quality_gate/)（4 段 quality gate）
- [50_branch_protection/01_branch_protection.md](50_branch_protection/01_branch_protection.md)（`ci-overall` 1 本必須）
- [plan/02_開発環境整備/11_monorepo_CI_orchestration.md](../../../plan/02_開発環境整備/11_monorepo_CI_orchestration.md)
- 配置: [`.github/workflows/`](../../../.github/workflows/) / [`tools/ci/actions/`](../../../tools/ci/actions/) / [`tools/ci/path-filter.yaml`](../../../tools/ci/path-filter.yaml)

## 1. workflow taxonomy（命名規約と配置）

`.github/workflows/` 配下のファイルは **prefix で 5 系統** に分類する。各 prefix は trigger と責務を一意に示す。

| prefix | trigger | 責務 | 例 |
|---|---|---|---|
| `_reusable-*.yml` | `workflow_call` のみ | 他 workflow から呼ばれる reusable | `_reusable-lint.yml` / `_reusable-test.yml` / `_reusable-build.yml` / `_reusable-push.yml` / `_reusable-precommit.yml` |
| `pr*.yml` | `pull_request` | PR 検証（path-filter で reusable を呼ぶ） | `pr.yml` |
| `release*.yml` | tag push or `workflow_dispatch` | リリース pipeline（image push / SDK 公開 / GH Release） | `release.yml`（plan 17-04 で配置） |
| `nightly*.yml` | `schedule:` 夜間 | 重い検証（fuzz / chaos / full matrix / load light） | `nightly-fuzz.yml`（plan 11-05）/ `nightly-load.yml`（plan 11-08） |
| `cron*.yml` | `schedule:` 定期 | 定期メンテ（Renovate / SBOM 再生成 / link-check） | `renovate.yml` / `cron-link-check.yml`（plan 15-11） |

**規約**:

- ファイル名はすべて lowercase（GitHub UI のソートを揺らさないため）
- prefix と trigger が **不整合**な workflow は禁止（例: `pr-*.yml` で `schedule:` を持つのは禁止）
- `_reusable-*.yml` は `name: _reusable-X` で本体を識別、他から呼ぶ際は `uses: ./.github/workflows/_reusable-X.yml@<ref>`（IMP-CI-RWF-017 に従い `@main` 直参照は本リポ内呼出のみ許可）
- 本リポジトリは **モノレポ単一リポ**で reusable workflow も同居させる方針（IMP-CI-RWF-020）。`@<tag>` 参照は外部リポからの呼出時にのみ必須

## 2. cache 階層（L1 / L2 / L3）

CI 時間 SLI（リリース時点 30 分 / リリース時点+ 20 分以内、NFR-C-NOP-004）を守るため、cache を 3 層に分ける。

### L1: job 内 cache（actions/cache）

各言語の dependency cache を job ごとに保持する。キー規約は IMP-CI-RWF-016 と整合。

| 言語 | path | key |
|---|---|---|
| Rust | `~/.cargo` + workspace `target/` | `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}` |
| Go | `~/go/pkg/mod` + `~/.cache/go-build` | `${{ runner.os }}-gomod-${{ hashFiles('**/go.sum') }}` |
| .NET | `~/.nuget/packages` | `${{ runner.os }}-nuget-${{ hashFiles('**/packages.lock.json', '**/*.csproj') }}` |
| Node | `~/.local/share/pnpm/store` | `${{ runner.os }}-pnpm-${{ hashFiles('**/pnpm-lock.yaml') }}` |
| buf | BSR plugin cache | `${{ runner.os }}-buf-${{ hashFiles('buf.lock') }}` |
| docker buildx | `/tmp/.buildx-cache` | `${{ runner.os }}-buildx-${{ github.sha }}` |

実装は `tools/ci/actions/setup-{rust,go,dotnet,node}/action.yml` の composite action で集約（IMP-CI-RWF-021）。

### L2: job 間 artifact

`_reusable-build.yml` が出力する image / SBOM を、`_reusable-push.yml` が読む形で **job 間で artifact 経由** で受け渡す。直接 docker daemon 経由で渡さない（push job が異なる runner に乗る場合に対応）。

### L3: main ブランチ pre-warm（オプション、リリース時点+）

`schedule:` で 1 日 1 回 main ブランチの cache を warm する `cron-cache-warm.yml` を導入予定（リリース時点 では未配置、PR の cache hit が 80% 未満になった時点で発動を検討）。

## 3. matrix scaling 方針

matrix の組合せ爆発を避けるため、**段階別に matrix の幅を変える**。

| 段階 | OS | 言語バージョン | 想定組合せ数 |
|---|---|---|---|
| **PR**（短時間検証） | `ubuntu-22.04` のみ | 各言語 stable のみ | 最小（4-5 jobs） |
| **main**（merge 後） | `ubuntu-22.04` のみ | 各言語 LTS 1 系のみ | 中（10 jobs） |
| **nightly**（夜間） | `ubuntu-22.04` + `ubuntu-24.04` | 各言語 LTS + nightly | 多（30+ jobs） |
| **release**（tag push） | nightly と同等 + `arm64`（リリース時点+） | 全 LTS | 最大 |

**禁止**:
- PR で OS matrix を回す（例: macos / windows を PR で回さない、コスト超過）
- nightly で `arm64` × 全言語の cross-matrix（リリース時点+ 3 ヶ月で導入予定、現状は warning 段階）

## 4. artifact passing 規約

### 命名規約

`<artifact-type>-<image-or-pkg-name>-<github.sha>` の 3 セクションで衝突回避。例:

- `build-k1s0-tier1-facade-abcdef1` — _reusable-build から _reusable-push への中間
- `coverage-tier1-rust-abcdef1` — _reusable-test から集約 job への coverage report
- `sbom-k1s0-tier1-facade-abcdef1` — SBOM 単独保管

### retention（保持期間）

| ブランチ | retention |
|---|---|
| PR ブランチ | 7 日 |
| main / develop | 30 日 |
| `release/*` ブランチ | 90 日 |
| tag（v\*） | **永続**（リリース成果物） |

### 受け渡しの原則

- 中間 artifact は **同一 commit SHA 内でのみ有効**（push job が違う SHA を pull することは禁止）
- `actions/upload-artifact@v4` / `actions/download-artifact@v4` で v3 系の deprecation を回避
- 大容量 artifact（>100 MB）は `compression-level: 9` を必須

## 5. concurrency 規約

| workflow | concurrency group | cancel-in-progress |
|---|---|---|
| `pr.yml` | `pr-${{ github.workflow }}-${{ github.ref }}` | `true`（PR 連投で旧実行をキャンセル） |
| `_reusable-lint/test/build` | `<phase>-<workflow>-<inputs>-<github.ref>` | `true` |
| `_reusable-push` | `push-${{ inputs.image_name }}-${{ github.ref }}` | **`false`**（途中停止で half-pushed image を残さない） |
| `release*.yml` | `release-${{ github.ref }}` | **`false`**（リリース途中停止禁止） |
| `nightly*.yml` | `nightly-${{ github.workflow }}` | `true`（重複起動を抑制） |
| `cron*.yml` | `cron-${{ github.workflow }}` | `true` |

## 6. composite action の配置（IMP-CI-RWF-021）

reusable workflow から呼び出す共通部品（言語 toolchain setup / cache 設定 / 共通 step）は `tools/ci/actions/<name>/action.yml` に配置する。`.github/actions/` ではなく `tools/ci/` 配下に置く理由:

- sparse-checkout cone で `infra-ops` 役にも `tools/ci/` を含めることで CI 部品の編集も役切替で完結
- `tools/ci/path-filter.yaml` と並置して CI 関連を集約
- IMP-CI-RWF-021 の正典

呼出側（reusable workflow）から:

```yaml
- uses: ./tools/ci/actions/setup-rust
  with:
    components: rustfmt,clippy
```

composite action は **reusable workflow の内部実装** であり、PR の直接呼出（`uses: ./tools/ci/actions/...` を `.github/workflows/pr.yml` から）は禁止する。

## 7. 観測性（CI 自体の SLI）

CI workflow 実行時間 / 成功率を以下のメトリクスで集計する:

- `ci.workflow.duration_seconds{workflow,status}` — 各 workflow の実行時間
- `ci.workflow.failure_count{workflow,reason}` — 失敗パターン分布
- `ci.cache.hit_ratio{cache_name}` — L1 cache のヒット率（80% 目標）

リリース時点 では実装簡略化のため、`workflow_run` event で集計する `tools/ci/jobs/ci-metrics.sh`（リリース時点+ で配置）が Mimir に push する。リリース時点 では GitHub Actions の Insights タブで確認。

## 8. 数値 SLO（目標値）

リリース時点 計測ベースで以下を目指す:

| 指標 | リリース時点 | リリース時点+ |
|---|---|---|
| PR CI 時間（path-filter 後） | 15 分以内 | 10 分以内 |
| main full CI 時間 | 30 分以内 | 20 分以内 |
| nightly 重 matrix 時間 | 90 分以内 | 60 分以内 |
| L1 cache hit rate | 80% | 90% |
| flaky test 率 | 2% 以下 | 1% 以下 |

実機計測は plan 12-06 DX メトリクスと連動する。

## 9. 対応 IMP ID

- IMP-CI-MR-001: workflow taxonomy 規約
- IMP-CI-MR-002: cache 3 層階層
- IMP-CI-MR-003: matrix scaling 方針
- IMP-CI-MR-004: artifact passing 規約
- IMP-CI-MR-005: concurrency 規約
- IMP-CI-MR-006: composite action 配置（`tools/ci/actions/` 集約）

## 関連

- [10_reusable_workflow/01_reusable_workflow設計.md](10_reusable_workflow/01_reusable_workflow設計.md)
- [`tools/ci/actions/`](../../../tools/ci/actions/) — composite action 雛形
- [`tools/ci/path-filter.yaml`](../../../tools/ci/path-filter.yaml) — path-filter 11 軸
