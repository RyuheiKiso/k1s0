// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'error_info.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ErrorInfoImpl _$$ErrorInfoImplFromJson(Map<String, dynamic> json) =>
    _$ErrorInfoImpl(
      type: json['type'] as String,
      message: json['message'] as String,
      stackTrace: json['stackTrace'] as String?,
      code: json['code'] as String?,
      cause: json['cause'] == null
          ? null
          : ErrorInfo.fromJson(json['cause'] as Map<String, dynamic>),
      timestamp: json['timestamp'] as String,
      traceId: json['traceId'] as String?,
      context: json['context'] as Map<String, dynamic>? ?? const {},
    );

Map<String, dynamic> _$$ErrorInfoImplToJson(_$ErrorInfoImpl instance) =>
    <String, dynamic>{
      'type': instance.type,
      'message': instance.message,
      'stackTrace': instance.stackTrace,
      'code': instance.code,
      'cause': instance.cause,
      'timestamp': instance.timestamp,
      'traceId': instance.traceId,
      'context': instance.context,
    };
