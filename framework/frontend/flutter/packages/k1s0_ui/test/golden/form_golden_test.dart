import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/k1s0_ui.dart';

/// Form ゴールデンテスト
///
/// ゴールデンファイルの更新:
/// ```bash
/// flutter test --update-goldens test/golden/form_golden_test.dart
/// ```
void main() {
  // テスト用スキーマ
  final basicSchema = K1s0FormSchema<Map<String, dynamic>>(
    fields: [
      K1s0FormFieldSchema(
        name: 'name',
        label: '氏名',
        required: true,
        placeholder: '山田太郎',
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
      K1s0FormFieldSchema(
        name: 'notifications',
        label: '通知を受け取る',
        type: K1s0FieldType.switchField,
        defaultValue: true,
      ),
    ],
    fromMap: (map) => map,
    toMap: (value) => value,
  );

  final allFieldsSchema = K1s0FormSchema<Map<String, dynamic>>(
    fields: [
      K1s0FormFieldSchema(
        name: 'text',
        label: 'テキスト',
        required: true,
      ),
      K1s0FormFieldSchema(
        name: 'email',
        label: 'メール',
        type: K1s0FieldType.email,
      ),
      K1s0FormFieldSchema(
        name: 'password',
        label: 'パスワード',
        type: K1s0FieldType.password,
      ),
      K1s0FormFieldSchema(
        name: 'number',
        label: '数値',
        type: K1s0FieldType.number,
      ),
      K1s0FormFieldSchema(
        name: 'textarea',
        label: 'テキストエリア',
        type: K1s0FieldType.textarea,
        rows: 3,
      ),
      K1s0FormFieldSchema(
        name: 'select',
        label: 'セレクト',
        type: K1s0FieldType.select,
        options: [
          K1s0FieldOption(label: 'オプション1', value: '1'),
          K1s0FieldOption(label: 'オプション2', value: '2'),
        ],
      ),
      K1s0FormFieldSchema(
        name: 'radio',
        label: 'ラジオ',
        type: K1s0FieldType.radio,
        options: [
          K1s0FieldOption(label: '選択肢A', value: 'a'),
          K1s0FieldOption(label: '選択肢B', value: 'b'),
        ],
      ),
      K1s0FormFieldSchema(
        name: 'checkbox',
        label: '同意する',
        type: K1s0FieldType.checkbox,
      ),
      K1s0FormFieldSchema(
        name: 'switch',
        label: '有効',
        type: K1s0FieldType.switchField,
      ),
      K1s0FormFieldSchema(
        name: 'slider',
        label: 'スライダー',
        type: K1s0FieldType.slider,
        min: 0,
        max: 100,
      ),
    ],
    fromMap: (map) => map,
    toMap: (value) => value,
  );

  Widget buildTestWidget(Widget child) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        colorSchemeSeed: Colors.blue,
      ),
      home: Scaffold(
        body: RepaintBoundary(
          child: SingleChildScrollView(
            child: SizedBox(
              width: 600,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: child,
              ),
            ),
          ),
        ),
      ),
    );
  }

  group('Form ゴールデンテスト', () {
    testWidgets('基本フォーム', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0Form<Map<String, dynamic>>(
          schema: basicSchema,
          onSubmit: (_) async {},
          submitLabel: '送信',
          showCancel: true,
        ),
      ));

      await tester.pumpAndSettle();

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/form_basic.png'),
      );
    });

    testWidgets('全フィールドタイプ', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0Form<Map<String, dynamic>>(
          schema: allFieldsSchema,
          onSubmit: (_) async {},
        ),
      ));

      await tester.pumpAndSettle();

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/form_all_fields.png'),
      );
    });

    testWidgets('入力済みフォーム', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0Form<Map<String, dynamic>>(
          schema: basicSchema,
          initialValues: const {
            'name': '山田太郎',
            'email': 'yamada@example.com',
            'role': 'admin',
            'notifications': true,
          },
          onSubmit: (_) async {},
          submitLabel: '更新',
        ),
      ));

      await tester.pumpAndSettle();

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/form_filled.png'),
      );
    });

    testWidgets('バリデーションエラー', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0Form<Map<String, dynamic>>(
          schema: basicSchema,
          onSubmit: (_) async {},
        ),
      ));

      // 送信ボタンをタップしてエラーを表示
      await tester.tap(find.text('送信'));
      await tester.pumpAndSettle();

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/form_validation_error.png'),
      );
    });

    testWidgets('ローディング状態', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0Form<Map<String, dynamic>>(
          schema: basicSchema,
          onSubmit: (_) async {},
          loading: true,
        ),
      ));

      await tester.pumpAndSettle();

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/form_loading.png'),
      );
    });

    testWidgets('無効状態', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        K1s0Form<Map<String, dynamic>>(
          schema: basicSchema,
          initialValues: const {
            'name': '山田太郎',
            'email': 'yamada@example.com',
            'role': 'user',
            'notifications': false,
          },
          onSubmit: (_) async {},
          disabled: true,
        ),
      ));

      await tester.pumpAndSettle();

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/form_disabled.png'),
      );
    });

    testWidgets('2列レイアウト', (tester) async {
      final gridSchema = K1s0FormSchema<Map<String, dynamic>>(
        fields: [
          K1s0FormFieldSchema(name: 'firstName', label: '姓'),
          K1s0FormFieldSchema(name: 'lastName', label: '名'),
          K1s0FormFieldSchema(name: 'email', label: 'メール', type: K1s0FieldType.email),
          K1s0FormFieldSchema(name: 'phone', label: '電話番号'),
        ],
        fromMap: (map) => map,
        toMap: (value) => value,
      );

      await tester.pumpWidget(buildTestWidget(
        K1s0Form<Map<String, dynamic>>(
          schema: gridSchema,
          onSubmit: (_) async {},
          columns: 2,
        ),
      ));

      await tester.pumpAndSettle();

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/form_grid.png'),
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
              child: SingleChildScrollView(
                child: SizedBox(
                  width: 600,
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: K1s0Form<Map<String, dynamic>>(
                      schema: basicSchema,
                      initialValues: const {
                        'name': '山田太郎',
                        'email': 'yamada@example.com',
                        'role': 'admin',
                        'notifications': true,
                      },
                      onSubmit: (_) async {},
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      await expectLater(
        find.byType(RepaintBoundary),
        matchesGoldenFile('goldens/form_dark.png'),
      );
    });
  });
}
