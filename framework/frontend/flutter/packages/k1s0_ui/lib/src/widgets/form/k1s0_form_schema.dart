/// K1s0 Form スキーマ定義
library;

import 'k1s0_field_type.dart';

/// フィールドオプション
class K1s0FieldOption {
  /// 表示ラベル
  final String label;

  /// 値
  final dynamic value;

  /// 無効化
  final bool disabled;

  const K1s0FieldOption({
    required this.label,
    required this.value,
    this.disabled = false,
  });
}

/// フォームフィールドスキーマ
class K1s0FormFieldSchema {
  /// フィールド名
  final String name;

  /// ラベル
  final String label;

  /// プレースホルダー
  final String? placeholder;

  /// ヘルプテキスト
  final String? helperText;

  /// 必須
  final bool required;

  /// フィールドタイプ
  final K1s0FieldType type;

  /// 選択肢（select/radio 用）
  final List<K1s0FieldOption>? options;

  /// デフォルト値
  final dynamic defaultValue;

  /// バリデーター
  final String? Function(dynamic value)? validator;

  /// 条件付き表示
  final bool Function(Map<String, dynamic> values)? condition;

  /// グリッドカラム数
  final int? gridColumn;

  /// 読み取り専用
  final bool readOnly;

  /// 無効化
  final bool disabled;

  /// 最小値（number/slider 用）
  final num? min;

  /// 最大値（number/slider 用）
  final num? max;

  /// ステップ（number/slider 用）
  final num? step;

  /// 最小文字数
  final int? minLength;

  /// 最大文字数
  final int? maxLength;

  /// 行数（textarea 用）
  final int? rows;

  /// 最大行数（textarea 用）
  final int? maxRows;

  const K1s0FormFieldSchema({
    required this.name,
    required this.label,
    this.placeholder,
    this.helperText,
    this.required = false,
    this.type = K1s0FieldType.text,
    this.options,
    this.defaultValue,
    this.validator,
    this.condition,
    this.gridColumn,
    this.readOnly = false,
    this.disabled = false,
    this.min,
    this.max,
    this.step,
    this.minLength,
    this.maxLength,
    this.rows,
    this.maxRows,
  });
}

/// フォームスキーマ
class K1s0FormSchema<T> {
  /// フィールド定義
  final List<K1s0FormFieldSchema> fields;

  /// Map から T への変換
  final T Function(Map<String, dynamic> values) fromMap;

  /// T から Map への変換
  final Map<String, dynamic> Function(T value) toMap;

  const K1s0FormSchema({
    required this.fields,
    required this.fromMap,
    required this.toMap,
  });

  /// フィールドを名前で取得
  K1s0FormFieldSchema? getField(String name) {
    try {
      return fields.firstWhere((field) => field.name == name);
    } catch (_) {
      return null;
    }
  }

  /// デフォルト値の Map を取得
  Map<String, dynamic> getDefaultValues() {
    final defaults = <String, dynamic>{};
    for (final field in fields) {
      if (field.defaultValue != null) {
        defaults[field.name] = field.defaultValue;
      }
    }
    return defaults;
  }
}
