# Day 1 クイックスタート

k1s0 チームへようこそ。このガイドでは初日に開発環境を整えるまでの最短手順を説明する。

> 問題が起きたら → [`docs/infrastructure/devenv/troubleshooting.md`](../infrastructure/devenv/troubleshooting.md)

---

## 前提確認（5分）

### Git 設定（Windows の場合は必須）

```bash
# CRLF 自動変換を無効化（クローン前に必ず実行）
git config --global core.autocrlf input
git config --global core.longpaths true
```

### Docker のメモリ設定

infra + system プロファイル全起動で **5GB 以上**のメモリを消費する。

- **Docker Desktop**: Settings → Resources → Memory を **8GB 以上**に設定
- **WSL2**: `~/.wslconfig` に `memory=8GB` を追加し `wsl --shutdown` で再起動

---

## セットアップ方法を選ぶ

| 方法 | 推奨対象 | 所要時間 |
|------|---------|---------|
| **A: devcontainer（推奨）** | 全開発（Rust/Go/TS/Dart/サーバー） | 10〜20分 |
| **B: WSL2 ネイティブ** | 全開発 | 約30分 |
| **C: Windows ネイティブ** | CLI・TS・Dart のみ | 約10分 |

> **Rust サーバー開発は A または B が必須**（rdkafka / zen-engine の Windows ネイティブビルド非対応のため）

---

## A: devcontainer（推奨）

```bash
# 1. リポジトリをクローン
git clone <repository-url> k1s0
cd k1s0

# 2. VS Code で開く
code .
```

3. コマンドパレット（`F1`）→ **"Dev Containers: Reopen in Container"** を選択
4. 初回は Docker イメージのビルドと `post-create.sh` が自動実行される（10〜20分）

完了後、コンテナ内で手順「環境診断 → 開発環境起動」へ進む。

---

## B: WSL2 ネイティブ

```bash
# WSL2 のホームディレクトリにクローン（/mnt/c 以下は I/O が遅いため避ける）
cd ~
git clone <repository-url> k1s0
cd k1s0

# セットアップスクリプトを実行
bash scripts/setup-wsl.sh
```

詳細: [`docs/infrastructure/devenv/WSL2開発環境セットアップ.md`](../infrastructure/devenv/WSL2開発環境セットアップ.md)

---

## C: Windows ネイティブ（限定的）

```powershell
# PowerShell で実行
.\scripts\setup-windows.ps1
```

その後 Rust をインストール（https://rustup.rs/）。

---

## 環境診断（必須）

セットアップ後、必ず診断スクリプトを実行して環境を確認する。

```bash
just doctor
```

全項目が `[OK]` になれば開発を開始できる。`[WARN]` / `[ERROR]` が出た場合は表示される指示に従う。

---

## k1s0 CLI のインストール

devcontainer 利用者は自動インストール済み。それ以外は以下を実行。

```bash
# リポジトリルートで実行
cargo install --path CLI/crates/k1s0-cli

# 確認
k1s0 --version
```

---

## 開発環境の起動

```bash
# 認証バイパス付き開発環境を起動（ローカル開発推奨）
just local-up-dev

# 起動確認
docker compose ps

# Keycloak 管理コンソール: http://localhost:8180 (admin/dev)
```

必要に応じて可観測性スタックも起動できる。

```bash
just observability-up
# Grafana: http://localhost:3200
# Jaeger:  http://localhost:16686
```

---

## Docker Compose profile について

k1s0 の `docker-compose.yaml` はサービスを **profile** で分類している。
PostgreSQL・Redis・Kafka・Keycloak 等のインフラサービスはすべて `infra` プロファイルに属しており、
`docker compose up` をそのまま実行しても起動しない。

### just コマンドを使う場合（推奨）

`just local-up-dev` は内部で `--profile infra --profile system` を自動付与するため、
追加の操作は不要。

```bash
# 開発環境一式を起動（infra + system プロファイルを自動付与）
just local-up-dev

# 可観測性スタックのみ起動
just observability-up

# 特定プロファイルのみ起動（例: infra のみ）
just local-up-profile infra
```

### docker compose を直接呼ぶ場合

> ⛔ **新規環境セットアップには `just local-up-dev` を使用してください（HIGH-007 監査対応）。**
> `docker compose` を直接呼ぶと以下の 2 つの問題が発生します:
> 1. **DB マイグレーションが実行されない** → `tenant-rust`, `config-rust`, `featureflag-rust` 等が unhealthy になる（CRIT-004）
> 2. **`--build` フラグが付かない** → スタレイメージによるコンフィグ不整合が発生する（CRIT-001）
>
> `just local-up-dev` はマイグレーション（Phase 1.5）と `--build` を自動で処理するため、
> **新規環境では必ず `just local-up-dev` を使用すること。**

docker compose を直接呼ぶ場合は `--profile infra` フラグとマイグレーションの手動実行が必要。

```bash
# インフラサービス（PostgreSQL / Redis / Kafka / Keycloak 等）を起動
docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml \
  --profile infra up -d

# DB マイグレーションを手動実行（必須: スキップするとアプリが unhealthy になる）
just migrate-all

# インフラ + system プロファイルを同時起動
docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml \
  --profile infra --profile system up -d

# 個別サービスの停止時も profile が必要
docker compose --profile infra down
```

> 注意: `docker compose up -d` のみでは infra サービスが起動しないため、
> アプリケーションサービスが DB / Kafka に接続できずに起動失敗する。

### Keycloak のポートについて

Keycloak は HTTP ポートと管理ポートの 2 つを公開している。用途によって接続先が異なるため注意すること。

| 用途 | ポート | URL |
|------|--------|-----|
| 認証コンソール・OIDC エンドポイント | 8180 (HTTP) | `http://localhost:8180` |
| ヘルスチェック・管理 API | 9000 (管理ポート) | `http://localhost:9000/health/ready` |

- **ヘルスチェック**は管理ポート（9000）の `/health/ready` エンドポイントで行う
- `http://localhost:8180/health/ready` にアクセスしても **404** が返る（管理ポートとは別のポート）
- docker-compose.yaml のヘルスチェックも `localhost:9000` に対して実行している

---

## 最初の操作

```bash
# k1s0 CLI の対話メニューを起動
k1s0

# よく使う操作（新規参加者向け）:
#   よく使う操作 > プロジェクト初期化  ← sparse-checkout の設定
#   よく使う操作 > ひな形生成          ← サーバー/ライブラリ/クライアントの生成

# ログの確認
just logs <サービス名>

# 例: auth サーバーのログ
just logs auth-rust
```

---

## 次のステップ

担当する Tier に応じてオンボーディングを進める。

| Tier | ガイド |
|------|-------|
| system | [`tier1/`](tier1/01-概要.md) |
| business | [`tier2/`](tier2/01-概要.md) |
| service | [`tier3/`](tier3/01-概要.md) |

---

## 困ったときは

- **環境トラブル全般**: [`docs/infrastructure/devenv/troubleshooting.md`](../infrastructure/devenv/troubleshooting.md)
- **Windows 固有の問題**: [`docs/infrastructure/devenv/windows-quickstart.md`](../infrastructure/devenv/windows-quickstart.md)
- **開発ルール・PR プロセス**: [`CONTRIBUTING.md`](../../CONTRIBUTING.md)
