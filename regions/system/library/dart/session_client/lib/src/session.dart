class Session {
  final String id;
  final String userId;
  final String token;
  final DateTime expiresAt;
  final DateTime createdAt;
  final bool revoked;
  final Map<String, String> metadata;

  const Session({
    required this.id,
    required this.userId,
    required this.token,
    required this.expiresAt,
    required this.createdAt,
    this.revoked = false,
    this.metadata = const {},
  });

  Session copyWith({
    String? id,
    String? userId,
    String? token,
    DateTime? expiresAt,
    DateTime? createdAt,
    bool? revoked,
    Map<String, String>? metadata,
  }) {
    return Session(
      id: id ?? this.id,
      userId: userId ?? this.userId,
      token: token ?? this.token,
      expiresAt: expiresAt ?? this.expiresAt,
      createdAt: createdAt ?? this.createdAt,
      revoked: revoked ?? this.revoked,
      metadata: metadata ?? this.metadata,
    );
  }
}

class CreateSessionRequest {
  final String userId;
  final int ttlSeconds;
  final Map<String, String>? metadata;

  const CreateSessionRequest({
    required this.userId,
    required this.ttlSeconds,
    this.metadata,
  });
}

class RefreshSessionRequest {
  final String id;
  final int ttlSeconds;

  const RefreshSessionRequest({
    required this.id,
    required this.ttlSeconds,
  });
}
