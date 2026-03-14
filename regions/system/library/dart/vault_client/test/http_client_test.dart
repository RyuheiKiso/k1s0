import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:test/test.dart';

import 'package:k1s0_vault_client/vault_client.dart';

void main() {
  group('HttpVaultClient', () {
    test('getSecretが成功してシークレットを返すこと', () async {
      final mockClient = MockClient((request) async {
        expect(request.url.path, equals('/api/v1/secrets/system/db'));
        return http.Response(
          json.encode({
            'path': 'system/db',
            'data': {'password': 's3cr3t'},
            'version': 1,
            'created_at': DateTime.now().toIso8601String(),
          }),
          200,
          headers: {'content-type': 'application/json'},
        );
      });

      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      final secret = await client.getSecret('system/db');
      expect(secret.path, equals('system/db'));
      expect(secret.data['password'], equals('s3cr3t'));
      expect(secret.version, equals(1));
    });

    test('getSecretで404が返された場合にnotFoundエラーがスローされること', () async {
      final mockClient =
          MockClient((_) async => http.Response('Not Found', 404));
      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      expect(
        () => client.getSecret('missing'),
        throwsA(
          isA<VaultError>()
              .having((e) => e.code, 'code', VaultErrorCode.notFound),
        ),
      );
    });

    test('getSecretで401が返された場合にpermissionDeniedエラーがスローされること', () async {
      final mockClient =
          MockClient((_) async => http.Response('Unauthorized', 401));
      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      expect(
        () => client.getSecret('restricted'),
        throwsA(
          isA<VaultError>()
              .having((e) => e.code, 'code', VaultErrorCode.permissionDenied),
        ),
      );
    });

    test('getSecretで500が返された場合にserverErrorがスローされること', () async {
      final mockClient =
          MockClient((_) async => http.Response('Internal Error', 500));
      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      expect(
        () => client.getSecret('broken'),
        throwsA(
          isA<VaultError>()
              .having((e) => e.code, 'code', VaultErrorCode.serverError),
        ),
      );
    });

    test('getSecretValueで指定したキーの値が返されること', () async {
      final mockClient = MockClient((_) async {
        return http.Response(
          json.encode({
            'path': 'system/db',
            'data': {'password': 's3cr3t', 'host': 'localhost'},
            'version': 1,
            'created_at': DateTime.now().toIso8601String(),
          }),
          200,
          headers: {'content-type': 'application/json'},
        );
      });

      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      final value = await client.getSecretValue('system/db', 'password');
      expect(value, equals('s3cr3t'));
    });

    test('getSecretValueで存在しないキーを指定した場合にnotFoundエラーがスローされること', () async {
      final mockClient = MockClient((_) async {
        return http.Response(
          json.encode({
            'path': 'system/db',
            'data': {'password': 's3cr3t'},
            'version': 1,
            'created_at': DateTime.now().toIso8601String(),
          }),
          200,
          headers: {'content-type': 'application/json'},
        );
      });

      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      expect(
        () => client.getSecretValue('system/db', 'missing_key'),
        throwsA(isA<VaultError>()),
      );
    });

    test('listSecretsが成功してパス一覧を返すこと', () async {
      final mockClient = MockClient((request) async {
        expect(request.url.queryParameters['prefix'], equals('system/'));
        return http.Response(
          json.encode(['system/db', 'system/api']),
          200,
          headers: {'content-type': 'application/json'},
        );
      });

      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      final paths = await client.listSecrets('system/');
      expect(paths, hasLength(2));
      expect(paths, contains('system/db'));
      expect(paths, contains('system/api'));
    });

    test('listSecretsが失敗した場合にserverErrorがスローされること', () async {
      final mockClient =
          MockClient((_) async => http.Response('Error', 500));
      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      expect(
        () => client.listSecrets('system/'),
        throwsA(
          isA<VaultError>()
              .having((e) => e.code, 'code', VaultErrorCode.serverError),
        ),
      );
    });

    test('getSecretのキャッシュヒット時に2回目のHTTPリクエストが発生しないこと', () async {
      int callCount = 0;
      final mockClient = MockClient((request) async {
        callCount++;
        return http.Response(
          json.encode({
            'path': 'system/db',
            'data': {'key': 'val'},
            'version': 1,
            'created_at': DateTime.now().toIso8601String(),
          }),
          200,
          headers: {'content-type': 'application/json'},
        );
      });

      final client = HttpVaultClient(
        const VaultClientConfig(serverUrl: 'http://vault:8080'),
        httpClient: mockClient,
      );
      await client.getSecret('system/db');
      await client.getSecret('system/db');
      expect(callCount, equals(1));
    });

    test('設定のcacheTtlが適用されること', () async {
      int callCount = 0;
      final mockClient = MockClient((request) async {
        callCount++;
        return http.Response(
          json.encode({
            'path': 'system/db',
            'data': {'key': 'val'},
            'version': 1,
            'created_at': DateTime.now().toIso8601String(),
          }),
          200,
          headers: {'content-type': 'application/json'},
        );
      });

      final client = HttpVaultClient(
        const VaultClientConfig(
          serverUrl: 'http://vault:8080',
          cacheTtl: Duration.zero,
        ),
        httpClient: mockClient,
      );
      await client.getSecret('system/db');
      await client.getSecret('system/db');
      expect(callCount, equals(2));
    });
  });
}
