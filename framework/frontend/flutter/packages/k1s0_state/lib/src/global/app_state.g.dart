// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_state.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$AppStateImpl _$$AppStateImplFromJson(Map<String, dynamic> json) =>
    _$AppStateImpl(
      initialized: json['initialized'] as bool? ?? false,
      loading: json['loading'] as bool? ?? false,
      environment: json['environment'] as String? ?? 'development',
      locale: json['locale'] as String? ?? 'en',
      isDarkMode: json['isDarkMode'] as bool? ?? false,
      featureFlags: (json['featureFlags'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(k, e as bool),
          ) ??
          const {},
      metadata: json['metadata'] as Map<String, dynamic>? ?? const {},
    );

Map<String, dynamic> _$$AppStateImplToJson(_$AppStateImpl instance) =>
    <String, dynamic>{
      'initialized': instance.initialized,
      'loading': instance.loading,
      'environment': instance.environment,
      'locale': instance.locale,
      'isDarkMode': instance.isDarkMode,
      'featureFlags': instance.featureFlags,
      'metadata': instance.metadata,
    };

_$UserPreferencesImpl _$$UserPreferencesImplFromJson(
        Map<String, dynamic> json) =>
    _$UserPreferencesImpl(
      themeMode: json['themeMode'] as String? ?? 'system',
      preferredLocale: json['preferredLocale'] as String?,
      notificationsEnabled: json['notificationsEnabled'] as bool? ?? true,
      analyticsConsent: json['analyticsConsent'] as bool? ?? false,
      custom: json['custom'] as Map<String, dynamic>? ?? const {},
    );

Map<String, dynamic> _$$UserPreferencesImplToJson(
        _$UserPreferencesImpl instance) =>
    <String, dynamic>{
      'themeMode': instance.themeMode,
      'preferredLocale': instance.preferredLocale,
      'notificationsEnabled': instance.notificationsEnabled,
      'analyticsConsent': instance.analyticsConsent,
      'custom': instance.custom,
    };

_$NavigationStateImpl _$$NavigationStateImplFromJson(
        Map<String, dynamic> json) =>
    _$NavigationStateImpl(
      currentPath: json['currentPath'] as String? ?? '/',
      previousPath: json['previousPath'] as String?,
      params: (json['params'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(k, e as String),
          ) ??
          const {},
      queryParams: (json['queryParams'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(k, e as String),
          ) ??
          const {},
      history: (json['history'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
    );

Map<String, dynamic> _$$NavigationStateImplToJson(
        _$NavigationStateImpl instance) =>
    <String, dynamic>{
      'currentPath': instance.currentPath,
      'previousPath': instance.previousPath,
      'params': instance.params,
      'queryParams': instance.queryParams,
      'history': instance.history,
    };

_$ConnectivityStateImpl _$$ConnectivityStateImplFromJson(
        Map<String, dynamic> json) =>
    _$ConnectivityStateImpl(
      isConnected: json['isConnected'] as bool? ?? true,
      connectionType: json['connectionType'] as String? ?? 'unknown',
      lastChecked: json['lastChecked'] == null
          ? null
          : DateTime.parse(json['lastChecked'] as String),
    );

Map<String, dynamic> _$$ConnectivityStateImplToJson(
        _$ConnectivityStateImpl instance) =>
    <String, dynamic>{
      'isConnected': instance.isConnected,
      'connectionType': instance.connectionType,
      'lastChecked': instance.lastChecked?.toIso8601String(),
    };
