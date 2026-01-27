// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'async_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$AsyncState<T> {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() initial,
    required TResult Function(T? previousData) loading,
    required TResult Function(T data) success,
    required TResult Function(
            Object error, StackTrace? stackTrace, T? previousData)
        failure,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? initial,
    TResult? Function(T? previousData)? loading,
    TResult? Function(T data)? success,
    TResult? Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? initial,
    TResult Function(T? previousData)? loading,
    TResult Function(T data)? success,
    TResult Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AsyncInitial<T> value) initial,
    required TResult Function(AsyncLoading<T> value) loading,
    required TResult Function(AsyncSuccess<T> value) success,
    required TResult Function(AsyncFailure<T> value) failure,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AsyncInitial<T> value)? initial,
    TResult? Function(AsyncLoading<T> value)? loading,
    TResult? Function(AsyncSuccess<T> value)? success,
    TResult? Function(AsyncFailure<T> value)? failure,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AsyncInitial<T> value)? initial,
    TResult Function(AsyncLoading<T> value)? loading,
    TResult Function(AsyncSuccess<T> value)? success,
    TResult Function(AsyncFailure<T> value)? failure,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AsyncStateCopyWith<T, $Res> {
  factory $AsyncStateCopyWith(
          AsyncState<T> value, $Res Function(AsyncState<T>) then) =
      _$AsyncStateCopyWithImpl<T, $Res, AsyncState<T>>;
}

/// @nodoc
class _$AsyncStateCopyWithImpl<T, $Res, $Val extends AsyncState<T>>
    implements $AsyncStateCopyWith<T, $Res> {
  _$AsyncStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$AsyncInitialImplCopyWith<T, $Res> {
  factory _$$AsyncInitialImplCopyWith(_$AsyncInitialImpl<T> value,
          $Res Function(_$AsyncInitialImpl<T>) then) =
      __$$AsyncInitialImplCopyWithImpl<T, $Res>;
}

/// @nodoc
class __$$AsyncInitialImplCopyWithImpl<T, $Res>
    extends _$AsyncStateCopyWithImpl<T, $Res, _$AsyncInitialImpl<T>>
    implements _$$AsyncInitialImplCopyWith<T, $Res> {
  __$$AsyncInitialImplCopyWithImpl(
      _$AsyncInitialImpl<T> _value, $Res Function(_$AsyncInitialImpl<T>) _then)
      : super(_value, _then);

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$AsyncInitialImpl<T> implements AsyncInitial<T> {
  const _$AsyncInitialImpl();

  @override
  String toString() {
    return 'AsyncState<$T>.initial()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is _$AsyncInitialImpl<T>);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() initial,
    required TResult Function(T? previousData) loading,
    required TResult Function(T data) success,
    required TResult Function(
            Object error, StackTrace? stackTrace, T? previousData)
        failure,
  }) {
    return initial();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? initial,
    TResult? Function(T? previousData)? loading,
    TResult? Function(T data)? success,
    TResult? Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
  }) {
    return initial?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? initial,
    TResult Function(T? previousData)? loading,
    TResult Function(T data)? success,
    TResult Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
    required TResult orElse(),
  }) {
    if (initial != null) {
      return initial();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AsyncInitial<T> value) initial,
    required TResult Function(AsyncLoading<T> value) loading,
    required TResult Function(AsyncSuccess<T> value) success,
    required TResult Function(AsyncFailure<T> value) failure,
  }) {
    return initial(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AsyncInitial<T> value)? initial,
    TResult? Function(AsyncLoading<T> value)? loading,
    TResult? Function(AsyncSuccess<T> value)? success,
    TResult? Function(AsyncFailure<T> value)? failure,
  }) {
    return initial?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AsyncInitial<T> value)? initial,
    TResult Function(AsyncLoading<T> value)? loading,
    TResult Function(AsyncSuccess<T> value)? success,
    TResult Function(AsyncFailure<T> value)? failure,
    required TResult orElse(),
  }) {
    if (initial != null) {
      return initial(this);
    }
    return orElse();
  }
}

abstract class AsyncInitial<T> implements AsyncState<T> {
  const factory AsyncInitial() = _$AsyncInitialImpl<T>;
}

/// @nodoc
abstract class _$$AsyncLoadingImplCopyWith<T, $Res> {
  factory _$$AsyncLoadingImplCopyWith(_$AsyncLoadingImpl<T> value,
          $Res Function(_$AsyncLoadingImpl<T>) then) =
      __$$AsyncLoadingImplCopyWithImpl<T, $Res>;
  @useResult
  $Res call({T? previousData});
}

/// @nodoc
class __$$AsyncLoadingImplCopyWithImpl<T, $Res>
    extends _$AsyncStateCopyWithImpl<T, $Res, _$AsyncLoadingImpl<T>>
    implements _$$AsyncLoadingImplCopyWith<T, $Res> {
  __$$AsyncLoadingImplCopyWithImpl(
      _$AsyncLoadingImpl<T> _value, $Res Function(_$AsyncLoadingImpl<T>) _then)
      : super(_value, _then);

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? previousData = freezed,
  }) {
    return _then(_$AsyncLoadingImpl<T>(
      previousData: freezed == previousData
          ? _value.previousData
          : previousData // ignore: cast_nullable_to_non_nullable
              as T?,
    ));
  }
}

/// @nodoc

class _$AsyncLoadingImpl<T> implements AsyncLoading<T> {
  const _$AsyncLoadingImpl({this.previousData});

  /// Optional previous data while loading.
  @override
  final T? previousData;

  @override
  String toString() {
    return 'AsyncState<$T>.loading(previousData: $previousData)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AsyncLoadingImpl<T> &&
            const DeepCollectionEquality()
                .equals(other.previousData, previousData));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(previousData));

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AsyncLoadingImplCopyWith<T, _$AsyncLoadingImpl<T>> get copyWith =>
      __$$AsyncLoadingImplCopyWithImpl<T, _$AsyncLoadingImpl<T>>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() initial,
    required TResult Function(T? previousData) loading,
    required TResult Function(T data) success,
    required TResult Function(
            Object error, StackTrace? stackTrace, T? previousData)
        failure,
  }) {
    return loading(previousData);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? initial,
    TResult? Function(T? previousData)? loading,
    TResult? Function(T data)? success,
    TResult? Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
  }) {
    return loading?.call(previousData);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? initial,
    TResult Function(T? previousData)? loading,
    TResult Function(T data)? success,
    TResult Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
    required TResult orElse(),
  }) {
    if (loading != null) {
      return loading(previousData);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AsyncInitial<T> value) initial,
    required TResult Function(AsyncLoading<T> value) loading,
    required TResult Function(AsyncSuccess<T> value) success,
    required TResult Function(AsyncFailure<T> value) failure,
  }) {
    return loading(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AsyncInitial<T> value)? initial,
    TResult? Function(AsyncLoading<T> value)? loading,
    TResult? Function(AsyncSuccess<T> value)? success,
    TResult? Function(AsyncFailure<T> value)? failure,
  }) {
    return loading?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AsyncInitial<T> value)? initial,
    TResult Function(AsyncLoading<T> value)? loading,
    TResult Function(AsyncSuccess<T> value)? success,
    TResult Function(AsyncFailure<T> value)? failure,
    required TResult orElse(),
  }) {
    if (loading != null) {
      return loading(this);
    }
    return orElse();
  }
}

abstract class AsyncLoading<T> implements AsyncState<T> {
  const factory AsyncLoading({final T? previousData}) = _$AsyncLoadingImpl<T>;

  /// Optional previous data while loading.
  T? get previousData;

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AsyncLoadingImplCopyWith<T, _$AsyncLoadingImpl<T>> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$AsyncSuccessImplCopyWith<T, $Res> {
  factory _$$AsyncSuccessImplCopyWith(_$AsyncSuccessImpl<T> value,
          $Res Function(_$AsyncSuccessImpl<T>) then) =
      __$$AsyncSuccessImplCopyWithImpl<T, $Res>;
  @useResult
  $Res call({T data});
}

/// @nodoc
class __$$AsyncSuccessImplCopyWithImpl<T, $Res>
    extends _$AsyncStateCopyWithImpl<T, $Res, _$AsyncSuccessImpl<T>>
    implements _$$AsyncSuccessImplCopyWith<T, $Res> {
  __$$AsyncSuccessImplCopyWithImpl(
      _$AsyncSuccessImpl<T> _value, $Res Function(_$AsyncSuccessImpl<T>) _then)
      : super(_value, _then);

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? data = freezed,
  }) {
    return _then(_$AsyncSuccessImpl<T>(
      freezed == data
          ? _value.data
          : data // ignore: cast_nullable_to_non_nullable
              as T,
    ));
  }
}

/// @nodoc

class _$AsyncSuccessImpl<T> implements AsyncSuccess<T> {
  const _$AsyncSuccessImpl(this.data);

  @override
  final T data;

  @override
  String toString() {
    return 'AsyncState<$T>.success(data: $data)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AsyncSuccessImpl<T> &&
            const DeepCollectionEquality().equals(other.data, data));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(data));

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AsyncSuccessImplCopyWith<T, _$AsyncSuccessImpl<T>> get copyWith =>
      __$$AsyncSuccessImplCopyWithImpl<T, _$AsyncSuccessImpl<T>>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() initial,
    required TResult Function(T? previousData) loading,
    required TResult Function(T data) success,
    required TResult Function(
            Object error, StackTrace? stackTrace, T? previousData)
        failure,
  }) {
    return success(data);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? initial,
    TResult? Function(T? previousData)? loading,
    TResult? Function(T data)? success,
    TResult? Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
  }) {
    return success?.call(data);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? initial,
    TResult Function(T? previousData)? loading,
    TResult Function(T data)? success,
    TResult Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(data);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AsyncInitial<T> value) initial,
    required TResult Function(AsyncLoading<T> value) loading,
    required TResult Function(AsyncSuccess<T> value) success,
    required TResult Function(AsyncFailure<T> value) failure,
  }) {
    return success(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AsyncInitial<T> value)? initial,
    TResult? Function(AsyncLoading<T> value)? loading,
    TResult? Function(AsyncSuccess<T> value)? success,
    TResult? Function(AsyncFailure<T> value)? failure,
  }) {
    return success?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AsyncInitial<T> value)? initial,
    TResult Function(AsyncLoading<T> value)? loading,
    TResult Function(AsyncSuccess<T> value)? success,
    TResult Function(AsyncFailure<T> value)? failure,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(this);
    }
    return orElse();
  }
}

abstract class AsyncSuccess<T> implements AsyncState<T> {
  const factory AsyncSuccess(final T data) = _$AsyncSuccessImpl<T>;

  T get data;

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AsyncSuccessImplCopyWith<T, _$AsyncSuccessImpl<T>> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$AsyncFailureImplCopyWith<T, $Res> {
  factory _$$AsyncFailureImplCopyWith(_$AsyncFailureImpl<T> value,
          $Res Function(_$AsyncFailureImpl<T>) then) =
      __$$AsyncFailureImplCopyWithImpl<T, $Res>;
  @useResult
  $Res call({Object error, StackTrace? stackTrace, T? previousData});
}

/// @nodoc
class __$$AsyncFailureImplCopyWithImpl<T, $Res>
    extends _$AsyncStateCopyWithImpl<T, $Res, _$AsyncFailureImpl<T>>
    implements _$$AsyncFailureImplCopyWith<T, $Res> {
  __$$AsyncFailureImplCopyWithImpl(
      _$AsyncFailureImpl<T> _value, $Res Function(_$AsyncFailureImpl<T>) _then)
      : super(_value, _then);

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? error = null,
    Object? stackTrace = freezed,
    Object? previousData = freezed,
  }) {
    return _then(_$AsyncFailureImpl<T>(
      null == error ? _value.error : error,
      stackTrace: freezed == stackTrace
          ? _value.stackTrace
          : stackTrace // ignore: cast_nullable_to_non_nullable
              as StackTrace?,
      previousData: freezed == previousData
          ? _value.previousData
          : previousData // ignore: cast_nullable_to_non_nullable
              as T?,
    ));
  }
}

/// @nodoc

class _$AsyncFailureImpl<T> implements AsyncFailure<T> {
  const _$AsyncFailureImpl(this.error, {this.stackTrace, this.previousData});

  @override
  final Object error;
  @override
  final StackTrace? stackTrace;

  /// Optional previous data when failure occurred.
  @override
  final T? previousData;

  @override
  String toString() {
    return 'AsyncState<$T>.failure(error: $error, stackTrace: $stackTrace, previousData: $previousData)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AsyncFailureImpl<T> &&
            const DeepCollectionEquality().equals(other.error, error) &&
            (identical(other.stackTrace, stackTrace) ||
                other.stackTrace == stackTrace) &&
            const DeepCollectionEquality()
                .equals(other.previousData, previousData));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(error),
      stackTrace,
      const DeepCollectionEquality().hash(previousData));

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AsyncFailureImplCopyWith<T, _$AsyncFailureImpl<T>> get copyWith =>
      __$$AsyncFailureImplCopyWithImpl<T, _$AsyncFailureImpl<T>>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() initial,
    required TResult Function(T? previousData) loading,
    required TResult Function(T data) success,
    required TResult Function(
            Object error, StackTrace? stackTrace, T? previousData)
        failure,
  }) {
    return failure(error, stackTrace, previousData);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? initial,
    TResult? Function(T? previousData)? loading,
    TResult? Function(T data)? success,
    TResult? Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
  }) {
    return failure?.call(error, stackTrace, previousData);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? initial,
    TResult Function(T? previousData)? loading,
    TResult Function(T data)? success,
    TResult Function(Object error, StackTrace? stackTrace, T? previousData)?
        failure,
    required TResult orElse(),
  }) {
    if (failure != null) {
      return failure(error, stackTrace, previousData);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AsyncInitial<T> value) initial,
    required TResult Function(AsyncLoading<T> value) loading,
    required TResult Function(AsyncSuccess<T> value) success,
    required TResult Function(AsyncFailure<T> value) failure,
  }) {
    return failure(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AsyncInitial<T> value)? initial,
    TResult? Function(AsyncLoading<T> value)? loading,
    TResult? Function(AsyncSuccess<T> value)? success,
    TResult? Function(AsyncFailure<T> value)? failure,
  }) {
    return failure?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AsyncInitial<T> value)? initial,
    TResult Function(AsyncLoading<T> value)? loading,
    TResult Function(AsyncSuccess<T> value)? success,
    TResult Function(AsyncFailure<T> value)? failure,
    required TResult orElse(),
  }) {
    if (failure != null) {
      return failure(this);
    }
    return orElse();
  }
}

abstract class AsyncFailure<T> implements AsyncState<T> {
  const factory AsyncFailure(final Object error,
      {final StackTrace? stackTrace,
      final T? previousData}) = _$AsyncFailureImpl<T>;

  Object get error;
  StackTrace? get stackTrace;

  /// Optional previous data when failure occurred.
  T? get previousData;

  /// Create a copy of AsyncState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AsyncFailureImplCopyWith<T, _$AsyncFailureImpl<T>> get copyWith =>
      throw _privateConstructorUsedError;
}
