// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'span.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

SpanInfo _$SpanInfoFromJson(Map<String, dynamic> json) {
  return _SpanInfo.fromJson(json);
}

/// @nodoc
mixin _$SpanInfo {
  /// Trace ID
  String get traceId => throw _privateConstructorUsedError;

  /// Span ID
  String get spanId => throw _privateConstructorUsedError;

  /// Parent span ID
  String? get parentSpanId => throw _privateConstructorUsedError;

  /// Span name
  String get name => throw _privateConstructorUsedError;

  /// Start time (Unix timestamp in milliseconds)
  int get startTime => throw _privateConstructorUsedError;

  /// End time (Unix timestamp in milliseconds)
  int? get endTime => throw _privateConstructorUsedError;

  /// Span status
  SpanStatus get status => throw _privateConstructorUsedError;

  /// Status message
  String? get statusMessage => throw _privateConstructorUsedError;

  /// Attributes
  Map<String, Object> get attributes => throw _privateConstructorUsedError;

  /// Serializes this SpanInfo to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of SpanInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $SpanInfoCopyWith<SpanInfo> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SpanInfoCopyWith<$Res> {
  factory $SpanInfoCopyWith(SpanInfo value, $Res Function(SpanInfo) then) =
      _$SpanInfoCopyWithImpl<$Res, SpanInfo>;
  @useResult
  $Res call(
      {String traceId,
      String spanId,
      String? parentSpanId,
      String name,
      int startTime,
      int? endTime,
      SpanStatus status,
      String? statusMessage,
      Map<String, Object> attributes});
}

/// @nodoc
class _$SpanInfoCopyWithImpl<$Res, $Val extends SpanInfo>
    implements $SpanInfoCopyWith<$Res> {
  _$SpanInfoCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of SpanInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? traceId = null,
    Object? spanId = null,
    Object? parentSpanId = freezed,
    Object? name = null,
    Object? startTime = null,
    Object? endTime = freezed,
    Object? status = null,
    Object? statusMessage = freezed,
    Object? attributes = null,
  }) {
    return _then(_value.copyWith(
      traceId: null == traceId
          ? _value.traceId
          : traceId // ignore: cast_nullable_to_non_nullable
              as String,
      spanId: null == spanId
          ? _value.spanId
          : spanId // ignore: cast_nullable_to_non_nullable
              as String,
      parentSpanId: freezed == parentSpanId
          ? _value.parentSpanId
          : parentSpanId // ignore: cast_nullable_to_non_nullable
              as String?,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      startTime: null == startTime
          ? _value.startTime
          : startTime // ignore: cast_nullable_to_non_nullable
              as int,
      endTime: freezed == endTime
          ? _value.endTime
          : endTime // ignore: cast_nullable_to_non_nullable
              as int?,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as SpanStatus,
      statusMessage: freezed == statusMessage
          ? _value.statusMessage
          : statusMessage // ignore: cast_nullable_to_non_nullable
              as String?,
      attributes: null == attributes
          ? _value.attributes
          : attributes // ignore: cast_nullable_to_non_nullable
              as Map<String, Object>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SpanInfoImplCopyWith<$Res>
    implements $SpanInfoCopyWith<$Res> {
  factory _$$SpanInfoImplCopyWith(
          _$SpanInfoImpl value, $Res Function(_$SpanInfoImpl) then) =
      __$$SpanInfoImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String traceId,
      String spanId,
      String? parentSpanId,
      String name,
      int startTime,
      int? endTime,
      SpanStatus status,
      String? statusMessage,
      Map<String, Object> attributes});
}

/// @nodoc
class __$$SpanInfoImplCopyWithImpl<$Res>
    extends _$SpanInfoCopyWithImpl<$Res, _$SpanInfoImpl>
    implements _$$SpanInfoImplCopyWith<$Res> {
  __$$SpanInfoImplCopyWithImpl(
      _$SpanInfoImpl _value, $Res Function(_$SpanInfoImpl) _then)
      : super(_value, _then);

  /// Create a copy of SpanInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? traceId = null,
    Object? spanId = null,
    Object? parentSpanId = freezed,
    Object? name = null,
    Object? startTime = null,
    Object? endTime = freezed,
    Object? status = null,
    Object? statusMessage = freezed,
    Object? attributes = null,
  }) {
    return _then(_$SpanInfoImpl(
      traceId: null == traceId
          ? _value.traceId
          : traceId // ignore: cast_nullable_to_non_nullable
              as String,
      spanId: null == spanId
          ? _value.spanId
          : spanId // ignore: cast_nullable_to_non_nullable
              as String,
      parentSpanId: freezed == parentSpanId
          ? _value.parentSpanId
          : parentSpanId // ignore: cast_nullable_to_non_nullable
              as String?,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      startTime: null == startTime
          ? _value.startTime
          : startTime // ignore: cast_nullable_to_non_nullable
              as int,
      endTime: freezed == endTime
          ? _value.endTime
          : endTime // ignore: cast_nullable_to_non_nullable
              as int?,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as SpanStatus,
      statusMessage: freezed == statusMessage
          ? _value.statusMessage
          : statusMessage // ignore: cast_nullable_to_non_nullable
              as String?,
      attributes: null == attributes
          ? _value._attributes
          : attributes // ignore: cast_nullable_to_non_nullable
              as Map<String, Object>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SpanInfoImpl extends _SpanInfo {
  const _$SpanInfoImpl(
      {required this.traceId,
      required this.spanId,
      this.parentSpanId,
      required this.name,
      required this.startTime,
      this.endTime,
      this.status = SpanStatus.unset,
      this.statusMessage,
      final Map<String, Object> attributes = const {}})
      : _attributes = attributes,
        super._();

  factory _$SpanInfoImpl.fromJson(Map<String, dynamic> json) =>
      _$$SpanInfoImplFromJson(json);

  /// Trace ID
  @override
  final String traceId;

  /// Span ID
  @override
  final String spanId;

  /// Parent span ID
  @override
  final String? parentSpanId;

  /// Span name
  @override
  final String name;

  /// Start time (Unix timestamp in milliseconds)
  @override
  final int startTime;

  /// End time (Unix timestamp in milliseconds)
  @override
  final int? endTime;

  /// Span status
  @override
  @JsonKey()
  final SpanStatus status;

  /// Status message
  @override
  final String? statusMessage;

  /// Attributes
  final Map<String, Object> _attributes;

  /// Attributes
  @override
  @JsonKey()
  Map<String, Object> get attributes {
    if (_attributes is EqualUnmodifiableMapView) return _attributes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_attributes);
  }

  @override
  String toString() {
    return 'SpanInfo(traceId: $traceId, spanId: $spanId, parentSpanId: $parentSpanId, name: $name, startTime: $startTime, endTime: $endTime, status: $status, statusMessage: $statusMessage, attributes: $attributes)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SpanInfoImpl &&
            (identical(other.traceId, traceId) || other.traceId == traceId) &&
            (identical(other.spanId, spanId) || other.spanId == spanId) &&
            (identical(other.parentSpanId, parentSpanId) ||
                other.parentSpanId == parentSpanId) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.startTime, startTime) ||
                other.startTime == startTime) &&
            (identical(other.endTime, endTime) || other.endTime == endTime) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.statusMessage, statusMessage) ||
                other.statusMessage == statusMessage) &&
            const DeepCollectionEquality()
                .equals(other._attributes, _attributes));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      traceId,
      spanId,
      parentSpanId,
      name,
      startTime,
      endTime,
      status,
      statusMessage,
      const DeepCollectionEquality().hash(_attributes));

  /// Create a copy of SpanInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SpanInfoImplCopyWith<_$SpanInfoImpl> get copyWith =>
      __$$SpanInfoImplCopyWithImpl<_$SpanInfoImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SpanInfoImplToJson(
      this,
    );
  }
}

abstract class _SpanInfo extends SpanInfo {
  const factory _SpanInfo(
      {required final String traceId,
      required final String spanId,
      final String? parentSpanId,
      required final String name,
      required final int startTime,
      final int? endTime,
      final SpanStatus status,
      final String? statusMessage,
      final Map<String, Object> attributes}) = _$SpanInfoImpl;
  const _SpanInfo._() : super._();

  factory _SpanInfo.fromJson(Map<String, dynamic> json) =
      _$SpanInfoImpl.fromJson;

  /// Trace ID
  @override
  String get traceId;

  /// Span ID
  @override
  String get spanId;

  /// Parent span ID
  @override
  String? get parentSpanId;

  /// Span name
  @override
  String get name;

  /// Start time (Unix timestamp in milliseconds)
  @override
  int get startTime;

  /// End time (Unix timestamp in milliseconds)
  @override
  int? get endTime;

  /// Span status
  @override
  SpanStatus get status;

  /// Status message
  @override
  String? get statusMessage;

  /// Attributes
  @override
  Map<String, Object> get attributes;

  /// Create a copy of SpanInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SpanInfoImplCopyWith<_$SpanInfoImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
