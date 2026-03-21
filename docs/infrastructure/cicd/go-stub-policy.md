# Go 空スタブ実装ポリシー

## 背景
k1s0 は4言語（Rust/Go/TypeScript/Dart）パリティを目指しているが、
一部ライブラリは Go 実装が `.gitkeep` のみの空ディレクトリとして存在する。
本ドキュメントはこれらの取り扱い方針を定める。

## 現状
`regions/system/library/go/` 配下には、以下のような空スタブが存在する:
- 本来は実装が必要だが、優先度が低く未着手のもの
- Rust/TypeScript 専用のため Go 実装が不要なもの

## 分類と対応方針

### カテゴリ A: 実装予定あり
- `.gitkeep` ファイルに `TODO: implementation planned - {担当者} {目標日}` コメントを記載
- `modules.yaml` の `maturity: template-only` として登録
- 着手時に `maturity: experimental` に昇格

### カテゴリ B: 実装不要（言語固有）
- ディレクトリごと削除する
- `modules.yaml` の `library_parity` セクションで `server_only` または `rust_only` として明示
- 削除PRのコミットメッセージに理由を記載（例: "Rust固有のprocマクロのため Go版は不要"）

### カテゴリ C: 将来的に実装
- `README.md` に実装方針と想定インターフェースを記載
- `modules.yaml` に `maturity: template-only` で登録
- 四半期ごとにレビューし、カテゴリを見直す

## 新規スタブ作成時のルール
1. カテゴリを明示してから作成する（`.gitkeep` のみは禁止）
2. `modules.yaml` への登録と同時に作成する
3. 実装なしのディレクトリを `maturity: experimental` 以上にしない

## 整理スケジュール
- 四半期ごとにスタブ一覧をレビュー
- 6ヶ月間更新がないスタブはカテゴリ B として削除を検討

## 更新履歴

| 日付 | 変更内容 |
|------|---------|
| 2026-03-21 | 初版作成（技術品質監査対応 P2-37） |
