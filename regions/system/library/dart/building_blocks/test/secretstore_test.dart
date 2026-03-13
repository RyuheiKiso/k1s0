import 'package:test/test.dart';
import 'package:building_blocks/building_blocks.dart';

void main() {
  group('SecretValue', () => {
    test('should create with key, value, and metadata', () {
      final secret = SecretValue(
        key: 'db-password',
        value: 's3cret!',
        metadata: {'store': 'vault', 'version': '2'},
      );

      expect(secret.key, 'db-password');
      expect(secret.value, 's3cret!');
      expect(secret.metadata, {'store': 'vault', 'version': '2'});
    });

    test('should support empty metadata', () {
      final secret = SecretValue(
        key: 'api-key',
        value: 'abc123',
        metadata: const {},
      );

      expect(secret.metadata, isEmpty);
    });

    test('should be immutable (final fields)', () {
      final secret = SecretValue(
        key: 'token',
        value: 'xyz',
        metadata: const {},
      );

      expect(secret.key, 'token');
      expect(secret.value, 'xyz');
    });
  });
}
