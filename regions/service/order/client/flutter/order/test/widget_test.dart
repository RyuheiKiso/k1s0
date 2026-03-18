import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:order/config/app_config.dart';
import 'package:order/config/config_provider.dart';
import 'package:order/main.dart';

/// テスト用のAppConfig定数
/// 実際のYAML読み込みを回避し、テスト環境向けの設定値を直接提供する
const _testConfig = AppConfig(
  appName: 'order-test',
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

/// 注文管理アプリの基本ウィジェットテスト
/// アプリが正常に起動し、タイトルが表示されることを確認する
void main() {
  testWidgets('アプリのタイトルが表示されることを確認する', (WidgetTester tester) async {
    await tester.pumpWidget(
      /// appConfigProviderをテスト用設定でoverrideし、UnimplementedErrorを回避する
      ProviderScope(
        overrides: [
          appConfigProvider.overrideWithValue(_testConfig),
        ],
        child: const OrderApp(),
      ),
    );

    /// アプリタイトル「注文一覧」がAppBarに表示されていることを検証する
    expect(find.text('注文一覧'), findsOneWidget);
  });
}
