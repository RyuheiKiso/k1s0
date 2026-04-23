# 05. CI 戦略と path-filter 統合

本ファイルは CI（GitHub Actions）でスパースチェックアウトと path-filter をどう組合せるかを規定する。

## CI における sparse-checkout の方針

CI は原則 `full` 役割で動作する。つまり sparse-checkout は CI では使わない。理由:

- CI 環境は毎回新規 clone のため、partial clone + sparse のセットアップ時間 > 全 clone 時間
- CI ランナーのディスクは使い捨て
- job 種別（tier1 ビルド / tier2 ビルド等）を path-filter で絞る方が柔軟

ただし以下の例外:

- **monorepo 全体 E2E テスト**: full でチェックアウトした上で各 tier の image をビルドし、kind cluster で結合テスト実行
- **サイズ巨大な artifact ビルド**（例: Backstage の Docker image）: 必要な tier のみ sparse checkout して build 時間短縮

## path-filter による job 絞り込み

`.github/workflows/ci-main.yml` でルーティング job を定義し、下流の tier 別 job を起動する。

```yaml
# .github/workflows/ci-main.yml
name: ci-main

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  changes:
    runs-on: ubuntu-latest
    outputs:
      contracts: ${{ steps.filter.outputs.contracts }}
      tier1-rust: ${{ steps.filter.outputs.tier1-rust }}
      tier1-go: ${{ steps.filter.outputs.tier1-go }}
      tier2: ${{ steps.filter.outputs.tier2 }}
      tier3-web: ${{ steps.filter.outputs.tier3-web }}
      tier3-native: ${{ steps.filter.outputs.tier3-native }}
      infra: ${{ steps.filter.outputs.infra }}
      deploy: ${{ steps.filter.outputs.deploy }}
    steps:
      - uses: actions/checkout@v4
      - uses: dorny/paths-filter@v3
        id: filter
        with:
          filters: |
            contracts:
              - 'src/contracts/**'
            tier1-rust:
              - 'src/tier1/rust/**'
              - 'src/sdk/rust/**'
            tier1-go:
              - 'src/tier1/go/**'
              - 'src/sdk/go/**'
            tier2:
              - 'src/tier2/**'
              - 'src/sdk/dotnet/**'
            tier3-web:
              - 'src/tier3/web/**'
              - 'src/tier3/bff/**'
              - 'src/sdk/typescript/**'
            tier3-native:
              - 'src/tier3/native/**'
              - 'src/tier3/legacy-wrap/**'
            infra:
              - 'infra/**'
            deploy:
              - 'deploy/**'

  ci-contracts:
    needs: changes
    if: needs.changes.outputs.contracts == 'true'
    uses: ./.github/workflows/ci-contracts.yml

  ci-tier1-rust:
    needs: [changes, ci-contracts]
    if: needs.changes.outputs.tier1-rust == 'true' || needs.changes.outputs.contracts == 'true'
    uses: ./.github/workflows/ci-tier1-rust.yml

  ci-tier1-go:
    needs: [changes, ci-contracts]
    if: needs.changes.outputs.tier1-go == 'true' || needs.changes.outputs.contracts == 'true'
    uses: ./.github/workflows/ci-tier1-go.yml

  ci-tier2:
    needs: [changes, ci-contracts]
    if: needs.changes.outputs.tier2 == 'true' || needs.changes.outputs.contracts == 'true'
    uses: ./.github/workflows/ci-tier2.yml

  ci-tier3-web:
    needs: [changes, ci-contracts]
    if: needs.changes.outputs.tier3-web == 'true' || needs.changes.outputs.contracts == 'true'
    uses: ./.github/workflows/ci-tier3-web.yml

  ci-tier3-native:
    needs: [changes]
    if: needs.changes.outputs.tier3-native == 'true'
    uses: ./.github/workflows/ci-tier3-native.yml

  ci-infra:
    needs: changes
    if: needs.changes.outputs.infra == 'true'
    uses: ./.github/workflows/ci-infra.yml

  ci-deploy:
    needs: changes
    if: needs.changes.outputs.deploy == 'true'
    uses: ./.github/workflows/ci-deploy.yml
```

## 依存の伝播

`src/contracts/` の変更は tier1 / tier2 / tier3 全 job を起動する。path-filter の if 条件で `contracts == 'true'` を OR で入れる。

## sparse-checkout を使う特殊 CI

以下のケースで sparse-checkout を活用:

### Backstage image ビルド（高コスト）

```yaml
# .github/workflows/ci-backstage.yml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          sparse-checkout: |
            src/platform/backstage-plugins
            src/contracts
            docs
          sparse-checkout-cone-mode: true
      - name: Build Backstage image
        run: |
          cd src/platform/backstage-plugins
          pnpm install --frozen-lockfile
          pnpm build
          docker build -t harbor.k1s0.internal/platform/backstage:${{ github.sha }} .
```

### 週次 E2E（full が必要）

```yaml
# .github/workflows/ci-e2e-weekly.yml
on:
  schedule:
    - cron: '0 2 * * 0'  # 毎週日曜 02:00 UTC

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4  # sparse-checkout 指定なし = full
      - name: Setup kind cluster
        run: ./tools/local-stack/kind/bootstrap.sh
      - name: Run E2E tests
        run: cd tests/e2e && go test -v ./...
```

## CI キャッシュ戦略

role 毎に cache を分離。

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: tier1-rust-${{ runner.os }}-${{ hashFiles('src/tier1/rust/**/Cargo.lock', 'src/contracts/**/*.proto') }}
```

## path-filter の保守

path-filter の規則が不正確だと「契約変更が tier2 CI を起動しない」等の事故が起きる。

- 四半期ごとに `docs/05_実装/00_ディレクトリ設計/90_トレーサビリティ/` と path-filter 規則の整合を手動レビュー
- `tools/ci/validate-path-filter.sh` で「変更があるはずの job が起動してない」現象を検出

## 対応 IMP-DIR ID

- IMP-DIR-SPARSE-130（CI 戦略と path-filter 統合）

## 対応 ADR / 要件

- ADR-DIR-003
- DX-CICD-\*
