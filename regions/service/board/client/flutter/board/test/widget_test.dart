import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:board/app/app.dart';
import 'package:board/config/app_config.dart';
import 'package:board/config/config_provider.dart';
import 'package:board/providers/board_provider.dart';

/// テスト用のAppConfig定数
/// 実際のYAML読み込みを回避し、テスト環境向けの設定値を直接提供する
const _testConfig = AppConfig(
  appName: 'board-test',
  version: '0.0.0',
  env: 'test',
  api: ApiConfig(
    baseUrl: 'http://localhost:8080',
    timeout: 5,
    retryMaxAttempts: 1,
    retryBackoffMs: 100,
  ),
  features: {},
);

/// テスト用のDioアダプター
/// 実際のHTTP通信を行わず、即座に空レスポンスを返すことでペンディングタイマーを防ぐ
class _MockHttpClientAdapter implements HttpClientAdapter {
  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<List<int>>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    /// カラム一覧APIへのリクエストに対し、空のカラムリストを返す
    return ResponseBody.fromString(
      '{"columns": []}',
      200,
      headers: {
        'content-type': ['application/json'],
      },
    );
  }

  @override
  void close({bool force = false}) {}
}

/// テスト用のDioインスタンスを生成する
/// 実際のHTTP通信を行わないモックアダプターを使用する
Dio _createTestDio() {
  final dio = Dio(BaseOptions(baseUrl: 'http://localhost:8080'));
  dio.httpClientAdapter = _MockHttpClientAdapter();
  return dio;
}

/// ボード管理アプリの基本ウィジェットテスト
/// アプリが正常に起動し、タイトルが表示されることを確認する
void main() {
  testWidgets('アプリのタイトルが表示されることを確認する', (WidgetTester tester) async {
    await tester.pumpWidget(
      /// appConfigProviderとdioProviderをテスト用にoverrideし、
      /// 実際のHTTP通信を回避する
      ProviderScope(
        overrides: [
          appConfigProvider.overrideWithValue(_testConfig),
          dioProvider.overrideWithValue(_createTestDio()),
        ],
        child: const BoardApp(),
      ),
    );

    /// Dioの非同期タイマーを消化し、ペンディングタイマーエラーを防ぐ
    await tester.pumpAndSettle();

    /// アプリタイトル「Kanbanボード」がAppBarに表示されていることを検証する
    expect(find.textContaining('Kanbanボード'), findsOneWidget);
  });
}
