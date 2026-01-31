# frontend-flutter テンプレート

← [テンプレート設計書](./)

---

## ディレクトリ構造

```
feature/frontend/flutter/{service_name}/
├── .k1s0/
│   └── manifest.json
├── pubspec.yaml.tera
├── README.md
├── lib/
│   ├── main.dart.tera
│   └── src/
│       ├── domain/
│       │   ├── entities/
│       │   └── repositories/
│       ├── application/
│       │   ├── services/
│       │   └── usecases/
│       └── presentation/
│           ├── widgets/
│           ├── pages/
│           └── providers/
├── test/
└── deploy/
    ├── base/
    └── overlays/
```

## pubspec.yaml.tera

```yaml
name: {{ feature_name_snake }}
description: {{ feature_name_pascal }} Flutter application
publish_to: 'none'
version: 0.1.0

environment:
  sdk: '>=3.3.0 <4.0.0'
  flutter: '>=3.19.0'

dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.5.0
  go_router: ^14.0.0
  freezed_annotation: ^2.4.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^4.0.0
  build_runner: ^2.4.0
  freezed: ^2.5.0

flutter:
  uses-material-design: true
```
