import 'package:flutter/material.dart';

import '../../config_types.dart';

class FloatField extends StatelessWidget {
  const FloatField({
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
  final ValueChanged<double> onChanged;

  @override
  Widget build(BuildContext context) {
    final currentValue = value is num
        ? (value as num).toDouble()
        : (schema.defaultValue as num?)?.toDouble() ?? 0;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        TextFormField(
          initialValue: currentValue.toString(),
          keyboardType: const TextInputType.numberWithOptions(decimal: true),
          decoration: InputDecoration(
            labelText: schema.label,
            helperText: schema.description,
            suffixText: schema.unit,
            errorText: errorText,
          ),
          onChanged: (raw) {
            final parsed = double.tryParse(raw);
            if (parsed != null) {
              onValidationChanged(null);
              onChanged(parsed);
              return;
            }
            onValidationChanged('Enter a number');
          },
        ),
        if (schema.min != null && schema.max != null)
          Slider(
            value: currentValue.clamp(
              schema.min!.toDouble(),
              schema.max!.toDouble(),
            ),
            min: schema.min!.toDouble(),
            max: schema.max!.toDouble(),
            label: currentValue.toString(),
            onChanged: onChanged,
          ),
      ],
    );
  }
}
