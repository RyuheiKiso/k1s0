# Framework Crates

feature サービスが依存する共通 crate 群。

## Crate 一覧

| Crate | 説明 | ステータス |
|-------|------|-----------|
| `k1s0-config` | 設定読み込み（`--env`/`--config`/`--secrets-dir`） | ✅ 実装済み |
| `k1s0-validation` | 入力バリデーション（problem+json / gRPC 対応） | ✅ 実装済み |
| `k1s0-error` | エラー表現の統一（層別責務、error_code） | ✅ 実装済み |
| `k1s0-observability` | ログ/トレース/メトリクス初期化（OTel） | ✅ 実装済み |
| `k1s0-grpc-client` | gRPC クライアント共通（deadline 必須、retry 原則禁止） | 未実装 |
| `k1s0-auth` | 認証・認可クライアント | 未実装 |
| `k1s0-db` | DB 接続・トランザクション | 未実装 |
| `k1s0-cache` | キャッシュ抽象 | 未実装 |

## 公開 API の安定性

- `pub` で公開される型/関数/trait は SemVer の互換性対象
- `pub(crate)` や非公開モジュールは互換性対象外
- 非推奨化は `#[deprecated]` で段階移行

## 関連ドキュメント

- [規約: エラーハンドリング](../../../../docs/conventions/error-handling.md)
- [規約: 観測性](../../../../docs/conventions/observability.md)
- [規約: 設定と秘密情報](../../../../docs/conventions/config-and-secrets.md)
