# Frontend Android Template

Android (Kotlin + Jetpack Compose) フロントエンドのテンプレート。

## ディレクトリ構成

```
frontend-android/
├── feature/      # 機能雛形（k1s0 new-feature 用）
└── domain/       # ドメインライブラリ雛形（k1s0 new-domain 用）
```

## feature/ の生成物

`k1s0 new-feature --type frontend-android --name {name}` で生成される構成：

```
feature/frontend/android/{name}/
├── build.gradle.kts
├── app/
│   ├── build.gradle.kts
│   └── src/main/
│       ├── AndroidManifest.xml
│       └── kotlin/{package}/
│           ├── domain/              # ビジネスロジック層
│           │   ├── entities/
│           │   ├── valueobjects/
│           │   ├── repositories/
│           │   └── services/
│           ├── application/         # アプリケーション層
│           │   ├── usecases/
│           │   ├── services/
│           │   └── dtos/
│           ├── infrastructure/      # インフラストラクチャ層
│           │   ├── repositories/
│           │   ├── external/
│           │   └── persistence/
│           └── presentation/        # プレゼンテーション層
│               ├── screens/
│               ├── components/
│               └── theme/
├── config/
│   └── default.yaml
└── .k1s0/
    └── manifest.json
```

## 技術スタック

- UI: Jetpack Compose + Material 3
- DI: Koin
- 状態管理: ViewModel + StateFlow
- HTTP: Ktor Client
- ナビゲーション: Navigation Compose
- ビルド: Gradle (Kotlin DSL)
- 設定: k1s0-config (YAML)
- ログ/トレース: OpenTelemetry
