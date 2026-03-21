import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:system_client/system_client.dart';

void main() {
  group('LoadingIndicator', () {
    testWidgets('CircularProgressIndicator が表示される', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: LoadingIndicator()),
        ),
      );
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('メッセージが指定された場合はテキストが表示される', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: LoadingIndicator(message: '読み込み中...')),
        ),
      );
      expect(find.text('読み込み中...'), findsOneWidget);
    });

    testWidgets('メッセージ未指定の場合はテキストが表示されない', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: LoadingIndicator()),
        ),
      );
      expect(find.byType(Text), findsNothing);
    });

    testWidgets('Center ウィジェットでラップされている', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: LoadingIndicator()),
        ),
      );
      expect(find.byType(Center), findsOneWidget);
    });

    testWidgets('カスタムサイズが適用される', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: LoadingIndicator(size: 80.0)),
        ),
      );
      final sizedBox = tester.widget<SizedBox>(
        find.ancestor(
          of: find.byType(CircularProgressIndicator),
          matching: find.byType(SizedBox),
        ).first,
      );
      expect(sizedBox.width, equals(80.0));
      expect(sizedBox.height, equals(80.0));
    });
  });
}
