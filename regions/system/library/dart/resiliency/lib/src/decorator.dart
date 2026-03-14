import 'dart:async';
import 'dart:math' as math;

import 'package:k1s0_circuit_breaker/circuit_breaker.dart' as standalone;

import 'bulkhead.dart';
import 'error.dart';
import 'policy.dart';

/// [ResiliencyPolicy] に基づき、リトライ・サーキットブレーカー・バルクヘッド・
/// タイムアウトを組み合わせて処理を実行するデコレーター。
///
/// 実行順序:
/// 1. サーキットブレーカーの状態確認
/// 2. バルクヘッドによるスロット取得
/// 3. タイムアウト付きでリトライ実行
/// 4. バルクヘッドスロット解放
class ResiliencyDecorator {
  final ResiliencyPolicy policy;

  /// バルクヘッドポリシーが有効な場合に使用するバルクヘッドインスタンス。
  final Bulkhead? _bulkhead;

  /// サーキットブレーカーポリシーが有効な場合に使用するサーキットブレーカー。
  final standalone.CircuitBreaker? _cb;

  ResiliencyDecorator(this.policy)
      : _bulkhead = policy.bulkhead != null
            ? Bulkhead.fromConfig(policy.bulkhead!)
            : null,
        _cb = policy.circuitBreaker != null
            ? standalone.CircuitBreaker(standalone.CircuitBreakerConfig(
                failureThreshold: policy.circuitBreaker!.failureThreshold,
                successThreshold: policy.circuitBreaker!.halfOpenMaxCalls,
                timeout: policy.circuitBreaker!.recoveryTimeout,
              ))
            : null;

  /// ポリシーを適用して [fn] を実行する。
  ///
  /// - サーキットブレーカーがオープンの場合は即座に [CircuitBreakerOpenError] を投げる。
  /// - バルクヘッドが満杯の場合は [BulkheadFullError] を投げる。
  /// - リトライ上限を超えた場合は [MaxRetriesExceededError] を投げる。
  /// - タイムアウトした場合は [TimeoutError] を投げる。
  Future<T> execute<T>(Future<T> Function() fn) async {
    _checkCircuitBreaker();

    if (_bulkhead != null) {
      await _bulkhead.acquire();
    }

    try {
      return await _executeWithRetry(fn);
    } finally {
      _bulkhead?.release();
    }
  }

  /// リトライポリシーに従って [fn] を繰り返し実行する。
  /// 失敗のたびに指数バックオフで待機し、上限に達すると [MaxRetriesExceededError] を投げる。
  Future<T> _executeWithRetry<T>(Future<T> Function() fn) async {
    final maxAttempts = policy.retry?.maxAttempts ?? 1;
    Object? lastError;

    for (var attempt = 0; attempt < maxAttempts; attempt++) {
      try {
        final result = await _executeWithTimeout(fn);
        _cb?.recordSuccess();
        return result;
      } on ResiliencyError {
        // ResiliencyError（タイムアウト等）はリトライせずそのまま再投げする
        rethrow;
      } catch (e) {
        _cb?.recordFailure();
        lastError = e;

        // 失敗後にサーキットブレーカーがオープンになっていれば即中断
        _checkCircuitBreaker();

        if (attempt + 1 < maxAttempts && policy.retry != null) {
          final delay = _calculateBackoff(
            attempt,
            policy.retry!.baseDelay,
            policy.retry!.maxDelay,
          );
          await Future<void>.delayed(delay);
        }
      }
    }

    throw MaxRetriesExceededError(maxAttempts, lastError);
  }

  /// タイムアウトポリシーが設定されている場合、[fn] にタイムアウトを適用して実行する。
  Future<T> _executeWithTimeout<T>(Future<T> Function() fn) async {
    if (policy.timeout == null) {
      return fn();
    }

    try {
      return await fn().timeout(policy.timeout!);
    } on TimeoutException {
      throw TimeoutError(policy.timeout!);
    }
  }

  /// サーキットブレーカーがオープン状態であれば [CircuitBreakerOpenError] を投げる。
  void _checkCircuitBreaker() {
    if (_cb == null) return;

    if (_cb.isOpen) {
      final elapsed = _cb.state == standalone.CircuitState.open
          ? policy.circuitBreaker!.recoveryTimeout
          : Duration.zero;
      throw CircuitBreakerOpenError(elapsed);
    }
  }

  /// 指数バックオフの待機時間を計算する。
  ///
  /// `baseDelay * 2^attempt` を計算し、[maxDelay] でキャップする。
  Duration _calculateBackoff(
    int attempt,
    Duration baseDelay,
    Duration maxDelay,
  ) {
    final delayMs =
        baseDelay.inMilliseconds * math.pow(2, attempt).toInt();
    final cappedMs = math.min(delayMs, maxDelay.inMilliseconds);
    return Duration(milliseconds: cappedMs);
  }
}

/// [ResiliencyPolicy] を適用して [fn] を実行する便利関数。
///
/// [ResiliencyDecorator] を都度インスタンス化するコストがあるため、
/// 同じポリシーを繰り返し使う場合は [ResiliencyDecorator] を直接利用すること。
Future<T> withResiliency<T>(
  ResiliencyPolicy policy,
  Future<T> Function() fn,
) {
  return ResiliencyDecorator(policy).execute(fn);
}
