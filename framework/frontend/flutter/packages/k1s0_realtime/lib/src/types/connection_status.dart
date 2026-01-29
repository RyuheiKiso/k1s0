/// 接続状態
enum ConnectionStatus {
  /// 接続中
  connecting,

  /// 接続済み
  connected,

  /// 切断中
  disconnecting,

  /// 切断済み
  disconnected,
}
