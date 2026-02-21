class ServiceToken {
  final String accessToken;
  final String tokenType;
  final DateTime expiresAt;
  final String? scope;

  const ServiceToken({
    required this.accessToken,
    required this.tokenType,
    required this.expiresAt,
    this.scope,
  });
}

bool isExpired(ServiceToken token) =>
    token.expiresAt.isBefore(DateTime.now());

/// Refresh 30 seconds before expiration (same as Go implementation).
bool shouldRefresh(ServiceToken token) =>
    token.expiresAt
        .isBefore(DateTime.now().add(const Duration(seconds: 30)));

String bearerHeader(ServiceToken token) => 'Bearer ${token.accessToken}';
