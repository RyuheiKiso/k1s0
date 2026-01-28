/// オフラインキュー設定
class OfflineQueueConfig {
  /// キューを有効にする
  final bool enabled;

  /// 最大キューサイズ
  final int maxSize;

  /// ストレージに永続化する
  final bool persistToStorage;

  /// ストレージキー
  final String storageKey;

  const OfflineQueueConfig({
    this.enabled = true,
    this.maxSize = 50,
    this.persistToStorage = true,
    this.storageKey = 'k1s0_realtime_queue',
  });
}
