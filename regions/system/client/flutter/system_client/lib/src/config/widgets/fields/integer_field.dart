import 'package:flutter/material.dart';

import '../../config_types.dart';

class IntegerField extends StatelessWidget {
  const IntegerField({
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
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    final currentValue =
        value is int ? value as int : (schema.defaultValue as int?) ?? 0;

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
            errorText: errorText,
          ),
          onChanged: (raw) {
            final parsed = int.tryParse(raw);
            if (parsed != null) {
              onValidationChanged(null);
              onChanged(parsed);
              return;
            }
            onValidationChanged('Enter an integer');
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
            onChanged: (value) => onChanged(value.round()),
          ),
      ],
    );
  }
}
