import 'dart:convert';

import 'package:flutter/material.dart';

import '../../config_types.dart';

class ObjectField extends StatelessWidget {
  const ObjectField({
    super.key,
    required this.schema,
    required this.value,
    required this.errorText,
    required this.onValidationChanged,
    required this.onChanged,
  });

  final ConfigFieldSchema schema;
  final dynamic value;
  final String? errorText;
  final ValueChanged<String?> onValidationChanged;
  final ValueChanged<Map<String, dynamic>> onChanged;

  @override
  Widget build(BuildContext context) {
    final currentValue = value is Map<String, dynamic>
        ? value as Map<String, dynamic>
        : (schema.defaultValue as Map<String, dynamic>?) ?? const {};

    return TextFormField(
      initialValue: const JsonEncoder.withIndent('  ').convert(currentValue),
      maxLines: 6,
      decoration: InputDecoration(
        labelText: schema.label,
        helperText: schema.description,
        errorText: errorText,
      ),
      onChanged: (raw) {
        try {
          final decoded = jsonDecode(raw);
          if (decoded is Map<String, dynamic>) {
            onValidationChanged(null);
            onChanged(decoded);
          } else {
            onValidationChanged('JSON object is required');
          }
        } catch (_) {
          onValidationChanged('Invalid JSON');
        }
      },
    );
  }
}
