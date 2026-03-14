import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';

import 'package:k1s0_schemaregistry/schemaregistry.dart';

class MockHttpClient extends Mock implements http.Client {}

http.Response mockResponse(Object body, {int statusCode = 200}) =>
    http.Response(jsonEncode(body), statusCode);

void main() {
  late MockHttpClient mockClient;

  setUpAll(() {
    registerFallbackValue(Uri.parse('http://localhost'));
  });

  setUp(() {
    mockClient = MockHttpClient();
  });

  group('SchemaRegistryConfig', () {
    test('subjectNameが正しい値を返すこと', () {
      expect(
        SchemaRegistryConfig.subjectName(
            'k1s0.system.user.created.v1', 'value'),
        equals('k1s0.system.user.created.v1-value'),
      );
      expect(
        SchemaRegistryConfig.subjectName(
            'k1s0.system.user.created.v1', 'key'),
        equals('k1s0.system.user.created.v1-key'),
      );
    });

    test('URLが空のとき検証で例外がスローされること', () {
      final config = SchemaRegistryConfig(url: '');
      expect(() => config.validate(), throwsA(isA<SchemaRegistryError>()));
    });
  });

  group('HttpSchemaRegistryClient.registerSchema', () {
    test('成功時にスキーマIDが返ること', () async {
      when(() => mockClient.post(
            any(),
            headers: any(named: 'headers'),
            body: any(named: 'body'),
          )).thenAnswer((_) async => mockResponse({'id': 42}));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      final id = await client.registerSchema(
          'user.created.v1-value', '{"type":"record"}', 'AVRO');
      expect(id, equals(42));
    });

    test('404レスポンス時にNotFoundErrorがスローされること', () async {
      when(() => mockClient.post(
            any(),
            headers: any(named: 'headers'),
            body: any(named: 'body'),
          )).thenAnswer((_) async => http.Response('', 404));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      expect(
        () => client.registerSchema('nonexistent', '{}', 'AVRO'),
        throwsA(isA<NotFoundError>()),
      );
    });
  });

  group('HttpSchemaRegistryClient.getSchemaById', () {
    test('成功時にスキーマが返ること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => mockResponse({
            'schema': '{"type":"record"}',
            'schemaType': 'AVRO',
          }));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      final schema = await client.getSchemaById(42);
      expect(schema.id, equals(42));
      expect(schema.schema, equals('{"type":"record"}'));
    });

    test('404レスポンス時にNotFoundErrorがスローされること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => http.Response('', 404));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      expect(
        () => client.getSchemaById(999),
        throwsA(isA<NotFoundError>()),
      );
    });
  });

  group('HttpSchemaRegistryClient.getLatestSchema', () {
    test('成功時に最新スキーマが返ること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => mockResponse({
            'id': 42,
            'subject': 'user.created.v1-value',
            'version': 3,
            'schema': '{"type":"record"}',
            'schemaType': 'AVRO',
          }));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      final schema = await client.getLatestSchema('user.created.v1-value');
      expect(schema.version, equals(3));
    });

    test('404レスポンス時にNotFoundErrorがスローされること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => http.Response('', 404));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      expect(
        () => client.getLatestSchema('nonexistent-subject'),
        throwsA(isA<NotFoundError>()),
      );
    });
  });

  group('HttpSchemaRegistryClient.getSchemaVersion', () {
    test('成功時に指定バージョンのスキーマが返ること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => mockResponse({
            'id': 10,
            'subject': 'user.created.v1-value',
            'version': 2,
            'schema': '{"type":"record"}',
            'schemaType': 'AVRO',
          }));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      final schema =
          await client.getSchemaVersion('user.created.v1-value', 2);
      expect(schema.version, equals(2));
    });

    test('404レスポンス時にNotFoundErrorがスローされること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => http.Response('', 404));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      expect(
        () => client.getSchemaVersion('nonexistent-subject', 99),
        throwsA(isA<NotFoundError>()),
      );
    });
  });

  group('HttpSchemaRegistryClient.listSubjects', () {
    test('成功時にサブジェクト一覧が返ること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => mockResponse(
            ['user.created.v1-value', 'order.placed.v1-value'],
          ));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      final subjects = await client.listSubjects();
      expect(subjects, hasLength(2));
      expect(subjects, contains('user.created.v1-value'));
    });

    test('サブジェクトが存在しない場合に空リストが返ること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => mockResponse([]));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      final subjects = await client.listSubjects();
      expect(subjects, isEmpty);
    });
  });

  group('HttpSchemaRegistryClient.checkCompatibility', () {
    test('互換性がある場合にtrueが返ること', () async {
      when(() => mockClient.post(
            any(),
            headers: any(named: 'headers'),
            body: any(named: 'body'),
          )).thenAnswer(
              (_) async => mockResponse({'is_compatible': true}));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      final compatible = await client.checkCompatibility(
          'user.created.v1-value', '{"type":"record"}');
      expect(compatible, isTrue);
    });

    test('互換性がない場合にfalseが返ること', () async {
      when(() => mockClient.post(
            any(),
            headers: any(named: 'headers'),
            body: any(named: 'body'),
          )).thenAnswer(
              (_) async => mockResponse({'is_compatible': false}));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      final compatible =
          await client.checkCompatibility('subject', '{}');
      expect(compatible, isFalse);
    });
  });

  group('HttpSchemaRegistryClient.healthCheck', () {
    test('200レスポンス時に正常終了すること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => http.Response('{}', 200));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      await client.healthCheck();
    });

    test('200以外のレスポンス時に例外がスローされること', () async {
      when(() => mockClient.get(
            any(),
            headers: any(named: 'headers'),
          )).thenAnswer((_) async => http.Response('', 503));

      final client = HttpSchemaRegistryClient(
        const SchemaRegistryConfig(url: 'http://localhost:8081'),
        httpClient: mockClient,
      );

      expect(
        () => client.healthCheck(),
        throwsA(isA<SchemaRegistryError>()),
      );
    });
  });

  group('Error types', () {
    test('NotFoundErrorが正しく生成されること', () {
      const err = NotFoundError('schema id=42');
      expect(err.toString(), contains('not found'));
      expect(err.resource, equals('schema id=42'));
    });

    test('NotFoundErrorに対してisNotFoundがtrueを返すこと', () {
      expect(isNotFound(const NotFoundError('test')), isTrue);
    });

    test('NotFoundError以外に対してisNotFoundがfalseを返すこと', () {
      expect(isNotFound(const SchemaRegistryError(500, 'error')), isFalse);
      expect(isNotFound(null), isFalse);
    });

    test('SchemaRegistryErrorが正しいフィールドを持つこと', () {
      const err = SchemaRegistryError(500, 'internal error');
      expect(err.statusCode, equals(500));
      expect(err.message, equals('internal error'));
      expect(err.toString(), contains('500'));
      expect(err.toString(), contains('internal error'));
    });
  });
}
