import 'dart:async';

class BulkheadConfig {
  final int maxConcurrentCalls;
  final Duration maxWaitDuration;

  const BulkheadConfig({
    required this.maxConcurrentCalls,
    required this.maxWaitDuration,
  });
}

class BulkheadFullException implements Exception {
  const BulkheadFullException();

  @override
  String toString() => 'BulkheadFullException: bulkhead is full';
}

class Bulkhead {
  final BulkheadConfig config;

  int _current = 0;
  final List<Completer<void>> _waiters = [];

  Bulkhead(this.config);

  Future<void> acquire() async {
    if (_current < config.maxConcurrentCalls) {
      _current++;
      return;
    }

    final completer = Completer<void>();
    _waiters.add(completer);

    final timer = Timer(config.maxWaitDuration, () {
      if (!completer.isCompleted) {
        _waiters.remove(completer);
        completer.completeError(const BulkheadFullException());
      }
    });

    try {
      await completer.future;
    } finally {
      timer.cancel();
    }
  }

  void release() {
    if (_waiters.isNotEmpty) {
      final waiter = _waiters.removeAt(0);
      if (!waiter.isCompleted) {
        waiter.complete();
      }
    } else {
      _current--;
    }
  }

  Future<T> call<T>(Future<T> Function() fn) async {
    await acquire();
    try {
      return await fn();
    } finally {
      release();
    }
  }
}
