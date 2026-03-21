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
    test('merges schema and versioned service entries by namespace plus key', () async {
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
          {
            'id': 'advanced',
            'label': 'Advanced',
            'namespaces': ['test.advanced'],
            'fields': [
              {
                'key': 'timeout',
                'label': 'Advanced Timeout',
                'type': 'float',
                'default': 1.5,
              },
              {
                'key': 'metadata',
                'label': 'Metadata',
                'type': 'object',
                'default': {'owner': 'system'},
              },
            ],
          },
        ],
      };

      final valuesJson = {
        'service_name': 'test-service',
        'entries': [
          {
            'namespace': 'test.general',
            'key': 'timeout',
            'value': 60,
            'version': 3,
          },
          {
            'namespace': 'test.advanced',
            'key': 'timeout',
            'value': 2.75,
            'version': 9,
          },
        ],
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

      expect(data.service, equals('test-service'));
      expect(data.categories, hasLength(2));
      expect(data.dirtyCount, equals(0));

      final general = data.categories.firstWhere((category) => category.schema.id == 'general');
      final advanced = data.categories.firstWhere((category) => category.schema.id == 'advanced');

      expect(general.fields.single.id, equals('test.general::timeout'));
      // value は ConfigValue 型: 数値は NumberConfigValue でラップされる
      expect(general.fields.single.value, equals(const NumberConfigValue(60)));
      expect(general.fields.single.version, equals(3));
      expect(general.fields.single.originalVersion, equals(3));

      expect(advanced.fields.first.id, equals('test.advanced::timeout'));
      // value は ConfigValue 型: 浮動小数点も NumberConfigValue でラップされる
      expect(advanced.fields.first.value, equals(const NumberConfigValue(2.75)));
      expect(advanced.fields.first.version, equals(9));
      expect(advanced.fields.first.originalValue, equals(const NumberConfigValue(2.75)));

      final metadata = advanced.fields.firstWhere((field) => field.key == 'metadata');
      // デフォルト値がない場合はデフォルト値（スキーマの default）が使用される
      expect(metadata.value, equals(MapConfigValue({'owner': const StringConfigValue('system')})));
      expect(metadata.version, equals(0));
      expect(metadata.error, isNull);
    });

    test('surfaces Dio errors when the API request fails', () async {
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
