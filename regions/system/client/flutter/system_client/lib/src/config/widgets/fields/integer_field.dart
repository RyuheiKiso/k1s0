import 'package:flutter/material.dart';

import '../../config_types.dart';

class IntegerField extends StatelessWidget {
  const IntegerField({
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
        value is num ? value as num : (schema.defaultValue as num?) ?? 0;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        TextFormField(
          initialValue: currentValue.toString(),
          keyboardType: TextInputType.number,
          decoration: InputDecoration(
            labelText: schema.label,
            helperText: schema.description,
            suffixText: schema.unit,
          ),
          validator: (v) {
            if (v == null || v.isEmpty) return null;
            final n = int.tryParse(v);
            if (n == null) return '整数を入力してください';
            if (schema.min != null && n < schema.min!) {
              return '${schema.min} 以上の値を入力してください';
            }
            if (schema.max != null && n > schema.max!) {
              return '${schema.max} 以下の値を入力してください';
            }
            return null;
          },
          onChanged: (v) {
            final n = int.tryParse(v);
            if (n != null) onChanged(n);
          },
        ),
        if (schema.min != null && schema.max != null)
          Slider(
            value: currentValue.toDouble().clamp(
                  schema.min!.toDouble(),
                  schema.max!.toDouble(),
                ),
            min: schema.min!.toDouble(),
            max: schema.max!.toDouble(),
            divisions: (schema.max! - schema.min!).toInt(),
            label: currentValue.toString(),
            onChanged: (v) => onChanged(v.round()),
          ),
      ],
    );
  }
}
