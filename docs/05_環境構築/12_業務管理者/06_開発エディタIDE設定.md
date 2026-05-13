---
id: env.overview.business_admin_editor_ide
axis: overview
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.overview.business_admin_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- 業務管理者のエディタはブラウザのみで完結し、IDE は不要である。Backstage プラグインが決定表エディタを提供し、テキストエディタはメモ帳または VS Code（任意）を使う。

## ブラウザ内エディタ（メイン）

業務管理者の主要な編集作業はすべて Backstage UI のプラグインで行う。

```
Backstage 決定表エディタの操作:
1. https://backstage.staging.example.internal にアクセスする
2. [Plugins] > [Decision Table Editor] を開く
3. 対象テナントの決定表を選択する
4. ルールを編集して [Save] をクリックする
5. [変更申請フロー] が起動することを確認する
6. staging での validation 結果を確認する
```

## テキストエディタ（任意）

runbook のメモや手順書の下書き作成には Windows 標準のメモ帳で十分。より高度な編集には VS Code（任意）を使うことができる。

```
VS Code の任意設定:
- Windows 側に VS Code をインストールする
- Remote - WSL extension を導入する（WSL2 利用時）
- YAML / Markdown のシンタックスハイライトが利用できる
```

VS Code での YAML 編集補助（WSL2 利用時）:

```bash
code --install-extension redhat.vscode-yaml
```

## tier2 admin API ドキュメント（ブラウザ）

API の操作手順は Swagger UI / ReDoc で参照する。

```
API ドキュメント URL（例）:
https://api.staging.example.internal/docs
```

k1s0-admin CLI のコマンドリファレンスも同 URL から参照できる。

## 検収コマンド

ブラウザ確認は目視。任意 VS Code 確認（WSL2 利用時）:

```bash
code --version 2>/dev/null && echo "VS Code: 利用可能" || echo "VS Code: 任意（未導入でも可）"
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
