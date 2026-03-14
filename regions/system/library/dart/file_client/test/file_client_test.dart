import 'package:test/test.dart';

import 'package:k1s0_file_client/file_client.dart';

void main() {
  late InMemoryFileClient client;

  setUp(() {
    client = InMemoryFileClient();
  });

  group('generateUploadUrl', () {
    test('アップロードURLを返すこと', () async {
      final url = await client.generateUploadUrl(
        'uploads/test.png',
        'image/png',
        const Duration(hours: 1),
      );
      expect(url.url, contains('uploads/test.png'));
      expect(url.method, equals('PUT'));
    });
  });

  group('generateDownloadUrl', () {
    test('既存ファイルのダウンロードURLを返すこと', () async {
      await client.generateUploadUrl(
        'uploads/test.png',
        'image/png',
        const Duration(hours: 1),
      );
      final url = await client.generateDownloadUrl(
        'uploads/test.png',
        const Duration(minutes: 5),
      );
      expect(url.url, contains('uploads/test.png'));
      expect(url.method, equals('GET'));
    });

    test('存在しないファイルでエラーを投げること', () async {
      expect(
        () => client.generateDownloadUrl(
          'nonexistent.txt',
          const Duration(minutes: 5),
        ),
        throwsA(isA<FileClientError>()),
      );
    });
  });

  group('delete', () {
    test('既存ファイルを削除すること', () async {
      await client.generateUploadUrl(
        'uploads/test.png',
        'image/png',
        const Duration(hours: 1),
      );
      await client.delete('uploads/test.png');
      expect(
        () => client.getMetadata('uploads/test.png'),
        throwsA(isA<FileClientError>()),
      );
    });
  });

  group('getMetadata', () {
    test('既存ファイルのメタデータを返すこと', () async {
      await client.generateUploadUrl(
        'uploads/test.png',
        'image/png',
        const Duration(hours: 1),
      );
      final meta = await client.getMetadata('uploads/test.png');
      expect(meta.path, equals('uploads/test.png'));
      expect(meta.contentType, equals('image/png'));
    });
  });

  group('list', () {
    test('プレフィックスに一致するファイル一覧を返すこと', () async {
      await client.generateUploadUrl(
        'uploads/a.png',
        'image/png',
        const Duration(hours: 1),
      );
      await client.generateUploadUrl(
        'uploads/b.jpg',
        'image/jpeg',
        const Duration(hours: 1),
      );
      await client.generateUploadUrl(
        'other/c.txt',
        'text/plain',
        const Duration(hours: 1),
      );
      final files = await client.list('uploads/');
      expect(files, hasLength(2));
    });
  });

  group('copy', () {
    test('ファイルを新しいパスにコピーすること', () async {
      await client.generateUploadUrl(
        'uploads/test.png',
        'image/png',
        const Duration(hours: 1),
      );
      await client.copy('uploads/test.png', 'archive/test.png');
      final meta = await client.getMetadata('archive/test.png');
      expect(meta.contentType, equals('image/png'));
      expect(meta.path, equals('archive/test.png'));
    });

    test('存在しないコピー元でエラーを投げること', () async {
      expect(
        () => client.copy('nonexistent.txt', 'dest.txt'),
        throwsA(isA<FileClientError>()),
      );
    });
  });

  group('storedFiles', () {
    test('初期状態で空であること', () {
      expect(client.storedFiles, isEmpty);
    });
  });
}
