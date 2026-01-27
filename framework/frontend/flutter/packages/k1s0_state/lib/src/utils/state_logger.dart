import 'dart:developer' as developer;

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// A logger for state changes in Riverpod.
///
/// Logs provider updates, errors, and disposals for debugging.
class StateLogger extends ProviderObserver {
  StateLogger({
    this.enabled = kDebugMode,
    this.logUpdates = true,
    this.logDispose = false,
    this.logErrors = true,
    this.filter,
  });

  /// Whether logging is enabled.
  final bool enabled;

  /// Whether to log state updates.
  final bool logUpdates;

  /// Whether to log provider disposals.
  final bool logDispose;

  /// Whether to log errors.
  final bool logErrors;

  /// Optional filter to include only specific providers.
  final bool Function(ProviderBase<Object?>)? filter;

  bool _shouldLog(ProviderBase<Object?> provider) {
    if (!enabled) return false;
    if (filter != null) return filter!(provider);
    return true;
  }

  String _formatProvider(ProviderBase<Object?> provider) {
    final name = provider.name ?? provider.runtimeType.toString();
    if (provider.argument != null) {
      return '$name(${provider.argument})';
    }
    return name;
  }

  String _formatValue(Object? value) {
    if (value == null) return 'null';
    final str = value.toString();
    if (str.length > 200) {
      return '${str.substring(0, 200)}...';
    }
    return str;
  }

  @override
  void didUpdateProvider(
    ProviderBase<Object?> provider,
    Object? previousValue,
    Object? newValue,
    ProviderContainer container,
  ) {
    if (!logUpdates || !_shouldLog(provider)) return;

    final name = _formatProvider(provider);
    final prev = _formatValue(previousValue);
    final next = _formatValue(newValue);

    _log('[STATE] $name: $prev -> $next');
  }

  @override
  void didDisposeProvider(
    ProviderBase<Object?> provider,
    ProviderContainer container,
  ) {
    if (!logDispose || !_shouldLog(provider)) return;

    final name = _formatProvider(provider);
    _log('[DISPOSE] $name');
  }

  @override
  void providerDidFail(
    ProviderBase<Object?> provider,
    Object error,
    StackTrace stackTrace,
    ProviderContainer container,
  ) {
    if (!logErrors || !_shouldLog(provider)) return;

    final name = _formatProvider(provider);
    _log('[ERROR] $name: $error\n$stackTrace', isError: true);
  }

  void _log(String message, {bool isError = false}) {
    if (kDebugMode) {
      developer.log(
        message,
        name: 'k1s0_state',
        level: isError ? 1000 : 0,
      );
    }
  }
}

/// A verbose logger that includes timing information.
class VerboseStateLogger extends StateLogger {
  VerboseStateLogger({
    super.enabled,
    super.filter,
  }) : super(
          logUpdates: true,
          logDispose: true,
          logErrors: true,
        );

  final Map<String, DateTime> _updateTimes = {};

  @override
  void didUpdateProvider(
    ProviderBase<Object?> provider,
    Object? previousValue,
    Object? newValue,
    ProviderContainer container,
  ) {
    if (!logUpdates || !_shouldLog(provider)) return;

    final name = _formatProvider(provider);
    final now = DateTime.now();
    final lastUpdate = _updateTimes[name];
    _updateTimes[name] = now;

    final prev = _formatValue(previousValue);
    final next = _formatValue(newValue);

    var message = '[STATE] $name: $prev -> $next';
    if (lastUpdate != null) {
      final diff = now.difference(lastUpdate).inMilliseconds;
      message += ' (${diff}ms since last update)';
    }

    _log(message);
  }

  @override
  void didAddProvider(
    ProviderBase<Object?> provider,
    Object? value,
    ProviderContainer container,
  ) {
    if (!_shouldLog(provider)) return;

    final name = _formatProvider(provider);
    final val = _formatValue(value);
    _log('[ADD] $name: $val');
  }
}

/// Creates a StateLogger that only logs specific providers.
StateLogger filteredStateLogger(List<String> providerNames) {
  return StateLogger(
    filter: (provider) {
      final name = provider.name ?? provider.runtimeType.toString();
      return providerNames.any((n) => name.contains(n));
    },
  );
}
