import 'dart:convert';

import '../types/offline_queue_config.dart';
import '../utils/storage.dart';

/// オフラインキュー
///
/// ネットワーク切断中のメッセージを蓄積し、復帰時にフラッシュする。
class OfflineQueue {
  final OfflineQueueConfig config;
  final RealtimeStorage _storage;
  final Map<String, List<dynamic>> _queues = {};

  OfflineQueue({
    this.config = const OfflineQueueConfig(),
    RealtimeStorage? storage,
  }) : _storage = storage ?? RealtimeStorage();

  /// ストレージからキューを復元する
  Future<void> restore() async {
    if (!config.persistToStorage) return;

    await _storage.init();
    final raw = _storage.getRaw(config.storageKey);
    if (raw == null) return;

    try {
      final map = jsonDecode(raw) as Map<String, dynamic>;
      for (final entry in map.entries) {
        _queues[entry.key] = (entry.value as List<dynamic>).toList();
      }
    } catch (_) {
      // パース失敗は無視
    }
  }

  /// アイテムをキューに追加する
  void queue<T>(String connectionId, T item) {
    if (!config.enabled) return;

    final items = _queues.putIfAbsent(connectionId, () => []);
    if (items.length >= config.maxSize) {
      items.removeAt(0);
    }
    items.add(item);
    _persist();
  }

  /// キューをフラッシュして全アイテムを返す
  List<T> flush<T>(String connectionId) {
    final items = _queues.remove(connectionId);
    _persist();
    if (items == null) return [];
    return items.cast<T>();
  }

  /// キュー内のアイテムを取得する（削除しない）
  List<T> getQueuedItems<T>(String connectionId) {
    return (_queues[connectionId] ?? []).cast<T>();
  }

  /// キューをクリアする
  void clearQueue(String connectionId) {
    _queues.remove(connectionId);
    _persist();
  }

  /// 全キューをクリアする
  void clearAll() {
    _queues.clear();
    _persist();
  }

  Future<void> _persist() async {
    if (!config.persistToStorage) return;
    await _storage.set(config.storageKey, _queues);
  }
}
