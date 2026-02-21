import 'error.dart';

final _topicNameRegex = RegExp(
    r'^k1s0\.(system|business|service)\.[a-z0-9-]+\.[a-z0-9-]+\.v[0-9]+$');

class TopicConfig {
  final String name;
  final int? partitions;
  final int? replicationFactor;
  final int? retentionMs;

  const TopicConfig({
    required this.name,
    this.partitions,
    this.replicationFactor,
    this.retentionMs,
  });

  void validateName() {
    if (name.isEmpty) {
      throw KafkaError('topic name must not be empty');
    }
    if (!_topicNameRegex.hasMatch(name)) {
      throw KafkaError('invalid topic name: $name');
    }
  }

  String tier() {
    final match = _topicNameRegex.firstMatch(name);
    return match?.group(1) ?? '';
  }
}
