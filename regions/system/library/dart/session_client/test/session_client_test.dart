import 'package:test/test.dart';
import 'package:k1s0_session_client/session_client.dart';

void main() {
  late InMemorySessionClient client;

  setUp(() {
    client = InMemorySessionClient();
  });

  group('Session', () {
    test('creates with required fields', () {
      final session = Session(
        id: '1',
        userId: 'u1',
        token: 'tok',
        expiresAt: DateTime(2030),
        createdAt: DateTime(2025),
      );
      expect(session.id, equals('1'));
      expect(session.revoked, isFalse);
      expect(session.metadata, isEmpty);
    });

    test('copyWith creates modified copy', () {
      final session = Session(
        id: '1',
        userId: 'u1',
        token: 'tok',
        expiresAt: DateTime(2030),
        createdAt: DateTime(2025),
      );
      final revoked = session.copyWith(revoked: true);
      expect(revoked.revoked, isTrue);
      expect(revoked.id, equals('1'));
    });
  });

  group('CreateSessionRequest', () {
    test('creates with fields', () {
      const req = CreateSessionRequest(userId: 'u1', ttlSeconds: 3600);
      expect(req.userId, equals('u1'));
      expect(req.ttlSeconds, equals(3600));
      expect(req.metadata, isNull);
    });
  });

  group('InMemorySessionClient', () {
    test('create returns session with generated id', () async {
      final session = await client.create(
        const CreateSessionRequest(userId: 'user1', ttlSeconds: 3600),
      );
      expect(session.id, isNotEmpty);
      expect(session.userId, equals('user1'));
      expect(session.token, isNotEmpty);
      expect(session.revoked, isFalse);
    });

    test('create with metadata', () async {
      final session = await client.create(
        const CreateSessionRequest(
          userId: 'user1',
          ttlSeconds: 3600,
          metadata: {'device': 'mobile'},
        ),
      );
      expect(session.metadata['device'], equals('mobile'));
    });

    test('get returns existing session', () async {
      final created = await client.create(
        const CreateSessionRequest(userId: 'user1', ttlSeconds: 3600),
      );
      final fetched = await client.get(created.id);
      expect(fetched, isNotNull);
      expect(fetched!.userId, equals('user1'));
    });

    test('get returns null for nonexistent', () async {
      final result = await client.get('nonexistent');
      expect(result, isNull);
    });

    test('refresh updates expiry and token', () async {
      final created = await client.create(
        const CreateSessionRequest(userId: 'user1', ttlSeconds: 60),
      );
      final refreshed = await client.refresh(
        RefreshSessionRequest(id: created.id, ttlSeconds: 7200),
      );
      expect(refreshed.id, equals(created.id));
      expect(refreshed.token, isNot(equals(created.token)));
      expect(refreshed.expiresAt.isAfter(created.expiresAt), isTrue);
    });

    test('refresh throws for nonexistent session', () async {
      expect(
        () => client.refresh(const RefreshSessionRequest(id: 'bad', ttlSeconds: 60)),
        throwsStateError,
      );
    });

    test('revoke marks session as revoked', () async {
      final created = await client.create(
        const CreateSessionRequest(userId: 'user1', ttlSeconds: 3600),
      );
      await client.revoke(created.id);
      final fetched = await client.get(created.id);
      expect(fetched!.revoked, isTrue);
    });

    test('revoke nonexistent does nothing', () async {
      await client.revoke('nonexistent');
    });

    test('listUserSessions returns user sessions', () async {
      await client.create(const CreateSessionRequest(userId: 'u1', ttlSeconds: 60));
      await client.create(const CreateSessionRequest(userId: 'u1', ttlSeconds: 60));
      await client.create(const CreateSessionRequest(userId: 'u2', ttlSeconds: 60));
      final sessions = await client.listUserSessions('u1');
      expect(sessions, hasLength(2));
    });

    test('revokeAll revokes all user sessions', () async {
      await client.create(const CreateSessionRequest(userId: 'u1', ttlSeconds: 60));
      await client.create(const CreateSessionRequest(userId: 'u1', ttlSeconds: 60));
      await client.create(const CreateSessionRequest(userId: 'u2', ttlSeconds: 60));
      final count = await client.revokeAll('u1');
      expect(count, equals(2));
      final sessions = await client.listUserSessions('u1');
      expect(sessions.every((s) => s.revoked), isTrue);
    });

    test('revokeAll returns 0 for no matching sessions', () async {
      final count = await client.revokeAll('nobody');
      expect(count, equals(0));
    });
  });
}
