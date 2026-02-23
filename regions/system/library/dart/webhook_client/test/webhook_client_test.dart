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
}
