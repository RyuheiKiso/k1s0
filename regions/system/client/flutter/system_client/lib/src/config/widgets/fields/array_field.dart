import 'package:flutter/material.dart';

import '../../config_types.dart';

class ArrayField extends StatelessWidget {
  const ArrayField({
    super.key,
    required this.schema,
    required this.value,
    required this.errorText,
    required this.onChanged,
  });

  final ConfigFieldSchema schema;
  final dynamic value;
  final String? errorText;
  final ValueChanged<List<String>> onChanged;

  @override
  Widget build(BuildContext context) {
    final List<String> currentValue = value is List
        ? (value as List<dynamic>).map((item) => item.toString()).toList()
        : (schema.defaultValue as List<dynamic>? ?? const [])
            .map((item) => item.toString())
            .toList();

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
}
