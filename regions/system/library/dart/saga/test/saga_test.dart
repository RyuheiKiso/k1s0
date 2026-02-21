import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';

import 'package:k1s0_saga/saga.dart';

class MockHttpClient extends Mock implements http.Client {}

http.Response sagaResponse(Map<String, dynamic> body) =>
    http.Response(jsonEncode(body), 200);

http.Response errorResponse(int statusCode) =>
    http.Response('error', statusCode);

void main() {
  late MockHttpClient mockClient;
  late SagaClient client;

  setUpAll(() {
    registerFallbackValue(Uri.parse('http://localhost'));
    registerFallbackValue(<String, String>{});
  });

  setUp(() {
    mockClient = MockHttpClient();
    client = SagaClient('http://localhost:8080', httpClient: mockClient);
  });

  group('SagaClient', () {
    group('startSaga', () {
      test('Saga を開始して saga_id を返す', () async {
        when(() => mockClient.post(
              Uri.parse('http://localhost:8080/api/v1/sagas'),
              headers: any(named: 'headers'),
              body: any(named: 'body'),
            )).thenAnswer(
          (_) async =>
              sagaResponse({'saga_id': 'saga-123', 'status': 'STARTED'}),
        );

        final resp = await client.startSaga(
          StartSagaRequest(
            workflowName: 'order-create',
            payload: {'order_id': '1'},
          ),
        );

        expect(resp.sagaId, equals('saga-123'));
      });

      test('エラーレスポンスで SagaException を投げる', () async {
        when(() => mockClient.post(
              any(),
              headers: any(named: 'headers'),
              body: any(named: 'body'),
            )).thenAnswer((_) async => errorResponse(500));

        expect(
          () => client.startSaga(
              StartSagaRequest(workflowName: 'test', payload: {})),
          throwsA(isA<SagaException>()),
        );
      });
    });

    group('getSaga', () {
      test('Saga 状態を返す', () async {
        when(() => mockClient.get(
              Uri.parse('http://localhost:8080/api/v1/sagas/saga-456'),
            )).thenAnswer(
          (_) async => sagaResponse({
            'saga': {
              'saga_id': 'saga-456',
              'workflow_name': 'order-create',
              'status': 'RUNNING',
              'step_logs': [],
              'created_at': '2024-01-01T00:00:00Z',
              'updated_at': '2024-01-01T00:00:00Z',
            }
          }),
        );

        final state = await client.getSaga('saga-456');
        expect(state.sagaId, equals('saga-456'));
        expect(state.workflowName, equals('order-create'));
        expect(state.status, equals(SagaStatus.running));
      });

      test('エラーレスポンスで SagaException を投げる', () async {
        when(() => mockClient.get(any()))
            .thenAnswer((_) async => errorResponse(404));

        expect(
          () => client.getSaga('unknown'),
          throwsA(isA<SagaException>()),
        );
      });
    });

    group('cancelSaga', () {
      test('Saga をキャンセルする', () async {
        when(() => mockClient.post(
              Uri.parse('http://localhost:8080/api/v1/sagas/saga-789/cancel'),
              headers: any(named: 'headers'),
              body: any(named: 'body'),
            )).thenAnswer((_) async => http.Response('{}', 200));

        await expectLater(client.cancelSaga('saga-789'), completes);
      });

      test('エラーレスポンスで SagaException を投げる', () async {
        when(() => mockClient.post(
              any(),
              headers: any(named: 'headers'),
              body: any(named: 'body'),
            )).thenAnswer((_) async => errorResponse(500));

        expect(
          () => client.cancelSaga('saga-789'),
          throwsA(isA<SagaException>()),
        );
      });
    });
  });
}
