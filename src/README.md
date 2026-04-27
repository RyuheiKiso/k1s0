# src — k1s0 ソースコード一次配置

本ディレクトリは k1s0 全コード（contracts / tier1 / SDK / tier2 / tier3 / platform）を集約する。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/02_src配下の層別分割.md`](../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/02_src配下の層別分割.md)。

## 配置構造

```text
src/
├── CLAUDE.md           # コーディング規約（日本語コメント / 500 行制限 / 言語別基本）
├── contracts/          # *.proto — single source of truth（tier1 公開 12 API + 内部 gRPC）
├── tier1/
│   ├── go/             # Dapr Go SDK ファサード（3 Pod: state / secret / workflow）
│   └── rust/           # ZEN Engine / 暗号 / 雛形 CLI（3 Pod: decision / audit / pii）
├── sdk/                # contracts から自動生成（4 言語: dotnet / go / rust / typescript）
├── tier2/              # ドメイン共通業務ロジック（C# / Go）
├── tier3/              # Web（React + TS）/ Native（.NET MAUI）/ BFF（Go）/ Legacy wrap（.NET Framework）
└── platform/           # scaffold CLI / dependency analyzer / Backstage plugins
```

## 依存方向（一方向のみ、逆向きは CI で遮断）

```text
tier3 → tier2 → (sdk ← contracts) → tier1 → infra
```

逆方向の参照は内製 analyzer（`src/platform/analyzer/`）と `tools/ci/{go,rust}-dep-check/` の二重で reject される。
詳細は [`docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md`](../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md)。

## コーディング規約（必読）

[`CLAUDE.md`](CLAUDE.md) を必ず参照すること。要点:

- **日本語コメント必須**: 全コードファイルの先頭に日本語の説明コメント、各行の 1 行上にも日本語コメント
- **1 ファイル 500 行以内**: 例外なし。`tools/git-hooks/file-length-guard.py` で機械的に強制
- **tier1 内部のサービス間通信は Protobuf gRPC 必須**
- **tier2 / tier3 から内部言語（Rust / Go）は不可視**: クライアントライブラリと gRPC エンドポイントのみ公開

## サブディレクトリ別の詳細設計

| ディレクトリ | 詳細設計 |
|---|---|
| `contracts/` | [docs/05_実装/00_ディレクトリ設計/30_共通契約レイアウト/](../docs/05_実装/00_ディレクトリ設計/30_共通契約レイアウト/) |
| `tier1/` | [docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/](../docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/) |
| `sdk/` | [docs/05_実装/00_ディレクトリ設計/30_共通契約レイアウト/](../docs/05_実装/00_ディレクトリ設計/30_共通契約レイアウト/) |
| `tier2/` | [docs/05_実装/00_ディレクトリ設計/35_tier2レイアウト/](../docs/05_実装/00_ディレクトリ設計/35_tier2レイアウト/) |
| `tier3/` | [docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/](../docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/) |
| `platform/` | [docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/02_src配下の層別分割.md](../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/02_src配下の層別分割.md) |

## 実装マチュリティの開示

採用検討者は [`docs/SHIP_STATUS.md`](../docs/SHIP_STATUS.md) を参照すること。docs（設計）が記述する全体像と
実コードの実装率（同梱済 / 雛形あり / 設計のみ）を領域別に開示している。
