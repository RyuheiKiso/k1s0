# playground コマンド (v0.2.5)

← [CLI 設計書](./)

## 目的

サンプルコード付きの playground 環境を一時的に生成・起動し、k1s0 の開発体験を即座に試せるようにする。`k1s0 init` 不要でどこからでも実行可能であり、動作する CRUD エンドポイントを含むため、開発者のオンボーディングや技術スタック比較に最適化されている。

## `new-feature` との差異

| 観点 | `new-feature` + `docker compose up` | `playground` |
|------|--------------------------------------|-------------|
| 目的 | 本番用スキャフォールド生成 | 体験・学習・検証 |
| 出力先 | `feature/{type}/{name}/`（永続） | `.k1s0/playground/{name}/`（一時的） |
| manifest 登録 | あり | なし |
| lint 対象 | 対象 | 対象外 |
| init 前提 | 必要 | **不要（どこでも実行可能）** |
| サンプルコード | 空の Clean Architecture 構造 | **動作する CRUD エンドポイント付き** |
| クリーンアップ | 手動 | `playground stop` で自動削除 |

## サブコマンド

### playground start

テンプレートからサンプルコード付き環境を一時生成し、Docker Compose またはローカルプロセスで起動する。

```bash
# インタラクティブに選択して起動
k1s0 playground start

# テンプレート指定で起動
k1s0 playground start --type backend-rust

# オプション付き
k1s0 playground start --type backend-rust --with-grpc --with-db

# ローカルモードで起動（Docker 不要）
k1s0 playground start --type backend-rust --mode local

# 名前指定 + ポートオフセット（並行起動）
k1s0 playground start --type backend-rust --name my-app --port-offset 100

# 確認スキップ
k1s0 playground start --type backend-rust -y
```

#### 引数

| 引数 | 短縮 | 型 | デフォルト | 説明 |
|------|------|-----|-----------|------|
| `--type` | - | String | - | テンプレートタイプ（必須） |
| `--name` | - | String | `playground-{YYYYMMDD-HHMMSS}` | playground 名 |
| `--mode` | - | String | 自動検出 | 起動モード（`docker` / `local`） |
| `--with-grpc` | - | bool | false | gRPC エンドポイントを有効化 |
| `--with-rest` | - | bool | true | REST エンドポイントを有効化 |
| `--with-db` | - | bool | false | データベースを有効化 |
| `--with-cache` | - | bool | false | キャッシュを有効化 |
| `--port-offset` | - | u16 | 0 | ポートオフセット（0-999） |
| `--yes` | `-y` | bool | false | 確認プロンプトをスキップ |

### playground stop

playground 環境を停止し、ファイルを削除する。

```bash
# 名前指定で停止
k1s0 playground stop --name playground-20260131-120000

# 全 playground を停止
k1s0 playground stop -y

# ボリュームも含めて削除
k1s0 playground stop --volumes
```

#### 引数

| 引数 | 短縮 | 型 | デフォルト | 説明 |
|------|------|-----|-----------|------|
| `--name` | - | String | - | 停止対象の playground 名（省略時は全て） |
| `--volumes` | - | bool | false | ボリュームも削除（Docker モード時） |
| `--yes` | `-y` | bool | false | 確認プロンプトをスキップ |

### playground status

稼働中の playground 環境の一覧と状態を表示する。

```bash
# テーブル形式で表示
k1s0 playground status

# JSON 形式で表示
k1s0 playground status --json
```

#### 出力例

```
Playground 一覧:

  名前: playground-20260131-120000
    テンプレート: backend-rust
    モード: docker
    状態: running
    REST: http://localhost:8080
    gRPC: http://localhost:50051
    ディスク使用量: 523.0 MB
    作成日時: 2026-01-31T12:00:00+0900

  名前: playground-20260131-130000
    テンプレート: backend-go
    モード: local
    状態: running
    REST: http://localhost:8180
    ディスク使用量: 12.0 MB
    作成日時: 2026-01-31T13:00:00+0900
```

### playground list

利用可能なテンプレートの一覧を表示する。

```bash
k1s0 playground list
```

#### 出力例

```
利用可能なテンプレート:

  backend-rust       Rust バックエンドサービス (axum + tokio)
  backend-go         Go バックエンドサービス
  backend-csharp     C# バックエンドサービス (ASP.NET Core)
  backend-python     Python バックエンドサービス (FastAPI)
  frontend-react     React フロントエンド (Material-UI)
  frontend-flutter   Flutter フロントエンド (Material 3)
```

---

## 実行モード

playground は Docker モードとローカルモードの2つの実行モードを提供する。

### モード比較

| 項目 | Docker モード | ローカルモード |
|------|:------------:|:------------:|
| コマンド | `--mode docker`（デフォルト） | `--mode local` |
| 起動方法 | docker compose up | cargo run / go run 等 |
| DB | PostgreSQL コンテナ | SQLite ファイル |
| キャッシュ | Redis コンテナ | インメモリ HashMap |
| 前提ツール | Docker + Docker Compose | 各言語ツールチェーン |
| ネットワーク | Docker ネットワーク | localhost |
| クリーンアップ | docker compose down + ファイル削除 | プロセス kill + ファイル削除 |

### 自動モード検出

`--mode` を省略した場合、以下のロジックでモードを自動選択する。

1. `docker --version` と `docker compose version` の両方が成功 → Docker モード
2. いずれか失敗 → ローカルモードにフォールバック（警告メッセージを表示）

明示的に `--mode docker` / `--mode local` を指定した場合はその通りに動作し、前提ツールがない場合はエラーとなる。

### ローカルモードの前提ツール

| テンプレート | 必須ツール | 確認コマンド |
|-------------|-----------|-------------|
| backend-rust | rustc 1.85+, cargo | `cargo --version` |
| backend-go | go 1.22+ | `go version` |
| backend-csharp | dotnet 8.0+ | `dotnet --version` |
| backend-python | python 3.12+, uv | `python --version` |
| frontend-react | node 20+, pnpm 9.15+ | `pnpm --version` |
| frontend-flutter | flutter 3.x | `flutter --version` |

---

## テンプレートオーバーレイシステム

既存のベーステンプレートに playground 専用のオーバーレイを上書きマージする。`TemplateRenderer::render_directory()` を複数回呼び出すことで実現する。

### レンダリング順序

```
Pass 1: ベーステンプレート    CLI/templates/{type}/feature/         → output/
Pass 2: playground オーバーレイ CLI/templates/playground/{type}/      → output/ (上書き)
Pass 3: ローカルモード差分     CLI/templates/playground/{type}-local/ → output/ (上書き、ローカル時のみ)
Pass 4: 共通オーバーレイ       CLI/templates/playground/common/       → output/ (追加)
```

- **Pass 1** で `new-feature` と同じ Clean Architecture 構造を生成
- **Pass 2** でサンプルハンドラ、CRUD エンドポイント、docker-compose.yml 等を上書き
- **Pass 3**（ローカルモードのみ）で `config/default.yaml` を SQLite / インメモリ設定に差し替え
- **Pass 4** で seed SQL、playground ガイド README 等の共通ファイルを追加

### ディレクトリ構成

```
CLI/templates/playground/
├── common/                          # 全テンプレート・全モード共通
│   ├── seed.sql.tera                #   DB シード（SQLite/PostgreSQL 両対応）
│   └── PLAYGROUND_README.md.tera    #   playground ガイド
│
├── backend-rust/                    # Rust サンプルコード（モード共通）
│   ├── src/main.rs.tera
│   ├── src/domain/entities/item.rs.tera
│   ├── src/application/usecases/item_usecase.rs.tera
│   ├── src/infrastructure/repositories/item_repository.rs.tera
│   ├── src/presentation/rest/handlers.rs.tera
│   └── docker-compose.yml.tera      #   ポートオフセット対応版
│
├── backend-rust-local/              # Rust ローカルモード差分（1ファイルのみ）
│   └── config/default.yaml.tera     #   SQLite + インメモリキャッシュ設定
│
├── backend-go/                      # Go サンプルコード
├── backend-go-local/                # Go ローカルモード差分
├── backend-csharp/                  # C# サンプルコード
├── backend-csharp-local/            # C# ローカルモード差分
├── backend-python/                  # Python サンプルコード
├── backend-python-local/            # Python ローカルモード差分
├── frontend-react/                  # React サンプルコード
└── frontend-flutter/                # Flutter サンプルコード
```

### モード差分の最小化

サンプルコード（ハンドラ、エンティティ、ユースケース等）はモード間で**完全に共通**である。差分は `config/default.yaml` の1ファイルのみとなるよう設計されている。infrastructure 層で DB ドライバを抽象化することで、同一コードが PostgreSQL（Docker モード）と SQLite（ローカルモード）の両方で動作する。この設計は k1s0 の「設定駆動」思想のデモとして教育的価値が高い。

---

## Playground ディレクトリの解決

playground 環境は以下の優先順位でディレクトリを決定する。

1. カレントディレクトリに `.k1s0/` が存在する場合 → `.k1s0/playground/{name}/`
2. 存在しない場合 → `$HOME/.k1s0/playground/{name}/`

これにより、k1s0 init 済みプロジェクト内では `.k1s0/playground/` に配置され、init 未実施の環境ではホームディレクトリ配下に配置される。いずれの場合も manifest.json には登録されず、lint 対象にもならない。

`playground stop` と `playground status` は両方のパスを走査し、重複を排除して表示する。

---

## ポートオフセット

複数の playground を並行起動するために、`--port-offset` オプションでポート番号をずらすことができる。

| サービス | ベースポート | オフセット例（100） |
|---------|:----------:|:-----------------:|
| REST | 8080 | 8180 |
| gRPC | 50051 | 50151 |
| PostgreSQL | 5432 | 5532 |
| Redis | 6379 | 6479 |

- オフセットの範囲は 0-999 に制限
- 起動時に `TcpListener::bind()` でポートの空き状況を確認
- 競合時はエラーメッセージと `--port-offset` による回避策を提案

---

## プロセス管理

### Docker モード

- `docker compose up -d --build` で起動
- `docker compose down [-v]` で停止
- `docker compose ps --format {{.State}}` で状態確認

### ローカルモード

- `std::process::Command::spawn()` でバックグラウンド起動
- PID を `.playground.json` に保存
- 停止時の処理:
  - Unix: `SIGTERM` → 3秒待機 → `SIGKILL`
  - Windows: `taskkill /PID {pid} /F`
- プロセス生存確認:
  - Unix: `kill -0 {pid}`
  - Windows: `tasklist /FI "PID eq {pid}"`
- サーバーログは `{playground_dir}/logs/server.log` に出力

### メタデータ（.playground.json）

各 playground ディレクトリに `.playground.json` を保存し、停止・状態確認に使用する。

```json
{
  "name": "playground-20260131-120000",
  "template_type": "backend-rust",
  "mode": "docker",
  "options": {
    "with_grpc": true,
    "with_rest": true,
    "with_db": true,
    "with_cache": false
  },
  "port_offset": 0,
  "ports": {
    "rest_port": 8080,
    "grpc_port": 50051,
    "db_port": 5432,
    "redis_port": 6379
  },
  "pid": null,
  "created_at": "2026-01-31T12:00:00+0900",
  "dir": "/path/to/.k1s0/playground/playground-20260131-120000"
}
```

---

## 処理フロー

### start（Docker モード）

```
start --mode docker
  │
  ├─ 1. Docker / Docker Compose の存在確認
  │     └─ 未インストール → ローカルモードへフォールバック
  │
  ├─ 2. 引数解決（--type 必須、--name は自動生成可）
  │
  ├─ 3. ポート競合チェック（TcpListener::bind）
  │
  ├─ 4. playground ディレクトリ決定
  │
  ├─ 5. テンプレートレンダリング（3段階マージ: base → playground → common）
  │
  ├─ 6. メタデータ保存（.playground.json）
  │
  ├─ 7. docker compose up -d --build
  │
  ├─ 8. ヘルスチェック待機（TCP 接続、最大 30 秒）
  │
  └─ 9. 起動完了メッセージ
```

### start（ローカルモード）

```
start --mode local
  │
  ├─ 1. 言語ツールチェーン確認
  │
  ├─ 2. 引数解決
  │
  ├─ 3. ポート競合チェック
  │
  ├─ 4. playground ディレクトリ決定
  │
  ├─ 5. テンプレートレンダリング（4段階マージ: base → playground → local → common）
  │
  ├─ 6. メタデータ保存
  │
  ├─ 7. ビルド（cargo build / go build / dotnet build 等）
  │
  ├─ 8. バックグラウンドプロセス起動 → PID 保存
  │
  ├─ 9. ヘルスチェック待機
  │
  └─ 10. 起動完了メッセージ
```

### stop

```
stop 実行
  │
  ├─ 1. playground ディレクトリ走査（.k1s0/playground/ + $HOME/.k1s0/playground/）
  │
  ├─ 2. --name によるフィルタ（未指定時は全て対象）
  │
  ├─ 3. 確認プロンプト（--yes 未指定時）
  │
  ├─ 4. モードに応じた停止処理（Docker / ローカル）
  │
  ├─ 5. --volumes 指定時はディレクトリ再帰削除
  │
  └─ 6. 完了メッセージ
```

---

## エラーハンドリング

| エラー | 復旧提案 |
|--------|---------|
| `--type` 未指定 | `k1s0 playground list` で利用可能なテンプレートを確認 |
| 無効なテンプレートタイプ | 有効な値の一覧を表示 |
| Docker 未インストール（Docker モード） | Docker Desktop のインストール URL を案内 |
| ツールチェーン未インストール（ローカルモード） | 各言語のインストール URL を案内 |
| ポート競合 | `--port-offset` による回避策を提案 |
| playground 名の重複 | `playground stop` で既存を削除するか別名を指定 |
| ビルド失敗 | ログ確認を案内 |
| ヘルスチェックタイムアウト | 警告表示（エラーにはしない） |

---

## 実装

実装は `CLI/crates/k1s0-cli/src/commands/playground.rs` の単一ファイルに収められている。clap のサブコマンド構造は `docker.rs` と同一パターンを採用している。

### 主要関数

| 関数 | 説明 |
|------|------|
| `execute()` | dispatch（Start/Stop/Status/List） |
| `execute_start()` | テンプレート生成 → 起動 → ヘルスチェック |
| `execute_stop()` | playground 走査 → 停止 → 削除 |
| `execute_status()` | playground 走査 → 状態表示 |
| `execute_list()` | テンプレート一覧表示 |
| `detect_mode()` | Docker 有無による自動モード選択 |
| `render_playground()` | 4段階オーバーレイマージ |
| `resolve_playground_base_dir()` | `.k1s0/playground/` or `$HOME/.k1s0/playground/` |
| `resolve_ports()` | ベースポート + オフセット計算 |
