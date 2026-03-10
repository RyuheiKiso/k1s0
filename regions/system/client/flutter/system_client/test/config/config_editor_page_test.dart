import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:system_client/system_client.dart';

class MockDio extends Mock implements Dio {}

void main() {
  late MockDio mockDio;

  const schemaJson = {
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
            'min': 0.5,
            'max': 10.0,
            'default': 1.5,
          },
          {
            'key': 'metadata',
            'label': 'Metadata',
            'type': 'object',
            'default': {'owner': 'system'},
          },
          {
            'key': 'origins',
            'label': 'Origins',
            'type': 'array',
            'default': ['app'],
          },
        ],
      },
    ],
  };

  const valuesJson = {
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
        'version': 8,
      },
      {
        'namespace': 'test.advanced',
        'key': 'metadata',
        'value': {'owner': 'ops'},
        'version': 4,
      },
      {
        'namespace': 'test.advanced',
        'key': 'origins',
        'value': ['app', 'worker'],
        'version': 5,
      },
    ],
  };

  setUp(() {
    mockDio = MockDio();

    when(() => mockDio.get<Map<String, dynamic>>(
        '/api/v1/config-schema/test-service')).thenAnswer(
      (_) async => Response(
        data: schemaJson,
        statusCode: 200,
        requestOptions:
            RequestOptions(path: '/api/v1/config-schema/test-service'),
      ),
    );

    when(() => mockDio.get<Map<String, dynamic>>(
        '/api/v1/config/services/test-service')).thenAnswer(
      (_) async => Response(
        data: valuesJson,
        statusCode: 200,
        requestOptions:
            RequestOptions(path: '/api/v1/config/services/test-service'),
      ),
    );
  });

  Future<void> pumpPage(WidgetTester tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: ConfigEditorPage(
          dio: mockDio,
          serviceName: 'test-service',
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  FilledButton saveButton(WidgetTester tester) {
    return tester
        .widget<FilledButton>(find.widgetWithText(FilledButton, 'Save'));
  }

  testWidgets(
      'saves namespace-specific versioned updates and renders object/array fields',
      (tester) async {
    when(
      () => mockDio.put<dynamic>(
        any(),
        data: any(named: 'data'),
      ),
    ).thenAnswer(
      (invocation) async => Response(
        data: {
          'namespace': invocation.positionalArguments[0] as String,
          'key': 'timeout',
          'value': (invocation.namedArguments[#data]
              as Map<String, dynamic>)['value'],
          'version': 9,
        },
        statusCode: 200,
        requestOptions:
            RequestOptions(path: invocation.positionalArguments[0] as String),
      ),
    );

    await pumpPage(tester);

    await tester.tap(find.text('Advanced'));
    await tester.pumpAndSettle();

    expect(find.text('Metadata'), findsOneWidget);
    expect(find.text('Origins'), findsOneWidget);

    await tester.enterText(find.byType(TextFormField).first, '3.5');
    await tester.pumpAndSettle();

    expect(saveButton(tester).onPressed, isNotNull);

    await tester.tap(find.widgetWithText(FilledButton, 'Save'));
    await tester.pumpAndSettle();

    final captured = verify(
      () => mockDio.put<dynamic>(
        captureAny(),
        data: captureAny(named: 'data'),
      ),
    ).captured;

    expect(captured[0], equals('/api/v1/config/test.advanced/timeout'));
    expect(captured[1], equals({'value': 3.5, 'version': 8}));
  });

  testWidgets('disables save while object JSON is invalid', (tester) async {
    await pumpPage(tester);

    await tester.tap(find.text('Advanced'));
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextFormField).first, '4.25');
    await tester.pumpAndSettle();
    expect(saveButton(tester).onPressed, isNotNull);

    await tester.enterText(find.byType(TextFormField).at(1), '{invalid');
    await tester.pumpAndSettle();

    expect(find.text('Invalid JSON'), findsOneWidget);
    expect(saveButton(tester).onPressed, isNull);

    await tester.enterText(
      find.byType(TextFormField).at(1),
      '{"owner":"platform"}',
    );
    await tester.pumpAndSettle();

    expect(find.text('Invalid JSON'), findsNothing);
    expect(saveButton(tester).onPressed, isNotNull);
  });

  testWidgets('shows conflict guidance when save receives 409', (tester) async {
    when(
      () => mockDio.put<dynamic>(
        any(),
        data: any(named: 'data'),
      ),
    ).thenThrow(
      DioException(
        requestOptions:
            RequestOptions(path: '/api/v1/config/test.general/timeout'),
        response: Response(
          statusCode: 409,
          requestOptions:
              RequestOptions(path: '/api/v1/config/test.general/timeout'),
        ),
        type: DioExceptionType.badResponse,
      ),
    );

    await pumpPage(tester);

    await tester.enterText(find.byType(TextFormField).first, '90');
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Save'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));

    expect(find.text('Conflict'), findsOneWidget);
    expect(
      find.text(
          'Another user updated this config. Reload and review before saving again.'),
      findsOneWidget,
    );

    await tester.tap(find.widgetWithText(TextButton, 'OK'));
    await tester.pumpAndSettle();

    expect(
      find.text('Conflict detected. Reload and review before saving again.'),
      findsOneWidget,
    );
  });
}
