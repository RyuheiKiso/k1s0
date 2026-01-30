# k1s0_config (Flutter)

← [Flutter パッケージ一覧](./)

## 目的

YAML 設定ファイルの読み込み、型付け、バリデーション、環境マージを提供する。

## 主要な型

```dart
@freezed
class AppConfig with _$AppConfig {
  const factory AppConfig({
    required ApiConfig api,
    required AuthConfig auth,
    required LoggingConfig logging,
    @Default({}) Map<String, bool> featureFlags,
  }) = _AppConfig;
}

class ConfigLoader {
  ConfigLoader({required String defaultPath, String? environment});
  Future<AppConfig> load();
}
```

## 使用例

```dart
final loader = ConfigLoader(
  defaultPath: 'assets/config/default.yaml',
  environment: 'production',
);
final config = await loader.load();

// Riverpod Provider経由でアクセス
ConfigScope(
  config: config,
  child: MyApp(),
)

// 子ウィジェットで使用
final config = ref.watch(configProvider);
```
