import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:k1s0_app_updater/app_updater.dart';

void main() {
  group('RegistryApiClient', () {
    test('/api/v1 配下の latest エンドポイントが呼び出されること', () async {
      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: MockClient((request) async {
          expect(request.url.path, '/api/v1/apps/test-app/latest');
          expect(request.url.queryParameters['platform'], 'windows');
          return http.Response(
            '''
            {
              "app_id": "test-app",
              "version": "2.0.0",
              "platform": "windows",
              "arch": "amd64",
              "checksum_sha256": "abc123",
              "mandatory": false,
              "published_at": "2026-03-10T09:00:00Z"
            }
            ''',
            200,
          );
        }),
      );

      final latest = await client.getLatestVersion(
        'test-app',
        platform: 'windows',
        arch: 'amd64',
      );

      expect(latest.version, '2.0.0');
    });

    test('versions ラッパーレスポンスを正しくパースできること', () async {
      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: MockClient((request) async {
          expect(request.url.path, '/api/v1/apps/test-app/versions');
          return http.Response(
            '''
            {
              "versions": [
                {
                  "app_id": "test-app",
                  "version": "2.0.0",
                  "platform": "windows",
                  "arch": "amd64",
                  "checksum_sha256": "abc123",
                  "mandatory": false,
                  "published_at": "2026-03-10T09:00:00Z"
                }
              ]
            }
            ''',
            200,
          );
        }),
      );

      final versions = await client.listVersions('test-app');

      expect(versions, hasLength(1));
      expect(versions.first.version, '2.0.0');
    });

    test('ダウンロードレスポンスを正しくマッピングできること', () async {
      final client = RegistryApiClient(
        baseUrl: 'https://registry.example.com',
        client: MockClient((request) async {
          expect(
            request.url.path,
            '/api/v1/apps/test-app/versions/2.0.0/download',
          );
          return http.Response(
            '''
            {
              "download_url": "https://cdn.example.com/file.exe",
              "expires_in": 3600,
              "checksum_sha256": "abc123",
              "size_bytes": 12345
            }
            ''',
            200,
          );
        }),
      );

      final artifact = await client.getDownloadInfo(
        'test-app',
        '2.0.0',
        platform: 'windows',
        arch: 'amd64',
      );

      expect(artifact.downloadUrl, 'https://cdn.example.com/file.exe');
      expect(artifact.expiresIn, 3600);
    });
  });
}
