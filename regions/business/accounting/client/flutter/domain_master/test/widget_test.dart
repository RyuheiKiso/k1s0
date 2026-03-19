import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:domain_master/config/app_config.dart';
import 'package:domain_master/config/config_provider.dart';
import 'package:domain_master/providers/domain_master_provider.dart';
import 'package:domain_master/screens/category_list_screen.dart';

/// テスト用のAppConfig定数
/// 実際のYAML読み込みを回避し、テスト環境向けの設定値を直接提供する
const _testConfig = AppConfig(
  appName: 'domain-master-test',
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
    /// カテゴリ一覧APIへのリクエストに対し、空のカテゴリリストを返す
    return ResponseBody.fromString(
      '{"categories": []}',
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

/// CategoryListScreenの基本的なウィジェットテスト
void main() {
  /// CategoryListScreenが正常にレンダリングされることを確認する
  testWidgets('CategoryListScreen が正常に表示される', (tester) async {
    await tester.pumpWidget(
      /// appConfigProviderとdioProviderをテスト用にoverrideし、
      /// 実際のHTTP通信を回避する
      ProviderScope(
        overrides: [
          appConfigProvider.overrideWithValue(_testConfig),
          dioProvider.overrideWithValue(_createTestDio()),
        ],
        child: const MaterialApp(
          home: CategoryListScreen(),
        ),
      ),
    );

    /// AppBarのタイトルが表示されることを検証する
    expect(find.text('マスタカテゴリ管理'), findsOneWidget);

    /// FABが表示されることを検証する
    expect(find.byType(FloatingActionButton), findsOneWidget);
  });

  /// FABタップでカテゴリ作成ダイアログが表示されることを確認する
  testWidgets('FABタップでカテゴリ作成ダイアログが表示される', (tester) async {
    await tester.pumpWidget(
      /// appConfigProviderとdioProviderをテスト用にoverrideし、
      /// 実際のHTTP通信を回避する
      ProviderScope(
        overrides: [
          appConfigProvider.overrideWithValue(_testConfig),
          dioProvider.overrideWithValue(_createTestDio()),
        ],
        child: const MaterialApp(
          home: CategoryListScreen(),
        ),
      ),
    );

    /// 非同期のAPI呼び出しが完了するまで待つ
    await tester.pumpAndSettle();

    /// FABをタップする
    await tester.tap(find.byType(FloatingActionButton));
    await tester.pumpAndSettle();

    /// ダイアログのタイトルが表示されることを検証する
    expect(find.text('カテゴリ作成'), findsOneWidget);

    /// フォームフィールドが表示されることを検証する
    expect(find.text('コード'), findsOneWidget);
    expect(find.text('表示名'), findsOneWidget);
  });
}
