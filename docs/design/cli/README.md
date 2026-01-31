# CLI 設計書

## 概要

k1s0 CLI は、サービスの雛形生成、規約チェック、テンプレート更新支援を行う開発支援ツールです。

## ドキュメント構成

| ドキュメント | 説明 |
|-------------|------|
| [対話モード](./interactive-mode.md) | 対話式インターフェースの設計 |
| [基本コマンド（init, new-feature, new-screen）](./commands-basic.md) | リポジトリ初期化、サービス・画面の雛形生成 |
| [ドメインコマンド](./commands-domain.md) | new-domain、ドメイン管理、カタログ、依存グラフ |
| [lint コマンド](./commands-lint.md) | 規約違反検査 |
| [upgrade コマンド](./commands-upgrade.md) | テンプレート更新 |
| [doctor コマンド](./commands-doctor.md) | 環境診断 |
| [docker コマンド](./commands-docker.md) | Docker ビルド・Compose 操作 |
| [playground コマンド](./commands-playground.md) | サンプル付き一時環境の生成・起動・停止 |
| [LSP サーバー](./lsp.md) | k1s0-lsp の設計 |
| [エラーハンドリング](./error-handling.md) | エラー型・終了コード |
| [出力制御・UX 機能](./output.md) | 出力モード、UX 改善 |
| [今後の拡張予定](./future.md) | ロードマップ |

## Crate 構成

```
CLI/crates/
├── k1s0-cli/           # CLI メインプログラム
│   └── src/
│       ├── main.rs     # エントリーポイント
│       ├── lib.rs      # CLI 定義（clap）
│       ├── error.rs    # エラー型
│       ├── output.rs   # 出力制御
│       ├── doctor/     # 環境診断モジュール
│       │   ├── mod.rs
│       │   ├── checker.rs
│       │   ├── requirements.rs
│       │   └── recommendation.rs
│       ├── prompts/   # 対話式プロンプト
│       │   ├── mod.rs
│       │   ├── command_select.rs
│       │   ├── template_type.rs
│       │   ├── name_input.rs
│       │   ├── options.rs
│       │   ├── confirm.rs
│       │   ├── version_input.rs
│       │   ├── feature_select.rs
│       │   └── init_options.rs
│       ├── settings.rs  # 設定管理
│       └── commands/   # サブコマンド実装
│           ├── init.rs
│           ├── new_feature.rs
│           ├── new_domain.rs
│           ├── new_screen.rs
│           ├── lint.rs
│           ├── upgrade.rs
│           ├── doctor.rs
│           ├── completions.rs
│           ├── registry.rs
│           ├── domain_list.rs
│           ├── domain_version.rs
│           ├── domain_dependents.rs
│           ├── domain_impact.rs
│           ├── domain_catalog.rs
│           ├── domain_graph.rs
│           ├── feature_update_domain.rs
│           ├── docker.rs
│           └── playground.rs
│
└── k1s0-generator/     # テンプレートエンジン（別設計書参照）
```

## コマンド一覧

| コマンド | 説明 | 主要オプション |
|---------|------|---------------|
| `init` | リポジトリ初期化 | `--force`, `--template-source` |
| `new-feature` | サービス雛形生成 | `-t/--type`, `-n/--name`, `--with-grpc`, `--with-rest`, `--with-db` |
| `new-screen` | 画面雛形生成 | `-t/--type`, `-s/--screen-id`, `-T/--title`, `-f/--feature-dir` |
| `lint` | 規約違反検査 | `--rules`, `--exclude-rules`, `--strict`, `--fix`, `--json`, `--watch`, `--diff`, `--debounce-ms`, `--no-config` |
| `upgrade` | テンプレート更新 | `--check`, `-y/--yes`, `--managed-only` |
| `doctor` | 環境診断 | `--verbose`, `--json`, `--check`, `--strict` |
| `completions` | シェル補完生成 | `--shell` |
| `domain-catalog` | ドメインカタログ表示 | `--language`, `--include-deprecated`, `--json` |
| `domain-list` | ドメイン一覧表示 | `--language`, `--json` |
| `domain-version` | ドメインバージョン管理 | `--name`, `--bump`, `--set`, `--message` |
| `domain-dependents` | ドメイン依存先表示 | `--name`, `--json` |
| `domain-impact` | バージョン影響分析 | `--name`, `--from`, `--to`, `--json` |
| `domain-graph` | ドメイン依存グラフ出力 | `--format`, `--root`, `--detect-cycles` |
| `feature-update-domain` | feature のドメイン依存更新 | `--name`, `--domain`, `--version` |
| `registry` | テンプレートレジストリ操作 | - |
| `docker build` | Docker イメージをビルド | `--tag`, `--no-cache`, `--http-proxy`, `--https-proxy` |
| `docker compose up` | docker compose サービスを起動 | `-d/--detach`, `--build` |
| `docker compose down` | docker compose サービスを停止 | `-v/--volumes` |
| `docker compose logs` | docker compose ログを表示 | `-f/--follow`, `<service>` |
| `docker status` | コンテナ状態を表示 | `--json` |
| `playground start` | サンプル付き playground 環境を起動 | `--type`, `--name`, `--mode`, `--with-grpc`, `--with-db`, `--with-cache`, `--port-offset`, `-y` |
| `playground stop` | playground 環境を停止 | `--name`, `--volumes`, `-y` |
| `playground status` | playground 環境の状態を表示 | `--json` |
| `playground list` | 利用可能なテンプレートを一覧表示 | - |

## グローバルオプション

```rust
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 詳細な出力を有効にする
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// カラー出力を無効にする
    #[arg(long, global = true)]
    pub no_color: bool,

    /// JSON 形式で出力する
    #[arg(long, global = true)]
    pub json: bool,
}
```

## バージョン管理

k1s0 のバージョンは `k1s0-version.txt` ファイルで一元管理されます。

```rust
static VERSION_STRING: Lazy<String> = Lazy::new(|| {
    include_str!("../../../../k1s0-version.txt").trim().to_string()
});
```

## 依存ライブラリ

| ライブラリ | バージョン | 用途 |
|-----------|----------|------|
| clap | 4.5 | CLI パーサー |
| clap_complete | 4.5 | シェル補完 |
| serde | 1.0 | シリアライゼーション |
| serde_json | 1.0 | JSON 処理 |
| chrono | 0.4 | 日時操作 |
| console | 0.15 | コンソール出力 |
| indicatif | 0.17 | プログレスバー |
| tokio | 1.0 | 非同期ランタイム |
| once_cell | 1.19 | 遅延初期化 |
