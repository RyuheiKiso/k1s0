import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/k1s0_ui.dart';

void main() {
  // テスト用のスキーマ
  final testSchema = K1s0FormSchema<Map<String, dynamic>>(
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
        name: 'age',
        label: '年齢',
        type: K1s0FieldType.number,
      ),
      K1s0FormFieldSchema(
        name: 'role',
        label: '権限',
        type: K1s0FieldType.select,
        options: [
          K1s0FieldOption(label: '管理者', value: 'admin'),
          K1s0FieldOption(label: '一般', value: 'user'),
        ],
      ),
      K1s0FormFieldSchema(
        name: 'active',
        label: '有効',
        type: K1s0FieldType.switchField,
        defaultValue: true,
      ),
    ],
    fromMap: (map) => map,
    toMap: (value) => value,
  );

  Widget buildTestWidget({
    K1s0FormSchema<Map<String, dynamic>>? schema,
    Map<String, dynamic>? initialValues,
    Future<void> Function(Map<String, dynamic>)? onSubmit,
    bool loading = false,
    bool disabled = false,
  }) {
    return MaterialApp(
      home: Scaffold(
        body: SingleChildScrollView(
          child: K1s0Form<Map<String, dynamic>>(
            schema: schema ?? testSchema,
            initialValues: initialValues,
            onSubmit: onSubmit ?? (_) async {},
            loading: loading,
            disabled: disabled,
            showCancel: true,
          ),
        ),
      ),
    );
  }

  group('K1s0Form', () {
    testWidgets('フィールドが正しく表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      expect(find.text('氏名'), findsOneWidget);
      expect(find.text('メールアドレス'), findsOneWidget);
      expect(find.text('年齢'), findsOneWidget);
      expect(find.text('権限'), findsOneWidget);
      expect(find.text('有効'), findsOneWidget);
    });

    testWidgets('送信ボタンが表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      expect(find.text('送信'), findsOneWidget);
    });

    testWidgets('キャンセルボタンが表示される', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      expect(find.text('キャンセル'), findsOneWidget);
    });

    testWidgets('初期値が反映される', (tester) async {
      await tester.pumpWidget(buildTestWidget(
        initialValues: {'name': '山田太郎'},
      ));

      expect(find.text('山田太郎'), findsOneWidget);
    });

    testWidgets('必須フィールドのバリデーションエラーが表示される', (tester) async {
      Map<String, dynamic>? submittedValues;

      await tester.pumpWidget(buildTestWidget(
        onSubmit: (values) async {
          submittedValues = values;
        },
      ));

      // 送信ボタンをタップ
      await tester.tap(find.text('送信'));
      await tester.pumpAndSettle();

      // エラーが表示される
      expect(find.text('氏名は必須です'), findsOneWidget);

      // onSubmit は呼ばれない
      expect(submittedValues, isNull);
    });

    testWidgets('ローディング中はボタンが無効化される', (tester) async {
      await tester.pumpWidget(buildTestWidget(loading: true));

      final submitButton = tester.widget<FilledButton>(
        find.byType(FilledButton),
      );
      expect(submitButton.onPressed, isNull);
    });

    testWidgets('disabled=true でフォームが無効化される', (tester) async {
      await tester.pumpWidget(buildTestWidget(disabled: true));

      final submitButton = tester.widget<FilledButton>(
        find.byType(FilledButton),
      );
      expect(submitButton.onPressed, isNull);
    });

    testWidgets('有効なフォームが送信される', (tester) async {
      Map<String, dynamic>? submittedValues;

      await tester.pumpWidget(buildTestWidget(
        onSubmit: (values) async {
          submittedValues = values;
        },
      ));

      // フォームに入力
      await tester.enterText(
        find.byType(TextFormField).first,
        '山田太郎',
      );
      await tester.enterText(
        find.byType(TextFormField).at(1),
        'yamada@example.com',
      );

      // 送信
      await tester.tap(find.text('送信'));
      await tester.pumpAndSettle();

      expect(submittedValues, isNotNull);
      expect(submittedValues!['name'], equals('山田太郎'));
      expect(submittedValues!['email'], equals('yamada@example.com'));
    });
  });

  group('バリデーター', () {
    test('RequiredValidator が正しく動作する', () {
      final validator = RequiredValidator();

      expect(validator.validate(null), isNotNull);
      expect(validator.validate(''), isNotNull);
      expect(validator.validate('test'), isNull);
    });

    test('EmailValidator が正しく動作する', () {
      final validator = EmailValidator();

      expect(validator.validate('invalid'), isNotNull);
      expect(validator.validate('test@example.com'), isNull);
    });

    test('MinLengthValidator が正しく動作する', () {
      final validator = MinLengthValidator(3);

      expect(validator.validate('ab'), isNotNull);
      expect(validator.validate('abc'), isNull);
    });

    test('MaxLengthValidator が正しく動作する', () {
      final validator = MaxLengthValidator(5);

      expect(validator.validate('abcdef'), isNotNull);
      expect(validator.validate('abcde'), isNull);
    });

    test('RangeValidator が正しく動作する', () {
      final validator = RangeValidator(min: 0, max: 100);

      expect(validator.validate('-1'), isNotNull);
      expect(validator.validate('101'), isNotNull);
      expect(validator.validate('50'), isNull);
    });

    test('CompositeValidator が正しく動作する', () {
      final validator = CompositeValidator([
        RequiredValidator(),
        MinLengthValidator(3),
      ]);

      expect(validator.validate(null), isNotNull);
      expect(validator.validate('ab'), isNotNull);
      expect(validator.validate('abc'), isNull);
    });
  });
}
