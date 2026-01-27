import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// A widget that builds UI based on AsyncValue state.
///
/// Provides convenient builders for loading, error, and data states.
class AsyncValueWidget<T> extends StatelessWidget {
  /// Creates an AsyncValueWidget.
  const AsyncValueWidget({
    required this.value,
    required this.data,
    this.loading,
    this.error,
    this.skipLoadingOnRefresh = true,
    this.onRetry,
    super.key,
  });

  /// The AsyncValue to display.
  final AsyncValue<T> value;

  /// Builder for the data state.
  final Widget Function(T data) data;

  /// Builder for the loading state.
  final Widget Function()? loading;

  /// Builder for the error state.
  final Widget Function(Object error, StackTrace stackTrace)? error;

  /// Whether to skip loading indicator on refresh.
  final bool skipLoadingOnRefresh;

  /// Callback for retry action in error state.
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) => value.when(
        skipLoadingOnRefresh: skipLoadingOnRefresh,
        data: data,
        loading: loading ?? _defaultLoading,
        error: error ?? _defaultError,
      );

  Widget _defaultLoading() => const Center(
        child: CircularProgressIndicator(),
      );

  Widget _defaultError(Object error, StackTrace stackTrace) => Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline, size: 48, color: Colors.red),
          const SizedBox(height: 16),
          Text(
            error.toString(),
            textAlign: TextAlign.center,
          ),
          if (onRetry != null) ...[
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: onRetry,
              child: const Text('Retry'),
            ),
          ],
        ],
      ),
    );
}

/// A Sliver version of AsyncValueWidget.
class AsyncValueSliverWidget<T> extends StatelessWidget {
  /// Creates an AsyncValueSliverWidget.
  const AsyncValueSliverWidget({
    required this.value,
    required this.data,
    this.loading,
    this.error,
    this.skipLoadingOnRefresh = true,
    super.key,
  });

  /// The AsyncValue to display.
  final AsyncValue<T> value;

  /// Builder for the data state (must return a Sliver).
  final Widget Function(T data) data;

  /// Builder for the loading state (must return a Sliver).
  final Widget Function()? loading;

  /// Builder for the error state (must return a Sliver).
  final Widget Function(Object error, StackTrace stackTrace)? error;

  /// Whether to skip loading indicator on refresh.
  final bool skipLoadingOnRefresh;

  @override
  Widget build(BuildContext context) => value.when(
        skipLoadingOnRefresh: skipLoadingOnRefresh,
        data: data,
        loading: loading ?? _defaultLoading,
        error: error ?? _defaultError,
      );

  Widget _defaultLoading() => const SliverFillRemaining(
        child: Center(child: CircularProgressIndicator()),
      );

  Widget _defaultError(Object error, StackTrace stackTrace) => SliverFillRemaining(
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 16),
            Text(error.toString(), textAlign: TextAlign.center),
          ],
        ),
      ),
    );
}

/// A widget that watches a provider and builds based on AsyncValue.
class AsyncProviderWidget<T> extends ConsumerWidget {
  /// Creates an AsyncProviderWidget.
  const AsyncProviderWidget({
    required this.provider,
    required this.data,
    this.loading,
    this.error,
    this.skipLoadingOnRefresh = true,
    this.onRetry,
    super.key,
  });

  /// The provider to watch.
  final ProviderListenable<AsyncValue<T>> provider;

  /// Builder for the data state.
  final Widget Function(T data, WidgetRef ref) data;

  /// Builder for the loading state.
  final Widget Function()? loading;

  /// Builder for the error state.
  final Widget Function(Object error, StackTrace stackTrace)? error;

  /// Whether to skip loading indicator on refresh.
  final bool skipLoadingOnRefresh;

  /// Callback for retry action.
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final value = ref.watch(provider);

    return AsyncValueWidget<T>(
      value: value,
      data: (data_) => data(data_, ref),
      loading: loading,
      error: error,
      skipLoadingOnRefresh: skipLoadingOnRefresh,
      onRetry: onRetry,
    );
  }
}

/// A widget for handling multiple AsyncValues.
class MultiAsyncValueWidget<T1, T2> extends StatelessWidget {
  /// Creates a MultiAsyncValueWidget.
  const MultiAsyncValueWidget({
    required this.value1,
    required this.value2,
    required this.data,
    this.loading,
    this.error,
    super.key,
  });

  /// The first AsyncValue.
  final AsyncValue<T1> value1;

  /// The second AsyncValue.
  final AsyncValue<T2> value2;

  /// Builder for when both values have data.
  final Widget Function(T1 data1, T2 data2) data;

  /// Builder for loading state.
  final Widget Function()? loading;

  /// Builder for error state.
  final Widget Function(Object error, StackTrace stackTrace)? error;

  @override
  Widget build(BuildContext context) {
    // Check for loading
    if (value1.isLoading && !value1.hasValue) {
      return loading?.call() ??
          const Center(child: CircularProgressIndicator());
    }
    if (value2.isLoading && !value2.hasValue) {
      return loading?.call() ??
          const Center(child: CircularProgressIndicator());
    }

    // Check for errors
    if (value1.hasError) {
      return error?.call(value1.error!, value1.stackTrace!) ??
          Center(child: Text('Error: ${value1.error}'));
    }
    if (value2.hasError) {
      return error?.call(value2.error!, value2.stackTrace!) ??
          Center(child: Text('Error: ${value2.error}'));
    }

    // Both have values
    return data(value1.value as T1, value2.value as T2);
  }
}

/// A widget for handling three AsyncValues.
class TripleAsyncValueWidget<T1, T2, T3> extends StatelessWidget {
  /// Creates a TripleAsyncValueWidget.
  const TripleAsyncValueWidget({
    required this.value1,
    required this.value2,
    required this.value3,
    required this.data,
    this.loading,
    this.error,
    super.key,
  });

  /// The first AsyncValue.
  final AsyncValue<T1> value1;

  /// The second AsyncValue.
  final AsyncValue<T2> value2;

  /// The third AsyncValue.
  final AsyncValue<T3> value3;

  /// Builder for when all values have data.
  final Widget Function(T1 data1, T2 data2, T3 data3) data;

  /// Builder for loading state.
  final Widget Function()? loading;

  /// Builder for error state.
  final Widget Function(Object error, StackTrace stackTrace)? error;

  @override
  Widget build(BuildContext context) {
    // Check for loading
    if ((value1.isLoading && !value1.hasValue) ||
        (value2.isLoading && !value2.hasValue) ||
        (value3.isLoading && !value3.hasValue)) {
      return loading?.call() ??
          const Center(child: CircularProgressIndicator());
    }

    // Check for errors
    if (value1.hasError) {
      return error?.call(value1.error!, value1.stackTrace!) ??
          Center(child: Text('Error: ${value1.error}'));
    }
    if (value2.hasError) {
      return error?.call(value2.error!, value2.stackTrace!) ??
          Center(child: Text('Error: ${value2.error}'));
    }
    if (value3.hasError) {
      return error?.call(value3.error!, value3.stackTrace!) ??
          Center(child: Text('Error: ${value3.error}'));
    }

    // All have values
    return data(
      value1.value as T1,
      value2.value as T2,
      value3.value as T3,
    );
  }
}
