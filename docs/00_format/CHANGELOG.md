# CHANGELOG フォーマット規約

本ドキュメントは、このリポジトリにおける CHANGELOG の記述形式を定義する。  
[Keep a Changelog](https://keepachangelog.com/ja/1.0.0/) に準拠し、[Semantic Versioning](https://semver.org/lang/ja/) を採用する。

---

## 基本構造

```markdown
# Changelog

## [Unreleased]

## [X.Y.Z] - YYYY-MM-DD HH:MM
### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security
```

---

## バージョン番号のルール（Semantic Versioning）

| 種別 | 説明 | 例 |
|------|------|----|
| MAJOR | 後方互換性のない変更 | `1.0.0` → `2.0.0` |
| MINOR | 後方互換性のある機能追加 | `1.0.0` → `1.1.0` |
| PATCH | 後方互換性のあるバグ修正 | `1.0.0` → `1.0.1` |

---

## セクションの定義

| セクション | 記載内容 |
|------------|----------|
| `Added`    | 新機能の追加 |
| `Changed`  | 既存機能の変更 |
| `Deprecated` | 将来削除予定の機能 |
| `Removed`  | 削除された機能 |
| `Fixed`    | バグ修正 |
| `Security` | 脆弱性対応 |

---

## 記述ルール

- 各エントリは **日本語** で記載する
- エントリは箇条書き（`-`）で記載する
- 最新バージョンをファイルの先頭に記載する
- リリース前の変更は `[Unreleased]` セクションにまとめる
- リリース時に `[Unreleased]` の内容をバージョン番号付きセクションへ移動する
- 日時は `YYYY-MM-DD HH:MM` 形式（例: `2026-04-11 09:30`）で記載する
- 一日に複数回リリースする場合は時刻で区別する（同日の場合も PATCH バージョンをインクリメントする）

---

## 記述例

```markdown
# Changelog

## [Unreleased]
### Added
- インストール確認コマンド `install-check` を追加

## [0.2.1] - 2026-05-01 15:00
### Fixed
- CLI 起動時のエラーメッセージを修正

## [0.2.0] - 2026-05-01 09:30
### Added
- GUI モード初期実装

### Changed
- CLI の出力フォーマットを改善

## [0.1.0] - 2026-04-11 10:00
### Added
- プロジェクト初期セットアップ
- README.md を新規作成
- `.gitignore` を新規作成

[Unreleased]: https://github.com/example/k1s0/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/example/k1s0/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/example/k1s0/releases/tag/v0.1.0
```
