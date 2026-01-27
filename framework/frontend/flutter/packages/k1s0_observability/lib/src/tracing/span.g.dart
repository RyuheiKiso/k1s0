// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'span.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$SpanInfoImpl _$$SpanInfoImplFromJson(Map<String, dynamic> json) =>
    _$SpanInfoImpl(
      traceId: json['traceId'] as String,
      spanId: json['spanId'] as String,
      parentSpanId: json['parentSpanId'] as String?,
      name: json['name'] as String,
      startTime: (json['startTime'] as num).toInt(),
      endTime: (json['endTime'] as num?)?.toInt(),
      status: $enumDecodeNullable(_$SpanStatusEnumMap, json['status']) ??
          SpanStatus.unset,
      statusMessage: json['statusMessage'] as String?,
      attributes: (json['attributes'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(k, e as Object),
          ) ??
          const {},
    );

Map<String, dynamic> _$$SpanInfoImplToJson(_$SpanInfoImpl instance) =>
    <String, dynamic>{
      'traceId': instance.traceId,
      'spanId': instance.spanId,
      'parentSpanId': instance.parentSpanId,
      'name': instance.name,
      'startTime': instance.startTime,
      'endTime': instance.endTime,
      'status': _$SpanStatusEnumMap[instance.status]!,
      'statusMessage': instance.statusMessage,
      'attributes': instance.attributes,
    };

const _$SpanStatusEnumMap = {
  SpanStatus.ok: 'ok',
  SpanStatus.error: 'error',
  SpanStatus.unset: 'unset',
};
