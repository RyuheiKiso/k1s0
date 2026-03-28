# ADR-0022: gRPC メッセージレベルバリデーション（buf validate）の導入計画

## ステータス

承認済み（2026-03-25）

> **外部監査 SM-4 対応**: バリデーションルールが各ハンドラーに分散し多言語間で不整合のリスクが継続している。本 ADR を承認し buf validate 導入を開始する。

## 実装タイムライン（SM-4 監査対応）

| フェーズ | 内容 | 期限 | 状態 |
|---------|------|------|------|
| Phase 1 | `auth.proto` と `task.proto` への buf validate 適用（buf.yaml 依存追加・フィールドルール追加） | 2026-04-14 | **✅ 完了（2026-03-29）** |
| Phase 2 | 全 proto ファイルへの拡張（Go: protovalidate-go Interceptor, Rust: prost-validate ミドルウェア） | 2026-05-12 | 未着手 |

### Phase 1 実施内容（H-15 監査対応）

- `api/proto/buf.yaml` に `buf.build/bufbuild/protovalidate` 依存を追加
- `auth.proto`: ValidateTokenRequest.token（min_len=1）、GetUserRequest.user_id（UUID）、GetUserRolesRequest.user_id（UUID）、CheckPermissionRequest.permission/resource（文字列長制限）にルールを追加
- `task.proto`: CreateTaskRequest.project_id/title（UUID/文字列長）、GetTaskRequest.task_id（UUID）、UpdateTaskStatusRequest.task_id/status/expected_version にルールを追加

## コンテキスト

現在の k1s0 gRPC サービスはバリデーションをアプリケーション層（Go / Rust のハンドラー関数内）
でのみ実施している。この方式では以下の課題がある。

1. **バリデーションロジックの分散**: 各ハンドラーが独自にバリデーションを実装するため、
   漏れや不整合が発生しやすい
2. **早期リジェクトの欠如**: 不正なメッセージがアプリケーション層まで到達してから
   エラーが返るため、無駄な処理コストが生じる
3. **proto との乖離**: バリデーションルールがコードに埋め込まれており、
   proto ファイル（インターフェース仕様）を見ただけでは制約を把握できない
4. **多言語間の不整合**: Go / Rust / TypeScript / Dart の各クライアントが
   それぞれバリデーションを再実装する必要があり、ルールがずれるリスクがある

`buf validate`（旧称: `protobuf-validate` / `protovalidate`）は proto ファイルに
バリデーションルールを宣言的に記述し、各言語のランタイムライブラリが
フレームワークレベルで自動的にバリデーションを実行する仕組みを提供する。

## 決定

中長期（3 ヶ月以内）に `buf validate` を導入する。

**導入スコープと優先順位**:

1. **フェーズ 1（1 ヶ月以内）**: `auth.proto` と `task.proto` の主要フィールドから適用開始
   - 必須フィールドの `required` 制約
   - 文字列長制限（`min_len` / `max_len`）
   - UUID フォーマット制約（`uuid` オプション）
2. **フェーズ 2（3 ヶ月以内）**: 全 proto ファイルへ拡張
   - 数値範囲制約（ページサイズ等）
   - 列挙値制約（`enum` の有効値チェック）
   - メッセージ間制約（フィールド間の条件付き必須等）

**技術的変更点**:

- `buf.yaml` に `buf.build/bufbuild/protovalidate` 依存を追加
- 各 proto ファイルに `import "buf/validate/validate.proto"` を追加
- Go サーバー: `protovalidate-go` を導入し、Interceptor としてバリデーションを実施
- Rust サーバー: `prost-validate` を導入し、tonic のミドルウェアとして組み込む
- gRPC バリデーションエラーは `INVALID_ARGUMENT` ステータスコードで返却する

## 理由

1. **スキーマファーストの一貫性**: proto ファイルがインターフェース仕様の
   single source of truth となり、バリデーションルールも proto に集約される
2. **多言語間の整合性**: proto から各言語のコードを生成するため、
   バリデーションルールが全言語で自動的に統一される
3. **早期リジェクト**: フレームワーク層でバリデーションを実施するため、
   不正なリクエストをハンドラー処理前に弾くことができる
4. **保守性の向上**: バリデーションルールの変更が proto ファイル 1 箇所の修正で済む
5. **ドキュメント性**: proto ファイルを読むだけでフィールドの制約条件が把握できる

## 影響

**ポジティブな影響**:

- 各ハンドラーのバリデーションコードを簡略化（または削除）できる
- クライアント側でも同じ proto 定義からバリデーターを生成できるため、
  送信前のクライアントサイドバリデーションも統一できる
- `buf lint` でバリデーションルールの記述ミスを CI で検出できる
- API ドキュメント（buf schema registry 等）にバリデーション制約が自動的に反映される

**ネガティブな影響・トレードオフ**:

- Go / Rust 双方にバリデーションライブラリを追加するビルド依存が増加する
- 既存のハンドラー内バリデーションコードとの二重チェックが発生する移行期間がある
- `prost-validate`（Rust）は `protovalidate-go`（Go）より機能が限定される場合があり、
  サポート状況を継続的に確認する必要がある
- proto ファイルの変更が多言語全体のビルドに影響するため、
  変更時のテスト範囲が広がる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| アプリケーション層バリデーションのまま維持 | 各ハンドラーで個別実装を継続 | バリデーションの漏れ・言語間不整合が解消されない。現状維持 |
| gRPC Interceptor でカスタムバリデーション | 共通 Interceptor に集約してハンドラーから外出し | proto 仕様とバリデーション定義が分離したままであり、根本的な課題が残る |
| Connect-RPC + Interceptor | Connect プロトコルへ移行してミドルウェア機構を活用 | プロトコル変更は大規模リファクタリングとなり、バリデーション改善のためだけに行うにはコスト過大 |

## 参考

- [ADR-0006: proto バージョニング](./0006-proto-versioning.md) — proto 管理の基盤方針
- buf validate 公式ドキュメント: [https://buf.build/bufbuild/protovalidate](https://buf.build/bufbuild/protovalidate)
- Go 実装: [protovalidate-go](https://github.com/bufbuild/protovalidate-go)
- Rust 実装: [prost-validate](https://github.com/linka-cloud/prost-validate)
- proto ファイル格納先: `regions/system/library/*/proto/`
