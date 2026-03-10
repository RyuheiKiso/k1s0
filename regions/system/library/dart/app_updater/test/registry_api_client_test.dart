import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:http/testing.dart' as http_testing;
import 'package:test/test.dart';

import 'package:k1s0_app_updater/app_updater.dart';

void main() {
  final sampleVersionJson = {
    'app_id': 'test-app',
    'version': '2.0.0',
    'platform': 'windows',
    'arch': 'x64',
    'checksum_sha256': 'abc123',
    'mandatory': false,
    'published_at': '2026-01-01T00:00:00.000',
    'download_url': 'https://cdn.example.com/test-app-2.0.0.exe',
  };

  group('getLatestVersion', () {
    test('returns AppVersion on success', () async {
      final mockClient = http_testing.MockClient((request) async {
        expect(request.url.path, contains('/apps/test-app/versions/latest'));
        expect(request.url.queryParameters['platform'], equals('windows'));
        expect(request.url.queryParameters['arch'], equals('x64'));
        return http.Response(jsonEncode(sampleVersionJson), 200);
      });

      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final version =
          await client.getLatestVersion('test-app', 'windows', 'x64');

      expect(version, isNotNull);
      expect(version!.appId, equals('test-app'));
      expect(version.version, equals('2.0.0'));
    });

    test('returns null on 404', () async {
      final mockClient = http_testing.MockClient((_) async {
        return http.Response('', 404);
      });

      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final version =
          await client.getLatestVersion('unknown-app', 'windows', 'x64');

      expect(version, isNull);
    });

    test('throws NetworkError on server error', () async {
      final mockClient = http_testing.MockClient((_) async {
        return http.Response('Internal Server Error', 500);
      });

      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: mockClient,
      );

      expect(
        () => client.getLatestVersion('test-app', 'windows', 'x64'),
        throwsA(isA<NetworkError>()),
      );
    });

    test('includes auth token in headers', () async {
      final mockClient = http_testing.MockClient((request) async {
        expect(
          request.headers['Authorization'],
          equals('Bearer test-token'),
        );
        return http.Response(jsonEncode(sampleVersionJson), 200);
      });

      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: mockClient,
        authToken: 'test-token',
      );

      await client.getLatestVersion('test-app', 'windows', 'x64');
    });
  });

  group('listVersions', () {
    test('returns list of AppVersion', () async {
      final mockClient = http_testing.MockClient((request) async {
        expect(request.url.path, contains('/apps/test-app/versions'));
        return http.Response(jsonEncode([sampleVersionJson]), 200);
      });

      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final versions = await client.listVersions('test-app');

      expect(versions, hasLength(1));
      expect(versions.first.version, equals('2.0.0'));
    });
  });

  group('getDownloadUrl', () {
    test('returns download URL', () async {
      final mockClient = http_testing.MockClient((request) async {
        expect(request.url.path, contains('/versions/2.0.0/download'));
        return http.Response(
          jsonEncode({'download_url': 'https://cdn.example.com/file.exe'}),
          200,
        );
      });

      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final url = await client.getDownloadUrl(
        'test-app',
        '2.0.0',
        'windows',
        'x64',
      );

      expect(url, equals('https://cdn.example.com/file.exe'));
    });
  });
}
