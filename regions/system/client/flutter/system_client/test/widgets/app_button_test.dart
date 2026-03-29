import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:system_client/system_client.dart';

void main() {
  group('AppButton', () {
    testWidgets('primary バリアントは ElevatedButton をレンダリングする', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AppButton(
              label: 'クリック',
              onPressed: () {},
            ),
          ),
        ),
      );
      expect(find.byType(ElevatedButton), findsOneWidget);
      expect(find.text('クリック'), findsOneWidget);
    });

    testWidgets('secondary バリアントは OutlinedButton をレンダリングする', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AppButton(
              label: 'キャンセル',
              onPressed: () {},
              variant: AppButtonVariant.secondary,
            ),
          ),
        ),
      );
      expect(find.byType(OutlinedButton), findsOneWidget);
    });

    testWidgets('text バリアントは TextButton をレンダリングする', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AppButton(
              label: 'リンク',
              onPressed: () {},
              variant: AppButtonVariant.text,
            ),
          ),
        ),
      );
      expect(find.byType(TextButton), findsOneWidget);
    });

    testWidgets('isLoading が true の場合は CircularProgressIndicator が表示される', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AppButton(
              label: '送信中',
              onPressed: () {},
              isLoading: true,
            ),
          ),
        ),
      );
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('送信中'), findsNothing);
    });

    testWidgets('isLoading が true の場合はボタンが無効化される', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AppButton(
              label: '送信',
              onPressed: () {},
              isLoading: true,
            ),
          ),
        ),
      );
      final button = tester.widget<ElevatedButton>(find.byType(ElevatedButton));
      expect(button.onPressed, isNull);
    });

    testWidgets('onPressed が呼ばれる', (tester) async {
      var tapped = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AppButton(
              label: 'タップ',
              onPressed: () => tapped = true,
            ),
          ),
        ),
      );
      await tester.tap(find.byType(ElevatedButton));
      expect(tapped, isTrue);
    });

    testWidgets('onPressed が null の場合はボタンが無効化される', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: AppButton(
              label: '無効',
              onPressed: null,
            ),
          ),
        ),
      );
      final button = tester.widget<ElevatedButton>(find.byType(ElevatedButton));
      expect(button.onPressed, isNull);
    });
  });
}
