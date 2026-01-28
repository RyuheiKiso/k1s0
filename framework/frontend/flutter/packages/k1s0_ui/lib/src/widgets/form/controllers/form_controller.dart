/// K1s0 Form コントローラー
library;

import 'package:flutter/foundation.dart';
import '../k1s0_form_schema.dart';

/// フォーム状態を管理するコントローラー
class K1s0FormController<T> extends ChangeNotifier {
  /// フォームスキーマ
  final K1s0FormSchema<T> schema;

  /// 現在の値
  Map<String, dynamic> _values;

  /// エラーメッセージ
  Map<String, String?> _errors;

  /// タッチ状態
  final Set<String> _touched;

  /// 送信中フラグ
  bool _isSubmitting;

  /// コンストラクタ
  K1s0FormController({
    required this.schema,
    T? initialValues,
  })  : _values = initialValues != null
            ? schema.toMap(initialValues)
            : schema.getDefaultValues(),
        _errors = {},
        _touched = {},
        _isSubmitting = false;

  /// 現在の値
  Map<String, dynamic> get values => Map.unmodifiable(_values);

  /// エラーメッセージ
  Map<String, String?> get errors => Map.unmodifiable(_errors);

  /// 送信中かどうか
  bool get isSubmitting => _isSubmitting;

  /// バリデーションエラーがあるかどうか
  bool get hasErrors => _errors.values.any((e) => e != null);

  /// フォームが有効かどうか
  bool get isValid => !hasErrors;

  /// フォームがダーティかどうか
  bool get isDirty => _touched.isNotEmpty;

  /// フィールド値を取得
  dynamic getValue(String name) => _values[name];

  /// フィールド値を設定
  void setValue(String name, dynamic value) {
    _values[name] = value;
    _touched.add(name);

    // 値変更時にバリデーション
    _validateField(name);

    notifyListeners();
  }

  /// 複数の値を一括設定
  void setValues(Map<String, dynamic> values) {
    _values.addAll(values);
    _touched.addAll(values.keys);

    // 各フィールドをバリデーション
    for (final name in values.keys) {
      _validateField(name);
    }

    notifyListeners();
  }

  /// フォームをリセット
  void reset([T? initialValues]) {
    _values = initialValues != null
        ? schema.toMap(initialValues)
        : schema.getDefaultValues();
    _errors = {};
    _touched.clear();
    _isSubmitting = false;
    notifyListeners();
  }

  /// フィールドがタッチされたかどうか
  bool isTouched(String name) => _touched.contains(name);

  /// フィールドのエラーを取得
  String? getError(String name) => _errors[name];

  /// フィールドにエラーがあるかどうか
  bool hasError(String name) => _errors[name] != null;

  /// フィールドをタッチ状態にする
  void touch(String name) {
    _touched.add(name);
    _validateField(name);
    notifyListeners();
  }

  /// 単一フィールドをバリデーション
  void _validateField(String name) {
    final field = schema.getField(name);
    if (field == null) return;

    final value = _values[name];
    String? error;

    // 必須チェック
    if (field.required && (value == null || value.toString().isEmpty)) {
      error = '${field.label}は必須です';
    }
    // カスタムバリデーター
    else if (field.validator != null) {
      error = field.validator!(value);
    }

    _errors[name] = error;
  }

  /// 全フィールドをバリデーション
  bool validate() {
    _errors.clear();

    for (final field in schema.fields) {
      // 条件付きフィールドで非表示の場合はスキップ
      if (field.condition != null && !field.condition!(_values)) {
        continue;
      }

      _validateField(field.name);
      _touched.add(field.name);
    }

    notifyListeners();
    return isValid;
  }

  /// フォームを送信
  Future<void> submit(Future<void> Function(T values) onSubmit) async {
    if (!validate()) return;

    _isSubmitting = true;
    notifyListeners();

    try {
      final data = schema.fromMap(_values);
      await onSubmit(data);
    } finally {
      _isSubmitting = false;
      notifyListeners();
    }
  }

  /// 型付きの値を取得
  T getTypedValues() => schema.fromMap(_values);

  /// フィールドが表示されるかどうか
  bool isFieldVisible(String name) {
    final field = schema.getField(name);
    if (field == null) return false;
    if (field.condition == null) return true;
    return field.condition!(_values);
  }
}
