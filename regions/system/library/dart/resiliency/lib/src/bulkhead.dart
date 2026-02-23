import 'dart:async';
import 'error.dart';

class Bulkhead {
  final int maxConcurrent;
  final Duration maxWait;
  int _current = 0;
  final List<Completer<void>> _waiters = [];

  Bulkhead({required this.maxConcurrent, required this.maxWait});

  Future<void> acquire() async {
    if (_current < maxConcurrent) {
      _current++;
      return;
    }

    final completer = Completer<void>();
    _waiters.add(completer);

    final timer = Timer(maxWait, () {
      if (!completer.isCompleted) {
        _waiters.remove(completer);
        completer.completeError(BulkheadFullError(maxConcurrent));
      }
    });

    try {
      await completer.future;
      timer.cancel();
    } catch (_) {
      timer.cancel();
      rethrow;
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
}
