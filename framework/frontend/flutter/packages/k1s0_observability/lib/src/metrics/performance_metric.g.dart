// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'performance_metric.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$PerformanceMetricImpl _$$PerformanceMetricImplFromJson(
        Map<String, dynamic> json) =>
    _$PerformanceMetricImpl(
      name: json['name'] as String,
      value: (json['value'] as num).toDouble(),
      unit: $enumDecode(_$MetricUnitEnumMap, json['unit']),
      timestamp: (json['timestamp'] as num).toInt(),
      tags: (json['tags'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(k, e as String),
          ) ??
          const {},
    );

Map<String, dynamic> _$$PerformanceMetricImplToJson(
        _$PerformanceMetricImpl instance) =>
    <String, dynamic>{
      'name': instance.name,
      'value': instance.value,
      'unit': _$MetricUnitEnumMap[instance.unit]!,
      'timestamp': instance.timestamp,
      'tags': instance.tags,
    };

const _$MetricUnitEnumMap = {
  MetricUnit.milliseconds: 'milliseconds',
  MetricUnit.bytes: 'bytes',
  MetricUnit.count: 'count',
  MetricUnit.percent: 'percent',
};
