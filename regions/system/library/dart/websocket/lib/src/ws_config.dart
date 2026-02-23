class WsConfig {
  final String url;
  final bool reconnect;
  final int maxReconnectAttempts;
  final Duration reconnectDelay;
  final Duration? pingInterval;

  const WsConfig({
    required this.url,
    this.reconnect = true,
    this.maxReconnectAttempts = 5,
    this.reconnectDelay = const Duration(seconds: 1),
    this.pingInterval,
  });

  static WsConfig get defaults => const WsConfig(
        url: 'ws://localhost',
        reconnect: true,
        maxReconnectAttempts: 5,
        reconnectDelay: Duration(seconds: 1),
      );
}
