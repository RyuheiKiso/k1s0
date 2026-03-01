import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:test/test.dart';

import 'package:k1s0_webhook_client/webhook_client.dart';

void main() {
  group('generateSignature', () {
    test('produces consistent signature', () {
      final s1 = generateSignature('secret', 'body');
      final s2 = generateSignature('secret', 'body');
      expect(s1, equals(s2));
    });

    test('different secrets produce different signatures', () {
      final s1 = generateSignature('secret1', 'body');
      final s2 = generateSignature('secret2', 'body');
      expect(s1, isNot(equals(s2)));
    });
  });

  group('verifySignature', () {
    test('returns true for valid signature', () {
      final sig = generateSignature('key', 'data');
      expect(verifySignature('key', 'data', sig), isTrue);
    });

    test('returns false for invalid signature', () {
      expect(verifySignature('key', 'data', 'wrong'), isFalse);
    });
  });

  group('InMemoryWebhookClient', () {
    test('records sent webhooks', () async {
      final client = InMemoryWebhookClient();
      final payload = WebhookPayload(
        eventType: 'test',
        timestamp: '2026-01-01T00:00:00Z',
        data: {'key': 'value'},
      );
      final status = await client.send('https://example.com/hook', payload);
      expect(status, equals(200));
      expect(client.sent, hasLength(1));
    });
  });

  group('WebhookPayload', () {
    test('stores fields', () {
      const payload = WebhookPayload(
        eventType: 'user.created',
        timestamp: '2026-01-01T00:00:00Z',
        data: {'id': '123'},
      );
      expect(payload.eventType, equals('user.created'));
    });
  });

  group('HttpWebhookClient', () {
    final testPayload = WebhookPayload(
      eventType: 'user.created',
      timestamp: '2026-01-01T00:00:00Z',
      data: {'userId': '123'},
    );

    // No-op delay for testing (no actual waiting)
    Future<void> noDelay(Duration _) async {}

    test('returns status code on success', () async {
      final mockClient = MockClient((_) async => http.Response('', 200));
      final client = HttpWebhookClient(
        httpClient: mockClient,
        delayFn: noDelay,
      );

      final status = await client.send('https://example.com/hook', testPayload);
      expect(status, equals(200));
    });

    test('includes Idempotency-Key header', () async {
      String? capturedKey;
      final mockClient = MockClient((request) async {
        capturedKey = request.headers['idempotency-key'];
        return http.Response('', 200);
      });
      final client = HttpWebhookClient(
        httpClient: mockClient,
        uuidGenerator: () => 'test-uuid-1234',
        delayFn: noDelay,
      );

      await client.send('https://example.com/hook', testPayload);
      expect(capturedKey, equals('test-uuid-1234'));
    });

    test('includes X-K1s0-Signature header when secret is set', () async {
      String? capturedSignature;
      String? capturedBody;
      final mockClient = MockClient((request) async {
        capturedSignature = request.headers['x-k1s0-signature'];
        capturedBody = request.body;
        return http.Response('', 200);
      });
      final client = HttpWebhookClient(
        secret: 'my-secret',
        httpClient: mockClient,
        delayFn: noDelay,
      );

      await client.send('https://example.com/hook', testPayload);
      final expectedSig = generateSignature('my-secret', capturedBody!);
      expect(capturedSignature, equals(expectedSig));
    });

    test('does not include X-K1s0-Signature header when secret is not set',
        () async {
      String? capturedSignature;
      final mockClient = MockClient((request) async {
        capturedSignature = request.headers['x-k1s0-signature'];
        return http.Response('', 200);
      });
      final client = HttpWebhookClient(
        httpClient: mockClient,
        delayFn: noDelay,
      );

      await client.send('https://example.com/hook', testPayload);
      expect(capturedSignature, isNull);
    });

    test('retries on 5xx responses', () async {
      int callCount = 0;
      final mockClient = MockClient((_) async {
        callCount++;
        if (callCount <= 2) {
          return http.Response('', 500);
        }
        return http.Response('', 200);
      });
      final client = HttpWebhookClient(
        config: WebhookConfig(
          maxRetries: 3,
          initialBackoffMs: 1,
          maxBackoffMs: 10,
        ),
        httpClient: mockClient,
        delayFn: noDelay,
      );

      final status = await client.send('https://example.com/hook', testPayload);
      expect(status, equals(200));
      expect(callCount, equals(3));
    });

    test('retries on 429 responses', () async {
      int callCount = 0;
      final mockClient = MockClient((_) async {
        callCount++;
        if (callCount <= 1) {
          return http.Response('', 429);
        }
        return http.Response('', 200);
      });
      final client = HttpWebhookClient(
        config: WebhookConfig(
          maxRetries: 3,
          initialBackoffMs: 1,
          maxBackoffMs: 10,
        ),
        httpClient: mockClient,
        delayFn: noDelay,
      );

      final status = await client.send('https://example.com/hook', testPayload);
      expect(status, equals(200));
      expect(callCount, equals(2));
    });

    test('does not retry on non-429 4xx responses', () async {
      int callCount = 0;
      final mockClient = MockClient((_) async {
        callCount++;
        return http.Response('', 400);
      });
      final client = HttpWebhookClient(
        config: WebhookConfig(
          maxRetries: 3,
          initialBackoffMs: 1,
          maxBackoffMs: 10,
        ),
        httpClient: mockClient,
        delayFn: noDelay,
      );

      final status = await client.send('https://example.com/hook', testPayload);
      expect(status, equals(400));
      expect(callCount, equals(1));
    });

    test('throws MAX_RETRIES_EXCEEDED error when retries exhausted', () async {
      final mockClient = MockClient((_) async => http.Response('', 500));
      final client = HttpWebhookClient(
        config: WebhookConfig(
          maxRetries: 2,
          initialBackoffMs: 1,
          maxBackoffMs: 10,
        ),
        httpClient: mockClient,
        delayFn: noDelay,
      );

      try {
        await client.send('https://example.com/hook', testPayload);
        fail('Expected WebhookError');
      } on WebhookError catch (e) {
        expect(e.code, equals(WebhookErrorCode.maxRetriesExceeded));
      }
    });

    test('retries on network errors', () async {
      int callCount = 0;
      final mockClient = MockClient((_) async {
        callCount++;
        if (callCount <= 2) {
          throw Exception('Network error');
        }
        return http.Response('', 200);
      });
      final client = HttpWebhookClient(
        config: WebhookConfig(
          maxRetries: 3,
          initialBackoffMs: 1,
          maxBackoffMs: 10,
        ),
        httpClient: mockClient,
        delayFn: noDelay,
      );

      final status = await client.send('https://example.com/hook', testPayload);
      expect(status, equals(200));
      expect(callCount, equals(3));
    });

    test('uses same Idempotency-Key across retries', () async {
      final capturedKeys = <String>[];
      final mockClient = MockClient((request) async {
        capturedKeys.add(request.headers['idempotency-key'] ?? '');
        return http.Response('', 500);
      });
      final client = HttpWebhookClient(
        config: WebhookConfig(
          maxRetries: 2,
          initialBackoffMs: 1,
          maxBackoffMs: 10,
        ),
        httpClient: mockClient,
        uuidGenerator: () => 'fixed-uuid',
        delayFn: noDelay,
      );

      try {
        await client.send('https://example.com/hook', testPayload);
      } on WebhookError {
        // expected
      }

      expect(capturedKeys, hasLength(3));
      expect(capturedKeys.toSet(), hasLength(1));
      expect(capturedKeys.first, equals('fixed-uuid'));
    });
  });
}
