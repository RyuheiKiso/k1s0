import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';

import 'package:k1s0_dlq_client/dlq_client.dart';

class MockHttpClient extends Mock implements http.Client {}

http.Response jsonResponse(Map<String, dynamic> body) =>
    http.Response(jsonEncode(body), 200);

http.Response errorResponse(int statusCode) =>
    http.Response('error', statusCode);

void main() {
  late MockHttpClient mockClient;
  late DlqClient client;

  setUpAll(() {
    registerFallbackValue(Uri.parse('http://localhost'));
    registerFallbackValue(<String, String>{});
  });

  setUp(() {
    mockClient = MockHttpClient();
    client = DlqClient('http://localhost:8080', httpClient: mockClient);
  });

  group('DlqClient', () {
    group('listMessages', () {
      test('DLQ メッセージ一覧を返す', () async {
        when(() => mockClient.get(any())).thenAnswer(
          (_) async => jsonResponse({
            'messages': [
              {
                'id': 'msg-1',
                'original_topic': 'orders.v1',
                'error_message': 'processing failed',
                'retry_count': 1,
                'max_retries': 3,
                'payload': {'order_id': '123'},
                'status': 'PENDING',
                'created_at': '2024-01-01T00:00:00Z',
                'last_retry_at': null,
              }
            ],
            'total': 1,
            'page': 1,
          }),
        );

        final resp = await client.listMessages('orders.dlq.v1', 1, 20);
        expect(resp.messages, hasLength(1));
        expect(resp.messages[0].id, equals('msg-1'));
        expect(resp.total, equals(1));
      });

      test('エラーレスポンスで DlqException を投げる', () async {
        when(() => mockClient.get(any()))
            .thenAnswer((_) async => errorResponse(500));

        expect(
          () => client.listMessages('orders.dlq.v1', 1, 20),
          throwsA(isA<DlqException>()),
        );
      });
    });

    group('getMessage', () {
      test('DLQ メッセージ詳細を返す', () async {
        when(() => mockClient.get(any())).thenAnswer(
          (_) async => jsonResponse({
            'id': 'msg-1',
            'original_topic': 'orders.v1',
            'error_message': 'error',
            'retry_count': 0,
            'max_retries': 3,
            'payload': {},
            'status': 'PENDING',
            'created_at': '2024-01-01T00:00:00Z',
            'last_retry_at': null,
          }),
        );

        final msg = await client.getMessage('msg-1');
        expect(msg.id, equals('msg-1'));
        expect(msg.status, equals(DlqStatus.pending));
      });

      test('エラーレスポンスで DlqException を投げる', () async {
        when(() => mockClient.get(any()))
            .thenAnswer((_) async => errorResponse(404));

        expect(
          () => client.getMessage('unknown'),
          throwsA(isA<DlqException>()),
        );
      });
    });

    group('retryMessage', () {
      test('再処理レスポンスを返す', () async {
        when(() => mockClient.post(
              any(),
              headers: any(named: 'headers'),
              body: any(named: 'body'),
            )).thenAnswer(
          (_) async =>
              jsonResponse({'message_id': 'msg-1', 'status': 'RETRYING'}),
        );

        final resp = await client.retryMessage('msg-1');
        expect(resp.messageId, equals('msg-1'));
        expect(resp.status, equals(DlqStatus.retrying));
      });

      test('エラーレスポンスで DlqException を投げる', () async {
        when(() => mockClient.post(
              any(),
              headers: any(named: 'headers'),
              body: any(named: 'body'),
            )).thenAnswer((_) async => errorResponse(500));

        expect(
          () => client.retryMessage('msg-1'),
          throwsA(isA<DlqException>()),
        );
      });
    });

    group('deleteMessage', () {
      test('正常に削除する', () async {
        when(() => mockClient.delete(any()))
            .thenAnswer((_) async => http.Response('', 200));

        await expectLater(client.deleteMessage('msg-1'), completes);
      });

      test('エラーレスポンスで DlqException を投げる', () async {
        when(() => mockClient.delete(any()))
            .thenAnswer((_) async => errorResponse(404));

        expect(
          () => client.deleteMessage('msg-1'),
          throwsA(isA<DlqException>()),
        );
      });
    });

    group('retryAll', () {
      test('一括再処理を実行する', () async {
        when(() => mockClient.post(
              any(),
              headers: any(named: 'headers'),
              body: any(named: 'body'),
            )).thenAnswer((_) async => http.Response('', 200));

        await expectLater(client.retryAll('orders.dlq.v1'), completes);
      });

      test('エラーレスポンスで DlqException を投げる', () async {
        when(() => mockClient.post(
              any(),
              headers: any(named: 'headers'),
              body: any(named: 'body'),
            )).thenAnswer((_) async => errorResponse(500));

        expect(
          () => client.retryAll('orders.dlq.v1'),
          throwsA(isA<DlqException>()),
        );
      });
    });
  });
}
