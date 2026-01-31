# Frontend Framework（Android）

k1s0 Frontend Framework（Android）は、Jetpack Compose ベースの Android アプリ開発のための共通パッケージ群を提供します。React 版・Flutter 版と同等の機能を Android ネイティブで実装しています。

## パッケージ一覧

```
framework/frontend/android/
├── build.gradle.kts               # ルートビルド定義
├── settings.gradle.kts             # マルチプロジェクト設定
├── packages/
│   ├── k1s0-navigation/           # Navigation Compose ルーティング
│   ├── k1s0-config/               # YAML 設定管理
│   ├── k1s0-http/                 # Ktor Client HTTP クライアント
│   ├── k1s0-ui/                   # Material 3 デザインシステム
│   ├── k1s0-auth/                 # JWT 認証クライアント
│   ├── k1s0-observability/        # ログ・トレーシング
│   ├── k1s0-state/                # ViewModel + StateFlow ユーティリティ
│   └── k1s0-realtime/             # WebSocket/SSE クライアント
└── tests/
```

## パッケージ詳細

| パッケージ | 説明 | 主要依存 |
|-----------|------|---------|
| k1s0-navigation | 設定駆動型ルーティング | Navigation Compose |
| k1s0-config | YAML 設定ファイル読み込み | kaml |
| k1s0-http | HTTP クライアント共通設定 | Ktor Client |
| k1s0-ui | Material 3 テーマ・共通コンポーネント | Jetpack Compose, Material 3 |
| k1s0-auth | JWT/OIDC 認証フロー | AppAuth, DataStore |
| k1s0-observability | 構造化ログ・トレーシング | OpenTelemetry Android |
| k1s0-state | ViewModel + StateFlow パターン | Lifecycle ViewModel, Koin |
| k1s0-realtime | WebSocket/SSE 再接続・ハートビート・オフラインキュー | Ktor Client WebSocket |

## 技術スタック

| 項目 | 技術 |
|------|------|
| 言語 | Kotlin 2.x |
| UI | Jetpack Compose + Material 3 |
| DI | Koin / Hilt |
| 状態管理 | ViewModel + StateFlow |
| HTTP | Ktor Client |
| ビルドツール | Gradle (Kotlin DSL) |
| Lint | ktlint, Android Lint |
| 静的解析 | detekt |
| 最小 SDK | API 26 (Android 8.0) |

## 関連ドキュメント

- [Framework 設計（トップ）](../README.md)
- [サービス構成規約](../../../conventions/service-structure.md)
- [設定・シークレット規約](../../../conventions/config-and-secrets.md)
