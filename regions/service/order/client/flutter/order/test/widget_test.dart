import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:order/main.dart';

/// 注文管理アプリの基本ウィジェットテスト
/// アプリが正常に起動し、タイトルが表示されることを確認する
void main() {
  testWidgets('アプリのタイトルが表示されることを確認する', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: OrderApp(),
      ),
    );

    /// アプリタイトル「注文一覧」がAppBarに表示されていることを検証する
    expect(find.text('注文一覧'), findsOneWidget);
  });
}
