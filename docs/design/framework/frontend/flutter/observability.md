# k1s0_observability (Flutter)

← [Flutter パッケージ一覧](./)

## 目的

フロントエンド向け観測性ライブラリ。構造化ログ、分散トレース、エラートラッキング、パフォーマンスメトリクスを提供。

## 必須フィールド（ログ）

バックエンド（k1s0-observability）と同じ必須フィールドをフロントエンドでも強制。

| フィールド | 説明 |
|-----------|------|
| `timestamp` | ISO 8601 形式のタイムスタンプ |
| `level` | ログレベル（debug/info/warn/error） |
| `message` | ログメッセージ |
| `service_name` | サービス名 |
| `env` | 環境名（dev/stg/prod） |
| `trace_id` | トレース ID（リクエスト相関用） |
| `span_id` | スパン ID |

## 主要な型

```dart
@freezed
class LogEntry with _$LogEntry {
  const factory LogEntry({
    required DateTime timestamp,
    required LogLevel level,
    required String message,
    required String serviceName,
    required String env,
    String? traceId,
    String? spanId,
    @Default({}) Map<String, dynamic> fields,
  }) = _LogEntry;
}

class Logger {
  void debug(String message, [Map<String, dynamic>? fields]);
  void info(String message, [Map<String, dynamic>? fields]);
  void warn(String message, [Map<String, dynamic>? fields]);
  void error(String message, [Object? error, StackTrace? stackTrace]);
}

class Tracer {
  Future<T> trace<T>(String name, Future<T> Function() fn);
  T traceSync<T>(String name, T Function() fn);
}
```

## 使用例

```dart
// Logger の使用
final logger = ref.read(loggerProvider);
logger.info('ユーザーがログインしました', {
  'userId': user.id,
  'loginMethod': 'oauth',
});

// Tracer の使用
final tracer = ref.read(tracerProvider);
final user = await tracer.trace('fetch-user-data', () async {
  return await api.getUser(userId);
});

// エラートラッキング
final errorTracker = ref.read(errorTrackerProvider);
try {
  await riskyOperation();
} catch (e, stackTrace) {
  errorTracker.capture(e, stackTrace);
}
```
