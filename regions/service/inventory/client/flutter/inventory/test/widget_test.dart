import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:inventory/config/app_config.dart';
import 'package:inventory/config/config_provider.dart';
import 'package:inventory/main.dart';

/// テスト用のAppConfig定数
/// 実際のYAML読み込みを回避し、テスト環境向けの設定値を直接提供する
const _testConfig = AppConfig(
  appName: 'inventory-test',
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

/// 在庫管理アプリの基本ウィジェットテスト
/// アプリが正常に起動し、タイトルが表示されることを確認する
void main() {
  testWidgets('アプリのタイトルが表示されることを確認する', (WidgetTester tester) async {
    await tester.pumpWidget(
      /// appConfigProviderをテスト用設定でoverrideし、UnimplementedErrorを回避する
      ProviderScope(
        overrides: [
          appConfigProvider.overrideWithValue(_testConfig),
        ],
        child: const InventoryApp(),
      ),
    );

    /// アプリタイトル「在庫一覧」がAppBarに表示されていることを検証する
    expect(find.text('在庫一覧'), findsOneWidget);
  });
}
