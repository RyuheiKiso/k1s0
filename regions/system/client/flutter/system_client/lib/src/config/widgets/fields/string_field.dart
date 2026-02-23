import 'package:flutter/material.dart';

import '../../config_types.dart';

class StringField extends StatelessWidget {
  const StringField({
    super.key,
    required this.schema,
    required this.value,
    required this.onChanged,
  });

  final ConfigFieldSchema schema;
  final dynamic value;
  final ValueChanged<dynamic> onChanged;

  @override
  Widget build(BuildContext context) {
    final currentValue =
        value is String ? value as String : (schema.defaultValue as String?) ?? '';

    return TextFormField(
      initialValue: currentValue,
      decoration: InputDecoration(
        labelText: schema.label,
        helperText: schema.description,
        suffixText: schema.unit,
      ),
      validator: (v) {
        if (v == null || v.isEmpty) return null;
        if (schema.pattern != null) {
          final regex = RegExp(schema.pattern!);
          if (!regex.hasMatch(v)) {
            return 'パターン ${schema.pattern} に一致しません';
          }
        }
        return null;
      },
      onChanged: (v) => onChanged(v),
    );
  }
}
