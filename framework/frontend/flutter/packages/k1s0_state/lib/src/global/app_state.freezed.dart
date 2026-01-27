// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'app_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

AppState _$AppStateFromJson(Map<String, dynamic> json) {
  return _AppState.fromJson(json);
}

/// @nodoc
mixin _$AppState {
  /// Whether the app has been initialized.
  bool get initialized => throw _privateConstructorUsedError;

  /// Whether the app is in a loading state.
  bool get loading => throw _privateConstructorUsedError;

  /// Current environment (e.g., 'development', 'staging', 'production').
  String get environment => throw _privateConstructorUsedError;

  /// Current locale code (e.g., 'en', 'ja').
  String get locale => throw _privateConstructorUsedError;

  /// Whether dark mode is enabled.
  bool get isDarkMode => throw _privateConstructorUsedError;

  /// Feature flags.
  Map<String, bool> get featureFlags => throw _privateConstructorUsedError;

  /// Custom metadata.
  Map<String, dynamic> get metadata => throw _privateConstructorUsedError;

  /// Serializes this AppState to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of AppState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AppStateCopyWith<AppState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AppStateCopyWith<$Res> {
  factory $AppStateCopyWith(AppState value, $Res Function(AppState) then) =
      _$AppStateCopyWithImpl<$Res, AppState>;
  @useResult
  $Res call(
      {bool initialized,
      bool loading,
      String environment,
      String locale,
      bool isDarkMode,
      Map<String, bool> featureFlags,
      Map<String, dynamic> metadata});
}

/// @nodoc
class _$AppStateCopyWithImpl<$Res, $Val extends AppState>
    implements $AppStateCopyWith<$Res> {
  _$AppStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AppState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? initialized = null,
    Object? loading = null,
    Object? environment = null,
    Object? locale = null,
    Object? isDarkMode = null,
    Object? featureFlags = null,
    Object? metadata = null,
  }) {
    return _then(_value.copyWith(
      initialized: null == initialized
          ? _value.initialized
          : initialized // ignore: cast_nullable_to_non_nullable
              as bool,
      loading: null == loading
          ? _value.loading
          : loading // ignore: cast_nullable_to_non_nullable
              as bool,
      environment: null == environment
          ? _value.environment
          : environment // ignore: cast_nullable_to_non_nullable
              as String,
      locale: null == locale
          ? _value.locale
          : locale // ignore: cast_nullable_to_non_nullable
              as String,
      isDarkMode: null == isDarkMode
          ? _value.isDarkMode
          : isDarkMode // ignore: cast_nullable_to_non_nullable
              as bool,
      featureFlags: null == featureFlags
          ? _value.featureFlags
          : featureFlags // ignore: cast_nullable_to_non_nullable
              as Map<String, bool>,
      metadata: null == metadata
          ? _value.metadata
          : metadata // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$AppStateImplCopyWith<$Res>
    implements $AppStateCopyWith<$Res> {
  factory _$$AppStateImplCopyWith(
          _$AppStateImpl value, $Res Function(_$AppStateImpl) then) =
      __$$AppStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {bool initialized,
      bool loading,
      String environment,
      String locale,
      bool isDarkMode,
      Map<String, bool> featureFlags,
      Map<String, dynamic> metadata});
}

/// @nodoc
class __$$AppStateImplCopyWithImpl<$Res>
    extends _$AppStateCopyWithImpl<$Res, _$AppStateImpl>
    implements _$$AppStateImplCopyWith<$Res> {
  __$$AppStateImplCopyWithImpl(
      _$AppStateImpl _value, $Res Function(_$AppStateImpl) _then)
      : super(_value, _then);

  /// Create a copy of AppState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? initialized = null,
    Object? loading = null,
    Object? environment = null,
    Object? locale = null,
    Object? isDarkMode = null,
    Object? featureFlags = null,
    Object? metadata = null,
  }) {
    return _then(_$AppStateImpl(
      initialized: null == initialized
          ? _value.initialized
          : initialized // ignore: cast_nullable_to_non_nullable
              as bool,
      loading: null == loading
          ? _value.loading
          : loading // ignore: cast_nullable_to_non_nullable
              as bool,
      environment: null == environment
          ? _value.environment
          : environment // ignore: cast_nullable_to_non_nullable
              as String,
      locale: null == locale
          ? _value.locale
          : locale // ignore: cast_nullable_to_non_nullable
              as String,
      isDarkMode: null == isDarkMode
          ? _value.isDarkMode
          : isDarkMode // ignore: cast_nullable_to_non_nullable
              as bool,
      featureFlags: null == featureFlags
          ? _value._featureFlags
          : featureFlags // ignore: cast_nullable_to_non_nullable
              as Map<String, bool>,
      metadata: null == metadata
          ? _value._metadata
          : metadata // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$AppStateImpl implements _AppState {
  const _$AppStateImpl(
      {this.initialized = false,
      this.loading = false,
      this.environment = 'development',
      this.locale = 'en',
      this.isDarkMode = false,
      final Map<String, bool> featureFlags = const {},
      final Map<String, dynamic> metadata = const {}})
      : _featureFlags = featureFlags,
        _metadata = metadata;

  factory _$AppStateImpl.fromJson(Map<String, dynamic> json) =>
      _$$AppStateImplFromJson(json);

  /// Whether the app has been initialized.
  @override
  @JsonKey()
  final bool initialized;

  /// Whether the app is in a loading state.
  @override
  @JsonKey()
  final bool loading;

  /// Current environment (e.g., 'development', 'staging', 'production').
  @override
  @JsonKey()
  final String environment;

  /// Current locale code (e.g., 'en', 'ja').
  @override
  @JsonKey()
  final String locale;

  /// Whether dark mode is enabled.
  @override
  @JsonKey()
  final bool isDarkMode;

  /// Feature flags.
  final Map<String, bool> _featureFlags;

  /// Feature flags.
  @override
  @JsonKey()
  Map<String, bool> get featureFlags {
    if (_featureFlags is EqualUnmodifiableMapView) return _featureFlags;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_featureFlags);
  }

  /// Custom metadata.
  final Map<String, dynamic> _metadata;

  /// Custom metadata.
  @override
  @JsonKey()
  Map<String, dynamic> get metadata {
    if (_metadata is EqualUnmodifiableMapView) return _metadata;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_metadata);
  }

  @override
  String toString() {
    return 'AppState(initialized: $initialized, loading: $loading, environment: $environment, locale: $locale, isDarkMode: $isDarkMode, featureFlags: $featureFlags, metadata: $metadata)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AppStateImpl &&
            (identical(other.initialized, initialized) ||
                other.initialized == initialized) &&
            (identical(other.loading, loading) || other.loading == loading) &&
            (identical(other.environment, environment) ||
                other.environment == environment) &&
            (identical(other.locale, locale) || other.locale == locale) &&
            (identical(other.isDarkMode, isDarkMode) ||
                other.isDarkMode == isDarkMode) &&
            const DeepCollectionEquality()
                .equals(other._featureFlags, _featureFlags) &&
            const DeepCollectionEquality().equals(other._metadata, _metadata));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      initialized,
      loading,
      environment,
      locale,
      isDarkMode,
      const DeepCollectionEquality().hash(_featureFlags),
      const DeepCollectionEquality().hash(_metadata));

  /// Create a copy of AppState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AppStateImplCopyWith<_$AppStateImpl> get copyWith =>
      __$$AppStateImplCopyWithImpl<_$AppStateImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$AppStateImplToJson(
      this,
    );
  }
}

abstract class _AppState implements AppState {
  const factory _AppState(
      {final bool initialized,
      final bool loading,
      final String environment,
      final String locale,
      final bool isDarkMode,
      final Map<String, bool> featureFlags,
      final Map<String, dynamic> metadata}) = _$AppStateImpl;

  factory _AppState.fromJson(Map<String, dynamic> json) =
      _$AppStateImpl.fromJson;

  /// Whether the app has been initialized.
  @override
  bool get initialized;

  /// Whether the app is in a loading state.
  @override
  bool get loading;

  /// Current environment (e.g., 'development', 'staging', 'production').
  @override
  String get environment;

  /// Current locale code (e.g., 'en', 'ja').
  @override
  String get locale;

  /// Whether dark mode is enabled.
  @override
  bool get isDarkMode;

  /// Feature flags.
  @override
  Map<String, bool> get featureFlags;

  /// Custom metadata.
  @override
  Map<String, dynamic> get metadata;

  /// Create a copy of AppState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AppStateImplCopyWith<_$AppStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

UserPreferences _$UserPreferencesFromJson(Map<String, dynamic> json) {
  return _UserPreferences.fromJson(json);
}

/// @nodoc
mixin _$UserPreferences {
  /// Preferred theme mode ('light', 'dark', 'system').
  String get themeMode => throw _privateConstructorUsedError;

  /// Preferred locale.
  String? get preferredLocale => throw _privateConstructorUsedError;

  /// Notification preferences.
  bool get notificationsEnabled => throw _privateConstructorUsedError;

  /// Analytics consent.
  bool get analyticsConsent => throw _privateConstructorUsedError;

  /// Custom preferences.
  Map<String, dynamic> get custom => throw _privateConstructorUsedError;

  /// Serializes this UserPreferences to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of UserPreferences
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $UserPreferencesCopyWith<UserPreferences> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UserPreferencesCopyWith<$Res> {
  factory $UserPreferencesCopyWith(
          UserPreferences value, $Res Function(UserPreferences) then) =
      _$UserPreferencesCopyWithImpl<$Res, UserPreferences>;
  @useResult
  $Res call(
      {String themeMode,
      String? preferredLocale,
      bool notificationsEnabled,
      bool analyticsConsent,
      Map<String, dynamic> custom});
}

/// @nodoc
class _$UserPreferencesCopyWithImpl<$Res, $Val extends UserPreferences>
    implements $UserPreferencesCopyWith<$Res> {
  _$UserPreferencesCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UserPreferences
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? themeMode = null,
    Object? preferredLocale = freezed,
    Object? notificationsEnabled = null,
    Object? analyticsConsent = null,
    Object? custom = null,
  }) {
    return _then(_value.copyWith(
      themeMode: null == themeMode
          ? _value.themeMode
          : themeMode // ignore: cast_nullable_to_non_nullable
              as String,
      preferredLocale: freezed == preferredLocale
          ? _value.preferredLocale
          : preferredLocale // ignore: cast_nullable_to_non_nullable
              as String?,
      notificationsEnabled: null == notificationsEnabled
          ? _value.notificationsEnabled
          : notificationsEnabled // ignore: cast_nullable_to_non_nullable
              as bool,
      analyticsConsent: null == analyticsConsent
          ? _value.analyticsConsent
          : analyticsConsent // ignore: cast_nullable_to_non_nullable
              as bool,
      custom: null == custom
          ? _value.custom
          : custom // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$UserPreferencesImplCopyWith<$Res>
    implements $UserPreferencesCopyWith<$Res> {
  factory _$$UserPreferencesImplCopyWith(_$UserPreferencesImpl value,
          $Res Function(_$UserPreferencesImpl) then) =
      __$$UserPreferencesImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String themeMode,
      String? preferredLocale,
      bool notificationsEnabled,
      bool analyticsConsent,
      Map<String, dynamic> custom});
}

/// @nodoc
class __$$UserPreferencesImplCopyWithImpl<$Res>
    extends _$UserPreferencesCopyWithImpl<$Res, _$UserPreferencesImpl>
    implements _$$UserPreferencesImplCopyWith<$Res> {
  __$$UserPreferencesImplCopyWithImpl(
      _$UserPreferencesImpl _value, $Res Function(_$UserPreferencesImpl) _then)
      : super(_value, _then);

  /// Create a copy of UserPreferences
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? themeMode = null,
    Object? preferredLocale = freezed,
    Object? notificationsEnabled = null,
    Object? analyticsConsent = null,
    Object? custom = null,
  }) {
    return _then(_$UserPreferencesImpl(
      themeMode: null == themeMode
          ? _value.themeMode
          : themeMode // ignore: cast_nullable_to_non_nullable
              as String,
      preferredLocale: freezed == preferredLocale
          ? _value.preferredLocale
          : preferredLocale // ignore: cast_nullable_to_non_nullable
              as String?,
      notificationsEnabled: null == notificationsEnabled
          ? _value.notificationsEnabled
          : notificationsEnabled // ignore: cast_nullable_to_non_nullable
              as bool,
      analyticsConsent: null == analyticsConsent
          ? _value.analyticsConsent
          : analyticsConsent // ignore: cast_nullable_to_non_nullable
              as bool,
      custom: null == custom
          ? _value._custom
          : custom // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UserPreferencesImpl implements _UserPreferences {
  const _$UserPreferencesImpl(
      {this.themeMode = 'system',
      this.preferredLocale,
      this.notificationsEnabled = true,
      this.analyticsConsent = false,
      final Map<String, dynamic> custom = const {}})
      : _custom = custom;

  factory _$UserPreferencesImpl.fromJson(Map<String, dynamic> json) =>
      _$$UserPreferencesImplFromJson(json);

  /// Preferred theme mode ('light', 'dark', 'system').
  @override
  @JsonKey()
  final String themeMode;

  /// Preferred locale.
  @override
  final String? preferredLocale;

  /// Notification preferences.
  @override
  @JsonKey()
  final bool notificationsEnabled;

  /// Analytics consent.
  @override
  @JsonKey()
  final bool analyticsConsent;

  /// Custom preferences.
  final Map<String, dynamic> _custom;

  /// Custom preferences.
  @override
  @JsonKey()
  Map<String, dynamic> get custom {
    if (_custom is EqualUnmodifiableMapView) return _custom;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_custom);
  }

  @override
  String toString() {
    return 'UserPreferences(themeMode: $themeMode, preferredLocale: $preferredLocale, notificationsEnabled: $notificationsEnabled, analyticsConsent: $analyticsConsent, custom: $custom)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserPreferencesImpl &&
            (identical(other.themeMode, themeMode) ||
                other.themeMode == themeMode) &&
            (identical(other.preferredLocale, preferredLocale) ||
                other.preferredLocale == preferredLocale) &&
            (identical(other.notificationsEnabled, notificationsEnabled) ||
                other.notificationsEnabled == notificationsEnabled) &&
            (identical(other.analyticsConsent, analyticsConsent) ||
                other.analyticsConsent == analyticsConsent) &&
            const DeepCollectionEquality().equals(other._custom, _custom));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      themeMode,
      preferredLocale,
      notificationsEnabled,
      analyticsConsent,
      const DeepCollectionEquality().hash(_custom));

  /// Create a copy of UserPreferences
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserPreferencesImplCopyWith<_$UserPreferencesImpl> get copyWith =>
      __$$UserPreferencesImplCopyWithImpl<_$UserPreferencesImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$UserPreferencesImplToJson(
      this,
    );
  }
}

abstract class _UserPreferences implements UserPreferences {
  const factory _UserPreferences(
      {final String themeMode,
      final String? preferredLocale,
      final bool notificationsEnabled,
      final bool analyticsConsent,
      final Map<String, dynamic> custom}) = _$UserPreferencesImpl;

  factory _UserPreferences.fromJson(Map<String, dynamic> json) =
      _$UserPreferencesImpl.fromJson;

  /// Preferred theme mode ('light', 'dark', 'system').
  @override
  String get themeMode;

  /// Preferred locale.
  @override
  String? get preferredLocale;

  /// Notification preferences.
  @override
  bool get notificationsEnabled;

  /// Analytics consent.
  @override
  bool get analyticsConsent;

  /// Custom preferences.
  @override
  Map<String, dynamic> get custom;

  /// Create a copy of UserPreferences
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserPreferencesImplCopyWith<_$UserPreferencesImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

NavigationState _$NavigationStateFromJson(Map<String, dynamic> json) {
  return _NavigationState.fromJson(json);
}

/// @nodoc
mixin _$NavigationState {
  /// Current route path.
  String get currentPath => throw _privateConstructorUsedError;

  /// Previous route path.
  String? get previousPath => throw _privateConstructorUsedError;

  /// Route parameters.
  Map<String, String> get params => throw _privateConstructorUsedError;

  /// Query parameters.
  Map<String, String> get queryParams => throw _privateConstructorUsedError;

  /// Navigation history stack.
  List<String> get history => throw _privateConstructorUsedError;

  /// Serializes this NavigationState to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of NavigationState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $NavigationStateCopyWith<NavigationState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $NavigationStateCopyWith<$Res> {
  factory $NavigationStateCopyWith(
          NavigationState value, $Res Function(NavigationState) then) =
      _$NavigationStateCopyWithImpl<$Res, NavigationState>;
  @useResult
  $Res call(
      {String currentPath,
      String? previousPath,
      Map<String, String> params,
      Map<String, String> queryParams,
      List<String> history});
}

/// @nodoc
class _$NavigationStateCopyWithImpl<$Res, $Val extends NavigationState>
    implements $NavigationStateCopyWith<$Res> {
  _$NavigationStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of NavigationState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? currentPath = null,
    Object? previousPath = freezed,
    Object? params = null,
    Object? queryParams = null,
    Object? history = null,
  }) {
    return _then(_value.copyWith(
      currentPath: null == currentPath
          ? _value.currentPath
          : currentPath // ignore: cast_nullable_to_non_nullable
              as String,
      previousPath: freezed == previousPath
          ? _value.previousPath
          : previousPath // ignore: cast_nullable_to_non_nullable
              as String?,
      params: null == params
          ? _value.params
          : params // ignore: cast_nullable_to_non_nullable
              as Map<String, String>,
      queryParams: null == queryParams
          ? _value.queryParams
          : queryParams // ignore: cast_nullable_to_non_nullable
              as Map<String, String>,
      history: null == history
          ? _value.history
          : history // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$NavigationStateImplCopyWith<$Res>
    implements $NavigationStateCopyWith<$Res> {
  factory _$$NavigationStateImplCopyWith(_$NavigationStateImpl value,
          $Res Function(_$NavigationStateImpl) then) =
      __$$NavigationStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String currentPath,
      String? previousPath,
      Map<String, String> params,
      Map<String, String> queryParams,
      List<String> history});
}

/// @nodoc
class __$$NavigationStateImplCopyWithImpl<$Res>
    extends _$NavigationStateCopyWithImpl<$Res, _$NavigationStateImpl>
    implements _$$NavigationStateImplCopyWith<$Res> {
  __$$NavigationStateImplCopyWithImpl(
      _$NavigationStateImpl _value, $Res Function(_$NavigationStateImpl) _then)
      : super(_value, _then);

  /// Create a copy of NavigationState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? currentPath = null,
    Object? previousPath = freezed,
    Object? params = null,
    Object? queryParams = null,
    Object? history = null,
  }) {
    return _then(_$NavigationStateImpl(
      currentPath: null == currentPath
          ? _value.currentPath
          : currentPath // ignore: cast_nullable_to_non_nullable
              as String,
      previousPath: freezed == previousPath
          ? _value.previousPath
          : previousPath // ignore: cast_nullable_to_non_nullable
              as String?,
      params: null == params
          ? _value._params
          : params // ignore: cast_nullable_to_non_nullable
              as Map<String, String>,
      queryParams: null == queryParams
          ? _value._queryParams
          : queryParams // ignore: cast_nullable_to_non_nullable
              as Map<String, String>,
      history: null == history
          ? _value._history
          : history // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$NavigationStateImpl implements _NavigationState {
  const _$NavigationStateImpl(
      {this.currentPath = '/',
      this.previousPath,
      final Map<String, String> params = const {},
      final Map<String, String> queryParams = const {},
      final List<String> history = const []})
      : _params = params,
        _queryParams = queryParams,
        _history = history;

  factory _$NavigationStateImpl.fromJson(Map<String, dynamic> json) =>
      _$$NavigationStateImplFromJson(json);

  /// Current route path.
  @override
  @JsonKey()
  final String currentPath;

  /// Previous route path.
  @override
  final String? previousPath;

  /// Route parameters.
  final Map<String, String> _params;

  /// Route parameters.
  @override
  @JsonKey()
  Map<String, String> get params {
    if (_params is EqualUnmodifiableMapView) return _params;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_params);
  }

  /// Query parameters.
  final Map<String, String> _queryParams;

  /// Query parameters.
  @override
  @JsonKey()
  Map<String, String> get queryParams {
    if (_queryParams is EqualUnmodifiableMapView) return _queryParams;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_queryParams);
  }

  /// Navigation history stack.
  final List<String> _history;

  /// Navigation history stack.
  @override
  @JsonKey()
  List<String> get history {
    if (_history is EqualUnmodifiableListView) return _history;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_history);
  }

  @override
  String toString() {
    return 'NavigationState(currentPath: $currentPath, previousPath: $previousPath, params: $params, queryParams: $queryParams, history: $history)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NavigationStateImpl &&
            (identical(other.currentPath, currentPath) ||
                other.currentPath == currentPath) &&
            (identical(other.previousPath, previousPath) ||
                other.previousPath == previousPath) &&
            const DeepCollectionEquality().equals(other._params, _params) &&
            const DeepCollectionEquality()
                .equals(other._queryParams, _queryParams) &&
            const DeepCollectionEquality().equals(other._history, _history));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      currentPath,
      previousPath,
      const DeepCollectionEquality().hash(_params),
      const DeepCollectionEquality().hash(_queryParams),
      const DeepCollectionEquality().hash(_history));

  /// Create a copy of NavigationState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$NavigationStateImplCopyWith<_$NavigationStateImpl> get copyWith =>
      __$$NavigationStateImplCopyWithImpl<_$NavigationStateImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$NavigationStateImplToJson(
      this,
    );
  }
}

abstract class _NavigationState implements NavigationState {
  const factory _NavigationState(
      {final String currentPath,
      final String? previousPath,
      final Map<String, String> params,
      final Map<String, String> queryParams,
      final List<String> history}) = _$NavigationStateImpl;

  factory _NavigationState.fromJson(Map<String, dynamic> json) =
      _$NavigationStateImpl.fromJson;

  /// Current route path.
  @override
  String get currentPath;

  /// Previous route path.
  @override
  String? get previousPath;

  /// Route parameters.
  @override
  Map<String, String> get params;

  /// Query parameters.
  @override
  Map<String, String> get queryParams;

  /// Navigation history stack.
  @override
  List<String> get history;

  /// Create a copy of NavigationState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$NavigationStateImplCopyWith<_$NavigationStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

ConnectivityState _$ConnectivityStateFromJson(Map<String, dynamic> json) {
  return _ConnectivityState.fromJson(json);
}

/// @nodoc
mixin _$ConnectivityState {
  /// Whether the device is connected to the internet.
  bool get isConnected => throw _privateConstructorUsedError;

  /// Connection type ('wifi', 'mobile', 'none').
  String get connectionType => throw _privateConstructorUsedError;

  /// Last time connectivity was checked.
  DateTime? get lastChecked => throw _privateConstructorUsedError;

  /// Serializes this ConnectivityState to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ConnectivityState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ConnectivityStateCopyWith<ConnectivityState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ConnectivityStateCopyWith<$Res> {
  factory $ConnectivityStateCopyWith(
          ConnectivityState value, $Res Function(ConnectivityState) then) =
      _$ConnectivityStateCopyWithImpl<$Res, ConnectivityState>;
  @useResult
  $Res call({bool isConnected, String connectionType, DateTime? lastChecked});
}

/// @nodoc
class _$ConnectivityStateCopyWithImpl<$Res, $Val extends ConnectivityState>
    implements $ConnectivityStateCopyWith<$Res> {
  _$ConnectivityStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ConnectivityState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? isConnected = null,
    Object? connectionType = null,
    Object? lastChecked = freezed,
  }) {
    return _then(_value.copyWith(
      isConnected: null == isConnected
          ? _value.isConnected
          : isConnected // ignore: cast_nullable_to_non_nullable
              as bool,
      connectionType: null == connectionType
          ? _value.connectionType
          : connectionType // ignore: cast_nullable_to_non_nullable
              as String,
      lastChecked: freezed == lastChecked
          ? _value.lastChecked
          : lastChecked // ignore: cast_nullable_to_non_nullable
              as DateTime?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ConnectivityStateImplCopyWith<$Res>
    implements $ConnectivityStateCopyWith<$Res> {
  factory _$$ConnectivityStateImplCopyWith(_$ConnectivityStateImpl value,
          $Res Function(_$ConnectivityStateImpl) then) =
      __$$ConnectivityStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({bool isConnected, String connectionType, DateTime? lastChecked});
}

/// @nodoc
class __$$ConnectivityStateImplCopyWithImpl<$Res>
    extends _$ConnectivityStateCopyWithImpl<$Res, _$ConnectivityStateImpl>
    implements _$$ConnectivityStateImplCopyWith<$Res> {
  __$$ConnectivityStateImplCopyWithImpl(_$ConnectivityStateImpl _value,
      $Res Function(_$ConnectivityStateImpl) _then)
      : super(_value, _then);

  /// Create a copy of ConnectivityState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? isConnected = null,
    Object? connectionType = null,
    Object? lastChecked = freezed,
  }) {
    return _then(_$ConnectivityStateImpl(
      isConnected: null == isConnected
          ? _value.isConnected
          : isConnected // ignore: cast_nullable_to_non_nullable
              as bool,
      connectionType: null == connectionType
          ? _value.connectionType
          : connectionType // ignore: cast_nullable_to_non_nullable
              as String,
      lastChecked: freezed == lastChecked
          ? _value.lastChecked
          : lastChecked // ignore: cast_nullable_to_non_nullable
              as DateTime?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ConnectivityStateImpl implements _ConnectivityState {
  const _$ConnectivityStateImpl(
      {this.isConnected = true,
      this.connectionType = 'unknown',
      this.lastChecked});

  factory _$ConnectivityStateImpl.fromJson(Map<String, dynamic> json) =>
      _$$ConnectivityStateImplFromJson(json);

  /// Whether the device is connected to the internet.
  @override
  @JsonKey()
  final bool isConnected;

  /// Connection type ('wifi', 'mobile', 'none').
  @override
  @JsonKey()
  final String connectionType;

  /// Last time connectivity was checked.
  @override
  final DateTime? lastChecked;

  @override
  String toString() {
    return 'ConnectivityState(isConnected: $isConnected, connectionType: $connectionType, lastChecked: $lastChecked)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ConnectivityStateImpl &&
            (identical(other.isConnected, isConnected) ||
                other.isConnected == isConnected) &&
            (identical(other.connectionType, connectionType) ||
                other.connectionType == connectionType) &&
            (identical(other.lastChecked, lastChecked) ||
                other.lastChecked == lastChecked));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, isConnected, connectionType, lastChecked);

  /// Create a copy of ConnectivityState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ConnectivityStateImplCopyWith<_$ConnectivityStateImpl> get copyWith =>
      __$$ConnectivityStateImplCopyWithImpl<_$ConnectivityStateImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ConnectivityStateImplToJson(
      this,
    );
  }
}

abstract class _ConnectivityState implements ConnectivityState {
  const factory _ConnectivityState(
      {final bool isConnected,
      final String connectionType,
      final DateTime? lastChecked}) = _$ConnectivityStateImpl;

  factory _ConnectivityState.fromJson(Map<String, dynamic> json) =
      _$ConnectivityStateImpl.fromJson;

  /// Whether the device is connected to the internet.
  @override
  bool get isConnected;

  /// Connection type ('wifi', 'mobile', 'none').
  @override
  String get connectionType;

  /// Last time connectivity was checked.
  @override
  DateTime? get lastChecked;

  /// Create a copy of ConnectivityState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ConnectivityStateImplCopyWith<_$ConnectivityStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
