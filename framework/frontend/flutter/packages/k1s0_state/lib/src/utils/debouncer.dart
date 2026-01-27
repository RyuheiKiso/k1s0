import 'dart:async';

/// A debouncer for delaying function execution.
///
/// Useful for debouncing user input or other rapid events.
class Debouncer {
  Debouncer({
    this.duration = const Duration(milliseconds: 300),
  });

  /// The delay duration.
  final Duration duration;

  Timer? _timer;

  /// Runs the action after the delay.
  ///
  /// If called again before the delay, the previous call is cancelled.
  void run(void Function() action) {
    _timer?.cancel();
    _timer = Timer(duration, action);
  }

  /// Runs the action immediately if not already running.
  void runImmediate(void Function() action) {
    if (_timer?.isActive != true) {
      action();
    }
    _timer?.cancel();
    _timer = Timer(duration, () {});
  }

  /// Cancels any pending action.
  void cancel() {
    _timer?.cancel();
    _timer = null;
  }

  /// Disposes the debouncer.
  void dispose() {
    cancel();
  }

  /// Whether there is a pending action.
  bool get isPending => _timer?.isActive ?? false;
}

/// A throttler for limiting function execution rate.
///
/// Useful for rate-limiting API calls or other expensive operations.
class Throttler {
  Throttler({
    this.duration = const Duration(milliseconds: 300),
  });

  /// The minimum time between executions.
  final Duration duration;

  DateTime? _lastRun;
  Timer? _timer;
  void Function()? _pendingAction;

  /// Runs the action if the throttle period has passed.
  ///
  /// If called during the throttle period, queues the action.
  void run(void Function() action) {
    final now = DateTime.now();
    final lastRun = _lastRun;

    if (lastRun == null ||
        now.difference(lastRun) >= duration) {
      _lastRun = now;
      action();
      _pendingAction = null;
    } else {
      _pendingAction = action;
      _scheduleNext();
    }
  }

  void _scheduleNext() {
    if (_timer?.isActive == true) return;

    final lastRun = _lastRun;
    if (lastRun == null) return;

    final elapsed = DateTime.now().difference(lastRun);
    final remaining = duration - elapsed;

    if (remaining.isNegative) {
      _runPending();
    } else {
      _timer = Timer(remaining, _runPending);
    }
  }

  void _runPending() {
    final action = _pendingAction;
    _pendingAction = null;
    if (action != null) {
      _lastRun = DateTime.now();
      action();
    }
  }

  /// Cancels any pending action.
  void cancel() {
    _timer?.cancel();
    _timer = null;
    _pendingAction = null;
  }

  /// Disposes the throttler.
  void dispose() {
    cancel();
  }

  /// Whether there is a pending action.
  bool get isPending => _pendingAction != null;
}

/// A debouncer for async functions.
class AsyncDebouncer<T> {
  AsyncDebouncer({
    this.duration = const Duration(milliseconds: 300),
  });

  /// The delay duration.
  final Duration duration;

  Timer? _timer;
  Completer<T>? _completer;

  /// Runs the async action after the delay.
  ///
  /// Returns a Future that completes with the result of the action.
  Future<T> run(Future<T> Function() action) {
    _timer?.cancel();
    _completer ??= Completer<T>();

    _timer = Timer(duration, () async {
      try {
        final result = await action();
        _completer?.complete(result);
      } catch (e, s) {
        _completer?.completeError(e, s);
      } finally {
        _completer = null;
      }
    });

    return _completer!.future;
  }

  /// Cancels any pending action.
  void cancel() {
    _timer?.cancel();
    _timer = null;
    _completer = null;
  }

  /// Disposes the debouncer.
  void dispose() {
    cancel();
  }
}

/// Extension methods for creating debouncers.
extension DebouncerExtension on Duration {
  /// Creates a debouncer with this duration.
  Debouncer get debouncer => Debouncer(duration: this);

  /// Creates a throttler with this duration.
  Throttler get throttler => Throttler(duration: this);

  /// Creates an async debouncer with this duration.
  AsyncDebouncer<T> asyncDebouncer<T>() => AsyncDebouncer<T>(duration: this);
}
