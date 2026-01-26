# ADR 0005: gRPC契約管理（buf lint/breaking）

## ステータス
承認済み

## コンテキスト
gRPCサービス間の契約を管理し、破壊的変更を検出する仕組みが必要である。
Protocol Buffersのスキーマ変更が既存のクライアントを壊さないことを保証する必要がある。

## 決定

### buf CLIの採用
- [buf](https://buf.build/)を契約管理ツールとして採用する
- 各サービスのprotoディレクトリに`buf.yaml`を配置する
- CIパイプラインで`buf lint`と`buf breaking`を必須チェックとする

### lint設定
```yaml
lint:
  use:
    - DEFAULT
    - COMMENTS  # コメント必須
  except:
    - PACKAGE_VERSION_SUFFIX  # v1, v2等のサフィックスは任意
  disallow_comment_ignores: true
  enum_zero_value_suffix: _UNSPECIFIED
  service_suffix: Service
```

### breaking変更検出
```yaml
breaking:
  use:
    - FILE  # ファイルレベルの破壊的変更を検出
  ignore_unstable_packages: true
```

### 破壊的変更の定義
以下の変更は破壊的変更として検出される：
- フィールド番号の変更
- フィールド型の変更
- 必須フィールドの追加（既存メッセージに対して）
- RPC署名の変更
- パッケージ名の変更
- サービス/メソッドの削除

### 許可される変更
- フィールドの追加（オプショナル）
- 新しいRPCメソッドの追加
- 新しいサービスの追加
- コメントの変更
- 予約フィールドの追加

## CIワークフロー

1. **lint**: すべてのPRで実行、失敗時はマージ不可
2. **breaking**: mainブランチとの差分を検出、警告として表示
3. **format**: protoファイルのフォーマットチェック

## ローカルでの実行

```bash
# lint実行
./scripts/buf-check.sh lint

# 破壊的変更チェック
./scripts/buf-check.sh breaking

# フォーマットチェック
./scripts/buf-check.sh format

# すべてのチェック
./scripts/buf-check.sh all
```

## 影響

### 開発プロセス
- protoファイルの変更時は必ず`buf lint`を実行する
- 破壊的変更が必要な場合はADRを作成し、移行計画を立てる
- クライアント側の更新を先に行い、後方互換性を保つ

### サービス間契約
- すべてのgRPCサービスはbuf設定を持つ
- 契約変更はPRレビューで明示的に承認される
- 破壊的変更は原則禁止、必要な場合はバージョニングで対応

## 代替案

### 1. protoc-gen-validateのみ
却下理由: バリデーションのみで契約管理機能がない

### 2. 手動レビューのみ
却下理由: 人的ミスのリスクが高い、自動化できない

### 3. OpenAPIへの統一
却下理由: gRPCの効率性を損なう、既存インフラとの互換性

## 参考

- [buf documentation](https://docs.buf.build/)
- [Protocol Buffers Best Practices](https://protobuf.dev/programming-guides/dos-donts/)
- [gRPC Breaking Change Guidelines](https://grpc.io/docs/what-is-grpc/core-concepts/)
