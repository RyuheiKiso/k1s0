
import 'package:test/test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart' as http_testing;

import 'package:k1s0_file_client/file_client.dart';

void main() {
  // ---------------------------------------------------------------------------
  // コンストラクタのバリデーション
  // ---------------------------------------------------------------------------

  group('S3FileClient コンストラクタ', () {
    test('必須フィールドが欠けている場合は FileClientError を投げる', () {
      expect(
        () => S3FileClient(const FileClientConfig()),
        throwsA(isA<FileClientError>()),
      );
      expect(
        () => S3FileClient(const FileClientConfig(s3Endpoint: 'http://s3')),
        throwsA(isA<FileClientError>()),
      );
      expect(
        () => S3FileClient(const FileClientConfig(
          s3Endpoint: 'http://s3',
          bucket: 'test',
        )),
        throwsA(isA<FileClientError>()),
      );
      expect(
        () => S3FileClient(const FileClientConfig(
          s3Endpoint: 'http://s3',
          bucket: 'test',
          region: 'us-east-1',
        )),
        throwsA(isA<FileClientError>()),
      );
      expect(
        () => S3FileClient(const FileClientConfig(
          s3Endpoint: 'http://s3',
          bucket: 'test',
          region: 'us-east-1',
          accessKeyId: 'AKID',
        )),
        throwsA(isA<FileClientError>()),
      );
    });

    test('全フィールドが設定されている場合は正常に生成される', () {
      final client = S3FileClient(_validConfig());
      expect(client, isA<FileClient>());
    });
  });

  // ---------------------------------------------------------------------------
  // generateUploadUrl
  // ---------------------------------------------------------------------------

  group('generateUploadUrl', () {
    test('PUT メソッドのプリサインド URL を返す', () async {
      final client = S3FileClient(
        _validConfig(),
        httpClient: _noopClient(),
      );
      final url = await client.generateUploadUrl(
        'uploads/test.png',
        'image/png',
        const Duration(hours: 1),
      );

      expect(url.method, equals('PUT'));
      expect(url.url, contains('test-bucket'));
      expect(url.url, contains('uploads'));
      expect(url.url, contains('test.png'));
      expect(url.url, contains('X-Amz-Algorithm=AWS4-HMAC-SHA256'));
      expect(url.url, contains('X-Amz-Credential='));
      expect(url.url, contains('X-Amz-Signature='));
      expect(url.url, contains('X-Amz-Expires=3600'));
      expect(url.headers['content-type'], equals('image/png'));
    });

    test('有効期限が正しく設定される', () async {
      final client = S3FileClient(
        _validConfig(),
        httpClient: _noopClient(),
      );
      final before = DateTime.now().toUtc();
      final url = await client.generateUploadUrl(
        'test.txt',
        'text/plain',
        const Duration(minutes: 30),
      );
      final after = DateTime.now().toUtc();

      expect(
        url.expiresAt.isAfter(before.add(const Duration(minutes: 29))),
        isTrue,
      );
      expect(
        url.expiresAt.isBefore(after.add(const Duration(minutes: 31))),
        isTrue,
      );
    });
  });

  // ---------------------------------------------------------------------------
  // generateDownloadUrl
  // ---------------------------------------------------------------------------

  group('generateDownloadUrl', () {
    test('GET メソッドのプリサインド URL を返す', () async {
      final client = S3FileClient(
        _validConfig(),
        httpClient: _noopClient(),
      );
      final url = await client.generateDownloadUrl(
        'downloads/file.pdf',
        const Duration(minutes: 15),
      );

      expect(url.method, equals('GET'));
      expect(url.url, contains('test-bucket'));
      expect(url.url, contains('file.pdf'));
      expect(url.url, contains('X-Amz-Signature='));
      expect(url.url, contains('X-Amz-Expires=900'));
      expect(url.headers, isEmpty);
    });
  });

  // ---------------------------------------------------------------------------
  // delete
  // ---------------------------------------------------------------------------

  group('delete', () {
    test('DELETE リクエストを送信する', () async {
      String? capturedMethod;
      final mockClient = http_testing.MockClient((request) async {
        capturedMethod = request.method;
        return http.Response('', 204);
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      await client.delete('uploads/test.png');
      expect(capturedMethod, equals('DELETE'));
    });

    test('404 レスポンスで FileClientError を投げる', () async {
      final mockClient = http_testing.MockClient((_) async {
        return http.Response('Not Found', 404);
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      expect(
        () => client.delete('nonexistent.txt'),
        throwsA(isA<FileClientError>()),
      );
    });

    test('リクエストに Authorization ヘッダーが含まれる', () async {
      Map<String, String>? capturedHeaders;
      final mockClient = http_testing.MockClient((request) async {
        capturedHeaders = request.headers;
        return http.Response('', 204);
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      await client.delete('test.txt');
      expect(capturedHeaders, isNotNull);
      expect(capturedHeaders!['authorization'],
          startsWith('AWS4-HMAC-SHA256'));
      expect(capturedHeaders!['x-amz-date'], isNotEmpty);
      expect(capturedHeaders!['x-amz-content-sha256'], isNotEmpty);
    });
  });

  // ---------------------------------------------------------------------------
  // getMetadata
  // ---------------------------------------------------------------------------

  group('getMetadata', () {
    test('HEAD レスポンスからメタデータを構築する', () async {
      final mockClient = http_testing.MockClient((request) async {
        expect(request.method, equals('HEAD'));
        return http.Response('', 200, headers: {
          'content-length': '1024',
          'content-type': 'image/png',
          'etag': '"abc123"',
          'last-modified': 'Mon, 01 Jan 2024 00:00:00 GMT',
        });
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      final meta = await client.getMetadata('uploads/test.png');
      expect(meta.path, equals('uploads/test.png'));
      expect(meta.sizeBytes, equals(1024));
      expect(meta.contentType, equals('image/png'));
      expect(meta.etag, equals('abc123'));
      expect(meta.lastModified.year, equals(2024));
    });

    test('404 レスポンスで FileClientError を投げる', () async {
      final mockClient = http_testing.MockClient((_) async {
        return http.Response('', 404);
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      expect(
        () => client.getMetadata('nonexistent.txt'),
        throwsA(isA<FileClientError>()),
      );
    });
  });

  // ---------------------------------------------------------------------------
  // list
  // ---------------------------------------------------------------------------

  group('list', () {
    test('ListObjectsV2 レスポンスをパースする', () async {
      const xmlResponse = '''<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>test-bucket</Name>
  <Prefix>uploads/</Prefix>
  <KeyCount>2</KeyCount>
  <MaxKeys>1000</MaxKeys>
  <IsTruncated>false</IsTruncated>
  <Contents>
    <Key>uploads/a.png</Key>
    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
    <ETag>&quot;etag1&quot;</ETag>
    <Size>512</Size>
    <StorageClass>STANDARD</StorageClass>
  </Contents>
  <Contents>
    <Key>uploads/b.jpg</Key>
    <LastModified>2024-01-02T12:00:00.000Z</LastModified>
    <ETag>&quot;etag2&quot;</ETag>
    <Size>2048</Size>
    <StorageClass>STANDARD</StorageClass>
  </Contents>
</ListBucketResult>''';

      final mockClient = http_testing.MockClient((request) async {
        expect(request.method, equals('GET'));
        expect(request.url.queryParameters['list-type'], equals('2'));
        expect(request.url.queryParameters['prefix'], equals('uploads/'));
        return http.Response(xmlResponse, 200);
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      final files = await client.list('uploads/');
      expect(files, hasLength(2));
      expect(files[0].path, equals('uploads/a.png'));
      expect(files[0].sizeBytes, equals(512));
      expect(files[0].etag, equals('etag1'));
      expect(files[1].path, equals('uploads/b.jpg'));
      expect(files[1].sizeBytes, equals(2048));
    });

    test('空の結果を正しく処理する', () async {
      const xmlResponse = '''<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>test-bucket</Name>
  <Prefix>empty/</Prefix>
  <KeyCount>0</KeyCount>
  <MaxKeys>1000</MaxKeys>
  <IsTruncated>false</IsTruncated>
</ListBucketResult>''';

      final mockClient = http_testing.MockClient((_) async {
        return http.Response(xmlResponse, 200);
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      final files = await client.list('empty/');
      expect(files, isEmpty);
    });
  });

  // ---------------------------------------------------------------------------
  // copy
  // ---------------------------------------------------------------------------

  group('copy', () {
    test('x-amz-copy-source ヘッダー付きの PUT リクエストを送信する', () async {
      String? capturedMethod;
      String? capturedCopySource;
      final mockClient = http_testing.MockClient((request) async {
        capturedMethod = request.method;
        capturedCopySource = request.headers['x-amz-copy-source'];
        return http.Response(
          '<?xml version="1.0"?><CopyObjectResult></CopyObjectResult>',
          200,
        );
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      await client.copy('src/file.txt', 'dst/file.txt');
      expect(capturedMethod, equals('PUT'));
      expect(capturedCopySource, contains('test-bucket'));
      expect(capturedCopySource, contains('src'));
      expect(capturedCopySource, contains('file.txt'));
    });

    test('S3 エラー時に FileClientError を投げる', () async {
      final mockClient = http_testing.MockClient((_) async {
        return http.Response('Internal Server Error', 500);
      });
      final client = S3FileClient(_validConfig(), httpClient: mockClient);

      expect(
        () => client.copy('a.txt', 'b.txt'),
        throwsA(isA<FileClientError>()),
      );
    });
  });

  // ---------------------------------------------------------------------------
  // HttpDate パーサー
  // ---------------------------------------------------------------------------

  group('HttpDate', () {
    test('RFC 1123 形式をパースする', () {
      final dt = HttpDate.parse('Mon, 01 Jan 2024 12:30:45 GMT');
      expect(dt.year, equals(2024));
      expect(dt.month, equals(1));
      expect(dt.day, equals(1));
      expect(dt.hour, equals(12));
      expect(dt.minute, equals(30));
      expect(dt.second, equals(45));
      expect(dt.isUtc, isTrue);
    });

    test('不正な形式で FormatException を投げる', () {
      expect(
        () => HttpDate.parse('invalid-date'),
        throwsA(isA<FormatException>()),
      );
    });
  });

  // ---------------------------------------------------------------------------
  // AWS Signature V4 署名の整合性
  // ---------------------------------------------------------------------------

  group('署名の整合性', () {
    test('同じパラメータで生成した URL は同じ署名構造を持つ', () async {
      final client = S3FileClient(_validConfig(), httpClient: _noopClient());

      final url1 = await client.generateUploadUrl(
        'test.txt',
        'text/plain',
        const Duration(hours: 1),
      );
      // URL には必要な署名パラメータが全て含まれる
      final uri = Uri.parse(url1.url);
      expect(uri.queryParameters.containsKey('X-Amz-Algorithm'), isTrue);
      expect(uri.queryParameters.containsKey('X-Amz-Credential'), isTrue);
      expect(uri.queryParameters.containsKey('X-Amz-Date'), isTrue);
      expect(uri.queryParameters.containsKey('X-Amz-Expires'), isTrue);
      expect(uri.queryParameters.containsKey('X-Amz-SignedHeaders'), isTrue);
      expect(uri.queryParameters.containsKey('X-Amz-Signature'), isTrue);

      // Credential にリージョンとサービスが含まれる
      final credential = uri.queryParameters['X-Amz-Credential']!;
      expect(credential, contains('us-east-1'));
      expect(credential, contains('s3'));
      expect(credential, contains('aws4_request'));
    });

    test('パスのエンコーディングが正しい', () async {
      final client = S3FileClient(_validConfig(), httpClient: _noopClient());

      final url = await client.generateDownloadUrl(
        'path/with spaces/file (1).txt',
        const Duration(hours: 1),
      );
      // スペースがエンコードされている
      expect(url.url, contains('with%20spaces'));
      expect(url.url, contains('file%20(1).txt'));
    });
  });
}

// ---------------------------------------------------------------------------
// テストヘルパー
// ---------------------------------------------------------------------------

FileClientConfig _validConfig() {
  return const FileClientConfig(
    s3Endpoint: 'http://localhost:9000',
    bucket: 'test-bucket',
    region: 'us-east-1',
    accessKeyId: 'AKIAIOSFODNN7EXAMPLE',
    secretAccessKey: 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY',
  );
}

http.Client _noopClient() {
  return http_testing.MockClient((_) async {
    return http.Response('', 200);
  });
}
