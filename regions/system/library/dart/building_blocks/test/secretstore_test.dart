import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

void main() {
  group('SecretValue', () {
    test('key・value・メタデータを指定して生成できること', () {
      const secret = SecretValue(
        key: 'db-password',
        value: 's3cret!',
        metadata: {'store': 'vault', 'version': '2'},
      );

      expect(secret.key, 'db-password');
      expect(secret.value, 's3cret!');
      expect(secret.metadata, {'store': 'vault', 'version': '2'});
    });

    test('空のメタデータをサポートすること', () {
      const secret = SecretValue(
        key: 'api-key',
        value: 'abc123',
        metadata: const {},
      );

      expect(secret.metadata, isEmpty);
    });

    test('イミュータブルであること（final フィールド）', () {
      const secret = SecretValue(
        key: 'token',
        value: 'xyz',
        metadata: const {},
      );

      expect(secret.key, 'token');
      expect(secret.value, 'xyz');
    });
  });
}
