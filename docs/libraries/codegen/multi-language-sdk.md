> **ステータス: 未実装（設計のみ）**

# Multi-language Client SDK 生成

## 概要

Proto 定義から Go / Rust / TypeScript / Dart の4言語クライアント SDK を自動生成するパイプライン。
各言語で Interface (trait) + gRPC 実装 + HTTP 実装 + Mock 実装のテンプレートを提供する。

## 使い方

```bash
# 全言語の SDK を生成
./scripts/generate-client-sdk.sh \
  --service user-profile \
  --proto api/proto/user-profile

# 特定言語のみ
./scripts/generate-client-sdk.sh \
  --service user-profile \
  --languages go,ts \
  --proto api/proto/user-profile

# justfile 経由
just gen-sdk service=user-profile proto=api/proto/user-profile
```

### オプション

| オプション | 必須 | デフォルト | 説明 |
|-----------|------|-----------|------|
| `--service` | Yes | - | サービス名 (e.g., `user-profile`) |
| `--languages` | No | `go,rust,ts,dart` | カンマ区切りの対象言語 |
| `--proto` | Yes | - | `.proto` ファイルを含むディレクトリ |

## 生成ファイル構成

### Go

```
regions/system/library/go/{service}_client/
  client.go         # Interface 定義
  grpc_client.go    # gRPC 実装
  http_client.go    # HTTP 実装
  mock_client.go    # Mock 実装
  go.mod
```

### Rust

```
regions/system/library/rust/{service}-client/
  Cargo.toml
  src/
    lib.rs          # Interface (trait) + re-export
    grpc_client.rs  # gRPC 実装
    http_client.rs  # HTTP 実装
    mock_client.rs  # Mock 実装
```

### TypeScript

```
regions/system/library/typescript/{service}_client/
  package.json
  tsconfig.json
  src/
    client.ts       # Interface 定義
    grpc-client.ts  # gRPC 実装
    http-client.ts  # HTTP 実装
    mock-client.ts  # Mock 実装
```

### Dart

```
regions/system/library/dart/{service}_client/
  pubspec.yaml
  lib/
    src/
      client.dart       # Abstract interface class
      grpc_client.dart  # gRPC 実装
      http_client.dart  # HTTP 実装
      mock_client.dart  # Mock 実装
```

## テンプレート配置先

```
CLI/crates/k1s0-cli/templates/client-sdk/
  go/           # Go テンプレート (.tera)
  rust/         # Rust テンプレート (.tera)
  typescript/   # TypeScript テンプレート (.tera)
  dart/         # Dart テンプレート (.tera)
```

テンプレートは [Tera](https://keats.github.io/tera/) 形式で、以下の変数が利用可能:

| 変数 | 例 | 説明 |
|------|-----|------|
| `{{ service_name }}` | `user-profile` | サービス名 (ケバブケース) |
| `{{ service_snake }}` | `user_profile` | スネークケース |
| `{{ service_pascal }}` | `UserProfile` | パスカルケース |

## Proto コード生成との関係

`buf.gen.yaml` で Proto から各言語の型・gRPC スタブを生成し、本スクリプトでクライアント SDK のスキャフォールドを生成する。
Proto 生成は `just proto`、SDK 生成は `just gen-sdk` で実行する。
