import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Extension methods for AsyncValue to simplify common patterns.
extension AsyncValueExtensions<T> on AsyncValue<T> {
  /// Returns true if the value is loading (initial load, not refresh).
  bool get isInitialLoading => isLoading && !hasValue;

  /// Returns true if the value is refreshing (has value but loading).
  bool get isRefreshing => isLoading && hasValue;

  /// Returns true if the value has an error.
  bool get hasFailure => hasError;

  /// Returns true if the value has data.
  bool get hasData => hasValue;

  /// Maps the value to a widget based on the current state.
  ///
  /// This is a convenience method for building UI based on AsyncValue state.
  Widget when2({
    required Widget Function(T data) data,
    required Widget Function() loading,
    required Widget Function(Object error, StackTrace stackTrace) error,
    Widget Function(T data)? refreshing,
  }) {
    return when(
      data: (value) {
        if (isRefreshing && refreshing != null) {
          return refreshing(value);
        }
        return data(value);
      },
      loading: loading,
      error: error,
    );
  }

  /// Maps the value with a skip loading on refresh option.
  Widget whenOrRefresh({
    required Widget Function(T data) data,
    required Widget Function() loading,
    required Widget Function(Object error, StackTrace stackTrace) error,
    bool skipLoadingOnRefresh = true,
  }) {
    return when(
      skipLoadingOnRefresh: skipLoadingOnRefresh,
      data: data,
      loading: loading,
      error: error,
    );
  }

  /// Returns the value or null if not available.
  T? get valueOrNull => hasValue ? value : null;

  /// Returns the error or null if not available.
  Object? get errorOrNull => hasError ? this.error : null;

  /// Transforms the value if present.
  AsyncValue<R> mapData<R>(R Function(T data) mapper) {
    return when(
      data: (data) => AsyncValue.data(mapper(data)),
      loading: () => const AsyncValue.loading(),
      error: (error, stackTrace) => AsyncValue.error(error, stackTrace),
    );
  }

  /// Chains another async operation.
  AsyncValue<R> flatMap<R>(AsyncValue<R> Function(T data) mapper) {
    return when(
      data: mapper,
      loading: () => const AsyncValue.loading(),
      error: (error, stackTrace) => AsyncValue.error(error, stackTrace),
    );
  }

  /// Returns the value or a default value.
  T getOrElse(T defaultValue) {
    return valueOrNull ?? defaultValue;
  }

  /// Returns the value or throws the error.
  T getOrThrow() {
    return when(
      data: (data) => data,
      loading: () => throw StateError('Value is still loading'),
      error: (error, stackTrace) => throw error,
    );
  }

  /// Combines two AsyncValues.
  AsyncValue<(T, R)> combine<R>(AsyncValue<R> other) {
    return when(
      data: (data1) => other.when(
        data: (data2) => AsyncValue.data((data1, data2)),
        loading: () => const AsyncValue.loading(),
        error: (error, stackTrace) => AsyncValue.error(error, stackTrace),
      ),
      loading: () => const AsyncValue.loading(),
      error: (error, stackTrace) => AsyncValue.error(error, stackTrace),
    );
  }
}

/// Extension methods for combining multiple AsyncValues.
extension AsyncValueCombineExtension<T> on List<AsyncValue<T>> {
  /// Combines all AsyncValues into a single AsyncValue<List<T>>.
  AsyncValue<List<T>> combine() {
    if (isEmpty) {
      return const AsyncValue.data([]);
    }

    // Check for any loading state
    if (any((v) => v.isLoading && !v.hasValue)) {
      return const AsyncValue.loading();
    }

    // Check for any error state
    for (final value in this) {
      if (value.hasError) {
        return AsyncValue.error(value.error!, value.stackTrace!);
      }
    }

    // All have values
    return AsyncValue.data(map((v) => v.value as T).toList());
  }
}

/// Extension for nullable AsyncValue.
extension AsyncValueNullableExtension<T> on AsyncValue<T?> {
  /// Returns a non-nullable AsyncValue, treating null as loading.
  AsyncValue<T> requireValue() {
    return when(
      data: (data) =>
          data != null ? AsyncValue.data(data) : const AsyncValue.loading(),
      loading: () => const AsyncValue.loading(),
      error: (error, stackTrace) => AsyncValue.error(error, stackTrace),
    );
  }
}
