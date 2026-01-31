# ADR-0012: Migrate コマンドの導入

## ステータス

承認済み（Accepted）

## 日付

2025-01-25

## コンテキスト

既存のマイクロサービスプロジェクトを k1s0 の管理下に移行する際、手動での対応が必要であった。ディレクトリ構造の変更、manifest.json の作成、Clean Architecture への準拠など、移行作業は煩雑でエラーを起こしやすかった。

## 決定

k1s0 CLI に `migrate` サブコマンドを追加し、既存プロジェクトの段階的な移行を支援する：

- `k1s0 migrate analyze`: 既存プロジェクトの k1s0 準拠状況を分析
- `k1s0 migrate plan`: 分析結果に基づく移行計画の生成
- `k1s0 migrate apply`: 移行計画の段階的な適用
- `k1s0 migrate status`: 移行進捗の確認

### 採用理由

- 自動化された分析と段階的な移行により、既存プロジェクトの取り込みリスクを最小化できる
- `--dry-run` と `--skip-backup` オプションにより安全な移行が可能

## 帰結

### 正の帰結

- 既存プロジェクトの移行が体系的に行える
- 移行状況の可視化により進捗管理が容易

### 負の帰結

- すべてのプロジェクト構造に対応できるわけではなく、手動対応が必要な場合がある

## 関連ドキュメント

- [CLI 設計書 - migrate コマンド](../design/cli/commands-migrate.md)
- [3層構造移行ガイド](../guides/migration-to-three-tier.md)
- [ADR-0006](ADR-0006-three-layer-architecture.md) - 3層アーキテクチャ
