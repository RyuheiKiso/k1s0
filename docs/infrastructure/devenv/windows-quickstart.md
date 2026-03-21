# Windows 開発クイックスタート

Windows での k1s0 開発環境構築手順。3つの方法から選択できる。

## 方法の比較

| 方法 | 全機能対応 | 所要時間 | 推奨度 |
|------|----------|----------|--------|
| **A: devcontainer** | ✅ | 10〜20分 | ★★★ 推奨 |
| **B: WSL2 ネイティブ** | ✅ | 30分 | ★★ |
| **C: Windows ネイティブ** | ❌（CLI/TS/Dart 限定） | 10分 | ★（限定用途） |

---

## 方法 A: devcontainer（推奨）

全機能に対応し、ツールチェーンのバージョン管理も自動。

### 前提ツール

1. **Docker Desktop**（WSL2 バックエンド有効）または WSL2 + Docker Engine CE
2. **VS Code** + [Dev Containers 拡張](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

### 手順

```powershell
# 1. Git 設定（PowerShell で一度だけ実行）
# *** クローン前に必ず実行すること ***
# Git for Windows はデフォルトで core.autocrlf=true のため、
# クローン後に実行しても既に CRLF が混入している場合がある
git config --global core.autocrlf input
git config --global core.longpaths true

# 2. リポジトリのクローン
git clone <repository-url> k1s0
cd k1s0
```

3. VS Code でフォルダを開く
4. 画面右下に「Reopen in Container」が表示されたらクリック（または `F1` → `Dev Containers: Reopen in Container`）
5. 初回ビルド後、インフラ（PostgreSQL, Redis, Kafka, Keycloak 等）が自動起動する

### devcontainer 内で利用できるツール

| ツール | バージョン |
|--------|----------|
| Rust | 1.93 |
| Go | 1.24 |
| Node.js | 22 |
| Flutter | 3.24.0 |
| buf | 1.47.2 |
| just | 最新 |
| sqlx-cli | 最新 |
| pnpm | 最新 |

---

## 方法 B: WSL2 ネイティブ

Docker Desktop を使わずに WSL2 上で直接開発する。詳細は [WSL2開発環境セットアップ.md](./WSL2開発環境セットアップ.md) を参照。

### 手順

```powershell
# 1. WSL2 のインストール（管理者 PowerShell）
wsl --install -d Ubuntu-24.04
# インストール後再起動してユーザー名・パスワードを設定
```

```powershell
# 2. .wslconfig でリソース割り当て（任意・推奨）
# %USERPROFILE%\.wslconfig に以下を記述:
# [wsl2]
# memory=8GB
# processors=4
```

```bash
# 3. WSL2 内でリポジトリをクローン（WSL ファイルシステム上に置くこと）
cd ~
git clone <repository-url> k1s0
cd k1s0

# 4. セットアップスクリプトを実行
bash scripts/setup-wsl.sh

# 5. WSL を再起動してグループ変更を反映
# （Windows PowerShell で: wsl --shutdown）

# 6. 開発開始
just local-up   # インフラ起動
just lint       # 全言語リント
just test       # 全言語テスト
```

---

## 方法 C: Windows ネイティブ（CLI・TS・Dart 開発のみ）

rdkafka/zen-engine の制約によりサーバービルドは不可。CLI・TypeScript・Dart の開発のみ対応。

> **制約**: `scripts/` 配下のシェルスクリプトは全て bash 4+ を前提としており、PowerShell / cmd.exe では動作しない。Git Bash 経由でも `mapfile`・`BASH_REMATCH` 等の bash 4+ 機能が必要なため、完全な互換性は保証されない。サーバー開発・Proto コード生成・インフラ操作には方法 A または B を使用すること。

### 前提ツール

- Git for Windows
- Rust（rustup 経由）
- Node.js 22+（TypeScript 開発の場合）
- Dart SDK / Flutter（Dart 開発の場合）

### 手順

```powershell
# 1. 初期設定スクリプトの実行（前提条件チェック + Git 設定）
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
.\scripts\setup-windows.ps1
```

```powershell
# 2. Rust のインストール（https://rustup.rs/ からインストーラを実行）

# 3. CLI 開発
just cli-build   # ビルド（Windows/Git Bash 両対応）
just cli-test    # テスト
just cli-lint    # リント
just cli-fmt     # フォーマット
```

### Windows ネイティブで可能な作業

| 作業 | 可否 | 備考 |
|------|------|------|
| CLI ビルド・テスト | ✅ | `just cli-build / cli-test / cli-lint / cli-fmt` |
| TypeScript リント・テスト | ✅ | 各モジュールで直接 `pnpm run lint / test` |
| Dart リント・テスト | ✅ | 各モジュールで直接 `dart analyze / dart test` |
| Rust ライブラリテスト（CLI workspace） | ✅ | rdkafka に依存しない |
| サーバービルド（Rust/Go） | ❌ | rdkafka/zen-engine の制約あり |
| `just local-up` | ❌ | WSL2/devcontainer が必要 |
| 統合テスト | ❌ | Docker が必要 |
| Proto コード生成 | ❌ | bash スクリプト依存 |

---

## トラブルシューティング

### devcontainer が起動しない

- Docker Desktop が起動しているか確認
- Docker Desktop のリソースを確認（メモリ 8GB 以上推奨）
- WSL2 バックエンドが有効か確認（Settings → General → Use WSL 2 based engine）

### WSL2 の I/O が遅い

Windows 側のパス（`/mnt/c/...`）ではなく WSL2 ファイルシステム（`~/...`）にリポジトリをクローンしていることを確認する。

### 改行コード関連のエラー

**原因**: Git for Windows はシステムデフォルトで `core.autocrlf=true` が設定されており、クローン時に LF → CRLF に変換される。本リポジトリの `.gitattributes` は `eol=lf` を強制しているため競合が生じる。

```bash
# 1. Git 設定を修正（クローン前に実行するのが理想）
git config --global core.autocrlf input

# 2. 未コミットの変更がある場合は先に退避
git stash

# 3. インデックスをクリアして .gitattributes を再適用
git rm -rf --cached .
git reset --hard HEAD

# 4. 退避していた変更を戻す
git stash pop
```

### WSL2 でメモリ不足

`%USERPROFILE%\.wslconfig` を作成・編集してメモリ制限を増やす:

```ini
[wsl2]
memory=8GB
processors=4
swap=4GB
```

変更後: `wsl --shutdown` で WSL2 を再起動。
