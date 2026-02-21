import 'error.dart';

class KafkaConfig {
  final List<String> bootstrapServers;
  final String? securityProtocol;
  final String? saslMechanism;
  final String? saslUsername;
  final String? saslPassword;

  const KafkaConfig({
    required this.bootstrapServers,
    this.securityProtocol,
    this.saslMechanism,
    this.saslUsername,
    this.saslPassword,
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
  }
}
