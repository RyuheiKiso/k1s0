import 'package:test/test.dart';
import 'package:k1s0_session_client/session_client.dart';

void main() {
  late InMemorySessionClient client;

  setUp(() {
    client = InMemorySessionClient();
  });

  group('Session', () {
    test('必須フィールドでセッションが生成されること', () {
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

    test('copyWithで変更されたコピーが生成されること', () {
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
    test('フィールドを指定してリクエストが生成されること', () {
      const req = CreateSessionRequest(userId: 'u1', ttlSeconds: 3600);
      expect(req.userId, equals('u1'));
      expect(req.ttlSeconds, equals(3600));
      expect(req.metadata, isNull);
    });
  });

  group('InMemorySessionClient', () {
    test('createで生成IDを持つセッションが返ること', () async {
      final session = await client.create(
        const CreateSessionRequest(userId: 'user1', ttlSeconds: 3600),
      );
      expect(session.id, isNotEmpty);
      expect(session.userId, equals('user1'));
      expect(session.token, isNotEmpty);
      expect(session.revoked, isFalse);
    });

    test('メタデータ付きでセッションが作成されること', () async {
      final session = await client.create(
        const CreateSessionRequest(
          userId: 'user1',
          ttlSeconds: 3600,
          metadata: {'device': 'mobile'},
        ),
      );
      expect(session.metadata['device'], equals('mobile'));
    });

    test('getで既存のセッションが返ること', () async {
      final created = await client.create(
        const CreateSessionRequest(userId: 'user1', ttlSeconds: 3600),
      );
      final fetched = await client.get(created.id);
      expect(fetched, isNotNull);
      expect(fetched!.userId, equals('user1'));
    });

    test('存在しないセッションのgetでnullが返ること', () async {
      final result = await client.get('nonexistent');
      expect(result, isNull);
    });

    test('refreshで有効期限とトークンが更新されること', () async {
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

    test('存在しないセッションのrefreshで例外がスローされること', () async {
      expect(
        () => client.refresh(const RefreshSessionRequest(id: 'bad', ttlSeconds: 60)),
        throwsStateError,
      );
    });

    test('revokeでセッションが失効状態になること', () async {
      final created = await client.create(
        const CreateSessionRequest(userId: 'user1', ttlSeconds: 3600),
      );
      await client.revoke(created.id);
      final fetched = await client.get(created.id);
      expect(fetched!.revoked, isTrue);
    });

    test('存在しないセッションのrevokeが何もしないこと', () async {
      await client.revoke('nonexistent');
    });

    test('listUserSessionsでユーザーのセッション一覧が返ること', () async {
      await client.create(const CreateSessionRequest(userId: 'u1', ttlSeconds: 60));
      await client.create(const CreateSessionRequest(userId: 'u1', ttlSeconds: 60));
      await client.create(const CreateSessionRequest(userId: 'u2', ttlSeconds: 60));
      final sessions = await client.listUserSessions('u1');
      expect(sessions, hasLength(2));
    });

    test('revokeAllでユーザーの全セッションが失効されること', () async {
      await client.create(const CreateSessionRequest(userId: 'u1', ttlSeconds: 60));
      await client.create(const CreateSessionRequest(userId: 'u1', ttlSeconds: 60));
      await client.create(const CreateSessionRequest(userId: 'u2', ttlSeconds: 60));
      final count = await client.revokeAll('u1');
      expect(count, equals(2));
      final sessions = await client.listUserSessions('u1');
      expect(sessions.every((s) => s.revoked), isTrue);
    });

    test('該当セッションがない場合にrevokeAllが0を返すこと', () async {
      final count = await client.revokeAll('nobody');
      expect(count, equals(0));
    });
  });
}
