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

  /// トピック設定の全フィールドを検証する。
  /// L-004 監査対応: name に加えて partitions/replicationFactor/retentionMs の正値チェックを追加する。
  /// null 値は「未指定」として許容する（ブローカーデフォルトを使用する意図）。
  void validate() {
    // トピック名の検証を先に実行する
    validateName();
    // パーティション数が指定されている場合は正の整数であることを検証する
    if (partitions != null && partitions! <= 0) {
      throw ArgumentError('partitions は正の整数でなければなりません: $partitions');
    }
    // レプリケーションファクターが指定されている場合は正の整数であることを検証する
    if (replicationFactor != null && replicationFactor! <= 0) {
      throw ArgumentError(
          'replicationFactor は正の整数でなければなりません: $replicationFactor');
    }
    // 保持時間（ms）が指定されている場合は正の整数であることを検証する
    if (retentionMs != null && retentionMs! <= 0) {
      throw ArgumentError('retentionMs は正の整数でなければなりません: $retentionMs');
    }
  }

  String tier() {
    final match = _topicNameRegex.firstMatch(name);
    return match?.group(1) ?? '';
  }

  /// tier 別デフォルトパーティション数を返す。
  int defaultPartitionsForTier() {
    return TopicConfig.partitionsForTier(tier());
  }

  /// tier 名からデフォルトパーティション数を返す。
  ///
  /// - system tier: 6 パーティション
  /// - business tier: 6 パーティション
  /// - service tier / その他: 3 パーティション
  static int partitionsForTier(String tier) {
    switch (tier) {
      case 'system':
      case 'business':
        return 6;
      default:
        return 3;
    }
  }
}
