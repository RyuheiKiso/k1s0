---
id: env.formal.formal_editor_ide
axis: formal
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.formal.formal_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- formal 軸エンジニアは VS Code + TLA+ extension + Dafny extension + Lean 4 extension（lean4）を標準エディタ環境とし、Stainless を使う場合は IntelliJ IDEA を代替として使用する。

## VS Code 必須 extension

| extension ID | 用途 |
|---|---|
| `alygin.vscode-tlaplus` | TLA+ / PlusCal の syntax highlight / TLC 実行 |
| `dafny-lang.ide-vscode` | Dafny の language server / inline verification |
| `leanprover.lean4` | Lean 4 の language server / infoview |

インストール手順:

```bash
code --install-extension alygin.vscode-tlaplus
code --install-extension dafny-lang.ide-vscode
code --install-extension leanprover.lean4
```

## TLA+ extension の設定

VS Code の `settings.json` に TLC のパスを設定する。

```json
{
  "tlaplus.java.home": "/usr/lib/jvm/java-21-openjdk-amd64",
  "tlaplus.tlcPath": "/home/<user>/tools/tla2tools.jar"
}
```

TLA+ extension は `apalache-mc` への連携も提供する。`tlaplus.apalache.jarPath` を設定すると Apalache 実行が VS Code から行える。

## Dafny extension の設定

Dafny extension は dafny language server を自動起動する。バイナリパスを明示する場合:

```json
{
  "dafny.serverRuntimePath": "/home/<user>/tools/dafny/DafnyLanguageServer.dll"
}
```

## Lean 4 extension の設定

lean4 extension は `elan` が管理するツールチェインを自動検出する。明示的に設定する場合:

```json
{
  "lean4.toolchainPath": "/home/<user>/.elan/toolchains/leanprover-lean4-stable"
}
```

Lean 4 の Infoview（ゴール表示）は `Ctrl+Shift+Enter` で開く。

## IntelliJ IDEA（Stainless 用 代替）

Stainless を Scala プロジェクトとして扱う場合、IntelliJ IDEA + Scala plugin を使う。

```bash
# snap 経由でのインストール（任意）
sudo snap install intellij-idea-community --classic
```

Stainless は CLI 実行が主体のため、エディタは任意。Stainless の verification 結果は terminal 出力で確認する。

## 検収コマンド

```bash
code --list-extensions | grep -E "tlaplus|dafny|lean4"
# 期待: 3 extension が表示される
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
