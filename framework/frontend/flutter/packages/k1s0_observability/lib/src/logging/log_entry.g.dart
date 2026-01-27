// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'log_entry.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$LogEntryImpl _$$LogEntryImplFromJson(Map<String, dynamic> json) =>
    _$LogEntryImpl(
      timestamp: json['timestamp'] as String,
      level: $enumDecode(_$LogLevelEnumMap, json['level']),
      message: json['message'] as String,
      serviceName: json['service_name'] as String,
      env: json['env'] as String,
      traceId: json['trace_id'] as String,
      spanId: json['span_id'] as String,
      requestId: json['request_id'] as String?,
      errorInfo: json['error'] as Map<String, dynamic>?,
      extra: json['extra'] as Map<String, dynamic>? ?? const {},
    );

Map<String, dynamic> _$$LogEntryImplToJson(_$LogEntryImpl instance) =>
    <String, dynamic>{
      'timestamp': instance.timestamp,
      'level': _$LogLevelEnumMap[instance.level]!,
      'message': instance.message,
      'service_name': instance.serviceName,
      'env': instance.env,
      'trace_id': instance.traceId,
      'span_id': instance.spanId,
      'request_id': instance.requestId,
      'error': instance.errorInfo,
      'extra': instance.extra,
    };

const _$LogLevelEnumMap = {
  LogLevel.debug: 'debug',
  LogLevel.info: 'info',
  LogLevel.warn: 'warn',
  LogLevel.error: 'error',
};
