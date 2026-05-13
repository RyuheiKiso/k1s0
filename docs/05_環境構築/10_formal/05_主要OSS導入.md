---
id: env.formal.formal_oss_install
axis: formal
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.formal.formal_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- TLA+ / Apalache / Dafny / Lean 4 / Kani / CBMC / Stainless / P language の 8 OSS をそれぞれのインストール手順に従い導入し、各コマンドが正常応答することを確認してから次のステップへ進む。

## 1. TLA+ CLI（tlc）

TLA+ のモデル検査を行う `tla2tools.jar` を取得し、エイリアスを設定する。

```bash
mkdir -p ~/tools
curl -L https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar \
  -o ~/tools/tla2tools.jar
# エイリアス設定（~/.bashrc に追加）
echo 'alias tlc="java -jar ~/tools/tla2tools.jar"' >> ~/.bashrc
source ~/.bashrc
tlc -help 2>&1 | head -3
```

## 2. Apalache（apalache-mc）

Apalache は TLA+ の bounded model checker（型付き）。Java 21 で動作する。

```bash
APALACHE_VER="0.46.0"
curl -L "https://github.com/apalache-mc/apalache/releases/download/v${APALACHE_VER}/apalache-${APALACHE_VER}.zip" \
  -o /tmp/apalache.zip
unzip /tmp/apalache.zip -d ~/tools/
echo "export PATH=\$PATH:~/tools/apalache/bin" >> ~/.bashrc
source ~/.bashrc
apalache-mc help 2>&1 | head -3
```

## 3. Dafny CLI

Dafny は証明付きプログラム検証言語。GitHub Releases からバイナリを取得する。

```bash
DAFNY_VER="4.6.0"
curl -L "https://github.com/dafny-lang/dafny/releases/download/v${DAFNY_VER}/dafny-${DAFNY_VER}-x64-ubuntu-20.04.zip" \
  -o /tmp/dafny.zip
unzip /tmp/dafny.zip -d ~/tools/dafny/
echo "export PATH=\$PATH:~/tools/dafny" >> ~/.bashrc
source ~/.bashrc
dafny --version
```

## 4. Lean 4 + mathlib（elan 経由）

Lean 4 はツールチェインマネージャー `elan` を介してインストールする。

```bash
curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh -s -- -y
source ~/.elan/env
elan default leanprover/lean4:stable
lean --version
# mathlib4 プロジェクトを作成する場合
lake new myproject math
```

mathlib のフルビルドは 16GB RAM 環境で 30 分以上かかる場合がある。

## 5. Kani（cargo install kani-verifier）

Kani は Rust コードの形式検証ツール。`cargo install` で導入する。

```bash
cargo install --locked kani-verifier
cargo kani --version
```

初回インストールは Rust ツールチェインを自動でセットアップする。完了まで数分かかる。

## 6. CBMC

CBMC は C / C++ のバウンデッドモデル検査ツール。Ubuntu apt で導入する。

```bash
sudo apt update
sudo apt install -y cbmc
cbmc --version
```

## 7. Stainless

Stainless は Scala コードの形式検証ツール。Java 17+ が必要。

```bash
STAINLESS_VER="0.9.8"
curl -L "https://github.com/epfl-lara/stainless/releases/download/v${STAINLESS_VER}/stainless-scalac-standalone-${STAINLESS_VER}-linux.zip" \
  -o /tmp/stainless.zip
unzip /tmp/stainless.zip -d ~/tools/stainless/
echo "export PATH=\$PATH:~/tools/stainless/bin" >> ~/.bashrc
source ~/.bashrc
stainless-dotty --version 2>&1 | head -3
```

## 8. P language（p）

P language は非同期・並行システムのモデル化と検証を行う DSL（microsoft/P）。.NET 8 SDK 上で動作する。

```bash
dotnet tool install --global p
p --help 2>&1 | head -3
```

グローバルツールとしてインストールされるため `~/.dotnet/tools` が PATH に含まれていること。

```bash
export PATH="$PATH:$HOME/.dotnet/tools"
echo 'export PATH="$PATH:$HOME/.dotnet/tools"' >> ~/.bashrc
```

## 検収コマンド

```bash
tlc -help 2>&1 | head -1
apalache-mc help 2>&1 | head -1
dafny --version
lean --version
cargo kani --version
cbmc --version
stainless-dotty --version 2>&1 | head -1
p --help 2>&1 | head -1
```

全コマンドが正常応答すれば OSS 導入完了。

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)
