# 対話モード

← [CLI 設計書](./)

## 概要

k1s0 CLI は対話式インターフェースをサポートしています。Vite のような洗練されたユーザー体験を提供します。

### 引数なし実行時の動作

`k1s0` を引数なしで実行した場合:

- **TTY 環境**: サブコマンド選択の対話モードが起動し、実行したいコマンドを選択できます
- **非 TTY 環境**: ヘルプが表示されます（clap のデフォルト動作）

```bash
# TTY 環境で引数なしで実行
$ k1s0

k1s0 - 高速な開発サイクルを実現する統合開発基盤

? 実行するコマンドを選択してください:
> new-feature     新しいフィーチャーサービスを作成
  new-domain      新しいドメインライブラリを作成
  new-screen      新しい画面を作成
  init            リポジトリを初期化
  lint            規約チェックを実行
  upgrade         テンプレートをアップグレード
  domain          ドメイン管理（list, version, dependents, impact）
  completions     シェル補完スクリプトを生成
```

選択後の動作:
- `new-feature`, `new-domain`, `new-screen`, `init`: 対話モードで継続（必要な情報を順次入力）
- `lint`, `upgrade`, `domain`, `completions`: ヘルプまたは使用方法を表示

### 対話モードの動作

1. **自動フォールバック**: 必須引数が不足している場合、TTY 環境であれば対話モードにフォールバックします
2. **強制対話モード**: `--interactive` / `-i` フラグで強制的に対話モードを起動できます
3. **非 TTY 環境**: CI/CD 環境など非 TTY 環境では、必須引数をすべて指定する必要があります

### 対話モードの判定ロジック

```
# 引数なし実行時
if 引数なし（プログラム名のみ）:
    if TTY環境: サブコマンド選択対話モード
    else: ヘルプ表示

# サブコマンド実行時
if --interactive フラグあり:
    if TTY環境: 対話モードで実行
    else: エラー（TTYが必要）
else if 必須引数がすべて揃っている:
    引数モードで実行
else:
    if TTY環境: 対話モードにフォールバック
    else: エラー（引数不足）
```

### 対話モード対応コマンド

| コマンド | 対話モードフラグ | 対話で入力可能な項目 |
|---------|----------------|---------------------|
| `new-feature` | `-i, --interactive` | type, name, domain, オプション（grpc/rest/db） |
| `new-domain` | `-i, --interactive` | type, name, version, with_events, with_repository |
| `new-screen` | `-i, --interactive` | type, screen_id, title, feature_dir |
| `init` | `-i, --interactive` | path, template_source, language |

### 使用例

```bash
# 対話モードで feature を作成（引数なしで実行）
k1s0 new-feature

# 強制的に対話モードを起動
k1s0 new-feature -i

# 一部引数を指定して残りを対話で入力
k1s0 new-feature --type backend-rust
# → name のみ対話で入力される

# 従来の引数指定方式（引き続き動作）
k1s0 new-feature --type backend-rust --name my-service
```

### prompts モジュール

対話式プロンプトは `src/prompts/` モジュールで実装されています。

```
src/prompts/
├── mod.rs              # モジュールルート、TTY 検出
├── command_select.rs   # サブコマンド選択（引数なし実行時）
├── template_type.rs    # テンプレートタイプ選択
├── name_input.rs       # 名前入力（バリデーション付き）
├── options.rs          # オプション選択（マルチセレクト）
├── confirm.rs          # 確認プロンプト
├── version_input.rs    # バージョン入力
├── feature_select.rs   # フィーチャー選択
└── init_options.rs     # init オプション選択
```

### 依存クレート

対話式プロンプトには [inquire](https://crates.io/crates/inquire) クレートを使用しています。
