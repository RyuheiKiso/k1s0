import 'session.dart';

abstract class SessionClient {
  Future<Session> create(CreateSessionRequest req);
  Future<Session?> get(String id);
  Future<Session> refresh(RefreshSessionRequest req);
  Future<void> revoke(String id);
  Future<List<Session>> listUserSessions(String userId);
  Future<int> revokeAll(String userId);
}

class InMemorySessionClient implements SessionClient {
  final _sessions = <String, Session>{};
  int _counter = 0;

  @override
  Future<Session> create(CreateSessionRequest req) async {
    _counter++;
    final id = _counter.toString();
    final now = DateTime.now();
    final session = Session(
      id: id,
      userId: req.userId,
      token: 'tok-$id',
      expiresAt: now.add(Duration(seconds: req.ttlSeconds)),
      createdAt: now,
      metadata: req.metadata ?? {},
    );
    _sessions[id] = session;
    return session;
  }

  @override
  Future<Session?> get(String id) async {
    return _sessions[id];
  }

  @override
  Future<Session> refresh(RefreshSessionRequest req) async {
    final existing = _sessions[req.id];
    if (existing == null) {
      throw StateError('Session not found: ${req.id}');
    }
    final refreshed = existing.copyWith(
      expiresAt: DateTime.now().add(Duration(seconds: req.ttlSeconds)),
      token: 'tok-${req.id}-refreshed',
    );
    _sessions[req.id] = refreshed;
    return refreshed;
  }

  @override
  Future<void> revoke(String id) async {
    final existing = _sessions[id];
    if (existing == null) return;
    _sessions[id] = existing.copyWith(revoked: true);
  }

  @override
  Future<List<Session>> listUserSessions(String userId) async {
    return _sessions.values
        .where((s) => s.userId == userId)
        .toList();
  }

  @override
  Future<int> revokeAll(String userId) async {
    var count = 0;
    for (final entry in _sessions.entries.toList()) {
      if (entry.value.userId == userId && !entry.value.revoked) {
        _sessions[entry.key] = entry.value.copyWith(revoked: true);
        count++;
      }
    }
    return count;
  }
}
