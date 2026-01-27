// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'config_types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ApiConfig _$ApiConfigFromJson(Map<String, dynamic> json) {
  return _ApiConfig.fromJson(json);
}

/// @nodoc
mixin _$ApiConfig {
  /// Base URL for API requests
  String get baseUrl => throw _privateConstructorUsedError;

  /// Request timeout in milliseconds
  int get timeout => throw _privateConstructorUsedError;

  /// Number of retry attempts
  int get retryCount => throw _privateConstructorUsedError;

  /// Delay between retries in milliseconds
  int get retryDelay => throw _privateConstructorUsedError;

  /// Serializes this ApiConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApiConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApiConfigCopyWith<ApiConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApiConfigCopyWith<$Res> {
  factory $ApiConfigCopyWith(ApiConfig value, $Res Function(ApiConfig) then) =
      _$ApiConfigCopyWithImpl<$Res, ApiConfig>;
  @useResult
  $Res call({String baseUrl, int timeout, int retryCount, int retryDelay});
}

/// @nodoc
class _$ApiConfigCopyWithImpl<$Res, $Val extends ApiConfig>
    implements $ApiConfigCopyWith<$Res> {
  _$ApiConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApiConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? baseUrl = null,
    Object? timeout = null,
    Object? retryCount = null,
    Object? retryDelay = null,
  }) {
    return _then(_value.copyWith(
      baseUrl: null == baseUrl
          ? _value.baseUrl
          : baseUrl // ignore: cast_nullable_to_non_nullable
              as String,
      timeout: null == timeout
          ? _value.timeout
          : timeout // ignore: cast_nullable_to_non_nullable
              as int,
      retryCount: null == retryCount
          ? _value.retryCount
          : retryCount // ignore: cast_nullable_to_non_nullable
              as int,
      retryDelay: null == retryDelay
          ? _value.retryDelay
          : retryDelay // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApiConfigImplCopyWith<$Res>
    implements $ApiConfigCopyWith<$Res> {
  factory _$$ApiConfigImplCopyWith(
          _$ApiConfigImpl value, $Res Function(_$ApiConfigImpl) then) =
      __$$ApiConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String baseUrl, int timeout, int retryCount, int retryDelay});
}

/// @nodoc
class __$$ApiConfigImplCopyWithImpl<$Res>
    extends _$ApiConfigCopyWithImpl<$Res, _$ApiConfigImpl>
    implements _$$ApiConfigImplCopyWith<$Res> {
  __$$ApiConfigImplCopyWithImpl(
      _$ApiConfigImpl _value, $Res Function(_$ApiConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApiConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? baseUrl = null,
    Object? timeout = null,
    Object? retryCount = null,
    Object? retryDelay = null,
  }) {
    return _then(_$ApiConfigImpl(
      baseUrl: null == baseUrl
          ? _value.baseUrl
          : baseUrl // ignore: cast_nullable_to_non_nullable
              as String,
      timeout: null == timeout
          ? _value.timeout
          : timeout // ignore: cast_nullable_to_non_nullable
              as int,
      retryCount: null == retryCount
          ? _value.retryCount
          : retryCount // ignore: cast_nullable_to_non_nullable
              as int,
      retryDelay: null == retryDelay
          ? _value.retryDelay
          : retryDelay // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApiConfigImpl implements _ApiConfig {
  const _$ApiConfigImpl(
      {required this.baseUrl,
      this.timeout = 30000,
      this.retryCount = 3,
      this.retryDelay = 1000});

  factory _$ApiConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApiConfigImplFromJson(json);

  /// Base URL for API requests
  @override
  final String baseUrl;

  /// Request timeout in milliseconds
  @override
  @JsonKey()
  final int timeout;

  /// Number of retry attempts
  @override
  @JsonKey()
  final int retryCount;

  /// Delay between retries in milliseconds
  @override
  @JsonKey()
  final int retryDelay;

  @override
  String toString() {
    return 'ApiConfig(baseUrl: $baseUrl, timeout: $timeout, retryCount: $retryCount, retryDelay: $retryDelay)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApiConfigImpl &&
            (identical(other.baseUrl, baseUrl) || other.baseUrl == baseUrl) &&
            (identical(other.timeout, timeout) || other.timeout == timeout) &&
            (identical(other.retryCount, retryCount) ||
                other.retryCount == retryCount) &&
            (identical(other.retryDelay, retryDelay) ||
                other.retryDelay == retryDelay));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, baseUrl, timeout, retryCount, retryDelay);

  /// Create a copy of ApiConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApiConfigImplCopyWith<_$ApiConfigImpl> get copyWith =>
      __$$ApiConfigImplCopyWithImpl<_$ApiConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApiConfigImplToJson(
      this,
    );
  }
}

abstract class _ApiConfig implements ApiConfig {
  const factory _ApiConfig(
      {required final String baseUrl,
      final int timeout,
      final int retryCount,
      final int retryDelay}) = _$ApiConfigImpl;

  factory _ApiConfig.fromJson(Map<String, dynamic> json) =
      _$ApiConfigImpl.fromJson;

  /// Base URL for API requests
  @override
  String get baseUrl;

  /// Request timeout in milliseconds
  @override
  int get timeout;

  /// Number of retry attempts
  @override
  int get retryCount;

  /// Delay between retries in milliseconds
  @override
  int get retryDelay;

  /// Create a copy of ApiConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApiConfigImplCopyWith<_$ApiConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

AuthConfig _$AuthConfigFromJson(Map<String, dynamic> json) {
  return _AuthConfig.fromJson(json);
}

/// @nodoc
mixin _$AuthConfig {
  /// Whether authentication is enabled
  bool get enabled => throw _privateConstructorUsedError;

  /// Authentication provider type
  String get provider => throw _privateConstructorUsedError;

  /// Token refresh threshold in seconds
  int get tokenRefreshThreshold => throw _privateConstructorUsedError;

  /// Token storage type
  String get storage => throw _privateConstructorUsedError;

  /// OIDC configuration (optional)
  OidcConfig? get oidc => throw _privateConstructorUsedError;

  /// Serializes this AuthConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AuthConfigCopyWith<AuthConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AuthConfigCopyWith<$Res> {
  factory $AuthConfigCopyWith(
          AuthConfig value, $Res Function(AuthConfig) then) =
      _$AuthConfigCopyWithImpl<$Res, AuthConfig>;
  @useResult
  $Res call(
      {bool enabled,
      String provider,
      int tokenRefreshThreshold,
      String storage,
      OidcConfig? oidc});

  $OidcConfigCopyWith<$Res>? get oidc;
}

/// @nodoc
class _$AuthConfigCopyWithImpl<$Res, $Val extends AuthConfig>
    implements $AuthConfigCopyWith<$Res> {
  _$AuthConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = null,
    Object? provider = null,
    Object? tokenRefreshThreshold = null,
    Object? storage = null,
    Object? oidc = freezed,
  }) {
    return _then(_value.copyWith(
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      provider: null == provider
          ? _value.provider
          : provider // ignore: cast_nullable_to_non_nullable
              as String,
      tokenRefreshThreshold: null == tokenRefreshThreshold
          ? _value.tokenRefreshThreshold
          : tokenRefreshThreshold // ignore: cast_nullable_to_non_nullable
              as int,
      storage: null == storage
          ? _value.storage
          : storage // ignore: cast_nullable_to_non_nullable
              as String,
      oidc: freezed == oidc
          ? _value.oidc
          : oidc // ignore: cast_nullable_to_non_nullable
              as OidcConfig?,
    ) as $Val);
  }

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $OidcConfigCopyWith<$Res>? get oidc {
    if (_value.oidc == null) {
      return null;
    }

    return $OidcConfigCopyWith<$Res>(_value.oidc!, (value) {
      return _then(_value.copyWith(oidc: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$AuthConfigImplCopyWith<$Res>
    implements $AuthConfigCopyWith<$Res> {
  factory _$$AuthConfigImplCopyWith(
          _$AuthConfigImpl value, $Res Function(_$AuthConfigImpl) then) =
      __$$AuthConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {bool enabled,
      String provider,
      int tokenRefreshThreshold,
      String storage,
      OidcConfig? oidc});

  @override
  $OidcConfigCopyWith<$Res>? get oidc;
}

/// @nodoc
class __$$AuthConfigImplCopyWithImpl<$Res>
    extends _$AuthConfigCopyWithImpl<$Res, _$AuthConfigImpl>
    implements _$$AuthConfigImplCopyWith<$Res> {
  __$$AuthConfigImplCopyWithImpl(
      _$AuthConfigImpl _value, $Res Function(_$AuthConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = null,
    Object? provider = null,
    Object? tokenRefreshThreshold = null,
    Object? storage = null,
    Object? oidc = freezed,
  }) {
    return _then(_$AuthConfigImpl(
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      provider: null == provider
          ? _value.provider
          : provider // ignore: cast_nullable_to_non_nullable
              as String,
      tokenRefreshThreshold: null == tokenRefreshThreshold
          ? _value.tokenRefreshThreshold
          : tokenRefreshThreshold // ignore: cast_nullable_to_non_nullable
              as int,
      storage: null == storage
          ? _value.storage
          : storage // ignore: cast_nullable_to_non_nullable
              as String,
      oidc: freezed == oidc
          ? _value.oidc
          : oidc // ignore: cast_nullable_to_non_nullable
              as OidcConfig?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$AuthConfigImpl implements _AuthConfig {
  const _$AuthConfigImpl(
      {this.enabled = true,
      this.provider = 'jwt',
      this.tokenRefreshThreshold = 300,
      this.storage = 'secure',
      this.oidc});

  factory _$AuthConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$AuthConfigImplFromJson(json);

  /// Whether authentication is enabled
  @override
  @JsonKey()
  final bool enabled;

  /// Authentication provider type
  @override
  @JsonKey()
  final String provider;

  /// Token refresh threshold in seconds
  @override
  @JsonKey()
  final int tokenRefreshThreshold;

  /// Token storage type
  @override
  @JsonKey()
  final String storage;

  /// OIDC configuration (optional)
  @override
  final OidcConfig? oidc;

  @override
  String toString() {
    return 'AuthConfig(enabled: $enabled, provider: $provider, tokenRefreshThreshold: $tokenRefreshThreshold, storage: $storage, oidc: $oidc)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AuthConfigImpl &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            (identical(other.provider, provider) ||
                other.provider == provider) &&
            (identical(other.tokenRefreshThreshold, tokenRefreshThreshold) ||
                other.tokenRefreshThreshold == tokenRefreshThreshold) &&
            (identical(other.storage, storage) || other.storage == storage) &&
            (identical(other.oidc, oidc) || other.oidc == oidc));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, enabled, provider, tokenRefreshThreshold, storage, oidc);

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AuthConfigImplCopyWith<_$AuthConfigImpl> get copyWith =>
      __$$AuthConfigImplCopyWithImpl<_$AuthConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$AuthConfigImplToJson(
      this,
    );
  }
}

abstract class _AuthConfig implements AuthConfig {
  const factory _AuthConfig(
      {final bool enabled,
      final String provider,
      final int tokenRefreshThreshold,
      final String storage,
      final OidcConfig? oidc}) = _$AuthConfigImpl;

  factory _AuthConfig.fromJson(Map<String, dynamic> json) =
      _$AuthConfigImpl.fromJson;

  /// Whether authentication is enabled
  @override
  bool get enabled;

  /// Authentication provider type
  @override
  String get provider;

  /// Token refresh threshold in seconds
  @override
  int get tokenRefreshThreshold;

  /// Token storage type
  @override
  String get storage;

  /// OIDC configuration (optional)
  @override
  OidcConfig? get oidc;

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AuthConfigImplCopyWith<_$AuthConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

OidcConfig _$OidcConfigFromJson(Map<String, dynamic> json) {
  return _OidcConfig.fromJson(json);
}

/// @nodoc
mixin _$OidcConfig {
  /// Issuer URL
  String get issuer => throw _privateConstructorUsedError;

  /// Client ID
  String get clientId => throw _privateConstructorUsedError;

  /// Redirect URI
  String get redirectUri => throw _privateConstructorUsedError;

  /// Scopes
  List<String> get scopes => throw _privateConstructorUsedError;

  /// Post-logout redirect URI
  String? get postLogoutRedirectUri => throw _privateConstructorUsedError;

  /// Serializes this OidcConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of OidcConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $OidcConfigCopyWith<OidcConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $OidcConfigCopyWith<$Res> {
  factory $OidcConfigCopyWith(
          OidcConfig value, $Res Function(OidcConfig) then) =
      _$OidcConfigCopyWithImpl<$Res, OidcConfig>;
  @useResult
  $Res call(
      {String issuer,
      String clientId,
      String redirectUri,
      List<String> scopes,
      String? postLogoutRedirectUri});
}

/// @nodoc
class _$OidcConfigCopyWithImpl<$Res, $Val extends OidcConfig>
    implements $OidcConfigCopyWith<$Res> {
  _$OidcConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of OidcConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? issuer = null,
    Object? clientId = null,
    Object? redirectUri = null,
    Object? scopes = null,
    Object? postLogoutRedirectUri = freezed,
  }) {
    return _then(_value.copyWith(
      issuer: null == issuer
          ? _value.issuer
          : issuer // ignore: cast_nullable_to_non_nullable
              as String,
      clientId: null == clientId
          ? _value.clientId
          : clientId // ignore: cast_nullable_to_non_nullable
              as String,
      redirectUri: null == redirectUri
          ? _value.redirectUri
          : redirectUri // ignore: cast_nullable_to_non_nullable
              as String,
      scopes: null == scopes
          ? _value.scopes
          : scopes // ignore: cast_nullable_to_non_nullable
              as List<String>,
      postLogoutRedirectUri: freezed == postLogoutRedirectUri
          ? _value.postLogoutRedirectUri
          : postLogoutRedirectUri // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$OidcConfigImplCopyWith<$Res>
    implements $OidcConfigCopyWith<$Res> {
  factory _$$OidcConfigImplCopyWith(
          _$OidcConfigImpl value, $Res Function(_$OidcConfigImpl) then) =
      __$$OidcConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String issuer,
      String clientId,
      String redirectUri,
      List<String> scopes,
      String? postLogoutRedirectUri});
}

/// @nodoc
class __$$OidcConfigImplCopyWithImpl<$Res>
    extends _$OidcConfigCopyWithImpl<$Res, _$OidcConfigImpl>
    implements _$$OidcConfigImplCopyWith<$Res> {
  __$$OidcConfigImplCopyWithImpl(
      _$OidcConfigImpl _value, $Res Function(_$OidcConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of OidcConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? issuer = null,
    Object? clientId = null,
    Object? redirectUri = null,
    Object? scopes = null,
    Object? postLogoutRedirectUri = freezed,
  }) {
    return _then(_$OidcConfigImpl(
      issuer: null == issuer
          ? _value.issuer
          : issuer // ignore: cast_nullable_to_non_nullable
              as String,
      clientId: null == clientId
          ? _value.clientId
          : clientId // ignore: cast_nullable_to_non_nullable
              as String,
      redirectUri: null == redirectUri
          ? _value.redirectUri
          : redirectUri // ignore: cast_nullable_to_non_nullable
              as String,
      scopes: null == scopes
          ? _value._scopes
          : scopes // ignore: cast_nullable_to_non_nullable
              as List<String>,
      postLogoutRedirectUri: freezed == postLogoutRedirectUri
          ? _value.postLogoutRedirectUri
          : postLogoutRedirectUri // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$OidcConfigImpl implements _OidcConfig {
  const _$OidcConfigImpl(
      {required this.issuer,
      required this.clientId,
      required this.redirectUri,
      final List<String> scopes = const ['openid', 'profile', 'email'],
      this.postLogoutRedirectUri})
      : _scopes = scopes;

  factory _$OidcConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$OidcConfigImplFromJson(json);

  /// Issuer URL
  @override
  final String issuer;

  /// Client ID
  @override
  final String clientId;

  /// Redirect URI
  @override
  final String redirectUri;

  /// Scopes
  final List<String> _scopes;

  /// Scopes
  @override
  @JsonKey()
  List<String> get scopes {
    if (_scopes is EqualUnmodifiableListView) return _scopes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_scopes);
  }

  /// Post-logout redirect URI
  @override
  final String? postLogoutRedirectUri;

  @override
  String toString() {
    return 'OidcConfig(issuer: $issuer, clientId: $clientId, redirectUri: $redirectUri, scopes: $scopes, postLogoutRedirectUri: $postLogoutRedirectUri)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$OidcConfigImpl &&
            (identical(other.issuer, issuer) || other.issuer == issuer) &&
            (identical(other.clientId, clientId) ||
                other.clientId == clientId) &&
            (identical(other.redirectUri, redirectUri) ||
                other.redirectUri == redirectUri) &&
            const DeepCollectionEquality().equals(other._scopes, _scopes) &&
            (identical(other.postLogoutRedirectUri, postLogoutRedirectUri) ||
                other.postLogoutRedirectUri == postLogoutRedirectUri));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, issuer, clientId, redirectUri,
      const DeepCollectionEquality().hash(_scopes), postLogoutRedirectUri);

  /// Create a copy of OidcConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$OidcConfigImplCopyWith<_$OidcConfigImpl> get copyWith =>
      __$$OidcConfigImplCopyWithImpl<_$OidcConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$OidcConfigImplToJson(
      this,
    );
  }
}

abstract class _OidcConfig implements OidcConfig {
  const factory _OidcConfig(
      {required final String issuer,
      required final String clientId,
      required final String redirectUri,
      final List<String> scopes,
      final String? postLogoutRedirectUri}) = _$OidcConfigImpl;

  factory _OidcConfig.fromJson(Map<String, dynamic> json) =
      _$OidcConfigImpl.fromJson;

  /// Issuer URL
  @override
  String get issuer;

  /// Client ID
  @override
  String get clientId;

  /// Redirect URI
  @override
  String get redirectUri;

  /// Scopes
  @override
  List<String> get scopes;

  /// Post-logout redirect URI
  @override
  String? get postLogoutRedirectUri;

  /// Create a copy of OidcConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$OidcConfigImplCopyWith<_$OidcConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

LoggingConfig _$LoggingConfigFromJson(Map<String, dynamic> json) {
  return _LoggingConfig.fromJson(json);
}

/// @nodoc
mixin _$LoggingConfig {
  /// Log level
  String get level => throw _privateConstructorUsedError;

  /// Whether console logging is enabled
  bool get enableConsole => throw _privateConstructorUsedError;

  /// Whether remote logging is enabled
  bool get enableRemote => throw _privateConstructorUsedError;

  /// Remote logging endpoint
  String? get remoteEndpoint => throw _privateConstructorUsedError;

  /// Serializes this LoggingConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of LoggingConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $LoggingConfigCopyWith<LoggingConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $LoggingConfigCopyWith<$Res> {
  factory $LoggingConfigCopyWith(
          LoggingConfig value, $Res Function(LoggingConfig) then) =
      _$LoggingConfigCopyWithImpl<$Res, LoggingConfig>;
  @useResult
  $Res call(
      {String level,
      bool enableConsole,
      bool enableRemote,
      String? remoteEndpoint});
}

/// @nodoc
class _$LoggingConfigCopyWithImpl<$Res, $Val extends LoggingConfig>
    implements $LoggingConfigCopyWith<$Res> {
  _$LoggingConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of LoggingConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? level = null,
    Object? enableConsole = null,
    Object? enableRemote = null,
    Object? remoteEndpoint = freezed,
  }) {
    return _then(_value.copyWith(
      level: null == level
          ? _value.level
          : level // ignore: cast_nullable_to_non_nullable
              as String,
      enableConsole: null == enableConsole
          ? _value.enableConsole
          : enableConsole // ignore: cast_nullable_to_non_nullable
              as bool,
      enableRemote: null == enableRemote
          ? _value.enableRemote
          : enableRemote // ignore: cast_nullable_to_non_nullable
              as bool,
      remoteEndpoint: freezed == remoteEndpoint
          ? _value.remoteEndpoint
          : remoteEndpoint // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$LoggingConfigImplCopyWith<$Res>
    implements $LoggingConfigCopyWith<$Res> {
  factory _$$LoggingConfigImplCopyWith(
          _$LoggingConfigImpl value, $Res Function(_$LoggingConfigImpl) then) =
      __$$LoggingConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String level,
      bool enableConsole,
      bool enableRemote,
      String? remoteEndpoint});
}

/// @nodoc
class __$$LoggingConfigImplCopyWithImpl<$Res>
    extends _$LoggingConfigCopyWithImpl<$Res, _$LoggingConfigImpl>
    implements _$$LoggingConfigImplCopyWith<$Res> {
  __$$LoggingConfigImplCopyWithImpl(
      _$LoggingConfigImpl _value, $Res Function(_$LoggingConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of LoggingConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? level = null,
    Object? enableConsole = null,
    Object? enableRemote = null,
    Object? remoteEndpoint = freezed,
  }) {
    return _then(_$LoggingConfigImpl(
      level: null == level
          ? _value.level
          : level // ignore: cast_nullable_to_non_nullable
              as String,
      enableConsole: null == enableConsole
          ? _value.enableConsole
          : enableConsole // ignore: cast_nullable_to_non_nullable
              as bool,
      enableRemote: null == enableRemote
          ? _value.enableRemote
          : enableRemote // ignore: cast_nullable_to_non_nullable
              as bool,
      remoteEndpoint: freezed == remoteEndpoint
          ? _value.remoteEndpoint
          : remoteEndpoint // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$LoggingConfigImpl implements _LoggingConfig {
  const _$LoggingConfigImpl(
      {this.level = 'info',
      this.enableConsole = true,
      this.enableRemote = false,
      this.remoteEndpoint});

  factory _$LoggingConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$LoggingConfigImplFromJson(json);

  /// Log level
  @override
  @JsonKey()
  final String level;

  /// Whether console logging is enabled
  @override
  @JsonKey()
  final bool enableConsole;

  /// Whether remote logging is enabled
  @override
  @JsonKey()
  final bool enableRemote;

  /// Remote logging endpoint
  @override
  final String? remoteEndpoint;

  @override
  String toString() {
    return 'LoggingConfig(level: $level, enableConsole: $enableConsole, enableRemote: $enableRemote, remoteEndpoint: $remoteEndpoint)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$LoggingConfigImpl &&
            (identical(other.level, level) || other.level == level) &&
            (identical(other.enableConsole, enableConsole) ||
                other.enableConsole == enableConsole) &&
            (identical(other.enableRemote, enableRemote) ||
                other.enableRemote == enableRemote) &&
            (identical(other.remoteEndpoint, remoteEndpoint) ||
                other.remoteEndpoint == remoteEndpoint));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, level, enableConsole, enableRemote, remoteEndpoint);

  /// Create a copy of LoggingConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$LoggingConfigImplCopyWith<_$LoggingConfigImpl> get copyWith =>
      __$$LoggingConfigImplCopyWithImpl<_$LoggingConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$LoggingConfigImplToJson(
      this,
    );
  }
}

abstract class _LoggingConfig implements LoggingConfig {
  const factory _LoggingConfig(
      {final String level,
      final bool enableConsole,
      final bool enableRemote,
      final String? remoteEndpoint}) = _$LoggingConfigImpl;

  factory _LoggingConfig.fromJson(Map<String, dynamic> json) =
      _$LoggingConfigImpl.fromJson;

  /// Log level
  @override
  String get level;

  /// Whether console logging is enabled
  @override
  bool get enableConsole;

  /// Whether remote logging is enabled
  @override
  bool get enableRemote;

  /// Remote logging endpoint
  @override
  String? get remoteEndpoint;

  /// Create a copy of LoggingConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$LoggingConfigImplCopyWith<_$LoggingConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

TelemetryConfig _$TelemetryConfigFromJson(Map<String, dynamic> json) {
  return _TelemetryConfig.fromJson(json);
}

/// @nodoc
mixin _$TelemetryConfig {
  /// Whether telemetry is enabled
  bool get enabled => throw _privateConstructorUsedError;

  /// Service name
  String get serviceName => throw _privateConstructorUsedError;

  /// OTLP endpoint
  String? get endpoint => throw _privateConstructorUsedError;

  /// Sampling rate (0.0 - 1.0)
  double get sampleRate => throw _privateConstructorUsedError;

  /// Serializes this TelemetryConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of TelemetryConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $TelemetryConfigCopyWith<TelemetryConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $TelemetryConfigCopyWith<$Res> {
  factory $TelemetryConfigCopyWith(
          TelemetryConfig value, $Res Function(TelemetryConfig) then) =
      _$TelemetryConfigCopyWithImpl<$Res, TelemetryConfig>;
  @useResult
  $Res call(
      {bool enabled, String serviceName, String? endpoint, double sampleRate});
}

/// @nodoc
class _$TelemetryConfigCopyWithImpl<$Res, $Val extends TelemetryConfig>
    implements $TelemetryConfigCopyWith<$Res> {
  _$TelemetryConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of TelemetryConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = null,
    Object? serviceName = null,
    Object? endpoint = freezed,
    Object? sampleRate = null,
  }) {
    return _then(_value.copyWith(
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      serviceName: null == serviceName
          ? _value.serviceName
          : serviceName // ignore: cast_nullable_to_non_nullable
              as String,
      endpoint: freezed == endpoint
          ? _value.endpoint
          : endpoint // ignore: cast_nullable_to_non_nullable
              as String?,
      sampleRate: null == sampleRate
          ? _value.sampleRate
          : sampleRate // ignore: cast_nullable_to_non_nullable
              as double,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$TelemetryConfigImplCopyWith<$Res>
    implements $TelemetryConfigCopyWith<$Res> {
  factory _$$TelemetryConfigImplCopyWith(_$TelemetryConfigImpl value,
          $Res Function(_$TelemetryConfigImpl) then) =
      __$$TelemetryConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {bool enabled, String serviceName, String? endpoint, double sampleRate});
}

/// @nodoc
class __$$TelemetryConfigImplCopyWithImpl<$Res>
    extends _$TelemetryConfigCopyWithImpl<$Res, _$TelemetryConfigImpl>
    implements _$$TelemetryConfigImplCopyWith<$Res> {
  __$$TelemetryConfigImplCopyWithImpl(
      _$TelemetryConfigImpl _value, $Res Function(_$TelemetryConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of TelemetryConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = null,
    Object? serviceName = null,
    Object? endpoint = freezed,
    Object? sampleRate = null,
  }) {
    return _then(_$TelemetryConfigImpl(
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      serviceName: null == serviceName
          ? _value.serviceName
          : serviceName // ignore: cast_nullable_to_non_nullable
              as String,
      endpoint: freezed == endpoint
          ? _value.endpoint
          : endpoint // ignore: cast_nullable_to_non_nullable
              as String?,
      sampleRate: null == sampleRate
          ? _value.sampleRate
          : sampleRate // ignore: cast_nullable_to_non_nullable
              as double,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$TelemetryConfigImpl implements _TelemetryConfig {
  const _$TelemetryConfigImpl(
      {this.enabled = false,
      this.serviceName = 'k1s0-flutter',
      this.endpoint,
      this.sampleRate = 0.1});

  factory _$TelemetryConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$TelemetryConfigImplFromJson(json);

  /// Whether telemetry is enabled
  @override
  @JsonKey()
  final bool enabled;

  /// Service name
  @override
  @JsonKey()
  final String serviceName;

  /// OTLP endpoint
  @override
  final String? endpoint;

  /// Sampling rate (0.0 - 1.0)
  @override
  @JsonKey()
  final double sampleRate;

  @override
  String toString() {
    return 'TelemetryConfig(enabled: $enabled, serviceName: $serviceName, endpoint: $endpoint, sampleRate: $sampleRate)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$TelemetryConfigImpl &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            (identical(other.serviceName, serviceName) ||
                other.serviceName == serviceName) &&
            (identical(other.endpoint, endpoint) ||
                other.endpoint == endpoint) &&
            (identical(other.sampleRate, sampleRate) ||
                other.sampleRate == sampleRate));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, enabled, serviceName, endpoint, sampleRate);

  /// Create a copy of TelemetryConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$TelemetryConfigImplCopyWith<_$TelemetryConfigImpl> get copyWith =>
      __$$TelemetryConfigImplCopyWithImpl<_$TelemetryConfigImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$TelemetryConfigImplToJson(
      this,
    );
  }
}

abstract class _TelemetryConfig implements TelemetryConfig {
  const factory _TelemetryConfig(
      {final bool enabled,
      final String serviceName,
      final String? endpoint,
      final double sampleRate}) = _$TelemetryConfigImpl;

  factory _TelemetryConfig.fromJson(Map<String, dynamic> json) =
      _$TelemetryConfigImpl.fromJson;

  /// Whether telemetry is enabled
  @override
  bool get enabled;

  /// Service name
  @override
  String get serviceName;

  /// OTLP endpoint
  @override
  String? get endpoint;

  /// Sampling rate (0.0 - 1.0)
  @override
  double get sampleRate;

  /// Create a copy of TelemetryConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$TelemetryConfigImplCopyWith<_$TelemetryConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

FeatureFlags _$FeatureFlagsFromJson(Map<String, dynamic> json) {
  return _FeatureFlags.fromJson(json);
}

/// @nodoc
mixin _$FeatureFlags {
  /// Feature flag map
  Map<String, bool> get flags => throw _privateConstructorUsedError;

  /// Serializes this FeatureFlags to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of FeatureFlags
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FeatureFlagsCopyWith<FeatureFlags> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FeatureFlagsCopyWith<$Res> {
  factory $FeatureFlagsCopyWith(
          FeatureFlags value, $Res Function(FeatureFlags) then) =
      _$FeatureFlagsCopyWithImpl<$Res, FeatureFlags>;
  @useResult
  $Res call({Map<String, bool> flags});
}

/// @nodoc
class _$FeatureFlagsCopyWithImpl<$Res, $Val extends FeatureFlags>
    implements $FeatureFlagsCopyWith<$Res> {
  _$FeatureFlagsCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FeatureFlags
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? flags = null,
  }) {
    return _then(_value.copyWith(
      flags: null == flags
          ? _value.flags
          : flags // ignore: cast_nullable_to_non_nullable
              as Map<String, bool>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$FeatureFlagsImplCopyWith<$Res>
    implements $FeatureFlagsCopyWith<$Res> {
  factory _$$FeatureFlagsImplCopyWith(
          _$FeatureFlagsImpl value, $Res Function(_$FeatureFlagsImpl) then) =
      __$$FeatureFlagsImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({Map<String, bool> flags});
}

/// @nodoc
class __$$FeatureFlagsImplCopyWithImpl<$Res>
    extends _$FeatureFlagsCopyWithImpl<$Res, _$FeatureFlagsImpl>
    implements _$$FeatureFlagsImplCopyWith<$Res> {
  __$$FeatureFlagsImplCopyWithImpl(
      _$FeatureFlagsImpl _value, $Res Function(_$FeatureFlagsImpl) _then)
      : super(_value, _then);

  /// Create a copy of FeatureFlags
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? flags = null,
  }) {
    return _then(_$FeatureFlagsImpl(
      flags: null == flags
          ? _value._flags
          : flags // ignore: cast_nullable_to_non_nullable
              as Map<String, bool>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$FeatureFlagsImpl extends _FeatureFlags {
  const _$FeatureFlagsImpl({final Map<String, bool> flags = const {}})
      : _flags = flags,
        super._();

  factory _$FeatureFlagsImpl.fromJson(Map<String, dynamic> json) =>
      _$$FeatureFlagsImplFromJson(json);

  /// Feature flag map
  final Map<String, bool> _flags;

  /// Feature flag map
  @override
  @JsonKey()
  Map<String, bool> get flags {
    if (_flags is EqualUnmodifiableMapView) return _flags;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_flags);
  }

  @override
  String toString() {
    return 'FeatureFlags(flags: $flags)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FeatureFlagsImpl &&
            const DeepCollectionEquality().equals(other._flags, _flags));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_flags));

  /// Create a copy of FeatureFlags
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FeatureFlagsImplCopyWith<_$FeatureFlagsImpl> get copyWith =>
      __$$FeatureFlagsImplCopyWithImpl<_$FeatureFlagsImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$FeatureFlagsImplToJson(
      this,
    );
  }
}

abstract class _FeatureFlags extends FeatureFlags {
  const factory _FeatureFlags({final Map<String, bool> flags}) =
      _$FeatureFlagsImpl;
  const _FeatureFlags._() : super._();

  factory _FeatureFlags.fromJson(Map<String, dynamic> json) =
      _$FeatureFlagsImpl.fromJson;

  /// Feature flag map
  @override
  Map<String, bool> get flags;

  /// Create a copy of FeatureFlags
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FeatureFlagsImplCopyWith<_$FeatureFlagsImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

AppConfig _$AppConfigFromJson(Map<String, dynamic> json) {
  return _AppConfig.fromJson(json);
}

/// @nodoc
mixin _$AppConfig {
  /// Environment
  Environment get env => throw _privateConstructorUsedError;

  /// Application name
  String get appName => throw _privateConstructorUsedError;

  /// Application version
  String? get version => throw _privateConstructorUsedError;

  /// API configuration
  ApiConfig? get api => throw _privateConstructorUsedError;

  /// Authentication configuration
  AuthConfig? get auth => throw _privateConstructorUsedError;

  /// Logging configuration
  LoggingConfig? get logging => throw _privateConstructorUsedError;

  /// Telemetry configuration
  TelemetryConfig? get telemetry => throw _privateConstructorUsedError;

  /// Feature flags
  FeatureFlags? get features => throw _privateConstructorUsedError;

  /// Custom configuration values
  Map<String, dynamic> get custom => throw _privateConstructorUsedError;

  /// Serializes this AppConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AppConfigCopyWith<AppConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AppConfigCopyWith<$Res> {
  factory $AppConfigCopyWith(AppConfig value, $Res Function(AppConfig) then) =
      _$AppConfigCopyWithImpl<$Res, AppConfig>;
  @useResult
  $Res call(
      {Environment env,
      String appName,
      String? version,
      ApiConfig? api,
      AuthConfig? auth,
      LoggingConfig? logging,
      TelemetryConfig? telemetry,
      FeatureFlags? features,
      Map<String, dynamic> custom});

  $ApiConfigCopyWith<$Res>? get api;
  $AuthConfigCopyWith<$Res>? get auth;
  $LoggingConfigCopyWith<$Res>? get logging;
  $TelemetryConfigCopyWith<$Res>? get telemetry;
  $FeatureFlagsCopyWith<$Res>? get features;
}

/// @nodoc
class _$AppConfigCopyWithImpl<$Res, $Val extends AppConfig>
    implements $AppConfigCopyWith<$Res> {
  _$AppConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? env = null,
    Object? appName = null,
    Object? version = freezed,
    Object? api = freezed,
    Object? auth = freezed,
    Object? logging = freezed,
    Object? telemetry = freezed,
    Object? features = freezed,
    Object? custom = null,
  }) {
    return _then(_value.copyWith(
      env: null == env
          ? _value.env
          : env // ignore: cast_nullable_to_non_nullable
              as Environment,
      appName: null == appName
          ? _value.appName
          : appName // ignore: cast_nullable_to_non_nullable
              as String,
      version: freezed == version
          ? _value.version
          : version // ignore: cast_nullable_to_non_nullable
              as String?,
      api: freezed == api
          ? _value.api
          : api // ignore: cast_nullable_to_non_nullable
              as ApiConfig?,
      auth: freezed == auth
          ? _value.auth
          : auth // ignore: cast_nullable_to_non_nullable
              as AuthConfig?,
      logging: freezed == logging
          ? _value.logging
          : logging // ignore: cast_nullable_to_non_nullable
              as LoggingConfig?,
      telemetry: freezed == telemetry
          ? _value.telemetry
          : telemetry // ignore: cast_nullable_to_non_nullable
              as TelemetryConfig?,
      features: freezed == features
          ? _value.features
          : features // ignore: cast_nullable_to_non_nullable
              as FeatureFlags?,
      custom: null == custom
          ? _value.custom
          : custom // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ) as $Val);
  }

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ApiConfigCopyWith<$Res>? get api {
    if (_value.api == null) {
      return null;
    }

    return $ApiConfigCopyWith<$Res>(_value.api!, (value) {
      return _then(_value.copyWith(api: value) as $Val);
    });
  }

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AuthConfigCopyWith<$Res>? get auth {
    if (_value.auth == null) {
      return null;
    }

    return $AuthConfigCopyWith<$Res>(_value.auth!, (value) {
      return _then(_value.copyWith(auth: value) as $Val);
    });
  }

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $LoggingConfigCopyWith<$Res>? get logging {
    if (_value.logging == null) {
      return null;
    }

    return $LoggingConfigCopyWith<$Res>(_value.logging!, (value) {
      return _then(_value.copyWith(logging: value) as $Val);
    });
  }

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $TelemetryConfigCopyWith<$Res>? get telemetry {
    if (_value.telemetry == null) {
      return null;
    }

    return $TelemetryConfigCopyWith<$Res>(_value.telemetry!, (value) {
      return _then(_value.copyWith(telemetry: value) as $Val);
    });
  }

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $FeatureFlagsCopyWith<$Res>? get features {
    if (_value.features == null) {
      return null;
    }

    return $FeatureFlagsCopyWith<$Res>(_value.features!, (value) {
      return _then(_value.copyWith(features: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$AppConfigImplCopyWith<$Res>
    implements $AppConfigCopyWith<$Res> {
  factory _$$AppConfigImplCopyWith(
          _$AppConfigImpl value, $Res Function(_$AppConfigImpl) then) =
      __$$AppConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {Environment env,
      String appName,
      String? version,
      ApiConfig? api,
      AuthConfig? auth,
      LoggingConfig? logging,
      TelemetryConfig? telemetry,
      FeatureFlags? features,
      Map<String, dynamic> custom});

  @override
  $ApiConfigCopyWith<$Res>? get api;
  @override
  $AuthConfigCopyWith<$Res>? get auth;
  @override
  $LoggingConfigCopyWith<$Res>? get logging;
  @override
  $TelemetryConfigCopyWith<$Res>? get telemetry;
  @override
  $FeatureFlagsCopyWith<$Res>? get features;
}

/// @nodoc
class __$$AppConfigImplCopyWithImpl<$Res>
    extends _$AppConfigCopyWithImpl<$Res, _$AppConfigImpl>
    implements _$$AppConfigImplCopyWith<$Res> {
  __$$AppConfigImplCopyWithImpl(
      _$AppConfigImpl _value, $Res Function(_$AppConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? env = null,
    Object? appName = null,
    Object? version = freezed,
    Object? api = freezed,
    Object? auth = freezed,
    Object? logging = freezed,
    Object? telemetry = freezed,
    Object? features = freezed,
    Object? custom = null,
  }) {
    return _then(_$AppConfigImpl(
      env: null == env
          ? _value.env
          : env // ignore: cast_nullable_to_non_nullable
              as Environment,
      appName: null == appName
          ? _value.appName
          : appName // ignore: cast_nullable_to_non_nullable
              as String,
      version: freezed == version
          ? _value.version
          : version // ignore: cast_nullable_to_non_nullable
              as String?,
      api: freezed == api
          ? _value.api
          : api // ignore: cast_nullable_to_non_nullable
              as ApiConfig?,
      auth: freezed == auth
          ? _value.auth
          : auth // ignore: cast_nullable_to_non_nullable
              as AuthConfig?,
      logging: freezed == logging
          ? _value.logging
          : logging // ignore: cast_nullable_to_non_nullable
              as LoggingConfig?,
      telemetry: freezed == telemetry
          ? _value.telemetry
          : telemetry // ignore: cast_nullable_to_non_nullable
              as TelemetryConfig?,
      features: freezed == features
          ? _value.features
          : features // ignore: cast_nullable_to_non_nullable
              as FeatureFlags?,
      custom: null == custom
          ? _value._custom
          : custom // ignore: cast_nullable_to_non_nullable
              as Map<String, dynamic>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$AppConfigImpl extends _AppConfig {
  const _$AppConfigImpl(
      {this.env = Environment.dev,
      this.appName = 'k1s0-app',
      this.version,
      this.api,
      this.auth,
      this.logging,
      this.telemetry,
      this.features,
      final Map<String, dynamic> custom = const {}})
      : _custom = custom,
        super._();

  factory _$AppConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$AppConfigImplFromJson(json);

  /// Environment
  @override
  @JsonKey()
  final Environment env;

  /// Application name
  @override
  @JsonKey()
  final String appName;

  /// Application version
  @override
  final String? version;

  /// API configuration
  @override
  final ApiConfig? api;

  /// Authentication configuration
  @override
  final AuthConfig? auth;

  /// Logging configuration
  @override
  final LoggingConfig? logging;

  /// Telemetry configuration
  @override
  final TelemetryConfig? telemetry;

  /// Feature flags
  @override
  final FeatureFlags? features;

  /// Custom configuration values
  final Map<String, dynamic> _custom;

  /// Custom configuration values
  @override
  @JsonKey()
  Map<String, dynamic> get custom {
    if (_custom is EqualUnmodifiableMapView) return _custom;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_custom);
  }

  @override
  String toString() {
    return 'AppConfig(env: $env, appName: $appName, version: $version, api: $api, auth: $auth, logging: $logging, telemetry: $telemetry, features: $features, custom: $custom)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AppConfigImpl &&
            (identical(other.env, env) || other.env == env) &&
            (identical(other.appName, appName) || other.appName == appName) &&
            (identical(other.version, version) || other.version == version) &&
            (identical(other.api, api) || other.api == api) &&
            (identical(other.auth, auth) || other.auth == auth) &&
            (identical(other.logging, logging) || other.logging == logging) &&
            (identical(other.telemetry, telemetry) ||
                other.telemetry == telemetry) &&
            (identical(other.features, features) ||
                other.features == features) &&
            const DeepCollectionEquality().equals(other._custom, _custom));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      env,
      appName,
      version,
      api,
      auth,
      logging,
      telemetry,
      features,
      const DeepCollectionEquality().hash(_custom));

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AppConfigImplCopyWith<_$AppConfigImpl> get copyWith =>
      __$$AppConfigImplCopyWithImpl<_$AppConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$AppConfigImplToJson(
      this,
    );
  }
}

abstract class _AppConfig extends AppConfig {
  const factory _AppConfig(
      {final Environment env,
      final String appName,
      final String? version,
      final ApiConfig? api,
      final AuthConfig? auth,
      final LoggingConfig? logging,
      final TelemetryConfig? telemetry,
      final FeatureFlags? features,
      final Map<String, dynamic> custom}) = _$AppConfigImpl;
  const _AppConfig._() : super._();

  factory _AppConfig.fromJson(Map<String, dynamic> json) =
      _$AppConfigImpl.fromJson;

  /// Environment
  @override
  Environment get env;

  /// Application name
  @override
  String get appName;

  /// Application version
  @override
  String? get version;

  /// API configuration
  @override
  ApiConfig? get api;

  /// Authentication configuration
  @override
  AuthConfig? get auth;

  /// Logging configuration
  @override
  LoggingConfig? get logging;

  /// Telemetry configuration
  @override
  TelemetryConfig? get telemetry;

  /// Feature flags
  @override
  FeatureFlags? get features;

  /// Custom configuration values
  @override
  Map<String, dynamic> get custom;

  /// Create a copy of AppConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AppConfigImplCopyWith<_$AppConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$ConfigLoadResult {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(AppConfig config) success,
    required TResult Function(
            String message, Object? error, StackTrace? stackTrace)
        failure,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(AppConfig config)? success,
    TResult? Function(String message, Object? error, StackTrace? stackTrace)?
        failure,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(AppConfig config)? success,
    TResult Function(String message, Object? error, StackTrace? stackTrace)?
        failure,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ConfigLoadSuccess value) success,
    required TResult Function(ConfigLoadFailure value) failure,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ConfigLoadSuccess value)? success,
    TResult? Function(ConfigLoadFailure value)? failure,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ConfigLoadSuccess value)? success,
    TResult Function(ConfigLoadFailure value)? failure,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ConfigLoadResultCopyWith<$Res> {
  factory $ConfigLoadResultCopyWith(
          ConfigLoadResult value, $Res Function(ConfigLoadResult) then) =
      _$ConfigLoadResultCopyWithImpl<$Res, ConfigLoadResult>;
}

/// @nodoc
class _$ConfigLoadResultCopyWithImpl<$Res, $Val extends ConfigLoadResult>
    implements $ConfigLoadResultCopyWith<$Res> {
  _$ConfigLoadResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ConfigLoadResult
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$ConfigLoadSuccessImplCopyWith<$Res> {
  factory _$$ConfigLoadSuccessImplCopyWith(_$ConfigLoadSuccessImpl value,
          $Res Function(_$ConfigLoadSuccessImpl) then) =
      __$$ConfigLoadSuccessImplCopyWithImpl<$Res>;
  @useResult
  $Res call({AppConfig config});

  $AppConfigCopyWith<$Res> get config;
}

/// @nodoc
class __$$ConfigLoadSuccessImplCopyWithImpl<$Res>
    extends _$ConfigLoadResultCopyWithImpl<$Res, _$ConfigLoadSuccessImpl>
    implements _$$ConfigLoadSuccessImplCopyWith<$Res> {
  __$$ConfigLoadSuccessImplCopyWithImpl(_$ConfigLoadSuccessImpl _value,
      $Res Function(_$ConfigLoadSuccessImpl) _then)
      : super(_value, _then);

  /// Create a copy of ConfigLoadResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? config = null,
  }) {
    return _then(_$ConfigLoadSuccessImpl(
      config: null == config
          ? _value.config
          : config // ignore: cast_nullable_to_non_nullable
              as AppConfig,
    ));
  }

  /// Create a copy of ConfigLoadResult
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AppConfigCopyWith<$Res> get config {
    return $AppConfigCopyWith<$Res>(_value.config, (value) {
      return _then(_value.copyWith(config: value));
    });
  }
}

/// @nodoc

class _$ConfigLoadSuccessImpl implements ConfigLoadSuccess {
  const _$ConfigLoadSuccessImpl({required this.config});

  @override
  final AppConfig config;

  @override
  String toString() {
    return 'ConfigLoadResult.success(config: $config)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ConfigLoadSuccessImpl &&
            (identical(other.config, config) || other.config == config));
  }

  @override
  int get hashCode => Object.hash(runtimeType, config);

  /// Create a copy of ConfigLoadResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ConfigLoadSuccessImplCopyWith<_$ConfigLoadSuccessImpl> get copyWith =>
      __$$ConfigLoadSuccessImplCopyWithImpl<_$ConfigLoadSuccessImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(AppConfig config) success,
    required TResult Function(
            String message, Object? error, StackTrace? stackTrace)
        failure,
  }) {
    return success(config);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(AppConfig config)? success,
    TResult? Function(String message, Object? error, StackTrace? stackTrace)?
        failure,
  }) {
    return success?.call(config);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(AppConfig config)? success,
    TResult Function(String message, Object? error, StackTrace? stackTrace)?
        failure,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(config);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ConfigLoadSuccess value) success,
    required TResult Function(ConfigLoadFailure value) failure,
  }) {
    return success(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ConfigLoadSuccess value)? success,
    TResult? Function(ConfigLoadFailure value)? failure,
  }) {
    return success?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ConfigLoadSuccess value)? success,
    TResult Function(ConfigLoadFailure value)? failure,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(this);
    }
    return orElse();
  }
}

abstract class ConfigLoadSuccess implements ConfigLoadResult {
  const factory ConfigLoadSuccess({required final AppConfig config}) =
      _$ConfigLoadSuccessImpl;

  AppConfig get config;

  /// Create a copy of ConfigLoadResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ConfigLoadSuccessImplCopyWith<_$ConfigLoadSuccessImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ConfigLoadFailureImplCopyWith<$Res> {
  factory _$$ConfigLoadFailureImplCopyWith(_$ConfigLoadFailureImpl value,
          $Res Function(_$ConfigLoadFailureImpl) then) =
      __$$ConfigLoadFailureImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String message, Object? error, StackTrace? stackTrace});
}

/// @nodoc
class __$$ConfigLoadFailureImplCopyWithImpl<$Res>
    extends _$ConfigLoadResultCopyWithImpl<$Res, _$ConfigLoadFailureImpl>
    implements _$$ConfigLoadFailureImplCopyWith<$Res> {
  __$$ConfigLoadFailureImplCopyWithImpl(_$ConfigLoadFailureImpl _value,
      $Res Function(_$ConfigLoadFailureImpl) _then)
      : super(_value, _then);

  /// Create a copy of ConfigLoadResult
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? message = null,
    Object? error = freezed,
    Object? stackTrace = freezed,
  }) {
    return _then(_$ConfigLoadFailureImpl(
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      error: freezed == error ? _value.error : error,
      stackTrace: freezed == stackTrace
          ? _value.stackTrace
          : stackTrace // ignore: cast_nullable_to_non_nullable
              as StackTrace?,
    ));
  }
}

/// @nodoc

class _$ConfigLoadFailureImpl implements ConfigLoadFailure {
  const _$ConfigLoadFailureImpl(
      {required this.message, this.error, this.stackTrace});

  @override
  final String message;
  @override
  final Object? error;
  @override
  final StackTrace? stackTrace;

  @override
  String toString() {
    return 'ConfigLoadResult.failure(message: $message, error: $error, stackTrace: $stackTrace)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ConfigLoadFailureImpl &&
            (identical(other.message, message) || other.message == message) &&
            const DeepCollectionEquality().equals(other.error, error) &&
            (identical(other.stackTrace, stackTrace) ||
                other.stackTrace == stackTrace));
  }

  @override
  int get hashCode => Object.hash(runtimeType, message,
      const DeepCollectionEquality().hash(error), stackTrace);

  /// Create a copy of ConfigLoadResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ConfigLoadFailureImplCopyWith<_$ConfigLoadFailureImpl> get copyWith =>
      __$$ConfigLoadFailureImplCopyWithImpl<_$ConfigLoadFailureImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(AppConfig config) success,
    required TResult Function(
            String message, Object? error, StackTrace? stackTrace)
        failure,
  }) {
    return failure(message, error, stackTrace);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(AppConfig config)? success,
    TResult? Function(String message, Object? error, StackTrace? stackTrace)?
        failure,
  }) {
    return failure?.call(message, error, stackTrace);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(AppConfig config)? success,
    TResult Function(String message, Object? error, StackTrace? stackTrace)?
        failure,
    required TResult orElse(),
  }) {
    if (failure != null) {
      return failure(message, error, stackTrace);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ConfigLoadSuccess value) success,
    required TResult Function(ConfigLoadFailure value) failure,
  }) {
    return failure(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ConfigLoadSuccess value)? success,
    TResult? Function(ConfigLoadFailure value)? failure,
  }) {
    return failure?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ConfigLoadSuccess value)? success,
    TResult Function(ConfigLoadFailure value)? failure,
    required TResult orElse(),
  }) {
    if (failure != null) {
      return failure(this);
    }
    return orElse();
  }
}

abstract class ConfigLoadFailure implements ConfigLoadResult {
  const factory ConfigLoadFailure(
      {required final String message,
      final Object? error,
      final StackTrace? stackTrace}) = _$ConfigLoadFailureImpl;

  String get message;
  Object? get error;
  StackTrace? get stackTrace;

  /// Create a copy of ConfigLoadResult
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ConfigLoadFailureImplCopyWith<_$ConfigLoadFailureImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
