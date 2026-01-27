import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Base class for AsyncNotifiers with common functionality.
///
/// Provides utility methods for handling async operations
/// with proper error handling and state management.
abstract class K1s0AsyncNotifier<T> extends AsyncNotifier<T> {
  /// Executes an async operation and updates the state.
  ///
  /// Automatically handles loading and error states.
  Future<void> execute(Future<T> Function() operation) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(operation);
  }

  /// Executes an async operation while preserving the previous value.
  ///
  /// Shows loading indicator but keeps the previous data visible.
  Future<void> refresh(Future<T> Function() operation) async {
    final previousValue = state.valueOrNull;
    state = AsyncValue<T>.loading().copyWithPrevious(
      previousValue != null
          ? AsyncValue.data(previousValue)
          : const AsyncValue.loading(),
    );
    state = await AsyncValue.guard(operation);
  }

  /// Executes an async operation that modifies the current state.
  ///
  /// Only runs if there is existing data.
  Future<void> modify(Future<T> Function(T current) operation) async {
    final current = state.valueOrNull;
    if (current == null) return;

    state = AsyncValue<T>.loading().copyWithPrevious(AsyncValue.data(current));
    state = await AsyncValue.guard(() => operation(current));
  }

  /// Sets the state to an error.
  void setError(Object error, [StackTrace? stackTrace]) {
    state = AsyncValue.error(error, stackTrace ?? StackTrace.current);
  }

  /// Sets the state to data.
  void setData(T data) {
    state = AsyncValue.data(data);
  }
}

/// Base class for family AsyncNotifiers.
abstract class K1s0FamilyAsyncNotifier<T, Arg> extends FamilyAsyncNotifier<T, Arg> {
  /// Executes an async operation and updates the state.
  Future<void> execute(Future<T> Function() operation) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(operation);
  }

  /// Executes an async operation while preserving the previous value.
  Future<void> refresh(Future<T> Function() operation) async {
    final previousValue = state.valueOrNull;
    state = AsyncValue<T>.loading().copyWithPrevious(
      previousValue != null
          ? AsyncValue.data(previousValue)
          : const AsyncValue.loading(),
    );
    state = await AsyncValue.guard(operation);
  }

  /// Executes an async operation that modifies the current state.
  Future<void> modify(Future<T> Function(T current) operation) async {
    final current = state.valueOrNull;
    if (current == null) return;

    state = AsyncValue<T>.loading().copyWithPrevious(AsyncValue.data(current));
    state = await AsyncValue.guard(() => operation(current));
  }

  /// Sets the state to an error.
  void setError(Object error, [StackTrace? stackTrace]) {
    state = AsyncValue.error(error, stackTrace ?? StackTrace.current);
  }

  /// Sets the state to data.
  void setData(T data) {
    state = AsyncValue.data(data);
  }
}

/// Base class for StateNotifiers with async operations.
abstract class K1s0StateNotifier<T> extends StateNotifier<T> {
  K1s0StateNotifier(super.state);

  /// Whether the notifier is disposed.
  bool _disposed = false;

  /// Whether the notifier is disposed.
  bool get disposed => _disposed;

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }

  /// Updates the state if not disposed.
  void safeUpdate(T newState) {
    if (!_disposed) {
      state = newState;
    }
  }

  /// Updates the state using a function if not disposed.
  void safeModify(T Function(T current) modifier) {
    if (!_disposed) {
      state = modifier(state);
    }
  }
}

/// Base class for Notifiers with common functionality.
abstract class K1s0Notifier<T> extends Notifier<T> {
  /// Updates the state.
  void update(T newState) {
    state = newState;
  }

  /// Modifies the state using a function.
  void modify(T Function(T current) modifier) {
    state = modifier(state);
  }
}

/// Base class for family Notifiers.
abstract class K1s0FamilyNotifier<T, Arg> extends FamilyNotifier<T, Arg> {
  /// Updates the state.
  void update(T newState) {
    state = newState;
  }

  /// Modifies the state using a function.
  void modify(T Function(T current) modifier) {
    state = modifier(state);
  }
}
