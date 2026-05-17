# client コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは client 固有の制約のみ記述する。

## 配置・構成

- **SDK**: `src/client/sdk/`（9 言語）
- **hlc_lib**: `src/client/hlc_lib/`（4 言語 HLC wrapper）
- **companion_mock**: `src/client/companion_mock/`
- **lock.yaml 配置先**: `src/client/lock/`（手書き禁止）
- **Cargo workspace**: `src/tier1/Cargo.toml` の member
- **go.work**: `src/tier1/go.work` の member
- **pnpm workspace**: `src/tier3/typescript/pnpm-workspace.yaml` の member

設計パターン・モジュール構成の詳細は `docs/03_概要設計/09_client設計方針/README.md` を単一の真とする。

## コーディング制約

### codegen

- protoc + Buf codegen pipeline が単一の真（手書き SDK 禁止）
- `src/client/sdk/<lang>/generated/` は codegen 出力専用（手書き edit 禁止 → diff で CI fail）
- Go SDK のみ `protoc-gen-k1s0-go-fsm` を適用（`generated/fsm/` に出力）
- Connect-RPC の .NET 8 実装は `src/_crosscutting/13_dotnet8_connect_inhouse/lib/` が primary

### SDK lockstep

- 9 言語 SDK の MAJOR.MINOR 同期（PATCH のみ skew 許容）
- 1 言語のみリリースを先行させることを禁止

### HLC

- `src/client/hlc_lib/` の言語別 wrapper のみを使う
- wall-clock TTL 禁止（全 deadline は sender-issued HLC tuple を使う）
- `CLOCK_MONOTONIC_RAW` 必須（Rust では `SystemTime::now()` を idempotency に使う path を `#[deny]` lint で reject）

### sdk/tauri

- `src/client/sdk/tauri/` は Tauri frontend から k1s0 API を呼ぶ SDK wrapper
- Connect-RPC の Tauri WebSocket bridge を含む
- companion/ と hlc_lib/typescript を依存に持つ

### companion_mock

- SDK 開発者向け local mock backend
- production code のテストで companion_mock を使わない（Testcontainers を使う）

## 関連参照

- `docs/03_概要設計/09_client設計方針/README.md` — 設計パターン
- `docs/04_詳細設計/02_強制機構/08_client強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md` — SDK 配布
