import 'package:flutter/material.dart';

import '../../config_types.dart';

class EnumField extends StatelessWidget {
  const EnumField({
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
        value is String ? value as String : schema.defaultValue as String?;
    final options = schema.options ?? [];

    return DropdownButtonFormField<String>(
      initialValue: options.contains(currentValue) ? currentValue : null,
      decoration: InputDecoration(
        labelText: schema.label,
        helperText: schema.description,
      ),
      items: options
          .map((o) => DropdownMenuItem(value: o, child: Text(o)))
          .toList(),
      onChanged: (v) {
        if (v != null) onChanged(v);
      },
    );
  }
}
