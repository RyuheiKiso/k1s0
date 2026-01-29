/// ハートビート設定
class HeartbeatConfig {
  /// ハートビートを有効にする
  final bool enabled;

  /// 送信間隔
  final Duration interval;

  /// 応答タイムアウト
  final Duration timeout;

  /// 送信メッセージ
  final String message;

  /// レスポンス検証関数
  final bool Function(dynamic)? validateResponse;

  const HeartbeatConfig({
    this.enabled = true,
    this.interval = const Duration(seconds: 30),
    this.timeout = const Duration(seconds: 5),
    this.message = '{"type":"ping"}',
    this.validateResponse,
  });
}
