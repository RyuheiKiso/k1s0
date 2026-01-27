// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'auth_config.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

AuthConfig _$AuthConfigFromJson(Map<String, dynamic> json) {
  return _AuthConfig.fromJson(json);
}

/// @nodoc
mixin _$AuthConfig {
  /// Whether authentication is enabled
  bool get enabled => throw _privateConstructorUsedError;

  /// Token storage type
  TokenStorageType get storageType => throw _privateConstructorUsedError;

  /// Time before expiration to trigger refresh (in seconds)
  int get refreshMarginSeconds => throw _privateConstructorUsedError;

  /// Whether to automatically refresh tokens
  bool get autoRefresh => throw _privateConstructorUsedError;

  /// Allowed issuers for token validation
  List<String>? get allowedIssuers => throw _privateConstructorUsedError;

  /// Allowed audiences for token validation
  List<String>? get allowedAudiences => throw _privateConstructorUsedError;

  /// OIDC configuration
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
      TokenStorageType storageType,
      int refreshMarginSeconds,
      bool autoRefresh,
      List<String>? allowedIssuers,
      List<String>? allowedAudiences,
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
    Object? storageType = null,
    Object? refreshMarginSeconds = null,
    Object? autoRefresh = null,
    Object? allowedIssuers = freezed,
    Object? allowedAudiences = freezed,
    Object? oidc = freezed,
  }) {
    return _then(_value.copyWith(
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      storageType: null == storageType
          ? _value.storageType
          : storageType // ignore: cast_nullable_to_non_nullable
              as TokenStorageType,
      refreshMarginSeconds: null == refreshMarginSeconds
          ? _value.refreshMarginSeconds
          : refreshMarginSeconds // ignore: cast_nullable_to_non_nullable
              as int,
      autoRefresh: null == autoRefresh
          ? _value.autoRefresh
          : autoRefresh // ignore: cast_nullable_to_non_nullable
              as bool,
      allowedIssuers: freezed == allowedIssuers
          ? _value.allowedIssuers
          : allowedIssuers // ignore: cast_nullable_to_non_nullable
              as List<String>?,
      allowedAudiences: freezed == allowedAudiences
          ? _value.allowedAudiences
          : allowedAudiences // ignore: cast_nullable_to_non_nullable
              as List<String>?,
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
      TokenStorageType storageType,
      int refreshMarginSeconds,
      bool autoRefresh,
      List<String>? allowedIssuers,
      List<String>? allowedAudiences,
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
    Object? storageType = null,
    Object? refreshMarginSeconds = null,
    Object? autoRefresh = null,
    Object? allowedIssuers = freezed,
    Object? allowedAudiences = freezed,
    Object? oidc = freezed,
  }) {
    return _then(_$AuthConfigImpl(
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      storageType: null == storageType
          ? _value.storageType
          : storageType // ignore: cast_nullable_to_non_nullable
              as TokenStorageType,
      refreshMarginSeconds: null == refreshMarginSeconds
          ? _value.refreshMarginSeconds
          : refreshMarginSeconds // ignore: cast_nullable_to_non_nullable
              as int,
      autoRefresh: null == autoRefresh
          ? _value.autoRefresh
          : autoRefresh // ignore: cast_nullable_to_non_nullable
              as bool,
      allowedIssuers: freezed == allowedIssuers
          ? _value._allowedIssuers
          : allowedIssuers // ignore: cast_nullable_to_non_nullable
              as List<String>?,
      allowedAudiences: freezed == allowedAudiences
          ? _value._allowedAudiences
          : allowedAudiences // ignore: cast_nullable_to_non_nullable
              as List<String>?,
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
      this.storageType = TokenStorageType.secure,
      this.refreshMarginSeconds = 300,
      this.autoRefresh = true,
      final List<String>? allowedIssuers,
      final List<String>? allowedAudiences,
      this.oidc})
      : _allowedIssuers = allowedIssuers,
        _allowedAudiences = allowedAudiences;

  factory _$AuthConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$AuthConfigImplFromJson(json);

  /// Whether authentication is enabled
  @override
  @JsonKey()
  final bool enabled;

  /// Token storage type
  @override
  @JsonKey()
  final TokenStorageType storageType;

  /// Time before expiration to trigger refresh (in seconds)
  @override
  @JsonKey()
  final int refreshMarginSeconds;

  /// Whether to automatically refresh tokens
  @override
  @JsonKey()
  final bool autoRefresh;

  /// Allowed issuers for token validation
  final List<String>? _allowedIssuers;

  /// Allowed issuers for token validation
  @override
  List<String>? get allowedIssuers {
    final value = _allowedIssuers;
    if (value == null) return null;
    if (_allowedIssuers is EqualUnmodifiableListView) return _allowedIssuers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  /// Allowed audiences for token validation
  final List<String>? _allowedAudiences;

  /// Allowed audiences for token validation
  @override
  List<String>? get allowedAudiences {
    final value = _allowedAudiences;
    if (value == null) return null;
    if (_allowedAudiences is EqualUnmodifiableListView)
      return _allowedAudiences;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  /// OIDC configuration
  @override
  final OidcConfig? oidc;

  @override
  String toString() {
    return 'AuthConfig(enabled: $enabled, storageType: $storageType, refreshMarginSeconds: $refreshMarginSeconds, autoRefresh: $autoRefresh, allowedIssuers: $allowedIssuers, allowedAudiences: $allowedAudiences, oidc: $oidc)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AuthConfigImpl &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            (identical(other.storageType, storageType) ||
                other.storageType == storageType) &&
            (identical(other.refreshMarginSeconds, refreshMarginSeconds) ||
                other.refreshMarginSeconds == refreshMarginSeconds) &&
            (identical(other.autoRefresh, autoRefresh) ||
                other.autoRefresh == autoRefresh) &&
            const DeepCollectionEquality()
                .equals(other._allowedIssuers, _allowedIssuers) &&
            const DeepCollectionEquality()
                .equals(other._allowedAudiences, _allowedAudiences) &&
            (identical(other.oidc, oidc) || other.oidc == oidc));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      enabled,
      storageType,
      refreshMarginSeconds,
      autoRefresh,
      const DeepCollectionEquality().hash(_allowedIssuers),
      const DeepCollectionEquality().hash(_allowedAudiences),
      oidc);

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
      final TokenStorageType storageType,
      final int refreshMarginSeconds,
      final bool autoRefresh,
      final List<String>? allowedIssuers,
      final List<String>? allowedAudiences,
      final OidcConfig? oidc}) = _$AuthConfigImpl;

  factory _AuthConfig.fromJson(Map<String, dynamic> json) =
      _$AuthConfigImpl.fromJson;

  /// Whether authentication is enabled
  @override
  bool get enabled;

  /// Token storage type
  @override
  TokenStorageType get storageType;

  /// Time before expiration to trigger refresh (in seconds)
  @override
  int get refreshMarginSeconds;

  /// Whether to automatically refresh tokens
  @override
  bool get autoRefresh;

  /// Allowed issuers for token validation
  @override
  List<String>? get allowedIssuers;

  /// Allowed audiences for token validation
  @override
  List<String>? get allowedAudiences;

  /// OIDC configuration
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

  /// Discovery URL (usually issuer + /.well-known/openid-configuration)
  String? get discoveryUrl => throw _privateConstructorUsedError;

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
      String? postLogoutRedirectUri,
      String? discoveryUrl});
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
    Object? discoveryUrl = freezed,
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
      discoveryUrl: freezed == discoveryUrl
          ? _value.discoveryUrl
          : discoveryUrl // ignore: cast_nullable_to_non_nullable
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
      String? postLogoutRedirectUri,
      String? discoveryUrl});
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
    Object? discoveryUrl = freezed,
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
      discoveryUrl: freezed == discoveryUrl
          ? _value.discoveryUrl
          : discoveryUrl // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$OidcConfigImpl extends _OidcConfig {
  const _$OidcConfigImpl(
      {required this.issuer,
      required this.clientId,
      required this.redirectUri,
      final List<String> scopes = const ['openid', 'profile', 'email'],
      this.postLogoutRedirectUri,
      this.discoveryUrl})
      : _scopes = scopes,
        super._();

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

  /// Discovery URL (usually issuer + /.well-known/openid-configuration)
  @override
  final String? discoveryUrl;

  @override
  String toString() {
    return 'OidcConfig(issuer: $issuer, clientId: $clientId, redirectUri: $redirectUri, scopes: $scopes, postLogoutRedirectUri: $postLogoutRedirectUri, discoveryUrl: $discoveryUrl)';
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
                other.postLogoutRedirectUri == postLogoutRedirectUri) &&
            (identical(other.discoveryUrl, discoveryUrl) ||
                other.discoveryUrl == discoveryUrl));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      issuer,
      clientId,
      redirectUri,
      const DeepCollectionEquality().hash(_scopes),
      postLogoutRedirectUri,
      discoveryUrl);

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

abstract class _OidcConfig extends OidcConfig {
  const factory _OidcConfig(
      {required final String issuer,
      required final String clientId,
      required final String redirectUri,
      final List<String> scopes,
      final String? postLogoutRedirectUri,
      final String? discoveryUrl}) = _$OidcConfigImpl;
  const _OidcConfig._() : super._();

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

  /// Discovery URL (usually issuer + /.well-known/openid-configuration)
  @override
  String? get discoveryUrl;

  /// Create a copy of OidcConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$OidcConfigImplCopyWith<_$OidcConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
