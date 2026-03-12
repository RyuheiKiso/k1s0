import 'component.dart';

class SecretValue {
  final String key;
  final String value;
  final Map<String, String> metadata;

  const SecretValue({
    required this.key,
    required this.value,
    required this.metadata,
  });
}

abstract class SecretStore implements Component {
  Future<SecretValue> getSecret(String key);
  Future<Map<String, SecretValue>> bulkGet(List<String> keys);
}
