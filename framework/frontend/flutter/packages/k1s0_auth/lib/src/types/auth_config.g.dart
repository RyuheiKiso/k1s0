// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'auth_config.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$AuthConfigImpl _$$AuthConfigImplFromJson(Map<String, dynamic> json) =>
    _$AuthConfigImpl(
      enabled: json['enabled'] as bool? ?? true,
      storageType:
          $enumDecodeNullable(_$TokenStorageTypeEnumMap, json['storageType']) ??
              TokenStorageType.secure,
      refreshMarginSeconds:
          (json['refreshMarginSeconds'] as num?)?.toInt() ?? 300,
      autoRefresh: json['autoRefresh'] as bool? ?? true,
      allowedIssuers: (json['allowedIssuers'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList(),
      allowedAudiences: (json['allowedAudiences'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList(),
      oidc: json['oidc'] == null
          ? null
          : OidcConfig.fromJson(json['oidc'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$$AuthConfigImplToJson(_$AuthConfigImpl instance) =>
    <String, dynamic>{
      'enabled': instance.enabled,
      'storageType': _$TokenStorageTypeEnumMap[instance.storageType]!,
      'refreshMarginSeconds': instance.refreshMarginSeconds,
      'autoRefresh': instance.autoRefresh,
      'allowedIssuers': instance.allowedIssuers,
      'allowedAudiences': instance.allowedAudiences,
      'oidc': instance.oidc,
    };

const _$TokenStorageTypeEnumMap = {
  TokenStorageType.secure: 'secure',
  TokenStorageType.memory: 'memory',
};

_$OidcConfigImpl _$$OidcConfigImplFromJson(Map<String, dynamic> json) =>
    _$OidcConfigImpl(
      issuer: json['issuer'] as String,
      clientId: json['clientId'] as String,
      redirectUri: json['redirectUri'] as String,
      scopes: (json['scopes'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const ['openid', 'profile', 'email'],
      postLogoutRedirectUri: json['postLogoutRedirectUri'] as String?,
      discoveryUrl: json['discoveryUrl'] as String?,
    );

Map<String, dynamic> _$$OidcConfigImplToJson(_$OidcConfigImpl instance) =>
    <String, dynamic>{
      'issuer': instance.issuer,
      'clientId': instance.clientId,
      'redirectUri': instance.redirectUri,
      'scopes': instance.scopes,
      'postLogoutRedirectUri': instance.postLogoutRedirectUri,
      'discoveryUrl': instance.discoveryUrl,
    };
