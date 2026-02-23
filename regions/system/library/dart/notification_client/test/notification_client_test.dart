import 'package:test/test.dart';

import 'package:k1s0_notification_client/notification_client.dart';

void main() {
  late InMemoryNotificationClient client;

  setUp(() {
    client = InMemoryNotificationClient();
  });

  group('send', () {
    test('returns sent status', () async {
      final req = NotificationRequest(
        id: 'n-1',
        channel: NotificationChannel.email,
        recipient: 'user@example.com',
        subject: 'Test',
        body: 'Hello',
      );
      final resp = await client.send(req);
      expect(resp.status, equals('sent'));
      expect(resp.id, equals('n-1'));
    });

    test('records sent notifications', () async {
      final req = NotificationRequest(
        id: 'n-1',
        channel: NotificationChannel.sms,
        recipient: '+1234567890',
        body: 'Test SMS',
      );
      await client.send(req);
      expect(client.sent, hasLength(1));
      expect(client.sent.first.channel, equals(NotificationChannel.sms));
    });
  });

  group('NotificationRequest', () {
    test('stores all fields', () {
      const req = NotificationRequest(
        id: 'n-2',
        channel: NotificationChannel.push,
        recipient: 'device-token',
        body: 'Push notification',
      );
      expect(req.id, equals('n-2'));
      expect(req.channel, equals(NotificationChannel.push));
      expect(req.subject, isNull);
    });
  });

  group('NotificationResponse', () {
    test('stores all fields', () {
      const resp = NotificationResponse(
        id: 'n-1',
        status: 'delivered',
        messageId: 'msg-123',
      );
      expect(resp.messageId, equals('msg-123'));
    });
  });
}
