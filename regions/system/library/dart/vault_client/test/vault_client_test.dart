import 'package:test/test.dart';

import 'package:k1s0_vault_client/vault_client.dart';

VaultClientConfig makeConfig() {
  return const VaultClientConfig(serverUrl: 'http://localhost:8080');
}

Secret makeSecret(String path) {
  return Secret(
    path: path,
    data: const {'password': 's3cr3t', 'username': 'admin'},
    version: 1,
    createdAt: DateTime.now(),
  );
}

void main() {
  late InMemoryVaultClient client;

  setUp(() {
    client = InMemoryVaultClient(makeConfig());
  });

  group('getSecret', () {
    test('returns secret when found', () async {
      client.putSecret(makeSecret('system/db/primary'));
      final secret = await client.getSecret('system/db/primary');
      expect(secret.path, equals('system/db/primary'));
      expect(secret.data['password'], equals('s3cr3t'));
    });

    test('throws VaultError when not found', () async {
      expect(
        () => client.getSecret('missing/path'),
        throwsA(isA<VaultError>()),
      );
    });
  });

  group('getSecretValue', () {
    test('returns value for key', () async {
      client.putSecret(makeSecret('system/db'));
      final value = await client.getSecretValue('system/db', 'password');
      expect(value, equals('s3cr3t'));
    });

    test('throws VaultError when key not found', () async {
      client.putSecret(makeSecret('system/db'));
      expect(
        () => client.getSecretValue('system/db', 'missing_key'),
        throwsA(isA<VaultError>()),
      );
    });
  });

  group('listSecrets', () {
    test('returns matching paths', () async {
      client.putSecret(makeSecret('system/db/primary'));
      client.putSecret(makeSecret('system/db/replica'));
      client.putSecret(makeSecret('business/api/key'));
      final paths = await client.listSecrets('system/');
      expect(paths, hasLength(2));
      expect(paths.every((p) => p.startsWith('system/')), isTrue);
    });

    test('returns empty list when no match', () async {
      final paths = await client.listSecrets('nothing/');
      expect(paths, isEmpty);
    });
  });

  group('watchSecret', () {
    test('returns a stream', () {
      final stream = client.watchSecret('system/db');
      expect(stream, isNotNull);
    });
  });

  group('Secret', () {
    test('stores all fields', () {
      final secret = makeSecret('test/path');
      expect(secret.path, equals('test/path'));
      expect(secret.version, equals(1));
      expect(secret.data['username'], equals('admin'));
    });
  });

  group('SecretRotatedEvent', () {
    test('stores all fields', () {
      const event = SecretRotatedEvent(path: 'system/db', version: 2);
      expect(event.path, equals('system/db'));
      expect(event.version, equals(2));
    });
  });

  group('VaultClientConfig', () {
    test('has default values', () {
      const config = VaultClientConfig(serverUrl: 'http://vault:8080');
      expect(config.cacheTtl, equals(const Duration(minutes: 10)));
      expect(config.cacheMaxCapacity, equals(500));
    });

    test('accepts custom values', () {
      const config = VaultClientConfig(
        serverUrl: 'http://vault:8080',
        cacheTtl: Duration(minutes: 5),
        cacheMaxCapacity: 100,
      );
      expect(config.cacheTtl, equals(const Duration(minutes: 5)));
      expect(config.cacheMaxCapacity, equals(100));
    });
  });

  group('VaultError', () {
    test('has code and message', () {
      const err = VaultError(VaultErrorCode.notFound, 'test/path');
      expect(err.code, equals(VaultErrorCode.notFound));
      expect(err.message, equals('test/path'));
    });

    test('toString includes code', () {
      const err = VaultError(VaultErrorCode.permissionDenied, 'secret');
      expect(err.toString(), contains('permissionDenied'));
    });
  });
}
