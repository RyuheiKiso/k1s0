import 'package:flutter/material.dart';

import '../config_types.dart';
import 'fields/boolean_field.dart';
import 'fields/enum_field.dart';
import 'fields/integer_field.dart';
import 'fields/string_field.dart';

class ConfigFieldList extends StatelessWidget {
  const ConfigFieldList({
    super.key,
    required this.fields,
    required this.values,
    required this.onFieldChanged,
  });

  final List<ConfigFieldSchema> fields;
  final Map<String, dynamic> values;
  final void Function(String key, dynamic value) onFieldChanged;

  @override
  Widget build(BuildContext context) {
    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: fields.length,
      separatorBuilder: (_, __) => const SizedBox(height: 16),
      itemBuilder: (context, index) {
        final field = fields[index];
        final value = values[field.key];

        return switch (field.type) {
          ConfigFieldType.integer ||
          ConfigFieldType.float =>
            IntegerField(
              schema: field,
              value: value,
              onChanged: (v) => onFieldChanged(field.key, v),
            ),
          ConfigFieldType.boolean => BooleanField(
              schema: field,
              value: value,
              onChanged: (v) => onFieldChanged(field.key, v),
            ),
          ConfigFieldType.enumType => EnumField(
              schema: field,
              value: value,
              onChanged: (v) => onFieldChanged(field.key, v),
            ),
          _ => StringField(
              schema: field,
              value: value,
              onChanged: (v) => onFieldChanged(field.key, v),
            ),
        };
      },
    );
  }
}
