# `examples/tier3-native-maui/` — .NET MAUI 最小アプリ

tier3 Native レイヤ（.NET MAUI、iOS / Android / Windows / macOS クロスプラットフォーム）の
典型的な実装パタンを示す最小アプリ例。

## 目的

- `src/tier3/native/apps/{Hub, Admin}` と同じ構造（apps / shared）を新規メンバーが真似できる
- `K1s0.Sdk` NuGet 経由で tier1 gRPC を呼び出すモバイル / デスクトップ実装パタン
- オフラインバッファリング・background sync の典型例

## scope

| 段階 | 提供範囲 |
|---|---|
| リリース時点 | 本 README のみ（構造規定） |
| 採用初期 | `Example.Native.sln` + MAUI app + ViewModel + K1s0.Sdk wrapper |
| 採用後の運用拡大時 | プッシュ通知 / オフライン同期 / バイオメトリクス認証 |

## 想定構成（採用初期）

```
tier3-native-maui/
├── README.md                       # 本ファイル
├── Example.Native.sln
├── apps/
│   └── Example.Native.Hub/         # MAUI app (iOS / Android / Windows / macOS)
│       ├── Example.Native.Hub.csproj
│       ├── App.xaml
│       ├── MainPage.xaml
│       └── ViewModels/
└── shared/
    └── Example.Native.Shared/      # ViewModel base / K1s0.Sdk wrapper
```

## 関連 docs / ADR

- `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/`（tier3 Native レイアウト準ずる）
- ADR-DEV-001（Paved Road）

## 参照する tier1 API（採用初期想定）

- StateService（ローカルキャッシュとサーバ状態の同期）
- LogService（モバイル発生エラーログの集約）
- AuditService（モバイル操作の監査ログ）
- FeatureService（モバイル向けフィーチャーフラグ）
