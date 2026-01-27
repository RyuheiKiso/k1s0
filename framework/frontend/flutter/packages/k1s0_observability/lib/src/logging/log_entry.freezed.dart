// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'log_entry.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

LogEntry _$LogEntryFromJson(Map<String, dynamic> json) {
  return _LogEntry.fromJson(json);
}

/// @nodoc
mixin _$LogEntry {
  /// ISO 8601 timestamp
  String get timestamp => throw _privateConstructorUsedError;

  /// Log level
  LogLevel get level => throw _privateConstructorUsedError;

  /// Log message
  String get message => throw _privateConstructorUsedError;

  /// Service name
  @JsonKey(name: 'service_name')
  String get serviceName => throw _privateConstructorUsedError;

  /// Environment (dev/stg/prod)
  String get env => throw _privateConstructorUsedError;

  /// Trace ID for request correlation
  @JsonKey(name: 'trace_id')
  String get traceId => throw _privateConstructorUsedError;

  /// Span ID
  @JsonKey(name: 'span_id')
  String get spanId => throw _privateConstructorUsedError;

  /// Request ID
  @JsonKey(name: 'request_id')
  String? get requestId => throw _privateConstructorUsedError;

  /// Error information
  @JsonKey(name: 'error')
  Map<String, dynamic>? get errorInfo => throw _privateConstructorUsedError;

  /// Additional fields
  Map<String, dynamic> get extra => throw _privateConstructorUsedError;

  /// Serializes this LogEntry to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of LogEntry
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $LogEntryCopyWith<LogEntry> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $LogEntryCopyWith<$Res> {
  factory $LogEntryCopyWith(LogEntry value, $Res Function(LogEntry) then) =
      _$LogEntryCopyWithImpl<$Res, LogEntry>;
  @useResult
  $Res call(
      {String timestamp,
      LogLevel level,
      String message,
      @JsonKey(name: 'service_name') String serviceName,
      String env,
      @JsonKey(name: 'trace_id') String traceId,
      @JsonKey(name: 'span_id') String spanId,
      @JsonKey(name: 'request_id') String? requestId,
      @JsonKey(name: 'error') Map<String, dynamic>? errorInfo,
      Map<String, dynamic> extra});
}

/// @nodoc
class _$LogEntryCopyWithImpl<$Res, $Val extends LogEntry>
    implements $LogEntryCopyWith<$Res> {
  _$LogEntryCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of LogEntry
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timestamp = null,
    Object? level = null,
    Object? message = null,
    Object? serviceName = null,
    Object? env = null,
    Object? traceId = null,
    Object? spanId = null,
    Object? requestId = freezed,
    Object? errorInfo = freezed,
    Object? extra = null,
  }) {
    return _then(_value.copyWith(
      timestamp: null == timestamp
          ? _value.timestamp
          : timestamp // ignore: cast_nullable_to_non_nullable
              as String,
      level: null == level
          ? _value.level
          : level // ignore: cast_nullable_to_non_nullable
              as LogLevel,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      serviceName: null == serviceName
          ? _value.serviceName
          : serviceName // ignore: cast_nullable_to_non_nullable
              as String,
      env: null == env
          ? _value.env
          : env // ignore: cast_nullable_to_non_nullable
              as String,
      traceId: null == traceId
          ? _value.traceId
          : traceId // ignore: cast_nullable_to_non_nullable
              as String,
      spanId: null == spanId
          ? _value.spanId
          : spanId // ignore: cast_nullable_to_non_nullable
              as String,
      requestId: freezed == requestId
          ? _value.requestId
          : requestId // ignore: cast_nullable_to_non_nullable
              as String?,
      errorInfo: freezed == errorInfo
          ? _value.errorInfo
          : errorInfo // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>?,
      extra: null == extra
          ? _value.extra
          : extra // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$LogEntryImplCopyWith<$Res>
    implements $LogEntryCopyWith<$Res> {
  factory _$$LogEntryImplCopyWith(
          _$LogEntryImpl value, $Res Function(_$LogEntryImpl) then) =
      __$$LogEntryImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String timestamp,
      LogLevel level,
      String message,
      @JsonKey(name: 'service_name') String serviceName,
      String env,
      @JsonKey(name: 'trace_id') String traceId,
      @JsonKey(name: 'span_id') String spanId,
      @JsonKey(name: 'request_id') String? requestId,
      @JsonKey(name: 'error') Map<String, dynamic>? errorInfo,
      Map<String, dynamic> extra});
}

/// @nodoc
class __$$LogEntryImplCopyWithImpl<$Res>
    extends _$LogEntryCopyWithImpl<$Res, _$LogEntryImpl>
    implements _$$LogEntryImplCopyWith<$Res> {
  __$$LogEntryImplCopyWithImpl(
      _$LogEntryImpl _value, $Res Function(_$LogEntryImpl) _then)
      : super(_value, _then);

  /// Create a copy of LogEntry
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timestamp = null,
    Object? level = null,
    Object? message = null,
    Object? serviceName = null,
    Object? env = null,
    Object? traceId = null,
    Object? spanId = null,
    Object? requestId = freezed,
    Object? errorInfo = freezed,
    Object? extra = null,
  }) {
    return _then(_$LogEntryImpl(
      timestamp: null == timestamp
          ? _value.timestamp
          : timestamp // ignore: cast_nullable_to_non_nullable
              as String,
      level: null == level
          ? _value.level
          : level // ignore: cast_nullable_to_non_nullable
              as LogLevel,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      serviceName: null == serviceName
          ? _value.serviceName
          : serviceName // ignore: cast_nullable_to_non_nullable
              as String,
      env: null == env
          ? _value.env
          : env // ignore: cast_nullable_to_non_nullable
              as String,
      traceId: null == traceId
          ? _value.traceId
          : traceId // ignore: cast_nullable_to_non_nullable
              as String,
      spanId: null == spanId
          ? _value.spanId
          : spanId // ignore: cast_nullable_to_non_nullable
              as String,
      requestId: freezed == requestId
          ? _value.requestId
          : requestId // ignore: cast_nullable_to_non_nullable
              as String?,
      errorInfo: freezed == errorInfo
          ? _value._errorInfo
          : errorInfo // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>?,
      extra: null == extra
          ? _value._extra
          : extra // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$LogEntryImpl extends _LogEntry {
  const _$LogEntryImpl(
      {required this.timestamp,
      required this.level,
      required this.message,
      @JsonKey(name: 'service_name') required this.serviceName,
      required this.env,
      @JsonKey(name: 'trace_id') required this.traceId,
      @JsonKey(name: 'span_id') required this.spanId,
      @JsonKey(name: 'request_id') this.requestId,
      @JsonKey(name: 'error') final Map<String, dynamic>? errorInfo,
      final Map<String, dynamic> extra = const {}})
      : _errorInfo = errorInfo,
        _extra = extra,
        super._();

  factory _$LogEntryImpl.fromJson(Map<String, dynamic> json) =>
      _$$LogEntryImplFromJson(json);

  /// ISO 8601 timestamp
  @override
  final String timestamp;

  /// Log level
  @override
  final LogLevel level;

  /// Log message
  @override
  final String message;

  /// Service name
  @override
  @JsonKey(name: 'service_name')
  final String serviceName;

  /// Environment (dev/stg/prod)
  @override
  final String env;

  /// Trace ID for request correlation
  @override
  @JsonKey(name: 'trace_id')
  final String traceId;

  /// Span ID
  @override
  @JsonKey(name: 'span_id')
  final String spanId;

  /// Request ID
  @override
  @JsonKey(name: 'request_id')
  final String? requestId;

  /// Error information
  final Map<String, dynamic>? _errorInfo;

  /// Error information
  @override
  @JsonKey(name: 'error')
  Map<String, dynamic>? get errorInfo {
    final value = _errorInfo;
    if (value == null) return null;
    if (_errorInfo is EqualUnmodifiableMapView) return _errorInfo;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(value);
  }

  /// Additional fields
  final Map<String, dynamic> _extra;

  /// Additional fields
  @override
  @JsonKey()
  Map<String, dynamic> get extra {
    if (_extra is EqualUnmodifiableMapView) return _extra;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_extra);
  }

  @override
  String toString() {
    return 'LogEntry(timestamp: $timestamp, level: $level, message: $message, serviceName: $serviceName, env: $env, traceId: $traceId, spanId: $spanId, requestId: $requestId, errorInfo: $errorInfo, extra: $extra)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$LogEntryImpl &&
            (identical(other.timestamp, timestamp) ||
                other.timestamp == timestamp) &&
            (identical(other.level, level) || other.level == level) &&
            (identical(other.message, message) || other.message == message) &&
            (identical(other.serviceName, serviceName) ||
                other.serviceName == serviceName) &&
            (identical(other.env, env) || other.env == env) &&
            (identical(other.traceId, traceId) || other.traceId == traceId) &&
            (identical(other.spanId, spanId) || other.spanId == spanId) &&
            (identical(other.requestId, requestId) ||
                other.requestId == requestId) &&
            const DeepCollectionEquality()
                .equals(other._errorInfo, _errorInfo) &&
            const DeepCollectionEquality().equals(other._extra, _extra));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      timestamp,
      level,
      message,
      serviceName,
      env,
      traceId,
      spanId,
      requestId,
      const DeepCollectionEquality().hash(_errorInfo),
      const DeepCollectionEquality().hash(_extra));

  /// Create a copy of LogEntry
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$LogEntryImplCopyWith<_$LogEntryImpl> get copyWith =>
      __$$LogEntryImplCopyWithImpl<_$LogEntryImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$LogEntryImplToJson(
      this,
    );
  }
}

abstract class _LogEntry extends LogEntry {
  const factory _LogEntry(
      {required final String timestamp,
      required final LogLevel level,
      required final String message,
      @JsonKey(name: 'service_name') required final String serviceName,
      required final String env,
      @JsonKey(name: 'trace_id') required final String traceId,
      @JsonKey(name: 'span_id') required final String spanId,
      @JsonKey(name: 'request_id') final String? requestId,
      @JsonKey(name: 'error') final Map<String, dynamic>? errorInfo,
      final Map<String, dynamic> extra}) = _$LogEntryImpl;
  const _LogEntry._() : super._();

  factory _LogEntry.fromJson(Map<String, dynamic> json) =
      _$LogEntryImpl.fromJson;

  /// ISO 8601 timestamp
  @override
  String get timestamp;

  /// Log level
  @override
  LogLevel get level;

  /// Log message
  @override
  String get message;

  /// Service name
  @override
  @JsonKey(name: 'service_name')
  String get serviceName;

  /// Environment (dev/stg/prod)
  @override
  String get env;

  /// Trace ID for request correlation
  @override
  @JsonKey(name: 'trace_id')
  String get traceId;

  /// Span ID
  @override
  @JsonKey(name: 'span_id')
  String get spanId;

  /// Request ID
  @override
  @JsonKey(name: 'request_id')
  String? get requestId;

  /// Error information
  @override
  @JsonKey(name: 'error')
  Map<String, dynamic>? get errorInfo;

  /// Additional fields
  @override
  Map<String, dynamic> get extra;

  /// Create a copy of LogEntry
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$LogEntryImplCopyWith<_$LogEntryImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
