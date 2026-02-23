import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:system_client/system_client.dart';

class MockDio extends Mock implements Dio {}

void main() {
  late MockDio mockDio;

  setUp(() {
    mockDio = MockDio();
  });

  group('ConfigInterpreter', () {
    test('schema と values が正しくマージされる', () async {
      final schemaJson = {
        'service': 'test-service',
        'namespace_prefix': 'test',
        'categories': [
          {
            'id': 'general',
            'label': 'General',
            'namespaces': ['test.general'],
            'fields': [
              {
                'key': 'timeout',
                'label': 'Timeout',
                'type': 'integer',
                'min': 1,
                'max': 300,
                'unit': 'seconds',
                'default': 30,
              },
            ],
          },
        ],
      };

      final valuesJson = {
        'values': {'timeout': 60},
      };

      when(() => mockDio.get<Map<String, dynamic>>(
            '/api/v1/config-schema/test-service',
          )).thenAnswer(
        (_) async => Response(
          data: schemaJson,
          statusCode: 200,
          requestOptions: RequestOptions(),
        ),
      );

      when(() => mockDio.get<Map<String, dynamic>>(
            '/api/v1/config/services/test-service',
          )).thenAnswer(
        (_) async => Response(
          data: valuesJson,
          statusCode: 200,
          requestOptions: RequestOptions(),
        ),
      );

      final interpreter = ConfigInterpreter(dio: mockDio);
      final data = await interpreter.build('test-service');

      expect(data.schema.service, equals('test-service'));
      expect(data.schema.namespacePrefix, equals('test'));
      expect(data.schema.categories, hasLength(1));
      expect(data.schema.categories.first.fields, hasLength(1));
      expect(data.schema.categories.first.fields.first.key, equals('timeout'));
      expect(data.values['timeout'], equals(60));
    });

    test('API 失敗時に例外がスローされる', () async {
      when(() => mockDio.get<Map<String, dynamic>>(
            '/api/v1/config-schema/bad-service',
          )).thenThrow(
        DioException(
          requestOptions: RequestOptions(),
          type: DioExceptionType.badResponse,
          response: Response(
            statusCode: 404,
            requestOptions: RequestOptions(),
          ),
        ),
      );

      when(() => mockDio.get<Map<String, dynamic>>(
            '/api/v1/config/services/bad-service',
          )).thenThrow(
        DioException(
          requestOptions: RequestOptions(),
          type: DioExceptionType.badResponse,
          response: Response(
            statusCode: 404,
            requestOptions: RequestOptions(),
          ),
        ),
      );

      final interpreter = ConfigInterpreter(dio: mockDio);

      expect(
        () => interpreter.build('bad-service'),
        throwsA(isA<DioException>()),
      );
    });
  });
}
