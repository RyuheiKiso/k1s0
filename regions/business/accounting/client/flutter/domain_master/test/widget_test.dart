import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:domain_master/screens/category_list_screen.dart';

/// CategoryListScreenの基本的なウィジェットテスト
void main() {
  /// CategoryListScreenが正常にレンダリングされることを確認する
  testWidgets('CategoryListScreen が正常に表示される', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: MaterialApp(
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
      const ProviderScope(
        child: MaterialApp(
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
