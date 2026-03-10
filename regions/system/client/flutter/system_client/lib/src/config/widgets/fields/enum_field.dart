import 'package:flutter/material.dart';

import '../../config_types.dart';

class EnumField extends StatelessWidget {
  const EnumField({
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
        value is String ? value as String : schema.defaultValue as String?;
    final options = schema.options ?? [];

    return DropdownButtonFormField<String>(
      initialValue: options.contains(currentValue) ? currentValue : null,
      decoration: InputDecoration(
        labelText: schema.label,
        helperText: schema.description,
        errorText: errorText,
      ),
      items: options
          .map((option) => DropdownMenuItem(value: option, child: Text(option)))
          .toList(),
      onChanged: (value) {
        if (value != null) {
          onChanged(value);
        }
      },
    );
  }
}
