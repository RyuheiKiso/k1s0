import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/k1s0_ui.dart';

/// DataTable + Form 統合テスト
void main() {
  // テスト用データモデル
  final testUsers = [
    {'id': '1', 'name': '山田太郎', 'email': 'yamada@example.com', 'role': 'admin'},
    {'id': '2', 'name': '鈴木花子', 'email': 'suzuki@example.com', 'role': 'user'},
    {'id': '3', 'name': '田中一郎', 'email': 'tanaka@example.com', 'role': 'guest'},
  ];

  // テスト用スキーマ
  final userSchema = K1s0FormSchema<Map<String, dynamic>>(
    fields: [
      K1s0FormFieldSchema(
        name: 'name',
        label: '氏名',
        required: true,
      ),
      K1s0FormFieldSchema(
        name: 'email',
        label: 'メールアドレス',
        type: K1s0FieldType.email,
        required: true,
      ),
      K1s0FormFieldSchema(
        name: 'role',
        label: '権限',
        type: K1s0FieldType.select,
        options: [
          K1s0FieldOption(label: '管理者', value: 'admin'),
          K1s0FieldOption(label: '一般', value: 'user'),
          K1s0FieldOption(label: 'ゲスト', value: 'guest'),
        ],
      ),
    ],
    fromMap: (map) => map,
    toMap: (value) => value,
  );

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
    ),
    K1s0Column(
      id: 'role',
      label: '権限',
      valueGetter: (row) => row['role'],
      type: K1s0ColumnType.chip,
    ),
  ];

  Widget buildTestApp(Widget child) {
    return MaterialApp(
      home: Scaffold(
        body: SingleChildScrollView(
          child: child,
        ),
      ),
    );
  }

  group('DataTable + Form 統合テスト', () {
    testWidgets('DataTableからフォームに選択データを渡せる', (tester) async {
      Map<String, dynamic>? selectedUser;

      await tester.pumpWidget(buildTestApp(
        Column(
          children: [
            SizedBox(
              height: 300,
              child: K1s0DataTable<Map<String, dynamic>>(
                rows: testUsers,
                columns: columns,
                getRowId: (row) => row['id'] as String,
                onRowTap: (row) {
                  selectedUser = row;
                },
              ),
            ),
            if (selectedUser != null)
              K1s0Form<Map<String, dynamic>>(
                schema: userSchema,
                initialValues: selectedUser,
                onSubmit: (_) async {},
              ),
          ],
        ),
      ));

      // 行をタップ
      await tester.tap(find.text('山田太郎'));
      await tester.pumpAndSettle();

      expect(selectedUser, isNotNull);
      expect(selectedUser!['name'], equals('山田太郎'));
    });

    testWidgets('フォーム送信後にDataTableを更新できる', (tester) async {
      final users = List<Map<String, dynamic>>.from(testUsers);
      Map<String, dynamic>? submittedValue;

      await tester.pumpWidget(
        StatefulBuilder(
          builder: (context, setState) {
            return buildTestApp(
              Column(
                children: [
                  SizedBox(
                    height: 300,
                    child: K1s0DataTable<Map<String, dynamic>>(
                      rows: users,
                      columns: columns,
                      getRowId: (row) => row['id'] as String,
                    ),
                  ),
                  K1s0Form<Map<String, dynamic>>(
                    schema: userSchema,
                    onSubmit: (values) async {
                      submittedValue = values;
                      setState(() {
                        users.add({
                          'id': '4',
                          ...values,
                        });
                      });
                    },
                  ),
                ],
              ),
            );
          },
        ),
      );

      // 最初は3行
      expect(find.byType(InkWell), findsNWidgets(3));

      // フォームに入力
      await tester.enterText(
        find.byType(TextFormField).first,
        '新規ユーザー',
      );
      await tester.enterText(
        find.byType(TextFormField).at(1),
        'new@example.com',
      );

      // 送信
      await tester.tap(find.text('送信'));
      await tester.pumpAndSettle();

      expect(submittedValue, isNotNull);
      expect(submittedValue!['name'], equals('新規ユーザー'));
    });

    testWidgets('選択モードで複数選択したIDをフォームに渡せる', (tester) async {
      final selectedIds = <String>{};

      await tester.pumpWidget(buildTestApp(
        K1s0DataTable<Map<String, dynamic>>(
          rows: testUsers,
          columns: columns,
          getRowId: (row) => row['id'] as String,
          selectionMode: K1s0SelectionMode.multiple,
          selectedIds: selectedIds,
          onSelectionChange: (ids) {
            selectedIds.clear();
            selectedIds.addAll(ids);
          },
        ),
      ));

      // チェックボックスをタップ
      final checkboxes = find.byType(Checkbox);
      expect(checkboxes, findsNWidgets(4)); // ヘッダー + 3行

      await tester.tap(checkboxes.at(1)); // 1行目
      await tester.pumpAndSettle();

      await tester.tap(checkboxes.at(2)); // 2行目
      await tester.pumpAndSettle();

      expect(selectedIds.length, equals(2));
      expect(selectedIds.contains('1'), isTrue);
      expect(selectedIds.contains('2'), isTrue);
    });
  });

  group('パフォーマンステスト', () {
    testWidgets('大量データ（500行）のDataTableがレンダリングできる', (tester) async {
      final largeDataset = List.generate(
        500,
        (i) => {
          'id': '${i + 1}',
          'name': 'ユーザー${i + 1}',
          'email': 'user${i + 1}@example.com',
          'role': ['admin', 'user', 'guest'][i % 3],
        },
      );

      final stopwatch = Stopwatch()..start();

      await tester.pumpWidget(buildTestApp(
        SizedBox(
          height: 600,
          child: K1s0DataTable<Map<String, dynamic>>(
            rows: largeDataset,
            columns: columns,
            getRowId: (row) => row['id'] as String,
            pagination: true,
            pageSize: 20,
          ),
        ),
      ));

      stopwatch.stop();

      // 5秒以内にレンダリング完了
      expect(stopwatch.elapsedMilliseconds, lessThan(5000));

      // DataTableが表示される
      expect(find.byType(K1s0DataTable<Map<String, dynamic>>), findsOneWidget);
    });

    testWidgets('複雑なフォーム（10フィールド）がレンダリングできる', (tester) async {
      final largeSchema = K1s0FormSchema<Map<String, dynamic>>(
        fields: List.generate(
          10,
          (i) => K1s0FormFieldSchema(
            name: 'field$i',
            label: 'フィールド${i + 1}',
          ),
        ),
        fromMap: (map) => map,
        toMap: (value) => value,
      );

      final stopwatch = Stopwatch()..start();

      await tester.pumpWidget(buildTestApp(
        K1s0Form<Map<String, dynamic>>(
          schema: largeSchema,
          onSubmit: (_) async {},
        ),
      ));

      stopwatch.stop();

      // 2秒以内にレンダリング完了
      expect(stopwatch.elapsedMilliseconds, lessThan(2000));

      // 全フィールドが表示される
      expect(find.text('フィールド1'), findsOneWidget);
      expect(find.text('フィールド10'), findsOneWidget);
    });
  });

  group('エラー状態テスト', () {
    testWidgets('フォームバリデーションエラーがDataTableと共存できる', (tester) async {
      await tester.pumpWidget(buildTestApp(
        Column(
          children: [
            SizedBox(
              height: 200,
              child: K1s0DataTable<Map<String, dynamic>>(
                rows: testUsers,
                columns: columns,
                getRowId: (row) => row['id'] as String,
              ),
            ),
            K1s0Form<Map<String, dynamic>>(
              schema: userSchema,
              onSubmit: (_) async {},
            ),
          ],
        ),
      ));

      // 空のまま送信
      await tester.tap(find.text('送信'));
      await tester.pumpAndSettle();

      // エラーメッセージが表示される
      expect(find.text('氏名は必須です'), findsOneWidget);

      // DataTableは正常に表示されている
      expect(find.text('山田太郎'), findsOneWidget);
    });

    testWidgets('DataTableローディング中にフォームが操作できる', (tester) async {
      await tester.pumpWidget(buildTestApp(
        Column(
          children: [
            const SizedBox(
              height: 200,
              child: K1s0DataTable<Map<String, dynamic>>(
                rows: [],
                columns: [],
                getRowId: (_) => '',
                loading: true,
              ),
            ),
            K1s0Form<Map<String, dynamic>>(
              schema: userSchema,
              onSubmit: (_) async {},
            ),
          ],
        ),
      ));

      // フォームに入力可能
      await tester.enterText(
        find.byType(TextFormField).first,
        'テスト入力',
      );
      await tester.pumpAndSettle();

      expect(find.text('テスト入力'), findsOneWidget);
    });
  });
}
