import 'package:flutter/material.dart';

import '../../config_types.dart';

class BooleanField extends StatelessWidget {
  const BooleanField({
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
        value is bool ? value as bool : (schema.defaultValue as bool?) ?? false;

    return SwitchListTile(
      title: Text(schema.label),
      subtitle:
          schema.description != null ? Text(schema.description!) : null,
      value: currentValue,
      onChanged: (v) => onChanged(v),
    );
  }
}
