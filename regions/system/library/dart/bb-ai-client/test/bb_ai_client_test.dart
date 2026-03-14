import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:test/test.dart';

import 'package:k1s0_bb_ai_client/bb_ai_client.dart';

void main() {
  // InMemoryAiClient のテスト
  group('InMemoryAiClient', () {
    late InMemoryAiClient client;

    setUp(() {
      client = InMemoryAiClient();
    });

    test('デフォルトでモックレスポンスを返すこと', () async {
      final res = await client.complete(CompleteRequest(
        model: 'test-model',
        messages: [const ChatMessage(role: 'user', content: 'hello')],
      ));
      expect(res.id, equals('mock-id'));
      expect(res.model, equals('test-model'));
      expect(res.content, equals('mock response'));
    });

    test('カスタム complete 関数を使用できること', () async {
      final customClient = InMemoryAiClient(
        complete: (req) async => CompleteResponse(
          id: 'custom-id',
          model: req.model,
          content: 'custom',
          usage: const Usage(inputTokens: 10, outputTokens: 20),
        ),
      );
      final res = await customClient.complete(CompleteRequest(
        model: 'gpt-4',
        messages: [const ChatMessage(role: 'user', content: 'test')],
      ));
      expect(res.id, equals('custom-id'));
      expect(res.usage.inputTokens, equals(10));
    });

    test('embed でデフォルト空埋め込みを返すこと', () async {
      final res = await client.embed(EmbedRequest(
        model: 'embed-model',
        texts: ['hello', 'world'],
      ));
      expect(res.model, equals('embed-model'));
      expect(res.embeddings, hasLength(2));
    });

    test('listModels でデフォルト空配列を返すこと', () async {
      final models = await client.listModels();
      expect(models, isEmpty);
    });
  });

  // HttpAiClient のテスト
  group('HttpAiClient', () {
    test('complete で正しいエンドポイントにPOSTすること', () async {
      final mockClient = MockClient((request) async {
        expect(request.url.path, equals('/v1/complete'));
        expect(request.method, equals('POST'));
        return http.Response(
          json.encode({
            'id': 'resp-id',
            'model': 'claude-3',
            'content': 'Hello!',
            'usage': {'input_tokens': 5, 'output_tokens': 10},
          }),
          200,
        );
      });

      final client = HttpAiClient(
        baseUrl: 'https://api.example.com',
        apiKey: 'test-key',
        httpClient: mockClient,
      );
      final res = await client.complete(CompleteRequest(
        model: 'claude-3',
        messages: [const ChatMessage(role: 'user', content: 'Hi')],
      ));

      expect(res.content, equals('Hello!'));
      expect(res.usage.inputTokens, equals(5));
      expect(res.usage.outputTokens, equals(10));
    });

    test('embed で正しいエンドポイントにPOSTすること', () async {
      final mockClient = MockClient((request) async {
        expect(request.url.path, equals('/v1/embed'));
        return http.Response(
          json.encode({
            'model': 'embed-v1',
            'embeddings': [[0.1, 0.2], [0.3, 0.4]],
          }),
          200,
        );
      });

      final client = HttpAiClient(
        baseUrl: 'https://api.example.com',
        httpClient: mockClient,
      );
      final res = await client.embed(EmbedRequest(
        model: 'embed-v1',
        texts: ['a', 'b'],
      ));
      expect(res.embeddings, hasLength(2));
    });

    test('listModels で正しいエンドポイントにGETすること', () async {
      final mockClient = MockClient((request) async {
        expect(request.url.path, equals('/v1/models'));
        expect(request.method, equals('GET'));
        return http.Response(
          json.encode([
            {'id': 'model-1', 'name': 'Model One', 'description': 'desc'},
          ]),
          200,
        );
      });

      final client = HttpAiClient(
        baseUrl: 'https://api.example.com',
        httpClient: mockClient,
      );
      final models = await client.listModels();
      expect(models, hasLength(1));
      expect(models.first.id, equals('model-1'));
    });

    test('APIエラー時に AiClientError をスローすること', () async {
      final mockClient = MockClient((_) async => http.Response('Unauthorized', 401));

      final client = HttpAiClient(
        baseUrl: 'https://api.example.com',
        httpClient: mockClient,
      );
      expect(
        () => client.complete(CompleteRequest(
          model: 'test',
          messages: [const ChatMessage(role: 'user', content: 'hi')],
        )),
        throwsA(isA<AiClientError>()),
      );
    });
  });

  // AiClientError のテスト
  group('AiClientError', () {
    test('正しいフィールドを持つこと', () {
      const err = AiClientError('test error', statusCode: 500);
      expect(err.message, equals('test error'));
      expect(err.statusCode, equals(500));
      expect(err.toString(), contains('500'));
    });
  });
}
