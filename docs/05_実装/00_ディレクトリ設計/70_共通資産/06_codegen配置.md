# 06. codegen 配置

本ファイルは `tools/codegen/` 配下の配置を確定する。Protobuf / OpenAPI / scaffold の各コード生成ツールを統合し、開発者と CI が同じ生成結果を得ることを保証する。

## codegen/ の役割

k1s0 は以下 3 種類のコード生成を行う。

- **Protobuf generation**: `src/contracts/` の .proto → Go / Rust / C# / TypeScript クライアント
- **OpenAPI generation**: tier3 BFF の OpenAPI spec → TypeScript / C# client
- **Scaffold generation**: 新サービス雛形（tier2 .NET / Go、tier3 web / BFF）

これら 3 つをまとめて `tools/codegen/` に集約し、開発者の手順と CI パイプラインで同じスクリプトを呼べるようにする。

## レイアウト

```
tools/codegen/
├── README.md
├── buf/
│   └── gen.sh                      # 単一 wrapper。buf.gen.*.yaml 実体は src/contracts/ に単一原典
├── openapi/
│   ├── openapi-generator-config.yaml
│   ├── templates/                  # カスタムテンプレート（必要時）
│   └── gen.sh
├── scaffold/
│   ├── handlebars/
│   │   ├── tier2-dotnet-service.hbs
│   │   ├── tier2-go-service.hbs
│   │   ├── tier3-web-app.hbs
│   │   ├── tier3-bff-graphql.hbs
│   │   └── partials/
│   │       ├── header-dotnet.hbs
│   │       └── header-go.hbs
│   ├── k1s0-scaffold/              # Rust 実装の scaffold ランナー
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── gen.sh
└── check-drift.sh                  # 生成結果と commit 済みの diff 検出
```

`tools/codegen/buf/` 配下には yaml 実体を置かない。設定ドリフトを防ぐため、`buf.gen.tier1.yaml` と `buf.gen.sdk.yaml` の 2 原典は `src/contracts/` に限定配置し、本 wrapper からはパス指定で参照する（詳細は `gen.sh` セクション）。

## buf/ の運用

### buf.gen.yaml

`src/contracts/` を入力に複数言語コードを生成する。buf v2 形式を採用。

本書は `buf.gen.*.yaml` の運用的補足（plugin 選定の根拠・tool version 連携）を扱う。`buf.gen.tier1.yaml` と `buf.gen.sdk.yaml` の正規定義は [../20_tier1レイアウト/02_contracts配置.md](../20_tier1レイアウト/02_contracts配置.md) に単一原典として集約する（二重管理による設定ドリフトを防ぐため）。tier1 内部生成と SDK 生成は yaml を物理的に分離して internal package の SDK 混入を構造的に防止する。以下は原典から抜粋した出力先の俯瞰表。

| 言語 | yaml | 用途 | 出力先 | 参考 plugin |
|---|---|---|---|---|
| Go | tier1 | tier1 実装 | `src/tier1/go/internal/proto/` | `buf.build/protocolbuffers/go` + `buf.build/grpc/go` |
| Go | sdk | SDK | `src/sdk/go/proto/` | 同上（`paths=source_relative`） |
| Rust | tier1 | tier1 実装 | `src/tier1/rust/crates/proto-gen/src/` | `buf.build/community/neoeinstein-prost` + `buf.build/community/neoeinstein-tonic` |
| Rust | sdk | SDK（Phase 2） | `src/sdk/rust/crates/k1s0-sdk-proto/src/gen/` | 同上 |
| C# | sdk | SDK | `src/sdk/dotnet/generated/` | `buf.build/protocolbuffers/csharp` + `buf.build/grpc/csharp` |
| TypeScript | sdk | SDK | `src/sdk/typescript/packages/proto/src/` | `buf.build/connectrpc/es`（connectrpc 採用） |

### gen.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

echo "=== Protobuf lint ==="
buf lint src/contracts

echo "=== Protobuf breaking change check ==="
buf breaking src/contracts --against ".git#branch=main,subdir=src/contracts"

echo "=== Protobuf code generation (tier1 internal) ==="
(cd src/contracts && buf generate --template buf.gen.tier1.yaml)

echo "=== Protobuf code generation (SDK public-only) ==="
(cd src/contracts && buf generate --template buf.gen.sdk.yaml)

echo "=== Go mod tidy ==="
(cd src/sdk/go && go mod tidy)

echo "=== Rust cargo fmt ==="
(cd src/sdk/rust && cargo fmt)

echo "=== Done ==="
```

## openapi/ の運用

### openapi-generator-config.yaml

tier3 BFF の OpenAPI spec を input に、TypeScript client と C# client を生成。

```yaml
# tools/codegen/openapi/openapi-generator-config.yaml
generatorName: typescript-fetch
inputSpec: src/tier3/bff/internal/rest/openapi.yaml
outputDir: src/sdk/typescript/src/rest-gen
additionalProperties:
  npmName: "@k1s0/rest-client"
  withInterfaces: true
  supportsES6: true
```

### gen.sh

```bash
#!/usr/bin/env bash
cd "$(git rev-parse --show-toplevel)"

openapi-generator-cli generate \
    --config tools/codegen/openapi/openapi-generator-config.yaml

(cd src/sdk/typescript && pnpm run build)
```

## scaffold/ の運用

### handlebars/

handlebars テンプレートで新サービスを生成。

```handlebars
{{!-- tools/codegen/scaffold/handlebars/tier2-go-service.hbs --}}
{{> header-go}}

package main

// {{service-name}} エントリポイント
import (
    "context"
    "log/slog"
    "github.com/k1s0/k1s0/src/tier2/go/shared/config"
    "github.com/k1s0/k1s0/src/tier2/go/shared/otel"
    "github.com/k1s0/k1s0/src/tier2/go/services/{{service-name}}/internal/api"
)

func main() {
    cfg := config.Load("{{service-name}}")
    shutdown := otel.Init(cfg.Otel)
    defer shutdown()

    slog.Info("starting {{service-name}}")

    srv := api.NewServer(cfg)
    if err := srv.Run(context.Background()); err != nil {
        slog.Error("server error", "err", err)
    }
}
```

### k1s0-scaffold/

scaffold の実体。Rust で handlebars を render し、ファイルをプロジェクトに書き出す。

```bash
# tools/codegen/scaffold/gen.sh
#!/usr/bin/env bash
cd "$(git rev-parse --show-toplevel)/tools/codegen/scaffold"
cargo run --release -- "$@"
```

使用例:

```bash
tools/codegen/scaffold/gen.sh \
    --type tier2-go-service \
    --name customer-billing \
    --owner tier2-fintech \
    --output src/tier2/go/services/customer-billing
```

## check-drift.sh

CI で「commit 済みの生成コードが最新 .proto / OpenAPI spec と一致するか」を検証。

```bash
#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# 現在の生成コードをバックアップ
cp -r src/sdk /tmp/sdk-backup

# 再生成
tools/codegen/buf/gen.sh
tools/codegen/openapi/gen.sh

# diff 検出
if ! diff -qr /tmp/sdk-backup src/sdk > /dev/null; then
    echo "ERROR: generated code is out of date."
    echo "Run tools/codegen/buf/gen.sh and tools/codegen/openapi/gen.sh then commit."
    diff -r /tmp/sdk-backup src/sdk || true
    exit 1
fi

echo "OK: generated code is up to date."
```

`.github/workflows/ci-drift-check.yml` で全 PR に対して実行。drift があれば CI fail。

## 生成コードの commit 方針

ADR-DIR-001 に従い、生成コードは全て commit する。理由:

- 初回 build 時間短縮（依存が揃っていなくても読める）
- レビュー時に生成 API の変更が diff で見える
- offline build 対応
- CI の generate step を省略可能

`.gitattributes` で `linguist-generated=true` を付与し、GitHub の PR diff から自動除外。

## 対応 IMP-DIR ID

- IMP-DIR-COMM-116（codegen 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-DIR-001（contracts 昇格）
- ADR-CODE-001（buf 採用）
- DX-GP-\* / DX-CICD-\*
