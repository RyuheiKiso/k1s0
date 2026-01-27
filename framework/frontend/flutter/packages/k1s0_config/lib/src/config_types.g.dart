// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'config_types.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ApiConfigImpl _$$ApiConfigImplFromJson(Map<String, dynamic> json) =>
    _$ApiConfigImpl(
      baseUrl: json['baseUrl'] as String,
      timeout: (json['timeout'] as num?)?.toInt() ?? 30000,
      retryCount: (json['retryCount'] as num?)?.toInt() ?? 3,
      retryDelay: (json['retryDelay'] as num?)?.toInt() ?? 1000,
    );

Map<String, dynamic> _$$ApiConfigImplToJson(_$ApiConfigImpl instance) =>
    <String, dynamic>{
      'baseUrl': instance.baseUrl,
      'timeout': instance.timeout,
      'retryCount': instance.retryCount,
      'retryDelay': instance.retryDelay,
    };

_$AuthConfigImpl _$$AuthConfigImplFromJson(Map<String, dynamic> json) =>
    _$AuthConfigImpl(
      enabled: json['enabled'] as bool? ?? true,
      provider: json['provider'] as String? ?? 'jwt',
      tokenRefreshThreshold:
          (json['tokenRefreshThreshold'] as num?)?.toInt() ?? 300,
      storage: json['storage'] as String? ?? 'secure',
      oidc: json['oidc'] == null
          ? null
          : OidcConfig.fromJson(json['oidc'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$$AuthConfigImplToJson(_$AuthConfigImpl instance) =>
    <String, dynamic>{
      'enabled': instance.enabled,
      'provider': instance.provider,
      'tokenRefreshThreshold': instance.tokenRefreshThreshold,
      'storage': instance.storage,
      'oidc': instance.oidc,
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
    );

Map<String, dynamic> _$$OidcConfigImplToJson(_$OidcConfigImpl instance) =>
    <String, dynamic>{
      'issuer': instance.issuer,
      'clientId': instance.clientId,
      'redirectUri': instance.redirectUri,
      'scopes': instance.scopes,
      'postLogoutRedirectUri': instance.postLogoutRedirectUri,
    };

_$LoggingConfigImpl _$$LoggingConfigImplFromJson(Map<String, dynamic> json) =>
    _$LoggingConfigImpl(
      level: json['level'] as String? ?? 'info',
      enableConsole: json['enableConsole'] as bool? ?? true,
      enableRemote: json['enableRemote'] as bool? ?? false,
      remoteEndpoint: json['remoteEndpoint'] as String?,
    );

Map<String, dynamic> _$$LoggingConfigImplToJson(_$LoggingConfigImpl instance) =>
    <String, dynamic>{
      'level': instance.level,
      'enableConsole': instance.enableConsole,
      'enableRemote': instance.enableRemote,
      'remoteEndpoint': instance.remoteEndpoint,
    };

_$TelemetryConfigImpl _$$TelemetryConfigImplFromJson(
        Map<String, dynamic> json) =>
    _$TelemetryConfigImpl(
      enabled: json['enabled'] as bool? ?? false,
      serviceName: json['serviceName'] as String? ?? 'k1s0-flutter',
      endpoint: json['endpoint'] as String?,
      sampleRate: (json['sampleRate'] as num?)?.toDouble() ?? 0.1,
    );

Map<String, dynamic> _$$TelemetryConfigImplToJson(
        _$TelemetryConfigImpl instance) =>
    <String, dynamic>{
      'enabled': instance.enabled,
      'serviceName': instance.serviceName,
      'endpoint': instance.endpoint,
      'sampleRate': instance.sampleRate,
    };

_$FeatureFlagsImpl _$$FeatureFlagsImplFromJson(Map<String, dynamic> json) =>
    _$FeatureFlagsImpl(
      flags: (json['flags'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(k, e as bool),
          ) ??
          const {},
    );

Map<String, dynamic> _$$FeatureFlagsImplToJson(_$FeatureFlagsImpl instance) =>
    <String, dynamic>{
      'flags': instance.flags,
    };

_$AppConfigImpl _$$AppConfigImplFromJson(Map<String, dynamic> json) =>
    _$AppConfigImpl(
      env: $enumDecodeNullable(_$EnvironmentEnumMap, json['env']) ??
          Environment.dev,
      appName: json['appName'] as String? ?? 'k1s0-app',
      version: json['version'] as String?,
      api: json['api'] == null
          ? null
          : ApiConfig.fromJson(json['api'] as Map<String, dynamic>),
      auth: json['auth'] == null
          ? null
          : AuthConfig.fromJson(json['auth'] as Map<String, dynamic>),
      logging: json['logging'] == null
          ? null
          : LoggingConfig.fromJson(json['logging'] as Map<String, dynamic>),
      telemetry: json['telemetry'] == null
          ? null
          : TelemetryConfig.fromJson(json['telemetry'] as Map<String, dynamic>),
      features: json['features'] == null
          ? null
          : FeatureFlags.fromJson(json['features'] as Map<String, dynamic>),
      custom: json['custom'] as Map<String, dynamic>? ?? const {},
    );

Map<String, dynamic> _$$AppConfigImplToJson(_$AppConfigImpl instance) =>
    <String, dynamic>{
      'env': _$EnvironmentEnumMap[instance.env]!,
      'appName': instance.appName,
      'version': instance.version,
      'api': instance.api,
      'auth': instance.auth,
      'logging': instance.logging,
      'telemetry': instance.telemetry,
      'features': instance.features,
      'custom': instance.custom,
    };

const _$EnvironmentEnumMap = {
  Environment.dev: 'dev',
  Environment.stg: 'stg',
  Environment.prod: 'prod',
};
