import 'package:k1s0_bulkhead/bulkhead.dart' as standalone;

import 'error.dart';
import 'policy.dart';

/// スタンドアロンの [standalone.Bulkhead] をラップし、
/// このライブラリのエラー型 ([BulkheadFullError]) に変換するアダプター。
class Bulkhead {
  final standalone.Bulkhead _inner;

  /// 許容する最大同時実行数。
  final int maxConcurrent;

  Bulkhead({required this.maxConcurrent, required Duration maxWait})
      : _inner = standalone.Bulkhead(standalone.BulkheadConfig(
          maxConcurrentCalls: maxConcurrent,
          maxWaitDuration: maxWait,
        ));

  /// [BulkheadConfig] からインスタンスを生成するファクトリーコンストラクタ。
  factory Bulkhead.fromConfig(BulkheadConfig config) => Bulkhead(
        maxConcurrent: config.maxConcurrentCalls,
        maxWait: config.maxWaitDuration,
      );

  /// 実行スロットを取得する。スロットが満杯の場合は [BulkheadFullError] を投げる。
  Future<void> acquire() async {
    try {
      await _inner.acquire();
    } on standalone.BulkheadFullException {
      throw BulkheadFullError(maxConcurrent);
    }
  }

  /// 実行スロットを解放する。[acquire] の呼び出し後に必ず呼ぶこと。
  void release() {
    _inner.release();
  }
}
