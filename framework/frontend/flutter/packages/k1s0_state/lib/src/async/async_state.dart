import 'package:freezed_annotation/freezed_annotation.dart';

part 'async_state.freezed.dart';

/// A sealed class representing async operation states.
///
/// This provides a more explicit alternative to AsyncValue for cases
/// where you need fine-grained control over state transitions.
@freezed
sealed class AsyncState<T> with _$AsyncState<T> {
  /// Initial state before any operation.
  const factory AsyncState.initial() = AsyncInitial<T>;

  /// Loading state during an operation.
  const factory AsyncState.loading({
    /// Optional previous data while loading.
    T? previousData,
  }) = AsyncLoading<T>;

  /// Success state with data.
  const factory AsyncState.success(T data) = AsyncSuccess<T>;

  /// Failure state with error.
  const factory AsyncState.failure(
    Object error, {
    StackTrace? stackTrace,

    /// Optional previous data when failure occurred.
    T? previousData,
  }) = AsyncFailure<T>;
}

/// Extension methods for AsyncState.
extension AsyncStateExtensions<T> on AsyncState<T> {
  /// Returns true if the state is initial.
  bool get isInitial => this is AsyncInitial<T>;

  /// Returns true if the state is loading.
  bool get isLoading => this is AsyncLoading<T>;

  /// Returns true if the state is success.
  bool get isSuccess => this is AsyncSuccess<T>;

  /// Returns true if the state is failure.
  bool get isFailure => this is AsyncFailure<T>;

  /// Returns the data if available.
  T? get dataOrNull => switch (this) {
        AsyncSuccess(:final data) => data,
        AsyncLoading(:final previousData) => previousData,
        AsyncFailure(:final previousData) => previousData,
        _ => null,
      };

  /// Returns true if the state has data.
  bool get hasData => dataOrNull != null;

  /// Returns the error if available.
  Object? get errorOrNull => switch (this) {
        AsyncFailure(:final error) => error,
        _ => null,
      };

  /// Maps the state to a value.
  R when<R>({
    required R Function() initial,
    required R Function(T? previousData) loading,
    required R Function(T data) success,
    required R Function(Object error, StackTrace? stackTrace, T? previousData)
        failure,
  }) =>
      switch (this) {
        AsyncInitial() => initial(),
        AsyncLoading(:final previousData) => loading(previousData),
        AsyncSuccess(:final data) => success(data),
        AsyncFailure(:final error, :final stackTrace, :final previousData) =>
          failure(error, stackTrace, previousData),
      };

  /// Maps the state to a value with optional handlers.
  R maybeWhen<R>({
    required R Function() orElse,
    R Function()? initial,
    R Function(T? previousData)? loading,
    R Function(T data)? success,
    R Function(Object error, StackTrace? stackTrace, T? previousData)? failure,
  }) =>
      switch (this) {
        AsyncInitial() => initial?.call() ?? orElse(),
        AsyncLoading(:final previousData) =>
          loading?.call(previousData) ?? orElse(),
        AsyncSuccess(:final data) => success?.call(data) ?? orElse(),
        AsyncFailure(:final error, :final stackTrace, :final previousData) =>
          failure?.call(error, stackTrace, previousData) ?? orElse(),
      };

  /// Transforms the data if success.
  AsyncState<R> map<R>(R Function(T data) mapper) => switch (this) {
        AsyncInitial() => const AsyncState.initial(),
        AsyncLoading(:final previousData) => AsyncState.loading(
            previousData: previousData != null ? mapper(previousData) : null,
          ),
        AsyncSuccess(:final data) => AsyncState.success(mapper(data)),
        AsyncFailure(:final error, :final stackTrace, :final previousData) =>
          AsyncState.failure(
            error,
            stackTrace: stackTrace,
            previousData: previousData != null ? mapper(previousData) : null,
          ),
      };

  /// Converts to loading state, preserving data.
  AsyncState<T> toLoading() => AsyncState.loading(previousData: dataOrNull);

  /// Converts to failure state, preserving data.
  AsyncState<T> toFailure(Object error, [StackTrace? stackTrace]) =>
      AsyncState.failure(
        error,
        stackTrace: stackTrace,
        previousData: dataOrNull,
      );
}
