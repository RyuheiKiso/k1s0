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
    test('adds event to buffer', () async {
      await client.record(makeEvent('create'));
      final events = await client.flush();
      expect(events, hasLength(1));
      expect(events.first.action, equals('create'));
    });
  });

  group('flush', () {
    test('returns all buffered events', () async {
      await client.record(makeEvent('create'));
      await client.record(makeEvent('update'));
      final events = await client.flush();
      expect(events, hasLength(2));
    });

    test('clears buffer after flush', () async {
      await client.record(makeEvent('create'));
      await client.flush();
      final events = await client.flush();
      expect(events, isEmpty);
    });
  });

  group('AuditEvent', () {
    test('stores all fields', () {
      final event = makeEvent('delete');
      expect(event.id, equals('evt-1'));
      expect(event.tenantId, equals('tenant-1'));
      expect(event.action, equals('delete'));
      expect(event.resourceType, equals('document'));
    });
  });
}
