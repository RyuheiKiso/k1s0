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
