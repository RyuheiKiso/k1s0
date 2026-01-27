// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'observability_config.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ObservabilityConfig _$ObservabilityConfigFromJson(Map<String, dynamic> json) {
  return _ObservabilityConfig.fromJson(json);
}

/// @nodoc
mixin _$ObservabilityConfig {
  /// Service name
  String get serviceName => throw _privateConstructorUsedError;

  /// Environment (dev/stg/prod)
  String get env => throw _privateConstructorUsedError;

  /// Service version
  String? get version => throw _privateConstructorUsedError;

  /// Minimum log level
  LogLevel get logLevel => throw _privateConstructorUsedError;

  /// Enable console logging
  bool get enableConsole => throw _privateConstructorUsedError;

  /// Enable tracing
  bool get enableTracing => throw _privateConstructorUsedError;

  /// Enable metrics collection
  bool get enableMetrics => throw _privateConstructorUsedError;

  /// Enable error tracking
  bool get enableErrorTracking => throw _privateConstructorUsedError;

  /// Sampling rate for tracing (0.0 - 1.0)
  double get tracingSampleRate => throw _privateConstructorUsedError;

  /// OTLP endpoint for exporting
  String? get otlpEndpoint => throw _privateConstructorUsedError;

  /// Batch size for exports
  int get batchSize => throw _privateConstructorUsedError;

  /// Flush interval in seconds
  int get flushIntervalSeconds => throw _privateConstructorUsedError;

  /// Serializes this ObservabilityConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ObservabilityConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ObservabilityConfigCopyWith<ObservabilityConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ObservabilityConfigCopyWith<$Res> {
  factory $ObservabilityConfigCopyWith(
          ObservabilityConfig value, $Res Function(ObservabilityConfig) then) =
      _$ObservabilityConfigCopyWithImpl<$Res, ObservabilityConfig>;
  @useResult
  $Res call(
      {String serviceName,
      String env,
      String? version,
      LogLevel logLevel,
      bool enableConsole,
      bool enableTracing,
      bool enableMetrics,
      bool enableErrorTracking,
      double tracingSampleRate,
      String? otlpEndpoint,
      int batchSize,
      int flushIntervalSeconds});
}

/// @nodoc
class _$ObservabilityConfigCopyWithImpl<$Res, $Val extends ObservabilityConfig>
    implements $ObservabilityConfigCopyWith<$Res> {
  _$ObservabilityConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ObservabilityConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? serviceName = null,
    Object? env = null,
    Object? version = freezed,
    Object? logLevel = null,
    Object? enableConsole = null,
    Object? enableTracing = null,
    Object? enableMetrics = null,
    Object? enableErrorTracking = null,
    Object? tracingSampleRate = null,
    Object? otlpEndpoint = freezed,
    Object? batchSize = null,
    Object? flushIntervalSeconds = null,
  }) {
    return _then(_value.copyWith(
      serviceName: null == serviceName
          ? _value.serviceName
          : serviceName // ignore: cast_nullable_to_non_nullable
              as String,
      env: null == env
          ? _value.env
          : env // ignore: cast_nullable_to_non_nullable
              as String,
      version: freezed == version
          ? _value.version
          : version // ignore: cast_nullable_to_non_nullable
              as String?,
      logLevel: null == logLevel
          ? _value.logLevel
          : logLevel // ignore: cast_nullable_to_non_nullable
              as LogLevel,
      enableConsole: null == enableConsole
          ? _value.enableConsole
          : enableConsole // ignore: cast_nullable_to_non_nullable
              as bool,
      enableTracing: null == enableTracing
          ? _value.enableTracing
          : enableTracing // ignore: cast_nullable_to_non_nullable
              as bool,
      enableMetrics: null == enableMetrics
          ? _value.enableMetrics
          : enableMetrics // ignore: cast_nullable_to_non_nullable
              as bool,
      enableErrorTracking: null == enableErrorTracking
          ? _value.enableErrorTracking
          : enableErrorTracking // ignore: cast_nullable_to_non_nullable
              as bool,
      tracingSampleRate: null == tracingSampleRate
          ? _value.tracingSampleRate
          : tracingSampleRate // ignore: cast_nullable_to_non_nullable
              as double,
      otlpEndpoint: freezed == otlpEndpoint
          ? _value.otlpEndpoint
          : otlpEndpoint // ignore: cast_nullable_to_non_nullable
              as String?,
      batchSize: null == batchSize
          ? _value.batchSize
          : batchSize // ignore: cast_nullable_to_non_nullable
              as int,
      flushIntervalSeconds: null == flushIntervalSeconds
          ? _value.flushIntervalSeconds
          : flushIntervalSeconds // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ObservabilityConfigImplCopyWith<$Res>
    implements $ObservabilityConfigCopyWith<$Res> {
  factory _$$ObservabilityConfigImplCopyWith(_$ObservabilityConfigImpl value,
          $Res Function(_$ObservabilityConfigImpl) then) =
      __$$ObservabilityConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String serviceName,
      String env,
      String? version,
      LogLevel logLevel,
      bool enableConsole,
      bool enableTracing,
      bool enableMetrics,
      bool enableErrorTracking,
      double tracingSampleRate,
      String? otlpEndpoint,
      int batchSize,
      int flushIntervalSeconds});
}

/// @nodoc
class __$$ObservabilityConfigImplCopyWithImpl<$Res>
    extends _$ObservabilityConfigCopyWithImpl<$Res, _$ObservabilityConfigImpl>
    implements _$$ObservabilityConfigImplCopyWith<$Res> {
  __$$ObservabilityConfigImplCopyWithImpl(_$ObservabilityConfigImpl _value,
      $Res Function(_$ObservabilityConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of ObservabilityConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? serviceName = null,
    Object? env = null,
    Object? version = freezed,
    Object? logLevel = null,
    Object? enableConsole = null,
    Object? enableTracing = null,
    Object? enableMetrics = null,
    Object? enableErrorTracking = null,
    Object? tracingSampleRate = null,
    Object? otlpEndpoint = freezed,
    Object? batchSize = null,
    Object? flushIntervalSeconds = null,
  }) {
    return _then(_$ObservabilityConfigImpl(
      serviceName: null == serviceName
          ? _value.serviceName
          : serviceName // ignore: cast_nullable_to_non_nullable
              as String,
      env: null == env
          ? _value.env
          : env // ignore: cast_nullable_to_non_nullable
              as String,
      version: freezed == version
          ? _value.version
          : version // ignore: cast_nullable_to_non_nullable
              as String?,
      logLevel: null == logLevel
          ? _value.logLevel
          : logLevel // ignore: cast_nullable_to_non_nullable
              as LogLevel,
      enableConsole: null == enableConsole
          ? _value.enableConsole
          : enableConsole // ignore: cast_nullable_to_non_nullable
              as bool,
      enableTracing: null == enableTracing
          ? _value.enableTracing
          : enableTracing // ignore: cast_nullable_to_non_nullable
              as bool,
      enableMetrics: null == enableMetrics
          ? _value.enableMetrics
          : enableMetrics // ignore: cast_nullable_to_non_nullable
              as bool,
      enableErrorTracking: null == enableErrorTracking
          ? _value.enableErrorTracking
          : enableErrorTracking // ignore: cast_nullable_to_non_nullable
              as bool,
      tracingSampleRate: null == tracingSampleRate
          ? _value.tracingSampleRate
          : tracingSampleRate // ignore: cast_nullable_to_non_nullable
              as double,
      otlpEndpoint: freezed == otlpEndpoint
          ? _value.otlpEndpoint
          : otlpEndpoint // ignore: cast_nullable_to_non_nullable
              as String?,
      batchSize: null == batchSize
          ? _value.batchSize
          : batchSize // ignore: cast_nullable_to_non_nullable
              as int,
      flushIntervalSeconds: null == flushIntervalSeconds
          ? _value.flushIntervalSeconds
          : flushIntervalSeconds // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ObservabilityConfigImpl extends _ObservabilityConfig {
  const _$ObservabilityConfigImpl(
      {required this.serviceName,
      required this.env,
      this.version,
      this.logLevel = LogLevel.info,
      this.enableConsole = true,
      this.enableTracing = true,
      this.enableMetrics = true,
      this.enableErrorTracking = true,
      this.tracingSampleRate = 1.0,
      this.otlpEndpoint,
      this.batchSize = 50,
      this.flushIntervalSeconds = 10})
      : super._();

  factory _$ObservabilityConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$ObservabilityConfigImplFromJson(json);

  /// Service name
  @override
  final String serviceName;

  /// Environment (dev/stg/prod)
  @override
  final String env;

  /// Service version
  @override
  final String? version;

  /// Minimum log level
  @override
  @JsonKey()
  final LogLevel logLevel;

  /// Enable console logging
  @override
  @JsonKey()
  final bool enableConsole;

  /// Enable tracing
  @override
  @JsonKey()
  final bool enableTracing;

  /// Enable metrics collection
  @override
  @JsonKey()
  final bool enableMetrics;

  /// Enable error tracking
  @override
  @JsonKey()
  final bool enableErrorTracking;

  /// Sampling rate for tracing (0.0 - 1.0)
  @override
  @JsonKey()
  final double tracingSampleRate;

  /// OTLP endpoint for exporting
  @override
  final String? otlpEndpoint;

  /// Batch size for exports
  @override
  @JsonKey()
  final int batchSize;

  /// Flush interval in seconds
  @override
  @JsonKey()
  final int flushIntervalSeconds;

  @override
  String toString() {
    return 'ObservabilityConfig(serviceName: $serviceName, env: $env, version: $version, logLevel: $logLevel, enableConsole: $enableConsole, enableTracing: $enableTracing, enableMetrics: $enableMetrics, enableErrorTracking: $enableErrorTracking, tracingSampleRate: $tracingSampleRate, otlpEndpoint: $otlpEndpoint, batchSize: $batchSize, flushIntervalSeconds: $flushIntervalSeconds)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ObservabilityConfigImpl &&
            (identical(other.serviceName, serviceName) ||
                other.serviceName == serviceName) &&
            (identical(other.env, env) || other.env == env) &&
            (identical(other.version, version) || other.version == version) &&
            (identical(other.logLevel, logLevel) ||
                other.logLevel == logLevel) &&
            (identical(other.enableConsole, enableConsole) ||
                other.enableConsole == enableConsole) &&
            (identical(other.enableTracing, enableTracing) ||
                other.enableTracing == enableTracing) &&
            (identical(other.enableMetrics, enableMetrics) ||
                other.enableMetrics == enableMetrics) &&
            (identical(other.enableErrorTracking, enableErrorTracking) ||
                other.enableErrorTracking == enableErrorTracking) &&
            (identical(other.tracingSampleRate, tracingSampleRate) ||
                other.tracingSampleRate == tracingSampleRate) &&
            (identical(other.otlpEndpoint, otlpEndpoint) ||
                other.otlpEndpoint == otlpEndpoint) &&
            (identical(other.batchSize, batchSize) ||
                other.batchSize == batchSize) &&
            (identical(other.flushIntervalSeconds, flushIntervalSeconds) ||
                other.flushIntervalSeconds == flushIntervalSeconds));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      serviceName,
      env,
      version,
      logLevel,
      enableConsole,
      enableTracing,
      enableMetrics,
      enableErrorTracking,
      tracingSampleRate,
      otlpEndpoint,
      batchSize,
      flushIntervalSeconds);

  /// Create a copy of ObservabilityConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ObservabilityConfigImplCopyWith<_$ObservabilityConfigImpl> get copyWith =>
      __$$ObservabilityConfigImplCopyWithImpl<_$ObservabilityConfigImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ObservabilityConfigImplToJson(
      this,
    );
  }
}

abstract class _ObservabilityConfig extends ObservabilityConfig {
  const factory _ObservabilityConfig(
      {required final String serviceName,
      required final String env,
      final String? version,
      final LogLevel logLevel,
      final bool enableConsole,
      final bool enableTracing,
      final bool enableMetrics,
      final bool enableErrorTracking,
      final double tracingSampleRate,
      final String? otlpEndpoint,
      final int batchSize,
      final int flushIntervalSeconds}) = _$ObservabilityConfigImpl;
  const _ObservabilityConfig._() : super._();

  factory _ObservabilityConfig.fromJson(Map<String, dynamic> json) =
      _$ObservabilityConfigImpl.fromJson;

  /// Service name
  @override
  String get serviceName;

  /// Environment (dev/stg/prod)
  @override
  String get env;

  /// Service version
  @override
  String? get version;

  /// Minimum log level
  @override
  LogLevel get logLevel;

  /// Enable console logging
  @override
  bool get enableConsole;

  /// Enable tracing
  @override
  bool get enableTracing;

  /// Enable metrics collection
  @override
  bool get enableMetrics;

  /// Enable error tracking
  @override
  bool get enableErrorTracking;

  /// Sampling rate for tracing (0.0 - 1.0)
  @override
  double get tracingSampleRate;

  /// OTLP endpoint for exporting
  @override
  String? get otlpEndpoint;

  /// Batch size for exports
  @override
  int get batchSize;

  /// Flush interval in seconds
  @override
  int get flushIntervalSeconds;

  /// Create a copy of ObservabilityConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ObservabilityConfigImplCopyWith<_$ObservabilityConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
