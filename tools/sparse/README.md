# `tools/sparse/` — Sparse-checkout 役割切替ツール

本ディレクトリは [`docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/`](../../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/) の運用設計を実装する。

## 配置

```text
tools/sparse/
├── README.md            # 本ファイル
├── checkout-role.sh     # 役割切替の実体スクリプト（IMP-DIR-SPARSE-127 + IMP-DEV-DC-010）
└── verify.sh            # 10 役 cone 定義の構文・整合性検証（CI / pre-commit から呼ぶ）
```

役割定義の cone は [`/.sparse-checkout/roles/<role>.txt`](../../.sparse-checkout/roles/) に保持される。

## Getting Started（採用側 / 新規 contributor 向け）

`git clone` 直後の最小手順:

```bash
# 1. partial clone + sparse-checkout 同時指定で軽量取得
git clone --filter=blob:none --sparse <url> k1s0
cd k1s0

# 2. cone mode 有効化（checkout-role.sh が自動実行するが、明示でも可）
git config core.sparseCheckoutCone true

# 3. 役を選んで cone を適用（.devcontainer/devcontainer.json も同時に張替）
./tools/sparse/checkout-role.sh tier1-rust-dev
```

### partial clone を忘れた場合の救済

`--filter=blob:none --sparse` を付けずにフルクローンしてしまった場合:

```bash
# 既存リポジトリで partial clone を有効化
git config remote.origin.partialclonefilter blob:none
git config remote.origin.promisor true
git config core.sparseCheckoutCone true

# sparse-checkout を初期化して再取得
git sparse-checkout init --cone
./tools/sparse/checkout-role.sh <role>

# 不要 blob を GC で削除（ディスク回収）
git gc --prune=now --aggressive
```

`--filter=blob:none` 適用後の `git pull` 系操作は通常通り動作する（必要な blob は lazy fetch）。完全オフライン環境では `--filter=blob:none` ではなく `--filter=tree:0` を選び、blob 取得を完全に止める運用も可能。

## 利用

```bash
# 役割切替（cone 設定 + .devcontainer/devcontainer.json symlink 張替え）
./tools/sparse/checkout-role.sh tier1-rust-dev

# 兼任（カンマ区切りで複数 role をマージ。symlink は先頭 role）
./tools/sparse/checkout-role.sh -m tier1-rust-dev,tier2-dev

# 検証（変更せず現在の整合のみ確認）
./tools/sparse/checkout-role.sh tier1-rust-dev --verify

# dry-run（変更内容を表示のみ）
./tools/sparse/checkout-role.sh tier1-rust-dev --dry-run

# 利用可能 role 一覧
./tools/sparse/checkout-role.sh --list

# 全 10 役の cone 定義を構文・整合性検証（CI 用）
./tools/sparse/verify.sh
./tools/sparse/verify.sh --strict        # プレースホルダ未存在も fail
./tools/sparse/verify.sh --json          # 機械可読出力
./tools/sparse/verify.sh tier1-rust-dev  # 単一役のみ
```

切替後に Dev Container を再起動（VS Code の "Rebuild Container"）すると、新しい役割の image でビルドされる。

## 役比較表（10 役）

各 role がカバーするディレクトリと、典型的な開発作業の対応を示す。**サイズ概算**は本リポジトリ初期段階（実装着手前、docs 主体）での値で、実装が進むと差分が顕著化する。

| Role | 主用途 | 主な include パス | include 数 | サイズ概算\* |
|---|---|---|---:|---:|
| `docs-writer` | docs / ADR / Knowledge 編集（リポジトリ既定） | `docs/`, `tools/sparse/`, `tools/devcontainer/profiles/docs-writer/`, `CLAUDE.md`, `.github/`, `.devcontainer/` | 6 | ≈255 MB |
| `tier1-rust-dev` | tier1 Rust コア（ZEN / 暗号 / Audit / scaffold CLI） | + `src/contracts/`, `src/tier1/rust/`, `src/platform/scaffold/`, `tools/codegen/buf/`, `tests/fuzz/rust/`, `tests/contract/`, `examples/tier1-rust-service/` | 13 | ≈255 MB |
| `tier1-go-dev` | tier1 Go ファサード（Dapr 統合） | + `src/contracts/`, `src/tier1/go/`, `src/sdk/go/`, `tools/codegen/buf/`, `tests/contract/`, `tests/integration/go/`, `examples/tier1-go-facade/` | 13 | ≈255 MB |
| `tier2-dev` | tier2 ドメイン共通（C# / Go） | + `src/contracts/`, `src/tier2/`, `src/sdk/go/`, `src/sdk/dotnet/`, `tools/codegen/`, `tests/contract/`, `tests/integration/`, `examples/tier2-{dotnet,go}-service/` | 15 | ≈255 MB |
| `tier3-web-dev` | tier3 Web (React + TS + Vite) / BFF (Go) | + `src/sdk/typescript/`, `src/sdk/go/`, `src/tier3/web/`, `src/tier3/bff/`, `tools/codegen/{openapi,buf}/`, `examples/tier3-{web-portal,bff-graphql}/` | 14 | ≈255 MB |
| `tier3-native-dev` | tier3 .NET MAUI / Legacy wrap (.NET Framework) | + `src/sdk/dotnet/`, `src/tier3/native/`, `src/tier3/legacy-wrap/`, `tools/codegen/buf/`, `examples/tier3-native-maui/` | 11 | ≈255 MB |
| `platform-cli-dev` | scaffold CLI / analyzer / Backstage plugin | + `src/contracts/`, `src/platform/`, `src/sdk/rust/`, `tools/codegen/`, `tests/golden/`, `examples/` | 12 | ≈255 MB |
| `sdk-dev` | 4 言語 SDK 横断開発 | + `src/contracts/`, `src/sdk/`, `tools/codegen/`, `tests/contract/`, `examples/` | 11 | ≈255 MB |
| `infra-ops` | infra / deploy / ops 横断（k8s / GitOps） | + `infra/`, `deploy/`, `ops/`, `tools/{ci,local-stack}/`, `tests/integration/` | 11 | ≈256 MB |
| `full` | 全可視（CI 専用、日常開発非推奨） | `/*` | 1 | ≈258 MB |

\* リリース前の実装ゼロ段階での値。サイズはほぼ docs (≈250 MB) に支配される。実装着手後は role 間で 数百 MB〜数 GB の差分が顕著化する見込み。

兼任モード（`-m role1,role2`）の場合、cone は両 role の union、symlink は先頭 role の Dev Container プロファイルが採用される。

## 検証スクリプト（verify.sh）

`verify.sh` は CI で `tools/sparse/checkout-role.sh` の妥当性を機械的に保証する。

| チェック項目 | 内容 |
|---|---|
| ファイル存在 | `.sparse-checkout/roles/<role>.txt` が全 10 役で配置されているか |
| プロファイル整合 | 対応する `tools/devcontainer/profiles/<role>/devcontainer.json` が存在するか |
| 構文（cone mode） | 各行が「先頭 `/`」「ディレクトリは末尾 `/`」「個別ファイル指定（CLAUDE.md 等）」を満たすか |
| パス実在 | include パスがリポジトリに実在するか（`--strict` で fail、デフォルトは warn） |
| `--list` 整合 | `checkout-role.sh --list` の出力と期待 role 集合が一致するか |

非 strict モードでは未実装ディレクトリ（`/tests/fuzz/rust/` `/examples/tier1-go-facade/` 等）は warning に留め、CI を fail させない。実装が進むに従い段階的に warn が消えていく前提。

## 新規ディレクトリの cone 漏れ防止

`src/` / `infra/` / `deploy/` 等に新規ディレクトリを追加した PR では、関連する `.sparse-checkout/roles/<role>.txt` の更新も必須。`CONTRIBUTING.md` のチェックリストに含める運用とする。

## 関連

- 設計: [`docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md`](../../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md)
- 切替運用: [`docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/04_役割切替運用.md`](../../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/04_役割切替運用.md)
- 10 役 Dev Container: [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md)
- ADR: [ADR-DIR-003](../../docs/02_構想設計/adr/ADR-DIR-003-sparse-checkout-cone-mode.md)（sparse-checkout cone mode）
- IMP ID: IMP-DIR-SPARSE-127 / IMP-DEV-DC-010
