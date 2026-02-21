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
    test('subjectName returns correct value', () {
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

    test('validate throws on empty URL', () {
      final config = SchemaRegistryConfig(url: '');
      expect(() => config.validate(), throwsA(isA<SchemaRegistryError>()));
    });
  });

  group('HttpSchemaRegistryClient.registerSchema', () {
    test('returns schema id on success', () async {
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

    test('throws NotFoundError on 404', () async {
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
    test('returns schema on success', () async {
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

    test('throws NotFoundError on 404', () async {
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
    test('returns latest schema on success', () async {
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

    test('throws NotFoundError on 404', () async {
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
    test('returns specific version on success', () async {
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

    test('throws NotFoundError on 404', () async {
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
    test('returns list of subjects on success', () async {
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

    test('returns empty list when no subjects', () async {
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
    test('returns true when compatible', () async {
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

    test('returns false when incompatible', () async {
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
    test('succeeds on 200', () async {
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

    test('throws on non-200', () async {
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
    test('NotFoundError is created correctly', () {
      const err = NotFoundError('schema id=42');
      expect(err.toString(), contains('not found'));
      expect(err.resource, equals('schema id=42'));
    });

    test('isNotFound returns true for NotFoundError', () {
      expect(isNotFound(const NotFoundError('test')), isTrue);
    });

    test('isNotFound returns false for other errors', () {
      expect(isNotFound(const SchemaRegistryError(500, 'error')), isFalse);
      expect(isNotFound(null), isFalse);
    });

    test('SchemaRegistryError has correct fields', () {
      const err = SchemaRegistryError(500, 'internal error');
      expect(err.statusCode, equals(500));
      expect(err.message, equals('internal error'));
      expect(err.toString(), contains('500'));
      expect(err.toString(), contains('internal error'));
    });
  });
}
