import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:domain_master/config/app_config.dart';
import 'package:domain_master/config/config_provider.dart';
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

/// CategoryListScreenの基本的なウィジェットテスト
void main() {
  /// CategoryListScreenが正常にレンダリングされることを確認する
  testWidgets('CategoryListScreen が正常に表示される', (tester) async {
    await tester.pumpWidget(
      /// appConfigProviderをテスト用設定でoverrideし、UnimplementedErrorを回避する
      ProviderScope(
        overrides: [
          appConfigProvider.overrideWithValue(_testConfig),
        ],
        child: const MaterialApp(
          home: CategoryListScreen(),
        ),
      ),
    );

    /// AppBarのタイトルが表示されることを検証する
    expect(find.text('マスタカテゴリ管理'), findsOneWidget);

    /// ローディングインジケーターが表示されることを検証する
    /// （API接続がないため、ローディング状態のままとなる）
    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    /// FABが表示されることを検証する
    expect(find.byType(FloatingActionButton), findsOneWidget);
  });

  /// FABタップでカテゴリ作成ダイアログが表示されることを確認する
  testWidgets('FABタップでカテゴリ作成ダイアログが表示される', (tester) async {
    await tester.pumpWidget(
      /// appConfigProviderをテスト用設定でoverrideし、UnimplementedErrorを回避する
      ProviderScope(
        overrides: [
          appConfigProvider.overrideWithValue(_testConfig),
        ],
        child: const MaterialApp(
          home: CategoryListScreen(),
        ),
      ),
    );

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
