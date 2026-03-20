import 'package:flutter/material.dart';

import '../../config_types.dart';

/// 配列型設定フィールドの入力ウィジェット
/// カンマ区切りの文字列で入力し、リストとして返す
class ArrayField extends StatelessWidget {
  const ArrayField({
    super.key,
    required this.schema,
    required this.value,
    required this.errorText,
    required this.onChanged,
  });

  final ConfigFieldSchema schema;
  /// 現在の設定値（ConfigValue sealed class で型安全に受け取る）
  final ConfigValue value;
  final String? errorText;
  final ValueChanged<List<String>> onChanged;

  @override
  Widget build(BuildContext context) {
    /// ConfigValue からリスト値を取り出す（型不一致時はデフォルト値にフォールバック）
    final List<String> currentValue = value is ListConfigValue
        ? (value as ListConfigValue)
            .values
            .map((item) =>
                item is StringConfigValue ? item.value : item.toJson().toString())
            .toList()
        : _defaultList();

    return TextFormField(
      initialValue: currentValue.join(', '),
      decoration: InputDecoration(
        labelText: schema.label,
        helperText: schema.description,
        errorText: errorText,
      ),
      onChanged: (raw) {
        onChanged(
          raw
              .split(',')
              .map((item) => item.trim())
              .where((item) => item.isNotEmpty)
              .toList(),
        );
      },
    );
  }

  /// スキーマのデフォルト値からリストを取得する（存在しない場合は空リスト）
  List<String> _defaultList() {
    final def = schema.defaultValue;
    if (def is ListConfigValue) {
      return def.values
          .map((item) =>
              item is StringConfigValue ? item.value : item.toJson().toString())
          .toList();
    }
    return const [];
  }
}
