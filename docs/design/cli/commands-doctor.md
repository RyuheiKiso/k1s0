# doctor コマンド

← [CLI 設計書](./)

開発環境の健全性をチェックし、問題を診断するコマンドです。

## 基本使用法

```bash
# 基本診断
k1s0 doctor

# 詳細情報（ツールのパス表示）
k1s0 doctor --verbose

# JSON 出力（CI 向け）
k1s0 doctor --json

# カテゴリ別チェック
k1s0 doctor --check rust
k1s0 doctor --check node
k1s0 doctor --check go
k1s0 doctor --check flutter
k1s0 doctor --check proto
k1s0 doctor --check docker

# 警告をエラーとして扱う
k1s0 doctor --strict
```

## 引数

```rust
#[derive(Args)]
pub struct DoctorArgs {
    /// 詳細情報を表示
    #[arg(short, long)]
    pub verbose: bool,

    /// JSON 形式で出力
    #[arg(long)]
    pub json: bool,

    /// チェックするカテゴリ
    #[arg(long, value_enum, default_value = "all")]
    pub check: CheckCategory,

    /// 警告をエラーとして扱う
    #[arg(long)]
    pub strict: bool,
}

#[derive(Clone, ValueEnum)]
pub enum CheckCategory {
    All,
    Rust,
    Go,
    Node,
    Flutter,
    Proto,
    Docker,
}
```

## チェック対象

### 必須ツール

| ツール | 最小バージョン | 用途 |
|--------|---------------|------|
| rustc | 1.85.0 | CLI 本体、backend-rust |
| cargo | - | Rust パッケージマネージャ |
| node | 20.0.0 | frontend-react |
| pnpm | 9.15.4 | Node パッケージ管理 |

### オプションツール

| ツール | 最小バージョン | 用途 |
|--------|---------------|------|
| go | 1.21.0 | backend-go |
| golangci-lint | 1.55.0 | Go リンター |
| buf | 1.28.0 | Protocol Buffers |
| flutter | 3.16.0 | frontend-flutter |
| dart | 3.2.0 | Dart SDK |
| docker | 24.0.0 | Docker ランタイム |
| docker compose | - | Docker Compose v2 |

## 出力例

```
k1s0 環境診断

k1s0 CLI
  k1s0: v0.1.0

必須ツール
  ✓ rustc: 1.92.0
  ✓ cargo: 1.92.0
  ✓ node: 22.14.0
  ✓ pnpm: 9.15.4

オプションツール
  ✓ go: 1.24.1
  - golangci-lint: not found
  - buf: not found
  - flutter: not found
  - dart: not found

推奨アクション:
  1. [推奨] buf をインストール: https://buf.build/docs/installation
  2. [推奨] flutter をインストール: https://flutter.dev/docs/get-started/install

✓ 全てのツールが正常にインストールされています
```

## JSON 出力

```json
{
  "k1s0_version": "0.1.0",
  "checks": [
    {
      "name": "rustc",
      "status": "ok",
      "version": "1.92.0",
      "required": true,
      "category": "Rust",
      "path": "/Users/user/.cargo/bin/rustc"
    },
    ...
  ],
  "recommendations": [
    {
      "tool": "buf",
      "action": "install",
      "message": "buf をインストールしてください",
      "url": "https://buf.build/docs/installation",
      "required": false
    }
  ],
  "summary": {
    "required_ok": 4,
    "required_failed": 0,
    "optional_ok": 1,
    "optional_missing": 4
  }
}
```

## 終了コード

| コード | 意味 |
|--------|------|
| 0 | 成功（必須ツール全て OK） |
| 5 | バリデーションエラー（必須ツールに問題あり） |

`--strict` モードでは、オプションツールの問題も終了コード 5 を返します。

## モジュール構成

```
src/doctor/
├── mod.rs              # モジュールルート
├── checker.rs          # ツールチェックロジック
├── requirements.rs     # バージョン要件定義
└── recommendation.rs   # 推奨アクション生成
```
