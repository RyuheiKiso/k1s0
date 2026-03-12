import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:k1s0_app_updater/app_updater.dart';

void main() {
  group('InMemoryAppUpdater', () {
    test('returns mandatory when current version is below minimum version',
        () async {
      final updater = InMemoryAppUpdater(
        versionInfo: const AppVersionInfo(
          latestVersion: '3.0.0',
          minimumVersion: '2.0.0',
          releaseNotes: 'Security fixes',
        ),
        currentVersion: '1.5.0',
      );

      final result = await updater.checkForUpdate();

      expect(result.type, UpdateType.mandatory);
      expect(result.isMandatory, isTrue);
    });

    test('returns optional when latest version is newer but not mandatory',
        () async {
      final updater = InMemoryAppUpdater(
        versionInfo: const AppVersionInfo(
          latestVersion: '2.1.0',
          minimumVersion: '1.0.0',
        ),
        currentVersion: '2.0.0',
      );

      final result = await updater.checkForUpdate();

      expect(result.type, UpdateType.optional);
      expect(result.needsUpdate, isTrue);
    });

    test('returns none when current version is latest', () async {
      final updater = InMemoryAppUpdater(
        versionInfo: const AppVersionInfo(
          latestVersion: '2.0.0',
          minimumVersion: '1.0.0',
        ),
        currentVersion: '2.0.0',
      );

      final result = await updater.checkForUpdate();

      expect(result.type, UpdateType.none);
      expect(result.needsUpdate, isFalse);
    });
  });

  group('AppRegistryAppUpdater', () {
    test('derives minimum version from the latest mandatory release', () async {
      var sawAuthorizationHeader = false;
      final client = MockClient((request) async {
        if (request.url.path.endsWith('/latest')) {
          sawAuthorizationHeader =
              request.headers['Authorization'] == 'Bearer token-123';
          expect(request.url.queryParameters['platform'], 'windows');
          expect(request.url.queryParameters['arch'], 'amd64');
          return http.Response(
            '''
            {
              "app_id": "order-client",
              "version": "2.3.0",
              "platform": "windows",
              "arch": "amd64",
              "size_bytes": 1048576,
              "checksum_sha256": "latest-checksum",
              "release_notes": "Latest release",
              "mandatory": false,
              "published_at": "2026-03-10T09:00:00Z"
            }
            ''',
            200,
          );
        }

        if (request.url.path.endsWith('/versions')) {
          return http.Response(
            '''
            {
              "versions": [
                {
                  "app_id": "order-client",
                  "version": "2.3.0",
                  "platform": "windows",
                  "arch": "amd64",
                  "checksum_sha256": "latest-checksum",
                  "mandatory": false,
                  "published_at": "2026-03-10T09:00:00Z"
                },
                {
                  "app_id": "order-client",
                  "version": "2.0.0",
                  "platform": "windows",
                  "arch": "amd64",
                  "checksum_sha256": "mandatory-checksum",
                  "mandatory": true,
                  "published_at": "2026-03-01T09:00:00Z"
                }
              ]
            }
            ''',
            200,
          );
        }

        return http.Response('Not Found', 404);
      });

      final updater = AppRegistryAppUpdater(
        AppUpdaterConfig(
          serverUrl: 'https://registry.example.com',
          appId: 'order-client',
          platform: 'windows',
          arch: 'amd64',
          tokenProvider: () async => 'token-123',
          currentVersionProvider: () async => '1.9.0',
          httpClient: client,
        ),
      );

      final result = await updater.checkForUpdate();

      expect(result.type, UpdateType.mandatory);
      expect(result.versionInfo.latestVersion, '2.3.0');
      expect(result.versionInfo.minimumVersion, '2.0.0');
      expect(sawAuthorizationHeader, isTrue);
    });

    test('fetchDownloadInfo maps download endpoint response', () async {
      final client = MockClient((request) async {
        expect(request.url.path,
            '/api/v1/apps/order-client/versions/2.3.0/download');
        expect(request.url.queryParameters['platform'], 'windows');
        expect(request.url.queryParameters['arch'], 'amd64');
        return http.Response(
          '''
          {
            "download_url": "https://cdn.example.com/order-client.exe",
            "expires_in": 3600,
            "checksum_sha256": "artifact-checksum",
            "size_bytes": 2048
          }
          ''',
          200,
        );
      });

      final updater = AppRegistryAppUpdater(
        AppUpdaterConfig(
          serverUrl: 'https://registry.example.com',
          appId: 'order-client',
          platform: 'windows',
          arch: 'amd64',
          currentVersionProvider: () async => '2.2.0',
          httpClient: client,
        ),
      );

      final artifact = await updater.fetchDownloadInfo(version: '2.3.0');

      expect(artifact.downloadUrl, 'https://cdn.example.com/order-client.exe');
      expect(artifact.expiresIn, 3600);
      expect(artifact.checksumSha256, 'artifact-checksum');
      expect(artifact.sizeBytes, 2048);
    });

    test('openStore uses configured launcher callback', () async {
      Uri? launchedUri;
      final updater = AppRegistryAppUpdater(
        AppUpdaterConfig(
          serverUrl: 'https://registry.example.com',
          appId: 'mobile-app',
          platform: 'android',
          androidStoreUrl:
              'https://play.google.com/store/apps/details?id=mobile-app',
          currentVersionProvider: () async => '1.0.0',
          urlLauncher: (uri) async {
            launchedUri = uri;
            return true;
          },
          httpClient: MockClient((_) async => http.Response('{}', 404)),
        ),
      );

      final opened = await updater.openStore();

      expect(opened, isTrue);
      expect(
        launchedUri.toString(),
        'https://play.google.com/store/apps/details?id=mobile-app',
      );
    });
  });

  group('MockAppUpdater', () {
    test('records calls and returns callback results', () async {
      final mock = MockAppUpdater()
        ..onCheckForUpdate = () async => const UpdateCheckResult(
              type: UpdateType.optional,
              currentVersion: '1.0.0',
              versionInfo: AppVersionInfo(
                latestVersion: '1.1.0',
                minimumVersion: '1.0.0',
              ),
            );

      final result = await mock.checkForUpdate();

      expect(result.type, UpdateType.optional);
      expect(mock.calls, contains('checkForUpdate'));
    });
  });
}
