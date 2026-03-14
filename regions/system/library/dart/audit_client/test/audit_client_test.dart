import 'package:test/test.dart';

import 'package:k1s0_audit_client/audit_client.dart';

void main() {
  late BufferedAuditClient client;

  setUp(() {
    client = BufferedAuditClient();
  });

  AuditEvent makeEvent(String action) => AuditEvent(
        id: 'evt-1',
        tenantId: 'tenant-1',
        actorId: 'user-1',
        action: action,
        resourceType: 'document',
        resourceId: 'doc-1',
        timestamp: DateTime.now(),
      );

  group('record', () {
    test('イベントをバッファに追加すること', () async {
      await client.record(makeEvent('create'));
      final events = await client.flush();
      expect(events, hasLength(1));
      expect(events.first.action, equals('create'));
    });
  });

  group('flush', () {
    test('バッファ内の全イベントを返すこと', () async {
      await client.record(makeEvent('create'));
      await client.record(makeEvent('update'));
      final events = await client.flush();
      expect(events, hasLength(2));
    });

    test('flush 後にバッファがクリアされること', () async {
      await client.record(makeEvent('create'));
      await client.flush();
      final events = await client.flush();
      expect(events, isEmpty);
    });
  });

  group('AuditEvent', () {
    test('全フィールドを保持すること', () {
      final event = makeEvent('delete');
      expect(event.id, equals('evt-1'));
      expect(event.tenantId, equals('tenant-1'));
      expect(event.action, equals('delete'));
      expect(event.resourceType, equals('document'));
    });
  });
}
