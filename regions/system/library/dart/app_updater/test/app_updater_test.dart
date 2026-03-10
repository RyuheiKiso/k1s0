import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';

import 'package:k1s0_app_updater/app_updater.dart';

class MockRegistryApiClient extends Mock implements RegistryApiClient {}

void main() {
  late MockRegistryApiClient mockClient;

  final sampleVersion = AppVersion(
    appId: 'test-app',
    version: '2.0.0',
    platform: 'windows',
    arch: 'x64',
    checksumSha256: 'abc123',
    mandatory: false,
    publishedAt: DateTime(2026, 1, 1),
    downloadUrl: 'https://example.com/test-app-2.0.0.exe',
  );

  final mandatoryVersion = AppVersion(
    appId: 'test-app',
    version: '3.0.0',
    platform: 'windows',
    arch: 'x64',
    checksumSha256: 'def456',
    mandatory: true,
    publishedAt: DateTime(2026, 2, 1),
  );

  setUp(() {
    mockClient = MockRegistryApiClient();
    when(() => mockClient.baseUrl).thenReturn('https://registry.example.com');
  });

  group('checkForUpdate', () {
    test('returns update available when newer version exists', () async {
      when(() => mockClient.getLatestVersion(any(), any(), any()))
          .thenAnswer((_) async => sampleVersion);

      final updater = AppUpdater(
        appId: 'test-app',
        currentVersion: '1.0.0',
        registryUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final info = await updater.checkForUpdate();

      expect(info.updateAvailable, isTrue);
      expect(info.latestVersion, isNotNull);
      expect(info.latestVersion!.version, equals('2.0.0'));
      expect(info.currentVersion, equals('1.0.0'));
    });

    test('returns no update when on latest version', () async {
      final currentVersion = AppVersion(
        appId: 'test-app',
        version: '2.0.0',
        platform: 'windows',
        arch: 'x64',
        checksumSha256: 'abc123',
        mandatory: false,
        publishedAt: DateTime(2026, 1, 1),
      );

      when(() => mockClient.getLatestVersion(any(), any(), any()))
          .thenAnswer((_) async => currentVersion);

      final updater = AppUpdater(
        appId: 'test-app',
        currentVersion: '2.0.0',
        registryUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final info = await updater.checkForUpdate();

      expect(info.updateAvailable, isFalse);
      expect(info.currentVersion, equals('2.0.0'));
    });

    test('returns no update when on newer version', () async {
      when(() => mockClient.getLatestVersion(any(), any(), any()))
          .thenAnswer((_) async => sampleVersion);

      final updater = AppUpdater(
        appId: 'test-app',
        currentVersion: '3.0.0',
        registryUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final info = await updater.checkForUpdate();

      expect(info.updateAvailable, isFalse);
    });

    test('returns no update when version not found', () async {
      when(() => mockClient.getLatestVersion(any(), any(), any()))
          .thenAnswer((_) async => null);

      final updater = AppUpdater(
        appId: 'test-app',
        currentVersion: '1.0.0',
        registryUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final info = await updater.checkForUpdate();

      expect(info.updateAvailable, isFalse);
      expect(info.latestVersion, isNull);
    });

    test('detects mandatory update', () async {
      when(() => mockClient.getLatestVersion(any(), any(), any()))
          .thenAnswer((_) async => mandatoryVersion);

      final updater = AppUpdater(
        appId: 'test-app',
        currentVersion: '1.0.0',
        registryUrl: 'https://registry.example.com',
        client: mockClient,
      );

      final info = await updater.checkForUpdate();

      expect(info.updateAvailable, isTrue);
      expect(info.isMandatory, isTrue);
    });
  });

  group('UpdateInfo', () {
    test('isMandatory returns false when no latest version', () {
      const info = UpdateInfo(
        updateAvailable: false,
        currentVersion: '1.0.0',
      );
      expect(info.isMandatory, isFalse);
    });
  });

  group('AppVersion', () {
    test('fromJson and toJson roundtrip', () {
      final json = {
        'app_id': 'test-app',
        'version': '1.0.0',
        'platform': 'windows',
        'arch': 'x64',
        'size_bytes': 1024,
        'checksum_sha256': 'abc123',
        'release_notes': 'Bug fixes',
        'mandatory': true,
        'published_at': '2026-01-01T00:00:00.000',
        'download_url': 'https://example.com/download',
      };

      final version = AppVersion.fromJson(json);
      expect(version.appId, equals('test-app'));
      expect(version.version, equals('1.0.0'));
      expect(version.sizeBytes, equals(1024));
      expect(version.mandatory, isTrue);
      expect(version.releaseNotes, equals('Bug fixes'));

      final output = version.toJson();
      expect(output['app_id'], equals('test-app'));
      expect(output['size_bytes'], equals(1024));
      expect(output['download_url'], equals('https://example.com/download'));
    });

    test('fromJson with minimal fields', () {
      final json = {
        'app_id': 'test-app',
        'version': '1.0.0',
        'platform': 'linux',
        'arch': 'arm64',
        'checksum_sha256': 'abc123',
        'mandatory': false,
        'published_at': '2026-01-01T00:00:00.000',
      };

      final version = AppVersion.fromJson(json);
      expect(version.sizeBytes, isNull);
      expect(version.releaseNotes, isNull);
      expect(version.downloadUrl, isNull);
    });

    test('toJson omits null fields', () {
      final version = AppVersion(
        appId: 'test-app',
        version: '1.0.0',
        platform: 'linux',
        arch: 'arm64',
        checksumSha256: 'abc123',
        mandatory: false,
        publishedAt: DateTime(2026, 1, 1),
      );

      final json = version.toJson();
      expect(json.containsKey('size_bytes'), isFalse);
      expect(json.containsKey('release_notes'), isFalse);
      expect(json.containsKey('download_url'), isFalse);
    });
  });

  group('AppUpdaterError', () {
    test('has correct fields', () {
      const err = AppUpdaterError('test message', 'TEST_CODE');
      expect(err.message, equals('test message'));
      expect(err.code, equals('TEST_CODE'));
      expect(err.toString(), contains('TEST_CODE'));
    });

    test('subtypes have correct codes', () {
      const networkErr = NetworkError('network issue');
      expect(networkErr.code, equals('NETWORK_ERROR'));

      const checksumErr = ChecksumError('bad checksum');
      expect(checksumErr.code, equals('CHECKSUM_ERROR'));

      const versionErr = VersionNotFoundError('not found');
      expect(versionErr.code, equals('VERSION_NOT_FOUND'));

      const downloadErr = DownloadError('download failed');
      expect(downloadErr.code, equals('DOWNLOAD_ERROR'));

      const applyErr = ApplyError('apply failed');
      expect(applyErr.code, equals('APPLY_ERROR'));
    });
  });
}
