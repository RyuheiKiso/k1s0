// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'error_info.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ErrorInfo _$ErrorInfoFromJson(Map<String, dynamic> json) {
  return _ErrorInfo.fromJson(json);
}

/// @nodoc
mixin _$ErrorInfo {
  /// Error type name
  String get type => throw _privateConstructorUsedError;

  /// Error message
  String get message => throw _privateConstructorUsedError;

  /// Stack trace
  String? get stackTrace => throw _privateConstructorUsedError;

  /// Error code
  String? get code => throw _privateConstructorUsedError;

  /// Original error info (for chained errors)
  ErrorInfo? get cause => throw _privateConstructorUsedError;

  /// Timestamp
  String get timestamp => throw _privateConstructorUsedError;

  /// Trace ID
  String? get traceId => throw _privateConstructorUsedError;

  /// Additional context
  Map<String, dynamic> get context => throw _privateConstructorUsedError;

  /// Serializes this ErrorInfo to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ErrorInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ErrorInfoCopyWith<ErrorInfo> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ErrorInfoCopyWith<$Res> {
  factory $ErrorInfoCopyWith(ErrorInfo value, $Res Function(ErrorInfo) then) =
      _$ErrorInfoCopyWithImpl<$Res, ErrorInfo>;
  @useResult
  $Res call(
      {String type,
      String message,
      String? stackTrace,
      String? code,
      ErrorInfo? cause,
      String timestamp,
      String? traceId,
      Map<String, dynamic> context});

  $ErrorInfoCopyWith<$Res>? get cause;
}

/// @nodoc
class _$ErrorInfoCopyWithImpl<$Res, $Val extends ErrorInfo>
    implements $ErrorInfoCopyWith<$Res> {
  _$ErrorInfoCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ErrorInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? type = null,
    Object? message = null,
    Object? stackTrace = freezed,
    Object? code = freezed,
    Object? cause = freezed,
    Object? timestamp = null,
    Object? traceId = freezed,
    Object? context = null,
  }) {
    return _then(_value.copyWith(
      type: null == type
          ? _value.type
          : type // ignore: cast_nullable_to_non_nullable
              as String,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      stackTrace: freezed == stackTrace
          ? _value.stackTrace
          : stackTrace // ignore: cast_nullable_to_non_nullable
              as String?,
      code: freezed == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String?,
      cause: freezed == cause
          ? _value.cause
          : cause // ignore: cast_nullable_to_non_nullable
              as ErrorInfo?,
      timestamp: null == timestamp
          ? _value.timestamp
          : timestamp // ignore: cast_nullable_to_non_nullable
              as String,
      traceId: freezed == traceId
          ? _value.traceId
          : traceId // ignore: cast_nullable_to_non_nullable
              as String?,
      context: null == context
          ? _value.context
          : context // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ) as $Val);
  }

  /// Create a copy of ErrorInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ErrorInfoCopyWith<$Res>? get cause {
    if (_value.cause == null) {
      return null;
    }

    return $ErrorInfoCopyWith<$Res>(_value.cause!, (value) {
      return _then(_value.copyWith(cause: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$ErrorInfoImplCopyWith<$Res>
    implements $ErrorInfoCopyWith<$Res> {
  factory _$$ErrorInfoImplCopyWith(
          _$ErrorInfoImpl value, $Res Function(_$ErrorInfoImpl) then) =
      __$$ErrorInfoImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String type,
      String message,
      String? stackTrace,
      String? code,
      ErrorInfo? cause,
      String timestamp,
      String? traceId,
      Map<String, dynamic> context});

  @override
  $ErrorInfoCopyWith<$Res>? get cause;
}

/// @nodoc
class __$$ErrorInfoImplCopyWithImpl<$Res>
    extends _$ErrorInfoCopyWithImpl<$Res, _$ErrorInfoImpl>
    implements _$$ErrorInfoImplCopyWith<$Res> {
  __$$ErrorInfoImplCopyWithImpl(
      _$ErrorInfoImpl _value, $Res Function(_$ErrorInfoImpl) _then)
      : super(_value, _then);

  /// Create a copy of ErrorInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? type = null,
    Object? message = null,
    Object? stackTrace = freezed,
    Object? code = freezed,
    Object? cause = freezed,
    Object? timestamp = null,
    Object? traceId = freezed,
    Object? context = null,
  }) {
    return _then(_$ErrorInfoImpl(
      type: null == type
          ? _value.type
          : type // ignore: cast_nullable_to_non_nullable
              as String,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      stackTrace: freezed == stackTrace
          ? _value.stackTrace
          : stackTrace // ignore: cast_nullable_to_non_nullable
              as String?,
      code: freezed == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String?,
      cause: freezed == cause
          ? _value.cause
          : cause // ignore: cast_nullable_to_non_nullable
              as ErrorInfo?,
      timestamp: null == timestamp
          ? _value.timestamp
          : timestamp // ignore: cast_nullable_to_non_nullable
              as String,
      traceId: freezed == traceId
          ? _value.traceId
          : traceId // ignore: cast_nullable_to_non_nullable
              as String?,
      context: null == context
          ? _value._context
          : context // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ErrorInfoImpl extends _ErrorInfo {
  const _$ErrorInfoImpl(
      {required this.type,
      required this.message,
      this.stackTrace,
      this.code,
      this.cause,
      required this.timestamp,
      this.traceId,
      final Map<String, dynamic> context = const {}})
      : _context = context,
        super._();

  factory _$ErrorInfoImpl.fromJson(Map<String, dynamic> json) =>
      _$$ErrorInfoImplFromJson(json);

  /// Error type name
  @override
  final String type;

  /// Error message
  @override
  final String message;

  /// Stack trace
  @override
  final String? stackTrace;

  /// Error code
  @override
  final String? code;

  /// Original error info (for chained errors)
  @override
  final ErrorInfo? cause;

  /// Timestamp
  @override
  final String timestamp;

  /// Trace ID
  @override
  final String? traceId;

  /// Additional context
  final Map<String, dynamic> _context;

  /// Additional context
  @override
  @JsonKey()
  Map<String, dynamic> get context {
    if (_context is EqualUnmodifiableMapView) return _context;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_context);
  }

  @override
  String toString() {
    return 'ErrorInfo(type: $type, message: $message, stackTrace: $stackTrace, code: $code, cause: $cause, timestamp: $timestamp, traceId: $traceId, context: $context)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ErrorInfoImpl &&
            (identical(other.type, type) || other.type == type) &&
            (identical(other.message, message) || other.message == message) &&
            (identical(other.stackTrace, stackTrace) ||
                other.stackTrace == stackTrace) &&
            (identical(other.code, code) || other.code == code) &&
            (identical(other.cause, cause) || other.cause == cause) &&
            (identical(other.timestamp, timestamp) ||
                other.timestamp == timestamp) &&
            (identical(other.traceId, traceId) || other.traceId == traceId) &&
            const DeepCollectionEquality().equals(other._context, _context));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, type, message, stackTrace, code,
      cause, timestamp, traceId, const DeepCollectionEquality().hash(_context));

  /// Create a copy of ErrorInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ErrorInfoImplCopyWith<_$ErrorInfoImpl> get copyWith =>
      __$$ErrorInfoImplCopyWithImpl<_$ErrorInfoImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ErrorInfoImplToJson(
      this,
    );
  }
}

abstract class _ErrorInfo extends ErrorInfo {
  const factory _ErrorInfo(
      {required final String type,
      required final String message,
      final String? stackTrace,
      final String? code,
      final ErrorInfo? cause,
      required final String timestamp,
      final String? traceId,
      final Map<String, dynamic> context}) = _$ErrorInfoImpl;
  const _ErrorInfo._() : super._();

  factory _ErrorInfo.fromJson(Map<String, dynamic> json) =
      _$ErrorInfoImpl.fromJson;

  /// Error type name
  @override
  String get type;

  /// Error message
  @override
  String get message;

  /// Stack trace
  @override
  String? get stackTrace;

  /// Error code
  @override
  String? get code;

  /// Original error info (for chained errors)
  @override
  ErrorInfo? get cause;

  /// Timestamp
  @override
  String get timestamp;

  /// Trace ID
  @override
  String? get traceId;

  /// Additional context
  @override
  Map<String, dynamic> get context;

  /// Create a copy of ErrorInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ErrorInfoImplCopyWith<_$ErrorInfoImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
