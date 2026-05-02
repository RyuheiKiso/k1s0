# 01. tools 配置

本ファイルは `tools/` 配下の配置を確定する。開発者ツール・横断スクリプトを集約する場所。

## tools/ の役割

`src/platform/scaffold/` にある Rust 実装の k1s0-scaffold CLI と、`ops/scripts/` の運用スクリプトの中間に位置し、「開発時に使うが、運用本番では実行されないスクリプト」を置く。

- Dev Container 設定
- Local-stack（kind / k3d のブートストラップ）
- コード生成（buf / openapi / scaffold）
- スパースチェックアウト CLI
- CI ヘルパー
- Git hooks
- 古いリポジトリからの migration 支援

## レイアウト

```text
tools/
├── README.md
├── devcontainer/
│   └── profiles/                   # 役割別 Dev Container 定義
│       ├── tier1-rust-dev/
│       ├── tier1-go-dev/
│       ├── tier2-dev/
│       ├── tier3-web-dev/
│       ├── tier3-native-dev/
│       ├── platform-cli-dev/
│       ├── infra-ops/
│       └── docs-writer/
├── banner/
│       └── generate_banner.py
├── local-stack/
│   ├── README.md
│   ├── kind/
│   │   ├── cluster-config.yaml
│   │   └── bootstrap.sh
│   ├── k3d/
│   │   ├── cluster-config.yaml
│   │   └── bootstrap.sh
│   └── teardown.sh
├── codegen/
│   ├── README.md
│   ├── buf/                        # Protobuf → Go / Rust / C# / TS
│   │   ├── buf.gen.yaml
│   │   └── gen.sh
│   ├── openapi/                    # OpenAPI → TS / C# client
│   │   ├── openapi-generator-config.yaml
│   │   └── gen.sh
│   └── scaffold/                   # 新サービス雛形
│       ├── tier2-dotnet-service.hbs
│       ├── tier2-go-service.hbs
│       ├── tier3-web-app.hbs
│       └── gen.sh
├── sparse/                         # スパースチェックアウト CLI
│   ├── README.md
│   ├── checkout-role.sh
│   └── list-roles.sh
├── ci/
│   ├── README.md
│   ├── detect-changed-tiers.sh     # path-filter で変更 tier を検出
│   ├── drift-check.sh              # 生成コード差分検出
│   └── release-notes-generator.sh
├── git-hooks/
│   ├── pre-commit
│   ├── pre-push
│   └── install.sh                  # hooks を .git/hooks に symlink
└── migration/                      # 既存 .NET Framework 資産の移行支援
    ├── README.md
    ├── framework-to-sidecar/
    └── framework-to-net8/
```

## devcontainer/profiles/ の構造

各ロール用の Dev Container を 8 個用意。ルート `.devcontainer/` は `docs-writer`（最も軽量）をデフォルトとし、開発者は VSCode の「Reopen in Container」で `tools/devcontainer/profiles/<role>/` を明示選択する。詳細は `05_devcontainer配置.md` で規定。

## local-stack/ の kind / k3d

開発者ローカルで k8s を動かすためのブートストラップ。

```bash
# tools/local-stack/kind/bootstrap.sh
#!/usr/bin/env bash
set -euo pipefail

kind create cluster --config="$(dirname "$0")/cluster-config.yaml" --name k1s0-dev

# CNI (Cilium)
kubectl apply -f ../../infra/k8s/bootstrap/cilium/

# 最小 Dapr インストール
kubectl apply -f ../../infra/dapr/control-plane/

# environments/dev/ の overlay を apply
kubectl apply -k ../../infra/environments/dev/

echo "k1s0-dev cluster ready. Run 'tools/sparse/checkout-role.sh <role>' to scope workspace."
```

## codegen/ の役割

### buf/

Protobuf → 各言語コード生成。`src/contracts/` の .proto を入力に Go / Rust / C# / TypeScript コードを生成する。`buf.gen.*.yaml`（4 ファイル: go / rust / ts / csharp）の実体は `src/contracts/` に置かれており、本 `tools/codegen/buf/gen.sh` はその wrapper として `buf generate` の実行順序・差分検知・ローカルツール版固定を行う。`buf.gen.*.yaml` の正規定義は [../20_tier1レイアウト/02_contracts配置.md](../20_tier1レイアウト/02_contracts配置.md) に単一原典として集約し、本書で重複定義しない（二重管理による設定ドリフトを防ぐため）。

```bash
# tools/codegen/buf/gen.sh（抜粋）
#!/usr/bin/env bash
set -euo pipefail

# .tool-versions の buf を起動、src/contracts/buf.gen.*.yaml を順次実行
cd "$(dirname "$0")/../../../src/contracts"
for tmpl in buf.gen.go.yaml buf.gen.rust.yaml buf.gen.ts.yaml buf.gen.csharp.yaml; do
    buf generate --template "$tmpl"
done
```

詳細は `06_codegen配置.md` で規定。

### openapi/

BFF の REST endpoint から OpenAPI schema を抽出し、tier3 Web 向け TypeScript client を生成。

### scaffold/

新サービスの雛形生成。handlebars template で Service 名・Owner team 等を差し込む。`src/platform/scaffold/` の Rust CLI から呼ばれる低レベル実装。

## sparse/ の CLI

役割選択でスパースチェックアウトを切り替える。

```bash
# tools/sparse/checkout-role.sh
#!/usr/bin/env bash
set -euo pipefail

ROLE="${1:-}"
if [[ -z "$ROLE" ]]; then
  echo "Usage: $0 <role>"
  echo "Roles:"
  ls -1 "$(dirname "$0")/../../.sparse-checkout/roles/" | sed 's/\.txt$//'
  exit 1
fi

ROLE_FILE="$(dirname "$0")/../../.sparse-checkout/roles/${ROLE}.txt"
if [[ ! -f "$ROLE_FILE" ]]; then
  echo "Role '$ROLE' not found. Available roles:"
  ls -1 "$(dirname "$0")/../../.sparse-checkout/roles/" | sed 's/\.txt$//'
  exit 1
fi

# cone mode + partial clone 前提
git sparse-checkout init --cone
git sparse-checkout set --stdin < "$ROLE_FILE"
echo "Switched to role: $ROLE"
```

リリース時点 では Rust 実装の CLI が `tools/sparse/checkout-role.sh` を置き換える。

## ci/ の役割

### detect-changed-tiers.sh

GitHub Actions の path-filter で変更対象 tier を検出、該当 workflow のみ起動。

```bash
# tools/ci/detect-changed-tiers.sh
#!/usr/bin/env bash
CHANGED_FILES=$(git diff --name-only HEAD~1 HEAD)
TIERS=()

if echo "$CHANGED_FILES" | grep -q "^src/tier1/rust/"; then
  TIERS+=("tier1-rust")
fi
if echo "$CHANGED_FILES" | grep -q "^src/tier1/go/"; then
  TIERS+=("tier1-go")
fi
if echo "$CHANGED_FILES" | grep -q "^src/tier2/"; then
  TIERS+=("tier2")
fi
if echo "$CHANGED_FILES" | grep -q "^src/tier3/web/"; then
  TIERS+=("tier3-web")
fi
if echo "$CHANGED_FILES" | grep -q "^src/contracts/"; then
  TIERS+=("contracts")
fi

echo "changed-tiers=${TIERS[*]}" >> "$GITHUB_OUTPUT"
```

### drift-check.sh

buf generate を実行し、生成コードと commit 済みコードの diff を検出。

## git-hooks/ の役割

- **pre-commit**: format 未実行 / trailing whitespace / 巨大ファイル検出
- **pre-push**: 簡易テスト実行 + import-boundary lint

Husky や pre-commit framework でなく、素の Git hooks + install.sh で symlink する軽量方式を採用。

### pre-push による import boundary 早期検出

pre-push は push 前に `tools/ci/lint-import-boundaries.go`（Go）/ `cargo-deny`（Rust）/ `eslint-plugin-boundaries`（TypeScript）/ NetArchTest（.NET）の 4 系統を並列に起動し、CLAUDE.md の依存方向違反を検出する。CI より前にローカルで fail するため、違反 PR が main に届く前に手戻りが生じる。

```bash
# tools/git-hooks/pre-push（抜粋）
#!/usr/bin/env bash
set -euo pipefail

# 変更 tier を検出
CHANGED=$(tools/ci/detect-changed-tiers.sh)

# 言語別に並列で lint
if echo "$CHANGED" | grep -q "tier1-go\|tier2-go\|tier3-bff\|sdk-go"; then
  go run ./tools/ci/lint-import-boundaries.go &
fi
if echo "$CHANGED" | grep -q "tier1-rust\|sdk-rust"; then
  cargo deny check bans sources &
fi
if echo "$CHANGED" | grep -q "tier3-web\|sdk-typescript"; then
  (cd src/tier3/web && pnpm lint) &
fi
if echo "$CHANGED" | grep -q "tier2-dotnet\|sdk-dotnet\|tier3-native"; then
  (cd tests && dotnet test --filter Category=Architecture) &
fi
wait
```

hook を skip したい場合は `git push --no-verify` が使えるが、CI 側で同じ lint を必須化しており bypass しても PR は merge できない。

## 対応 IMP-DIR ID

- IMP-DIR-COMM-111（tools 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-DEVEX-002（Dev Container 標準化）
- DX-GP-\* / DX-CICD-\*
