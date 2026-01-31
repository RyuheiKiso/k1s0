# frontend-android テンプレート

← [テンプレート設計書](./)

---

## ディレクトリ構造

```
feature/frontend/android/{service_name}/
├── .k1s0/
│   └── manifest.json.tera
├── build.gradle.kts.tera
├── settings.gradle.kts
├── gradle.properties
├── app/
│   ├── build.gradle.kts.tera
│   └── src/
│       └── main/
│           ├── AndroidManifest.xml.tera
│           └── kotlin/
│               └── {{ package_path }}/
│                   ├── MainActivity.kt.tera
│                   ├── MainApplication.kt.tera
│                   ├── domain/
│                   │   ├── entities/
│                   │   ├── valueobjects/
│                   │   ├── repositories/
│                   │   └── services/
│                   ├── application/
│                   │   ├── usecases/
│                   │   ├── services/
│                   │   └── dtos/
│                   ├── infrastructure/
│                   │   ├── repositories/
│                   │   ├── external/
│                   │   └── persistence/
│                   └── presentation/
│                       ├── screens/
│                       ├── components/
│                       ├── navigation/
│                       └── theme/
├── config/
│   ├── default.yaml.tera
│   └── dev.yaml.tera
└── README.md.tera
```

## 特徴

- **Jetpack Compose** ベースの UI
- **Material 3** デザインシステム
- **Kotlin Gradle DSL** によるビルド管理
- **Koin** で依存性注入
- **Ktor Client** で HTTP 通信
- **Navigation Compose** でルーティング
- **ktlint** でフォーマット
- **detekt** で静的解析
- **JUnit 5 + Compose UI Test** でテスト
- **MVVM パターン**: ViewModel + StateFlow による状態管理
