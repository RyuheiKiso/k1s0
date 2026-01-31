# migrate コマンド

← [CLI 設計書](./)

## 目的

既存プロジェクトを k1s0 プラットフォームに取り込むためのマイグレーション支援コマンド。既存のディレクトリ構造を分析し、k1s0 規約に準拠させるための移行計画を生成・適用する。

## サブコマンド

### migrate analyze

既存プロジェクトを分析し、k1s0 規約への準拠状況を報告する。

```bash
k1s0 migrate analyze --path ./existing-project --type backend-go
k1s0 migrate analyze --path ./existing-project --type backend-go --json
k1s0 migrate analyze --path ./existing-project --type backend-go --verbose
```

#### 引数

| 引数 | 短縮 | 型 | デフォルト | 説明 |
|------|------|-----|-----------|------|
| `--path` | `-p` | String | `.` | 分析対象のプロジェクトパス |
| `--type` | `-t` | String | 必須 | テンプレートタイプ（backend-rust, backend-go, backend-csharp, backend-python, backend-kotlin, frontend-react, frontend-flutter, frontend-android） |
| `--json` | | Flag | false | JSON 形式で出力 |
| `--verbose` | `-v` | Flag | false | 詳細出力 |

### migrate plan

分析結果に基づき、移行計画を生成する。

```bash
k1s0 migrate plan --path ./existing-project --name my-service --type backend-go
k1s0 migrate plan --path ./existing-project --name my-service --type backend-go --output plan.json
k1s0 migrate plan --path ./existing-project --name my-service --type backend-go --dry-run
```

#### 引数

| 引数 | 短縮 | 型 | デフォルト | 説明 |
|------|------|-----|-----------|------|
| `--path` | `-p` | String | `.` | 対象プロジェクトパス |
| `--name` | `-n` | String | 必須 | k1s0 サービス名（kebab-case） |
| `--type` | `-t` | String | 必須 | テンプレートタイプ |
| `--output` | `-o` | String | `.k1s0/migration-plan.json` | 計画ファイル出力先 |
| `--dry-run` | | Flag | false | 計画内容を表示するのみ |

### migrate apply

生成された移行計画を適用する。

```bash
k1s0 migrate apply --path ./existing-project --plan .k1s0/migration-plan.json
k1s0 migrate apply --path ./existing-project --plan .k1s0/migration-plan.json --phase 1
k1s0 migrate apply --path ./existing-project --plan .k1s0/migration-plan.json --dry-run
k1s0 migrate apply --path ./existing-project --plan .k1s0/migration-plan.json --yes --skip-backup
```

#### 引数

| 引数 | 短縮 | 型 | デフォルト | 説明 |
|------|------|-----|-----------|------|
| `--path` | `-p` | String | `.` | 対象プロジェクトパス |
| `--plan` | | String | `.k1s0/migration-plan.json` | 計画ファイル |
| `--phase` | | u32 | なし | 特定フェーズのみ適用 |
| `--dry-run` | | Flag | false | 変更内容を表示するのみ |
| `--yes` | `-y` | Flag | false | 確認をスキップ |
| `--skip-backup` | | Flag | false | バックアップの作成をスキップ |

### migrate status

移行の進捗状況を表示する。

```bash
k1s0 migrate status --path ./existing-project
k1s0 migrate status --path ./existing-project --plan .k1s0/migration-plan.json --json
```

#### 引数

| 引数 | 短縮 | 型 | デフォルト | 説明 |
|------|------|-----|-----------|------|
| `--path` | `-p` | String | `.` | 対象プロジェクトパス |
| `--plan` | | String | `.k1s0/migration-plan.json` | 計画ファイル |
| `--json` | | Flag | false | JSON 形式で出力 |

## ワークフロー

```
1. analyze  → 現状の分析（規約準拠状況を報告）
2. plan     → 移行計画の生成（ステップごとの変更内容）
3. apply    → 計画の適用（フェーズ単位で段階適用可能）
4. status   → 進捗の確認
```
