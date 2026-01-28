import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/k1s0_ui.dart';

/// DataTable ゴールデンテスト
///
/// ゴールデンファイルの更新:
/// ```bash
/// flutter test --update-goldens test/golden/data_table_golden_test.dart
/// ```
void main() {
  // テスト用データ
  final testUsers = [
    {'id': '1', 'name': '山田太郎', 'email': 'yamada@example.com', 'role': 'admin', 'active': true},
    {'id': '2', 'name': '鈴木花子', 'email': 'suzuki@example.com', 'role': 'user', 'active': true},
    {'id': '3', 'name': '田中一郎', 'email': 'tanaka@example.com', 'role': 'guest', 'active': false},
  ];

  // テスト用カラム
  final columns = <K1s0Column<Map<String, dynamic>>>[
    K1s0Column(
      id: 'name',
      label: '氏名',
      valueGetter: (row) => row['name'],
      sortable: true,
    ),
    K1s0Column(
      id: 'email',
      label: 'メール',
      valueGetter: (row) => row['email'],
      flex: 2,
    ),
    K1s0Column(
      id: 'role',
      label: '権限',
      valueGetter: (row) => row['role'],
      type: K1s0ColumnType.chip,
      width: 100,
    ),
    K1s0Column(
      id: 'active',
      label: '有効',
      valueGetter: (row) => row['active'],
      type: K1s0ColumnType.boolean,
      width: 80,
    ),
  ];

  Widget buildTestWidget(Widget child) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        colorSchemeSeed: Colors.blue,
      ),
      home: Scaffold(
        body: RepaintBoundary(
          child: SizedBox(
            width: 800,
            height: 400,
            child: child,
          ),
        ),
      ),
    );
  }

  group('DataTable ゴールデンテスト', () {
    testWidgets('基本表示', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0DataTable<Map<String, dynamic>>(
          rows: testUsers,
          columns: columns,
          getRowId: (row) => row['id'] as String,
        ),
      ));

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/data_table_basic.png'),
      );
    });

    testWidgets('選択状態', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0DataTable<Map<String, dynamic>>(
          rows: testUsers,
          columns: columns,
          getRowId: (row) => row['id'] as String,
          selectionMode: K1s0SelectionMode.multiple,
          selectedIds: const {'1', '2'},
        ),
      ));

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/data_table_selected.png'),
      );
    });

    testWidgets('ローディング状態', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0DataTable<Map<String, dynamic>>(
          rows: const [],
          columns: columns,
          getRowId: (row) => row['id'] as String,
          loading: true,
        ),
      ));

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/data_table_loading.png'),
      );
    });

    testWidgets('空状態', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0DataTable<Map<String, dynamic>>(
          rows: const [],
          columns: columns,
          getRowId: (row) => row['id'] as String,
          emptyMessage: 'データがありません',
        ),
      ));

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/data_table_empty.png'),
      );
    });

    testWidgets('ページネーション付き', (tester) async {
      final manyUsers = List.generate(
        25,
        (i) => {
          'id': '${i + 1}',
          'name': 'ユーザー${i + 1}',
          'email': 'user${i + 1}@example.com',
          'role': ['admin', 'user', 'guest'][i % 3],
          'active': i % 2 == 0,
        },
      );

      await tester.pumpWidget(buildTestWidget(
        K1s0DataTable<Map<String, dynamic>>(
          rows: manyUsers,
          columns: columns,
          getRowId: (row) => row['id'] as String,
          pagination: true,
          pageSize: 10,
        ),
      ));

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/data_table_pagination.png'),
      );
    });

    testWidgets('ソート状態（昇順）', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0DataTable<Map<String, dynamic>>(
          rows: testUsers,
          columns: columns,
          getRowId: (row) => row['id'] as String,
          initialSortColumn: 'name',
          initialSortOrder: K1s0SortOrder.ascending,
        ),
      ));

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/data_table_sorted_asc.png'),
      );
    });

    testWidgets('ダークモード', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          debugShowCheckedModeBanner: false,
          theme: ThemeData(
            useMaterial3: true,
            colorSchemeSeed: Colors.blue,
            brightness: Brightness.dark,
          ),
          home: Scaffold(
            body: RepaintBoundary(
              child: SizedBox(
                width: 800,
                height: 400,
                child: K1s0DataTable<Map<String, dynamic>>(
                  rows: testUsers,
                  columns: columns,
                  getRowId: (row) => row['id'] as String,
                ),
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/data_table_dark.png'),
      );
    });
  });
}
