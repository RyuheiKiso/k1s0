import 'dart:math';

/// テスト用フィクスチャビルダー。
class FixtureBuilder {
  static final _random = Random();

  /// ランダム UUID v4 を生成する。
  static String uuid() {
    final bytes = List<int>.generate(16, (_) => _random.nextInt(256));
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return '${hex.substring(0, 8)}-${hex.substring(8, 12)}-'
        '${hex.substring(12, 16)}-${hex.substring(16, 20)}-'
        '${hex.substring(20, 32)}';
  }

  /// ランダムなテスト用メールアドレスを生成する。
  static String email() => 'test-${uuid().substring(0, 8)}@example.com';

  /// ランダムなテスト用ユーザー名を生成する。
  static String name() => 'user-${uuid().substring(0, 8)}';

  /// 指定範囲のランダム整数を生成する。
  static int intValue({int min = 0, int max = 100}) {
    if (min >= max) return min;
    return min + _random.nextInt(max - min);
  }

  /// テスト用テナント ID を生成する。
  static String tenantId() => 'tenant-${uuid().substring(0, 8)}';
}
