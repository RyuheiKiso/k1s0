# tests — tier 横断テスト

各 tier 内部の unit test は言語標準の場所（`src/tier1/go/**/*_test.go` 等）に配置する。本ディレクトリは tier 横断の以下 6 カテゴリを集約する。

## 設計正典

- [`docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md`](../docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md) — 配置仕様
- [`docs/04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md`](../docs/04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md) — テスト戦略
- [`docs/04_概要設計/10_システム方式設計/05_システム結合テスト要件.md`](../docs/04_概要設計/10_システム方式設計/05_システム結合テスト要件.md) — システム結合テスト
- [`docs/04_概要設計/20_ソフトウェア方式設計/06_ソフトウェア結合テスト要件.md`](../docs/04_概要設計/20_ソフトウェア方式設計/06_ソフトウェア結合テスト要件.md) — ソフトウェア結合テスト
- [`docs/02_構想設計/04_CICDと配信/03_テスト戦略.md`](../docs/02_構想設計/04_CICDと配信/03_テスト戦略.md) — 構想設計

## 6 カテゴリ

| ディレクトリ | 役割 | 実行環境 | 起動 |
|---|---|---|---|
| [`e2e/`](e2e/) | tier1→tier2→tier3 を通す end-to-end シナリオ | kind cluster + `infra/environments/dev/` | CI (週次) / 手動 |
| [`contract/`](contract/) | tier 間 API 契約整合（Pact / OpenAPI） | Pact Broker（local-stack） | CI (PR 毎) |
| [`integration/`](integration/) | testcontainers で Dapr / CNPG / Kafka 連携検証 | Docker / testcontainers | CI (PR 毎) |
| [`fuzz/`](fuzz/) | Protobuf / REST 入力の Fuzz | cargo-fuzz / go-fuzz | CI (週次) / 手動 |
| [`golden/`](golden/) | scaffold 等の出力固定 snapshot | 単体 binary 実行 | CI (PR 毎) |
| [`fixtures/`](fixtures/) | 全テスト共有の test data / seed / 試験用 TLS 証明書 | — | — |

## リリース時点 のスコープ

リリース時点 では各カテゴリとも **README + 動作可能な最小サンプル + CI hook 連携の入口** を同梱する。本格的なシナリオ拡充は採用初期 段階で（`docs/03_要件定義/50_開発者体験/01_テスト戦略.md` のテスト工数見積に従う）。

## CI 連携

`.github/workflows/_reusable-test.yml` から各カテゴリのランナーが呼ばれる:

- `pnpm` / `go test ./...` / `cargo test --workspace` / `dotnet test` の各言語標準テストはまず unit test として実行
- 本ディレクトリの統合系テストは matrix で並列実行、失敗時に PR を block

## 依存関係

- `e2e/` は `tools/local-stack/up.sh` で kind cluster を起動してから実行
- `integration/` は Docker daemon が必須（GitHub Actions の `docker` services を利用）
- `fuzz/` は CI の compute time が高いため週次実行（必要時はラベル `run-fuzz` で PR でも起動可能）
