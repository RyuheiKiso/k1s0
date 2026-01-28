import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/k1s0_ui.dart';

void main() {
  // テスト用のデータ
  final testUsers = [
    {'id': '1', 'name': '山田太郎', 'email': 'yamada@example.com', 'age': 30},
    {'id': '2', 'name': '佐藤花子', 'email': 'sato@example.com', 'age': 25},
    {'id': '3', 'name': '鈴木一郎', 'email': 'suzuki@example.com', 'age': 35},
  ];

  // テスト用のカラム
  final testColumns = [
    K1s0Column<Map<String, dynamic>>(
      field: 'name',
      headerName: '氏名',
      flex: 1,
      sortable: true,
    ),
    K1s0Column<Map<String, dynamic>>(
      field: 'email',
      headerName: 'メール',
      flex: 1,
    ),
    K1s0Column<Map<String, dynamic>>(
      field: 'age',
      headerName: '年齢',
      width: 100,
      type: K1s0ColumnType.number,
    ),
  ];

  Widget buildTestWidget({
    List<Map<String, dynamic>>? rows,
    List<K1s0Column<Map<String, dynamic>>>? columns,
    bool loading = false,
    bool checkboxSelection = false,
    void Function(Map<String, dynamic>)? onRowTap,
  }) {
    return MaterialApp(
      home: Scaffold(
        body: SizedBox(
          height: 400,
          child: K1s0DataTable<Map<String, dynamic>>(
            rows: rows ?? testUsers,
            columns: columns ?? testColumns,
            getRowId: (row) => row['id'] as String,
            loading: loading,
            checkboxSelection: checkboxSelection,
            onRowTap: onRowTap,
          ),
        ),
      ),
    );
  }

  group('K1s0DataTable', () {
    testWidgets('データが正しく表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      expect(find.text('山田太郎'), findsOneWidget);
      expect(find.text('佐藤花子'), findsOneWidget);
      expect(find.text('鈴木一郎'), findsOneWidget);
    });

    testWidgets('カラムヘッダーが表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      expect(find.text('氏名'), findsOneWidget);
      expect(find.text('メール'), findsOneWidget);
      expect(find.text('年齢'), findsOneWidget);
    });

    testWidgets('空データで空状態が表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget(rows: []));

      expect(find.text('データがありません'), findsOneWidget);
    });

    testWidgets('ローディング状態が表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget(loading: true));

      // ローディング中はデータが表示されない
      expect(find.text('山田太郎'), findsNothing);
    });

    testWidgets('チェックボックスが表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget(checkboxSelection: true));

      // ヘッダーのチェックボックス + 各行のチェックボックス
      expect(find.byType(Checkbox), findsNWidgets(4));
    });

    testWidgets('行タップでコールバックが呼ばれる', (tester) async {
      Map<String, dynamic>? tappedRow;

      await tester.pumpWidget(buildTestWidget(
        onRowTap: (row) => tappedRow = row,
      ));

      await tester.tap(find.text('山田太郎'));
      await tester.pumpAndSettle();

      expect(tappedRow, isNotNull);
      expect(tappedRow!['name'], equals('山田太郎'));
    });

    testWidgets('ソートアイコンが表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      // ソート可能なカラムのヘッダーをタップ
      await tester.tap(find.text('氏名'));
      await tester.pumpAndSettle();

      // ソートアイコンが表示される
      expect(find.byIcon(Icons.arrow_upward), findsOneWidget);
    });

    testWidgets('ページネーションが表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      expect(find.text('表示件数:'), findsOneWidget);
      expect(find.byIcon(Icons.chevron_left), findsOneWidget);
      expect(find.byIcon(Icons.chevron_right), findsOneWidget);
    });
  });

  group('K1s0Column', () {
    test('数値フォーマットが正しく動作する', () {
      final column = K1s0Column<Map<String, dynamic>>(
        field: 'price',
        headerName: '価格',
        type: K1s0ColumnType.number,
        prefix: '¥',
        thousandSeparator: true,
      );

      expect(column.formatValue(1234567), equals('¥1,234,567'));
    });

    test('日付フォーマットが正しく動作する', () {
      final column = K1s0Column<Map<String, dynamic>>(
        field: 'date',
        headerName: '日付',
        type: K1s0ColumnType.date,
      );

      final date = DateTime(2024, 6, 15);
      expect(column.formatValue(date), equals('2024/06/15'));
    });

    test('Boolean フォーマットが正しく動作する', () {
      final column = K1s0Column<Map<String, dynamic>>(
        field: 'active',
        headerName: '有効',
        type: K1s0ColumnType.boolean,
      );

      expect(column.formatValue(true), equals('はい'));
      expect(column.formatValue(false), equals('いいえ'));
    });

    test('singleSelect フォーマットが正しく動作する', () {
      final column = K1s0Column<Map<String, dynamic>>(
        field: 'role',
        headerName: '権限',
        type: K1s0ColumnType.singleSelect,
        valueOptions: [
          K1s0ValueOption(label: '管理者', value: 'admin'),
          K1s0ValueOption(label: '一般', value: 'user'),
        ],
      );

      expect(column.formatValue('admin'), equals('管理者'));
      expect(column.formatValue('user'), equals('一般'));
    });
  });
}
