import 'error.dart';

class SchemaRegistryConfig {
  final String url;
  final String? username;
  final String? password;

  const SchemaRegistryConfig({
    required this.url,
    this.username,
    this.password,
  });

  /// Confluent naming convention: <topic>-value or <topic>-key
  static String subjectName(String topic, String keyOrValue) =>
      '$topic-$keyOrValue';

  void validate() {
    if (url.isEmpty) {
      throw SchemaRegistryError(0, 'schema registry URL must not be empty');
    }
  }
}
