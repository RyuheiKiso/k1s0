# E2E テスト設計

## 概要

CLIバイナリ（k1s0.exe）をPythonから呼び出し、対話式メニューのフローをE2Eテストする。

## 方針

- Windows上で実行する
- Python + pytest で実装する
- メンテナンス性を重視し、テストシナリオをデータ駆動で管理する

## 前提: dialoguer の制約

CLIの対話UIは `dialoguer` クレートで実装されている。`Select::interact()` はTTY（端末）を要求するため、単純な `subprocess.run(input=...)` では動作しない。

### 対応方針

CLI側に環境変数 `K1S0_STDIN_MODE=1` による stdin入力モードを追加する。このモードでは dialoguer の代わりに stdin から選択番号を1行ずつ読み取る。

```
通常モード:    dialoguer (矢印キー + Enter で選択)
stdinモード:   stdin から行番号を読み取り (E2Eテスト用)
```

CLI側の変更は `UserPrompt` トレイトの別実装（`StdinPrompt`）を追加するだけで、既存コードへの影響はない。

## テスト環境

### ディレクトリ構成

```
e2e/
├── conftest.py             # pytest共通フィクスチャ
├── test_main_menu.py       # メインメニューのテスト
├── test_settings.py        # 設定フローのテスト
├── test_create_project.py  # プロジェクト作成フローのテスト
└── scenarios/
    └── create_project.yaml # データ駆動シナリオ定義
```

### 環境変数

| 変数名 | 説明 |
|--------|------|
| `K1S0_STDIN_MODE` | `1` に設定すると StdinPrompt に切り替わる |
| `K1S0_CONFIG_DIR` | 設定ファイルの保存先ディレクトリ（テスト隔離用） |

### フィクスチャ

- `workspace` — テスト用の一時ワークスペースディレクトリ (pytest `tmp_path` 内)
- `config_dir` — テスト用の一時設定ディレクトリ (pytest `tmp_path` 内)
- `run_cli()` — CLIをstdinモードで実行するヘルパー関数 (`encoding="utf-8"` でWindows CP932問題を回避)

## テスト対象フロー

### メニュー選択番号マッピング

| メニュー | 0 | 1 | 2 |
|---------|---|---|---|
| メインメニュー | プロジェクト作成 | 設定 | 終了 |
| 設定メニュー | パス確認 | パス設定 | 戻る |
| リージョン選択 | system-region | business-region | service-region |
| プロジェクト種別 | Library | Service | - |
| 同上 (business) | Library | Service | Client |
| 言語選択 | Rust | Go | - |
| サービス種別 | Client | Server | - |
| クライアントFW | React | Flutter | - |
| 部門領域操作 | 既存選択 | 新規追加 | - |

### テストケース一覧

#### メインメニュー (test_main_menu.py)

| テスト | フロー | 検証内容 |
|--------|--------|----------|
| 起動→終了 | `[2]` | 正常終了、「終了します」表示 |
| バナー表示 | `[2]` | 「k1s0」がstdoutに含まれる |
| 未設定でプロジェクト作成 | `[0, 2]` | 「未設定」エラーメッセージ |

#### 設定フロー (test_settings.py)

| テスト | フロー | 検証内容 |
|--------|--------|----------|
| パス設定→確認 | `[1, 1, path, 0, 2, 2]` | 「保存しました」、パス表示 |
| 設定永続化 | 2回実行 | TOMLファイル存在、再読み込み |
| 未設定時の確認 | `[1, 0, 2, 2]` | 「未設定」表示 |
| 相対パス拒否 | `[1, 1, relative, 2, 2]` | 「無効」表示 |
| 空パス拒否 | `[1, 1, "", 2, 2]` | 「無効」表示 |

#### プロジェクト作成 (test_create_project.py)

| テスト | フロー | 検証内容 |
|--------|--------|----------|
| System / Library / Rust | `[0, 0, 0, 0, 2]` | 「チェックアウト」メッセージ |
| Business / 新規 / Service / Go | `[0, 1, name, 1, 1, 2]` | 「チェックアウト」メッセージ |
| Business / 空の領域名 | `[0, 1, "", 2]` | 「領域名が不正です」表示 |
| checkout失敗 | `[0, 0, 0, 0, 2]` | 「チェックアウトに失敗しました」表示 |
| Service / 部門なし | `[0, 2, 2]` | 「部門固有領域が存在しません」表示 |

#### データ駆動シナリオ (scenarios/create_project.yaml)

| シナリオ | selections |
|---------|------------|
| System / Library / Rust | `[0, 0, 0, 0, 2]` |
| System / Library / Go | `[0, 0, 0, 1, 2]` |
| System / Service / Rust | `[0, 0, 1, 0, 2]` |
| System / Service / Go | `[0, 0, 1, 1, 2]` |
| Business / 新規 / Library / Rust | `[0, 1, name, 0, 0, 2]` |
| Business / 新規 / Library / Go | `[0, 1, name, 0, 1, 2]` |
| Business / 新規 / Service / Rust | `[0, 1, name, 1, 0, 2]` |
| Business / 新規 / Service / Go | `[0, 1, name, 1, 1, 2]` |
| Business / 新規 / Client / React | `[0, 1, name, 2, 0, 2]` |
| Business / 新規 / Client / Flutter | `[0, 1, name, 2, 1, 2]` |

## CLI側の変更

### 追加ファイル

```
CLI/src/infrastructure/stdin_prompt.rs  # StdinPrompt (E2Eテスト用)
```

### 変更ファイル

```
CLI/src/infrastructure/mod.rs  # pub mod stdin_prompt 追加
CLI/src/main.rs                # 環境変数による切り替え追加
```

### main.rs の切り替えロジック

```rust
// K1S0_CONFIG_DIR で設定ファイルパスを切り替え
let config_path = match std::env::var("K1S0_CONFIG_DIR") {
    Ok(dir) => PathBuf::from(dir).join("config.toml"),
    Err(_) => TomlConfigStore::default_path(),
};

// K1S0_STDIN_MODE で入力モードを切り替え
if std::env::var("K1S0_STDIN_MODE").is_ok() {
    let prompt = StdinPrompt::new();
    // ...
} else {
    let prompt = DialoguerPrompt;
    // ...
}
```

## 実行方法

```bash
# CLIをビルド
cargo build --manifest-path CLI/Cargo.toml

# E2Eテスト実行
cd e2e
pip install pytest pyyaml
pytest -v
```

## テスト結果 (23件)

```
test_main_menu.py       3件 (起動終了、バナー、未設定エラー)
test_settings.py        5件 (設定、永続化、未設定、相対パス、空パス)
test_create_project.py 15件 (データ駆動10件 + 個別5件)
```
