// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'problem_details.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ProblemDetailsImpl _$$ProblemDetailsImplFromJson(Map<String, dynamic> json) =>
    _$ProblemDetailsImpl(
      type: json['type'] as String? ?? 'about:blank',
      title: json['title'] as String,
      status: (json['status'] as num).toInt(),
      detail: json['detail'] as String?,
      instance: json['instance'] as String?,
      errorCode: json['error_code'] as String,
      traceId: json['trace_id'] as String?,
      errors: (json['errors'] as List<dynamic>?)
          ?.map((e) => FieldError.fromJson(e as Map<String, dynamic>))
          .toList(),
    );

Map<String, dynamic> _$$ProblemDetailsImplToJson(
        _$ProblemDetailsImpl instance) =>
    <String, dynamic>{
      'type': instance.type,
      'title': instance.title,
      'status': instance.status,
      'detail': instance.detail,
      'instance': instance.instance,
      'error_code': instance.errorCode,
      'trace_id': instance.traceId,
      'errors': instance.errors,
    };

_$FieldErrorImpl _$$FieldErrorImplFromJson(Map<String, dynamic> json) =>
    _$FieldErrorImpl(
      field: json['field'] as String,
      message: json['message'] as String,
      code: json['code'] as String?,
    );

Map<String, dynamic> _$$FieldErrorImplToJson(_$FieldErrorImpl instance) =>
    <String, dynamic>{
      'field': instance.field,
      'message': instance.message,
      'code': instance.code,
    };
