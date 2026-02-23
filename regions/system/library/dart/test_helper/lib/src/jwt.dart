import 'dart:convert';

/// テスト用 JWT クレーム。
class TestClaims {
  final String sub;
  final List<String> roles;
  final String? tenantId;
  final int iat;
  final int exp;

  TestClaims({
    required this.sub,
    this.roles = const [],
    this.tenantId,
    int? iat,
    int? exp,
  })  : iat = iat ?? DateTime.now().millisecondsSinceEpoch ~/ 1000,
        exp = exp ??
            (DateTime.now().millisecondsSinceEpoch ~/ 1000 + 3600);

  Map<String, dynamic> toJson() {
    final map = <String, dynamic>{
      'sub': sub,
      'roles': roles,
      'iat': iat,
      'exp': exp,
    };
    if (tenantId != null) {
      map['tenant_id'] = tenantId;
    }
    return map;
  }

  factory TestClaims.fromJson(Map<String, dynamic> json) {
    return TestClaims(
      sub: json['sub'] as String,
      roles: (json['roles'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          [],
      tenantId: json['tenant_id'] as String?,
      iat: json['iat'] as int?,
      exp: json['exp'] as int?,
    );
  }
}

/// テスト用 JWT トークン生成ヘルパー。
class JwtTestHelper {
  final String secret;

  JwtTestHelper({required this.secret});

  /// 管理者トークンを生成する。
  String createAdminToken() {
    return createToken(TestClaims(sub: 'admin', roles: ['admin']));
  }

  /// ユーザートークンを生成する。
  String createUserToken(String userId, List<String> roles) {
    return createToken(TestClaims(sub: userId, roles: roles));
  }

  /// カスタムクレームでトークンを生成する。
  String createToken(TestClaims claims) {
    final header = _base64UrlEncode('{"alg":"HS256","typ":"JWT"}');
    final payload = _base64UrlEncode(jsonEncode(claims.toJson()));
    final signingInput = '$header.$payload';
    final signature = _base64UrlEncode('$signingInput:$secret');
    return '$signingInput.$signature';
  }

  /// トークンのペイロードをデコードしてクレームを返す。
  TestClaims? decodeClaims(String token) {
    final parts = token.split('.');
    if (parts.length != 3) return null;
    try {
      final payloadJson = _base64UrlDecode(parts[1]);
      final map = jsonDecode(payloadJson) as Map<String, dynamic>;
      return TestClaims.fromJson(map);
    } catch (_) {
      return null;
    }
  }

  String _base64UrlEncode(String input) {
    return base64Url.encode(utf8.encode(input)).replaceAll('=', '');
  }

  String _base64UrlDecode(String input) {
    var padded = input;
    final mod = padded.length % 4;
    if (mod != 0) {
      padded += '=' * (4 - mod);
    }
    return utf8.decode(base64Url.decode(padded));
  }
}
