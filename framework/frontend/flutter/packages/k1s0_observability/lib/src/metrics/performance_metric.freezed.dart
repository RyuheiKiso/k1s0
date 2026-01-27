// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'performance_metric.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

PerformanceMetric _$PerformanceMetricFromJson(Map<String, dynamic> json) {
  return _PerformanceMetric.fromJson(json);
}

/// @nodoc
mixin _$PerformanceMetric {
  /// Metric name
  String get name => throw _privateConstructorUsedError;

  /// Metric value
  double get value => throw _privateConstructorUsedError;

  /// Metric unit
  MetricUnit get unit => throw _privateConstructorUsedError;

  /// Timestamp (Unix milliseconds)
  int get timestamp => throw _privateConstructorUsedError;

  /// Tags for grouping
  Map<String, String> get tags => throw _privateConstructorUsedError;

  /// Serializes this PerformanceMetric to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PerformanceMetric
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PerformanceMetricCopyWith<PerformanceMetric> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PerformanceMetricCopyWith<$Res> {
  factory $PerformanceMetricCopyWith(
          PerformanceMetric value, $Res Function(PerformanceMetric) then) =
      _$PerformanceMetricCopyWithImpl<$Res, PerformanceMetric>;
  @useResult
  $Res call(
      {String name,
      double value,
      MetricUnit unit,
      int timestamp,
      Map<String, String> tags});
}

/// @nodoc
class _$PerformanceMetricCopyWithImpl<$Res, $Val extends PerformanceMetric>
    implements $PerformanceMetricCopyWith<$Res> {
  _$PerformanceMetricCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PerformanceMetric
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? value = null,
    Object? unit = null,
    Object? timestamp = null,
    Object? tags = null,
  }) {
    return _then(_value.copyWith(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as double,
      unit: null == unit
          ? _value.unit
          : unit // ignore: cast_nullable_to_non_nullable
              as MetricUnit,
      timestamp: null == timestamp
          ? _value.timestamp
          : timestamp // ignore: cast_nullable_to_non_nullable
              as int,
      tags: null == tags
          ? _value.tags
          : tags // ignore: cast_nullable_to_non_nullable
              as Map<String, String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$PerformanceMetricImplCopyWith<$Res>
    implements $PerformanceMetricCopyWith<$Res> {
  factory _$$PerformanceMetricImplCopyWith(_$PerformanceMetricImpl value,
          $Res Function(_$PerformanceMetricImpl) then) =
      __$$PerformanceMetricImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String name,
      double value,
      MetricUnit unit,
      int timestamp,
      Map<String, String> tags});
}

/// @nodoc
class __$$PerformanceMetricImplCopyWithImpl<$Res>
    extends _$PerformanceMetricCopyWithImpl<$Res, _$PerformanceMetricImpl>
    implements _$$PerformanceMetricImplCopyWith<$Res> {
  __$$PerformanceMetricImplCopyWithImpl(_$PerformanceMetricImpl _value,
      $Res Function(_$PerformanceMetricImpl) _then)
      : super(_value, _then);

  /// Create a copy of PerformanceMetric
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? value = null,
    Object? unit = null,
    Object? timestamp = null,
    Object? tags = null,
  }) {
    return _then(_$PerformanceMetricImpl(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as double,
      unit: null == unit
          ? _value.unit
          : unit // ignore: cast_nullable_to_non_nullable
              as MetricUnit,
      timestamp: null == timestamp
          ? _value.timestamp
          : timestamp // ignore: cast_nullable_to_non_nullable
              as int,
      tags: null == tags
          ? _value._tags
          : tags // ignore: cast_nullable_to_non_nullable
              as Map<String, String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$PerformanceMetricImpl extends _PerformanceMetric {
  const _$PerformanceMetricImpl(
      {required this.name,
      required this.value,
      required this.unit,
      required this.timestamp,
      final Map<String, String> tags = const {}})
      : _tags = tags,
        super._();

  factory _$PerformanceMetricImpl.fromJson(Map<String, dynamic> json) =>
      _$$PerformanceMetricImplFromJson(json);

  /// Metric name
  @override
  final String name;

  /// Metric value
  @override
  final double value;

  /// Metric unit
  @override
  final MetricUnit unit;

  /// Timestamp (Unix milliseconds)
  @override
  final int timestamp;

  /// Tags for grouping
  final Map<String, String> _tags;

  /// Tags for grouping
  @override
  @JsonKey()
  Map<String, String> get tags {
    if (_tags is EqualUnmodifiableMapView) return _tags;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_tags);
  }

  @override
  String toString() {
    return 'PerformanceMetric(name: $name, value: $value, unit: $unit, timestamp: $timestamp, tags: $tags)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PerformanceMetricImpl &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.value, value) || other.value == value) &&
            (identical(other.unit, unit) || other.unit == unit) &&
            (identical(other.timestamp, timestamp) ||
                other.timestamp == timestamp) &&
            const DeepCollectionEquality().equals(other._tags, _tags));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, name, value, unit, timestamp,
      const DeepCollectionEquality().hash(_tags));

  /// Create a copy of PerformanceMetric
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PerformanceMetricImplCopyWith<_$PerformanceMetricImpl> get copyWith =>
      __$$PerformanceMetricImplCopyWithImpl<_$PerformanceMetricImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PerformanceMetricImplToJson(
      this,
    );
  }
}

abstract class _PerformanceMetric extends PerformanceMetric {
  const factory _PerformanceMetric(
      {required final String name,
      required final double value,
      required final MetricUnit unit,
      required final int timestamp,
      final Map<String, String> tags}) = _$PerformanceMetricImpl;
  const _PerformanceMetric._() : super._();

  factory _PerformanceMetric.fromJson(Map<String, dynamic> json) =
      _$PerformanceMetricImpl.fromJson;

  /// Metric name
  @override
  String get name;

  /// Metric value
  @override
  double get value;

  /// Metric unit
  @override
  MetricUnit get unit;

  /// Timestamp (Unix milliseconds)
  @override
  int get timestamp;

  /// Tags for grouping
  @override
  Map<String, String> get tags;

  /// Create a copy of PerformanceMetric
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PerformanceMetricImplCopyWith<_$PerformanceMetricImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
