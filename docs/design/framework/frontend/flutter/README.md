# Flutter パッケージ一覧

```
framework/frontend/flutter/packages/
├── k1s0_navigation/     # 設定駆動ナビゲーション（NEW）
├── k1s0_config/         # YAML設定管理
├── k1s0_http/           # API通信クライアント
├── k1s0_auth/           # 認証クライアント
├── k1s0_observability/  # OTel/ログ
├── k1s0_ui/             # Design System
├── k1s0_state/          # 状態管理
└── k1s0_realtime/       # WebSocket/SSEリアルタイム通信（NEW）
```

## 実装状況

| パッケージ | 状態 | 説明 |
|-----------|:----:|------|
| k1s0_navigation | ✅ | 設定駆動ナビゲーション、go_router統合、ルートガード |
| k1s0_config | ✅ | YAML設定管理、Zodスキーマバリデーション、環境マージ |
| k1s0_http | ✅ | Dioベース通信クライアント、OTel計測、ProblemDetails対応 |
| k1s0_auth | ✅ | JWT/OIDC認証、SecureStorage、トークン自動更新 |
| k1s0_observability | ✅ | 構造化ログ、分散トレース、メトリクス収集 |
| k1s0_ui | ✅ | Material 3 Design System、共通ウィジェット、テーマ |
| k1s0_state | ✅ | Riverpod状態管理、AsyncValueヘルパー、永続化 |
| k1s0_realtime | ✅ | WebSocket/SSEクライアント、自動再接続、ハートビート、オフラインキュー |

## パッケージ詳細

| パッケージ | ドキュメント |
|-----------|-------------|
| k1s0_navigation | [navigation.md](./navigation.md) |
| k1s0_config | [config.md](./config.md) |
| k1s0_http | [http.md](./http.md) |
| k1s0_auth | [auth.md](./auth.md) |
| k1s0_observability | [observability.md](./observability.md) |
| k1s0_ui | [ui.md](./ui.md) |
| k1s0_state | [state.md](./state.md) |
