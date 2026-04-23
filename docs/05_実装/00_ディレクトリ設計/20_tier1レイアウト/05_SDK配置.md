# 05. SDK 配置

本ファイルは `src/sdk/` 配下の 4 言語 SDK（C# / Go / TypeScript / Rust）の物理配置を確定する。SDK は tier1 公開 11 API の多言語クライアントライブラリとして、tier1 実装と独立に配布される。

## SDK 独立配置の根拠

SDK を tier1 配下ではなく `src/sdk/` に独立配置する理由は以下。

- SDK は tier2 / tier3 / 外部システム（tier4 顧客）が使うクライアントライブラリであり、tier1 所有物ではない
- 4 言語それぞれが独自のパッケージ管理（NuGet / Go module / npm / cargo）でリリースされる
- SDK のバージョニングは tier1 実装のバージョンと独立（SDK は tier1 v1 API を表現し、tier1 内部実装の変更に影響されない）
- tier2 / tier3 開発者は SDK のみを依存関係に持ち、tier1 の実装を意識しない（ADR-TIER1-003 の内部言語不可視を物理的に保証）

## レイアウト

```
src/sdk/
├── README.md
├── dotnet/                 # C# NuGet
│   ├── Sdk.sln
│   ├── Directory.Build.props
│   ├── generated/          # buf generate の出力（gRPC stub）
│   ├── src/
│   │   ├── K1s0.Sdk/
│   │   │   ├── K1s0.Sdk.csproj
│   │   │   ├── StateClient.cs
│   │   │   ├── SecretsClient.cs
│   │   │   ├── WorkflowClient.cs
│   │   │   └── ...
│   │   └── K1s0.Sdk.Auth/
│   │       ├── K1s0.Sdk.Auth.csproj
│   │       └── JwtValidator.cs
│   ├── tests/
│   │   ├── K1s0.Sdk.Tests/
│   │   │   └── K1s0.Sdk.Tests.csproj
│   │   └── K1s0.Sdk.IntegrationTests/
│   └── samples/
│       └── HelloSdk/
├── go/                     # Go module（独立）
│   ├── README.md
│   ├── go.mod              # module github.com/k1s0/k1s0/src/sdk/go
│   ├── go.sum
│   ├── proto/              # buf generate の出力
│   │   ├── v1/
│   │   │   ├── state.pb.go
│   │   │   └── ...
│   ├── client/
│   │   ├── state.go
│   │   ├── secrets.go
│   │   └── workflow.go
│   ├── auth/
│   └── examples/
├── typescript/             # npm workspace
│   ├── README.md
│   ├── package.json
│   ├── pnpm-workspace.yaml
│   ├── tsconfig.base.json
│   ├── packages/
│   │   ├── proto/          # buf generate の出力
│   │   │   ├── package.json
│   │   │   └── src/
│   │   ├── client/
│   │   │   ├── package.json
│   │   │   ├── src/
│   │   │   │   ├── state.ts
│   │   │   │   ├── secrets.ts
│   │   │   │   └── ...
│   │   │   └── test/
│   │   └── auth/
│   │       ├── package.json
│   │       └── src/
│   └── examples/
└── rust/                   # Cargo workspace（Phase 2 骨組み）
    ├── Cargo.toml
    ├── rust-toolchain.toml
    └── crates/
        ├── k1s0-sdk/
        │   ├── Cargo.toml
        │   └── src/lib.rs
        └── k1s0-sdk-auth/
            ├── Cargo.toml
            └── src/lib.rs
```

## 4 言語の独立ビルド

各 SDK は独立したビルドツールを持ち、相互に依存しない。

### dotnet

`.sln` 単位で管理。`src/sdk/dotnet/Sdk.sln` に 2 csproj（`K1s0.Sdk` / `K1s0.Sdk.Auth`）を含める。

`Directory.Build.props` で共通プロパティ（LangVersion / Nullable / TargetFramework）を定義。

### go

独立 go.mod。`github.com/k1s0/k1s0/src/sdk/go` モジュール。tier1 Go（`src/tier1/go/`）の go.mod とは別。

### typescript

pnpm workspace。`packages/proto` / `packages/client` / `packages/auth` の 3 パッケージで構成。`pnpm-workspace.yaml` は src/sdk/typescript/ に配置し、tier3/web/ の workspace とは独立。

### rust

Phase 2 骨組みとして最小実装のみ（`k1s0-sdk` crate の `lib.rs` に TODO コメント）。Phase 2 で Dapr Rust SDK の stable 化状況を見て本実装に進む。

## 生成コードの配置

buf generate による gRPC stub 生成物は各言語の `generated/` または `proto/` 配下に commit。

- `dotnet/generated/` : C# generated classes
- `go/proto/v1/` : Go generated .pb.go + _grpc.pb.go
- `typescript/packages/proto/src/` : TypeScript generated .ts
- `rust/crates/proto-gen/` : Rust generated .rs（Phase 2）

CI で `buf generate` 実行後、diff が 0 であることを検証。

## 公開（publish）戦略

Phase 1a: 社内限定。GitHub Packages / 社内 NuGet / 社内 npm registry に publish。外部公開しない。

Phase 1b: 必要に応じて public publish を検討。ただし OSS ライセンス・商標確認が必要。

Phase 1c: 公式 SDK として nuget.org / npmjs.com / pkg.go.dev / crates.io に publish 検討（Phase 0 稟議時点では未決定）。

## 依存方向

- SDK は `src/contracts/tier1/v1/*.proto` のみを入力とする
- SDK は tier1 / tier2 / tier3 / infra を参照しない
- tier2 / tier3 は SDK を介して tier1 にアクセスする（直接参照禁止）

CI lint で依存方向を検証する（[../10_ルートレイアウト/05_依存方向ルール.md](../10_ルートレイアウト/05_依存方向ルール.md) 参照）。

## CODEOWNERS

```
/src/sdk/                    @k1s0/sdk-team
/src/sdk/dotnet/             @k1s0/sdk-team @k1s0/tier3-native
/src/sdk/go/                 @k1s0/sdk-team @k1s0/tier2-dev
/src/sdk/typescript/         @k1s0/sdk-team @k1s0/tier3-web
/src/sdk/rust/               @k1s0/sdk-team @k1s0/tier1-rust
```

## スパースチェックアウト cone

各 SDK は対応する tier 開発者の cone に含める。

- `tier1-rust-dev` cone : `src/sdk/rust/`
- `tier1-go-dev` cone : `src/sdk/go/`（tier1 Go 側からの参照なしだが、SDK 単体動作確認用）
- `tier2-dev` cone : `src/sdk/dotnet/` + `src/sdk/go/`
- `tier3-web-dev` cone : `src/sdk/typescript/`
- `tier3-native-dev` cone : `src/sdk/dotnet/`
- `platform-cli-dev` cone : 全 SDK（雛形生成時に参照）
- `full` cone : すべて含む

## テスト戦略

各 SDK で以下を実施。

- **unit test**: 各言語のネイティブ test framework（xUnit / go test / vitest / cargo test）
- **integration test**: モック gRPC サーバまたは実際の tier1 Pod 起動で動作確認
- **contract test**: [../../70_共通資産/02_tests配置.md](../../70_共通資産/02_tests配置.md) の Pact 経由で tier1 側と契約整合を検証

## 対応 IMP-DIR ID

- IMP-DIR-T1-025（src/sdk/ 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-003（内部言語不可視）/ ADR-TIER1-002（Protobuf）
- DS-SW-COMP-122 / DS-SW-COMP-139（Phase 2 拡張）
- FR-\*（tier1 公開 11 API 全般）/ DX-GP-\* / DX-CICD-\*
