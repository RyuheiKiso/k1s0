import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:payment/main.dart';

/// 決済管理アプリの基本ウィジェットテスト
/// アプリが正常に起動し、タイトルが表示されることを確認する
void main() {
  testWidgets('アプリのタイトルが表示されることを確認する', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: PaymentApp(),
      ),
    );

    /// アプリタイトル「決済一覧」がAppBarに表示されていることを検証する
    expect(find.text('決済一覧'), findsOneWidget);
  });
}
