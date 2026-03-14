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
    test('シークレットが見つかった場合に返されること', () async {
      client.putSecret(makeSecret('system/db/primary'));
      final secret = await client.getSecret('system/db/primary');
      expect(secret.path, equals('system/db/primary'));
      expect(secret.data['password'], equals('s3cr3t'));
    });

    test('見つからない場合にVaultErrorがスローされること', () async {
      expect(
        () => client.getSecret('missing/path'),
        throwsA(isA<VaultError>()),
      );
    });
  });

  group('getSecretValue', () {
    test('指定したキーの値が返されること', () async {
      client.putSecret(makeSecret('system/db'));
      final value = await client.getSecretValue('system/db', 'password');
      expect(value, equals('s3cr3t'));
    });

    test('キーが見つからない場合にVaultErrorがスローされること', () async {
      client.putSecret(makeSecret('system/db'));
      expect(
        () => client.getSecretValue('system/db', 'missing_key'),
        throwsA(isA<VaultError>()),
      );
    });
  });

  group('listSecrets', () {
    test('一致するパスが返されること', () async {
      client.putSecret(makeSecret('system/db/primary'));
      client.putSecret(makeSecret('system/db/replica'));
      client.putSecret(makeSecret('business/api/key'));
      final paths = await client.listSecrets('system/');
      expect(paths, hasLength(2));
      expect(paths.every((p) => p.startsWith('system/')), isTrue);
    });

    test('一致するパスがない場合に空リストが返されること', () async {
      final paths = await client.listSecrets('nothing/');
      expect(paths, isEmpty);
    });
  });

  group('watchSecret', () {
    test('ストリームが返されること', () {
      final stream = client.watchSecret('system/db');
      expect(stream, isNotNull);
    });
  });

  group('Secret', () {
    test('全フィールドが保持されること', () {
      final secret = makeSecret('test/path');
      expect(secret.path, equals('test/path'));
      expect(secret.version, equals(1));
      expect(secret.data['username'], equals('admin'));
    });
  });

  group('SecretRotatedEvent', () {
    test('全フィールドが保持されること', () {
      const event = SecretRotatedEvent(path: 'system/db', version: 2);
      expect(event.path, equals('system/db'));
      expect(event.version, equals(2));
    });
  });

  group('VaultClientConfig', () {
    test('デフォルト値が設定されていること', () {
      const config = VaultClientConfig(serverUrl: 'http://vault:8080');
      expect(config.cacheTtl, equals(const Duration(minutes: 10)));
      expect(config.cacheMaxCapacity, equals(500));
    });

    test('カスタム値が受け入れられること', () {
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
    test('コードとメッセージが保持されること', () {
      const err = VaultError(VaultErrorCode.notFound, 'test/path');
      expect(err.code, equals(VaultErrorCode.notFound));
      expect(err.message, equals('test/path'));
    });

    test('toStringにコードが含まれること', () {
      const err = VaultError(VaultErrorCode.permissionDenied, 'secret');
      expect(err.toString(), contains('permissionDenied'));
    });
  });
}
