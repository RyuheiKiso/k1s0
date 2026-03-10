import 'package:flutter/material.dart';

import '../../config_types.dart';

class StringField extends StatelessWidget {
  const StringField({
    super.key,
    required this.schema,
    required this.value,
    required this.errorText,
    required this.onChanged,
  });

  final ConfigFieldSchema schema;
  final dynamic value;
  final String? errorText;
  final ValueChanged<String> onChanged;

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
        errorText: errorText,
      ),
      onChanged: onChanged,
    );
  }
}
