# k1s0_http (Flutter)

← [Flutter パッケージ一覧](./)

## 目的

Dio ベースの HTTP クライアント。トレース伝播、エラーハンドリング、ProblemDetails 対応を提供。

## 主要な型

```dart
class K1s0HttpClient {
  K1s0HttpClient({required HttpClientConfig config});

  Future<K1s0Response<T>> get<T>(String path, {RequestOptions? options});
  Future<K1s0Response<T>> post<T>(String path, {dynamic data, RequestOptions? options});
  Future<K1s0Response<T>> put<T>(String path, {dynamic data, RequestOptions? options});
  Future<K1s0Response<T>> delete<T>(String path, {RequestOptions? options});
}

@freezed
class ProblemDetails with _$ProblemDetails {
  const factory ProblemDetails({
    required String type,
    required String title,
    required int status,
    String? detail,
    String? instance,
    String? errorCode,
    String? traceId,
  }) = _ProblemDetails;
}

class ApiError {
  final ApiErrorKind kind;
  final String message;
  final ProblemDetails? problemDetails;
}
```

## 使用例

```dart
final client = K1s0HttpClient(
  config: HttpClientConfig(
    baseUrl: 'https://api.example.com',
    timeout: Duration(seconds: 30),
  ),
);

try {
  final response = await client.get<User>('/users/123');
  print(response.data);
} on ApiError catch (e) {
  print('Error: ${e.message}');
  if (e.problemDetails != null) {
    print('Error Code: ${e.problemDetails!.errorCode}');
  }
}
```
