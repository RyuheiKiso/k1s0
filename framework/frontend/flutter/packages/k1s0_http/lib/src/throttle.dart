import 'dart:async';
import 'dart:collection';

/// リクエストスロットル設定
class ThrottleConfig {
  /// 1秒あたりの最大リクエスト数
  final int maxRequestsPerSecond;

  /// 同時接続数の上限
  final int maxConcurrent;

  /// キュー上限（超過時は即座にリジェクト）
  final int maxQueueSize;

  const ThrottleConfig({
    this.maxRequestsPerSecond = 10,
    this.maxConcurrent = 5,
    this.maxQueueSize = 50,
  });
}

/// リクエストスロットル
///
/// トークンバケット + 同時接続制限 + キュー上限によるリクエストレート制御。
class RequestThrottle {
  /// Current configuration.
  final ThrottleConfig config;

  int _tokens;
  int _activeCount = 0;
  int _allowed = 0;
  int _rejected = 0;
  final Queue<Completer<void>> _queue = Queue();
  late final Timer _timer;

  RequestThrottle({ThrottleConfig? config})
      : config = config ?? const ThrottleConfig(),
        _tokens = (config ?? const ThrottleConfig()).maxRequestsPerSecond {
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      _tokens = this.config.maxRequestsPerSecond;
      _processQueue();
    });
  }

  /// Acquires a slot. Returns immediately if available, otherwise queues.
  /// Throws [StateError] if the queue is full.
  Future<void> acquire() async {
    if (_tokens > 0 && _activeCount < config.maxConcurrent) {
      _tokens--;
      _activeCount++;
      _allowed++;
      return;
    }
    if (_queue.length >= config.maxQueueSize) {
      _rejected++;
      throw StateError('Request throttle queue full');
    }
    final completer = Completer<void>();
    _queue.add(completer);
    return completer.future;
  }

  /// Releases a slot after the request completes.
  void release() {
    _activeCount--;
    _processQueue();
  }

  /// Returns current throttle statistics.
  ({int allowed, int rejected, int queued, int active}) get stats => (
        allowed: _allowed,
        rejected: _rejected,
        queued: _queue.length,
        active: _activeCount,
      );

  /// Disposes of the throttle, cancelling the refill timer and rejecting
  /// all queued requests.
  void dispose() {
    _timer.cancel();
    for (final c in _queue) {
      c.completeError(StateError('Throttle disposed'));
    }
    _queue.clear();
  }

  void _processQueue() {
    while (_queue.isNotEmpty &&
        _tokens > 0 &&
        _activeCount < config.maxConcurrent) {
      _tokens--;
      _activeCount++;
      _allowed++;
      _queue.removeFirst().complete();
    }
  }
}
