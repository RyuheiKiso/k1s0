import 'error.dart';

class KafkaConfig {
  final List<String> bootstrapServers;
  final String? securityProtocol;
  final String? saslMechanism;
  final String? saslUsername;
  final String? saslPassword;

  /// コンシューマーグループID
  final String consumerGroup;

  /// 接続タイムアウト（ミリ秒）。デフォルト: 5000
  final int connectionTimeoutMs;

  /// リクエストタイムアウト（ミリ秒）。デフォルト: 30000
  final int requestTimeoutMs;

  /// 最大メッセージサイズ（バイト）。デフォルト: 1000000
  final int maxMessageBytes;

  const KafkaConfig({
    required this.bootstrapServers,
    this.securityProtocol,
    this.saslMechanism,
    this.saslUsername,
    this.saslPassword,
    required this.consumerGroup,
    this.connectionTimeoutMs = 5000,
    this.requestTimeoutMs = 30000,
    this.maxMessageBytes = 1000000,
  });

  String bootstrapServersString() => bootstrapServers.join(',');

  bool usesTLS() =>
      securityProtocol == 'SSL' || securityProtocol == 'SASL_SSL';

  void validate() {
    if (bootstrapServers.isEmpty) {
      throw KafkaError('bootstrap servers must not be empty');
    }
    const validProtocols = {'PLAINTEXT', 'SSL', 'SASL_PLAINTEXT', 'SASL_SSL'};
    if (securityProtocol != null &&
        !validProtocols.contains(securityProtocol)) {
      throw KafkaError('invalid security protocol: $securityProtocol');
    }
    // コンシューマーグループIDが空でないことを検証する
    if (consumerGroup.isEmpty) {
      throw KafkaError('consumer group must not be empty');
    }
    // 接続タイムアウトが正の値であることを検証する
    if (connectionTimeoutMs <= 0) {
      throw KafkaError('connection timeout must be positive');
    }
    // リクエストタイムアウトが正の値であることを検証する
    if (requestTimeoutMs <= 0) {
      throw KafkaError('request timeout must be positive');
    }
    // 最大メッセージサイズが正の値であることを検証する
    if (maxMessageBytes <= 0) {
      throw KafkaError('max message bytes must be positive');
    }
  }
}
