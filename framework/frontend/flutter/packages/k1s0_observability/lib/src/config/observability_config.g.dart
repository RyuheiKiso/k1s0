// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'observability_config.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ObservabilityConfigImpl _$$ObservabilityConfigImplFromJson(
        Map<String, dynamic> json) =>
    _$ObservabilityConfigImpl(
      serviceName: json['serviceName'] as String,
      env: json['env'] as String,
      version: json['version'] as String?,
      logLevel: $enumDecodeNullable(_$LogLevelEnumMap, json['logLevel']) ??
          LogLevel.info,
      enableConsole: json['enableConsole'] as bool? ?? true,
      enableTracing: json['enableTracing'] as bool? ?? true,
      enableMetrics: json['enableMetrics'] as bool? ?? true,
      enableErrorTracking: json['enableErrorTracking'] as bool? ?? true,
      tracingSampleRate: (json['tracingSampleRate'] as num?)?.toDouble() ?? 1.0,
      otlpEndpoint: json['otlpEndpoint'] as String?,
      batchSize: (json['batchSize'] as num?)?.toInt() ?? 50,
      flushIntervalSeconds:
          (json['flushIntervalSeconds'] as num?)?.toInt() ?? 10,
    );

Map<String, dynamic> _$$ObservabilityConfigImplToJson(
        _$ObservabilityConfigImpl instance) =>
    <String, dynamic>{
      'serviceName': instance.serviceName,
      'env': instance.env,
      'version': instance.version,
      'logLevel': _$LogLevelEnumMap[instance.logLevel]!,
      'enableConsole': instance.enableConsole,
      'enableTracing': instance.enableTracing,
      'enableMetrics': instance.enableMetrics,
      'enableErrorTracking': instance.enableErrorTracking,
      'tracingSampleRate': instance.tracingSampleRate,
      'otlpEndpoint': instance.otlpEndpoint,
      'batchSize': instance.batchSize,
      'flushIntervalSeconds': instance.flushIntervalSeconds,
    };

const _$LogLevelEnumMap = {
  LogLevel.debug: 'debug',
  LogLevel.info: 'info',
  LogLevel.warn: 'warn',
  LogLevel.error: 'error',
};
