# CLI 設計書

k1s0 CLI（Rust 製対話式 CLI）の設計・コード生成・マイグレーション・GUI に関するドキュメント一覧。

## トップレベル設計書

| ドキュメント | 内容 |
|------------|------|
| [テンプレート設計.md](./テンプレート設計.md) | CLI テンプレートシステム全体設計 |
| [コード生成.md](./コード生成.md) | CLI によるコード生成機能設計 |

---

## Doctor コマンド（開発環境診断）

`k1s0 doctor` コマンドは開発環境の依存ツールや設定を診断するサブコマンド。

```bash
k1s0 doctor
```

### 動作

- `scripts/doctor.sh` を検出して実行する
- ツールのインストール確認（go, cargo, node, pnpm, flutter, just, sqlx-cli 等）
- Docker / Kubernetes 設定の確認

### スクリプト探索順序（CLI-02 監査対応）

1. `K1S0_ROOT` 環境変数が設定されている場合: `$K1S0_ROOT/scripts/doctor.sh`
2. 実行ファイルの上位ディレクトリを遡って `scripts/doctor.sh` を探す
3. カレントディレクトリの `scripts/doctor.sh`（フォールバック）

### エラー時の対処

`doctor.sh が見つかりません` と表示された場合は、以下のように `K1S0_ROOT` を設定する:

```bash
export K1S0_ROOT=/path/to/k1s0
k1s0 doctor
```

### 現在の制限事項（MED-10 監査対応）

- `--json` 等の非インタラクティブ出力フォーマットは未実装。CI での使用にはシェルスクリプトを直接実行すること:
  ```bash
  bash scripts/doctor.sh
  ```
- 個別チェックの選択実行（`--check <name>`）は将来実装予定（別チケット管理）。

---

## flow — CLI フロー設計

| ドキュメント | 内容 |
|------------|------|
| [flow/CLIフロー.md](./flow/CLIフロー.md) | CLI の対話フロー・コマンド体系・ユーザー操作フロー |

---

## config — 設定・ナビゲーション設計

| ドキュメント | 内容 |
|------------|------|
| [config/config設計.md](./config/config設計.md) | `config.yaml` スキーマ設計・環境別管理・バリデーション |
| [config/navigation設計.md](./config/navigation設計.md) | CLI ナビゲーション・メニュー構成設計 |
| [config/config-editor設計.md](./config/config-editor設計.md) | 対話式 config エディター設計 |

---

## codegen — コード生成設計

| ドキュメント | 内容 |
|------------|------|
| [codegen/イベントコード生成設計.md](./codegen/イベントコード生成設計.md) | Kafka イベントコード自動生成・Proto からの型生成 |

---

## migration — マイグレーション設計

| ドキュメント | 内容 |
|------------|------|
| [migration/マイグレーション管理設計.md](./migration/マイグレーション管理設計.md) | DB マイグレーション管理・バージョン追跡・ロールバック設計 |
| [migration/テンプレートマイグレーション設計.md](./migration/テンプレートマイグレーション設計.md) | テンプレートバージョンアップ時のマイグレーション設計 |

---

## deps — 依存関係設計

| ドキュメント | 内容 |
|------------|------|
| [deps/依存関係マップ設計.md](./deps/依存関係マップ設計.md) | CLI が依存するサービス・ライブラリの依存関係マップ |

---

## dev — ローカル開発設計

| ドキュメント | 内容 |
|------------|------|
| [dev/ローカル開発環境設計.md](./dev/ローカル開発環境設計.md) | CLI 自体のローカル開発環境・ビルド手順 |

---

## gui — Tauri GUI 設計

| ドキュメント | 内容 |
|------------|------|
| [gui/TauriGUI設計.md](./gui/TauriGUI設計.md) | Tauri + React による GUI モード設計・UI コンポーネント構成 |

---

## 関連ドキュメント

- [テンプレート仕様](../templates/README.md) — CLI が使用するコード生成テンプレート
- [インフラ設計書](../infrastructure/README.md) — 開発環境セットアップ
- [アーキテクチャ設計書](../architecture/README.md) — 全体設計方針
- [ライブラリ設計書](../libraries/README.md) — codegen ライブラリ設計
