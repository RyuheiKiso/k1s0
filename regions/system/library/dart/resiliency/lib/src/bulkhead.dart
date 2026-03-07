import 'package:k1s0_bulkhead/bulkhead.dart' as standalone;

import 'error.dart';
import 'policy.dart';

class Bulkhead {
  final standalone.Bulkhead _inner;
  final int maxConcurrent;

  Bulkhead({required this.maxConcurrent, required Duration maxWait})
      : _inner = standalone.Bulkhead(standalone.BulkheadConfig(
          maxConcurrentCalls: maxConcurrent,
          maxWaitDuration: maxWait,
        ));

  factory Bulkhead.fromConfig(BulkheadConfig config) => Bulkhead(
        maxConcurrent: config.maxConcurrentCalls,
        maxWait: config.maxWaitDuration,
      );

  Future<void> acquire() async {
    try {
      await _inner.acquire();
    } on standalone.BulkheadFullException {
      throw BulkheadFullError(maxConcurrent);
    }
  }

  void release() {
    _inner.release();
  }
}
