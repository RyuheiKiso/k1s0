/// K1s0 Form メインウィジェット
library;

import 'package:flutter/material.dart';
import 'k1s0_form_schema.dart';
import 'k1s0_field_type.dart';
import 'controllers/form_controller.dart';
import 'components/form_container.dart';
import 'components/form_grid.dart';
import 'components/form_actions.dart';
import 'fields/text_form_field.dart';
import 'fields/dropdown_form_field.dart';
import 'fields/radio_form_field.dart';
import 'fields/checkbox_form_field.dart';
import 'fields/switch_form_field.dart';
import 'fields/date_form_field.dart';
import 'fields/slider_form_field.dart';

/// フォームレイアウト
enum K1s0FormLayout {
  /// 縦並び
  vertical,

  /// 横並び
  horizontal,
}

/// K1s0 Form ウィジェット
///
/// スキーマ駆動のフォームウィジェット。
///
/// 例:
/// ```dart
/// final schema = K1s0FormSchema<UserInput>(
///   fields: [
///     K1s0FormFieldSchema(name: 'name', label: '氏名', required: true),
///     K1s0FormFieldSchema(name: 'email', label: 'メール', type: K1s0FieldType.email),
///   ],
///   fromMap: (map) => UserInput.fromMap(map),
///   toMap: (user) => user.toMap(),
/// );
///
/// K1s0Form<UserInput>(
///   schema: schema,
///   onSubmit: (values) => saveUser(values),
///   submitLabel: '保存',
/// )
/// ```
class K1s0Form<T> extends StatefulWidget {
  /// フォームスキーマ
  final K1s0FormSchema<T> schema;

  /// 初期値
  final T? initialValues;

  /// 送信時コールバック
  final Future<void> Function(T values) onSubmit;

  /// キャンセル時コールバック
  final VoidCallback? onCancel;

  /// 値変更時コールバック
  final void Function(Map<String, dynamic> values)? onChange;

  /// 無効化
  final bool disabled;

  /// ローディング状態
  final bool loading;

  /// 読み取り専用
  final bool readOnly;

  /// 送信ボタンラベル
  final String submitLabel;

  /// キャンセルボタンラベル
  final String? cancelLabel;

  /// キャンセルボタン表示
  final bool showCancel;

  /// リセットボタン表示
  final bool showReset;

  /// レイアウト
  final K1s0FormLayout layout;

  /// カラム数
  final int columns;

  /// 間隔
  final double spacing;

  /// パディング
  final EdgeInsets? padding;

  const K1s0Form({
    super.key,
    required this.schema,
    this.initialValues,
    required this.onSubmit,
    this.onCancel,
    this.onChange,
    this.disabled = false,
    this.loading = false,
    this.readOnly = false,
    this.submitLabel = '送信',
    this.cancelLabel,
    this.showCancel = false,
    this.showReset = false,
    this.layout = K1s0FormLayout.vertical,
    this.columns = 1,
    this.spacing = 16,
    this.padding,
  });

  @override
  State<K1s0Form<T>> createState() => _K1s0FormState<T>();
}

class _K1s0FormState<T> extends State<K1s0Form<T>> {
  late K1s0FormController<T> _controller;
  final _formKey = GlobalKey<FormState>();

  @override
  void initState() {
    super.initState();
    _controller = K1s0FormController<T>(
      schema: widget.schema,
      initialValues: widget.initialValues,
    );
    _controller.addListener(_onControllerChange);
  }

  @override
  void dispose() {
    _controller.removeListener(_onControllerChange);
    _controller.dispose();
    super.dispose();
  }

  void _onControllerChange() {
    widget.onChange?.call(_controller.values);
    setState(() {});
  }

  Future<void> _handleSubmit() async {
    if (!_controller.validate()) return;
    if (_formKey.currentState?.validate() ?? false) {
      await _controller.submit(widget.onSubmit);
    }
  }

  void _handleReset() {
    _controller.reset(widget.initialValues);
    _formKey.currentState?.reset();
  }

  @override
  Widget build(BuildContext context) {
    final visibleFields = widget.schema.fields
        .where((field) => _controller.isFieldVisible(field.name))
        .toList();

    final fieldWidgets = visibleFields.map((field) {
      return _buildField(field);
    }).toList();

    return K1s0FormContainer(
      formKey: _formKey,
      padding: widget.padding,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (widget.columns > 1)
            K1s0FormGrid(
              columns: widget.columns,
              horizontalSpacing: widget.spacing,
              children: fieldWidgets,
            )
          else
            ...fieldWidgets,
          K1s0FormActions(
            submitLabel: widget.submitLabel,
            cancelLabel: widget.cancelLabel,
            showCancel: widget.showCancel,
            showReset: widget.showReset,
            onSubmit: _handleSubmit,
            onCancel: widget.onCancel,
            onReset: _handleReset,
            loading: widget.loading || _controller.isSubmitting,
            disabled: widget.disabled,
          ),
        ],
      ),
    );
  }

  Widget _buildField(K1s0FormFieldSchema field) {
    final value = _controller.getValue(field.name);
    final error = _controller.getError(field.name);
    final isDisabled = widget.disabled || widget.loading || field.disabled;
    final isReadOnly = widget.readOnly || field.readOnly;

    switch (field.type) {
      case K1s0FieldType.text:
      case K1s0FieldType.email:
      case K1s0FieldType.password:
        return K1s0TextFormField(
          value: value?.toString(),
          onChanged: (v) => _controller.setValue(field.name, v),
          label: field.label,
          placeholder: field.placeholder,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
          readOnly: isReadOnly,
          keyboardType: field.type == K1s0FieldType.email
              ? TextInputType.emailAddress
              : null,
          obscureText: field.type == K1s0FieldType.password,
          maxLength: field.maxLength,
        );

      case K1s0FieldType.number:
        return K1s0TextFormField(
          value: value?.toString(),
          onChanged: (v) {
            final num = double.tryParse(v);
            _controller.setValue(field.name, num);
          },
          label: field.label,
          placeholder: field.placeholder,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
          readOnly: isReadOnly,
          keyboardType: TextInputType.number,
        );

      case K1s0FieldType.textarea:
        return K1s0TextFormField(
          value: value?.toString(),
          onChanged: (v) => _controller.setValue(field.name, v),
          label: field.label,
          placeholder: field.placeholder,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
          readOnly: isReadOnly,
          multiline: true,
          maxLines: field.maxRows ?? 5,
          minLines: field.rows ?? 3,
          maxLength: field.maxLength,
        );

      case K1s0FieldType.select:
        return K1s0DropdownFormField(
          value: value,
          onChanged: (v) => _controller.setValue(field.name, v),
          options: field.options ?? [],
          label: field.label,
          placeholder: field.placeholder,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
        );

      case K1s0FieldType.radio:
        return K1s0RadioFormField(
          value: value,
          onChanged: (v) => _controller.setValue(field.name, v),
          options: field.options ?? [],
          label: field.label,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
        );

      case K1s0FieldType.checkbox:
        return K1s0CheckboxFormField(
          value: value == true,
          onChanged: (v) => _controller.setValue(field.name, v),
          label: field.label,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
        );

      case K1s0FieldType.switchField:
        return K1s0SwitchFormField(
          value: value == true,
          onChanged: (v) => _controller.setValue(field.name, v),
          label: field.label,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
        );

      case K1s0FieldType.date:
      case K1s0FieldType.dateTime:
        return K1s0DateFormField(
          value: value is DateTime ? value : null,
          onChanged: (v) => _controller.setValue(field.name, v),
          label: field.label,
          placeholder: field.placeholder,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
          readOnly: isReadOnly,
        );

      case K1s0FieldType.time:
        return K1s0DateFormField(
          value: value is DateTime ? value : null,
          onChanged: (v) => _controller.setValue(field.name, v),
          label: field.label,
          placeholder: field.placeholder,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
          readOnly: isReadOnly,
        );

      case K1s0FieldType.slider:
        return K1s0SliderFormField(
          value: (value is num ? value : 0).toDouble(),
          onChanged: (v) => _controller.setValue(field.name, v),
          label: field.label,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
          min: (field.min ?? 0).toDouble(),
          max: (field.max ?? 100).toDouble(),
        );

      case K1s0FieldType.rating:
        return K1s0SliderFormField(
          value: (value is num ? value : 0).toDouble(),
          onChanged: (v) => _controller.setValue(field.name, v),
          label: field.label,
          helperText: field.helperText,
          errorText: error,
          required: field.required,
          disabled: isDisabled,
          min: 0,
          max: 5,
          divisions: 5,
        );
    }
  }
}
