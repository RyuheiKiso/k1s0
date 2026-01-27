// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'claims.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ClaimsImpl _$$ClaimsImplFromJson(Map<String, dynamic> json) => _$ClaimsImpl(
      sub: json['sub'] as String,
      iss: json['iss'] as String,
      aud: const _AudienceConverter().fromJson(json['aud']),
      exp: (json['exp'] as num).toInt(),
      iat: (json['iat'] as num).toInt(),
      nbf: (json['nbf'] as num?)?.toInt(),
      jti: json['jti'] as String?,
      roles:
          (json['roles'] as List<dynamic>?)?.map((e) => e as String).toList() ??
              const [],
      permissions: (json['permissions'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
      tenantId: json['tenant_id'] as String?,
      scope: json['scope'] as String?,
    );

Map<String, dynamic> _$$ClaimsImplToJson(_$ClaimsImpl instance) =>
    <String, dynamic>{
      'sub': instance.sub,
      'iss': instance.iss,
      'aud': const _AudienceConverter().toJson(instance.aud),
      'exp': instance.exp,
      'iat': instance.iat,
      'nbf': instance.nbf,
      'jti': instance.jti,
      'roles': instance.roles,
      'permissions': instance.permissions,
      'tenant_id': instance.tenantId,
      'scope': instance.scope,
    };
