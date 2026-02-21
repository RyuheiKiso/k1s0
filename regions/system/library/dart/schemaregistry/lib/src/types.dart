/// スキーマ形式。
enum SchemaType {
  avro,
  json,
  protobuf;

  String toJson() => name.toUpperCase();

  static SchemaType fromString(String value) {
    return switch (value.toUpperCase()) {
      'AVRO' => SchemaType.avro,
      'JSON' => SchemaType.json,
      'PROTOBUF' => SchemaType.protobuf,
      _ => throw ArgumentError('Unknown SchemaType: $value'),
    };
  }
}

class RegisteredSchema {
  final int id;
  final String subject;
  final int version;
  final String schema;
  final String schemaType;

  const RegisteredSchema({
    required this.id,
    required this.subject,
    required this.version,
    required this.schema,
    required this.schemaType,
  });

  factory RegisteredSchema.fromJson(Map<String, dynamic> json) =>
      RegisteredSchema(
        id: json['id'] as int? ?? 0,
        subject: json['subject'] as String? ?? '',
        version: json['version'] as int? ?? 0,
        schema: json['schema'] as String? ?? '',
        schemaType: json['schemaType'] as String? ?? '',
      );
}
