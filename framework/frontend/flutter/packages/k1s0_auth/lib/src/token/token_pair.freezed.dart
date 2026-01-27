// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'token_pair.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

TokenPair _$TokenPairFromJson(Map<String, dynamic> json) {
  return _TokenPair.fromJson(json);
}

/// @nodoc
mixin _$TokenPair {
  /// Access token (JWT)
  @JsonKey(name: 'access_token')
  String get accessToken => throw _privateConstructorUsedError;

  /// Refresh token
  @JsonKey(name: 'refresh_token')
  String? get refreshToken => throw _privateConstructorUsedError;

  /// ID token (OIDC)
  @JsonKey(name: 'id_token')
  String? get idToken => throw _privateConstructorUsedError;

  /// Access token expiration time (Unix timestamp in milliseconds)
  @JsonKey(name: 'expires_at')
  int? get expiresAt => throw _privateConstructorUsedError;

  /// Token type (usually "Bearer")
  @JsonKey(name: 'token_type')
  String get tokenType => throw _privateConstructorUsedError;

  /// Scopes
  String? get scope => throw _privateConstructorUsedError;

  /// Serializes this TokenPair to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of TokenPair
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $TokenPairCopyWith<TokenPair> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $TokenPairCopyWith<$Res> {
  factory $TokenPairCopyWith(TokenPair value, $Res Function(TokenPair) then) =
      _$TokenPairCopyWithImpl<$Res, TokenPair>;
  @useResult
  $Res call(
      {@JsonKey(name: 'access_token') String accessToken,
      @JsonKey(name: 'refresh_token') String? refreshToken,
      @JsonKey(name: 'id_token') String? idToken,
      @JsonKey(name: 'expires_at') int? expiresAt,
      @JsonKey(name: 'token_type') String tokenType,
      String? scope});
}

/// @nodoc
class _$TokenPairCopyWithImpl<$Res, $Val extends TokenPair>
    implements $TokenPairCopyWith<$Res> {
  _$TokenPairCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of TokenPair
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? accessToken = null,
    Object? refreshToken = freezed,
    Object? idToken = freezed,
    Object? expiresAt = freezed,
    Object? tokenType = null,
    Object? scope = freezed,
  }) {
    return _then(_value.copyWith(
      accessToken: null == accessToken
          ? _value.accessToken
          : accessToken // ignore: cast_nullable_to_non_nullable
              as String,
      refreshToken: freezed == refreshToken
          ? _value.refreshToken
          : refreshToken // ignore: cast_nullable_to_non_nullable
              as String?,
      idToken: freezed == idToken
          ? _value.idToken
          : idToken // ignore: cast_nullable_to_non_nullable
              as String?,
      expiresAt: freezed == expiresAt
          ? _value.expiresAt
          : expiresAt // ignore: cast_nullable_to_non_nullable
              as int?,
      tokenType: null == tokenType
          ? _value.tokenType
          : tokenType // ignore: cast_nullable_to_non_nullable
              as String,
      scope: freezed == scope
          ? _value.scope
          : scope // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$TokenPairImplCopyWith<$Res>
    implements $TokenPairCopyWith<$Res> {
  factory _$$TokenPairImplCopyWith(
          _$TokenPairImpl value, $Res Function(_$TokenPairImpl) then) =
      __$$TokenPairImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'access_token') String accessToken,
      @JsonKey(name: 'refresh_token') String? refreshToken,
      @JsonKey(name: 'id_token') String? idToken,
      @JsonKey(name: 'expires_at') int? expiresAt,
      @JsonKey(name: 'token_type') String tokenType,
      String? scope});
}

/// @nodoc
class __$$TokenPairImplCopyWithImpl<$Res>
    extends _$TokenPairCopyWithImpl<$Res, _$TokenPairImpl>
    implements _$$TokenPairImplCopyWith<$Res> {
  __$$TokenPairImplCopyWithImpl(
      _$TokenPairImpl _value, $Res Function(_$TokenPairImpl) _then)
      : super(_value, _then);

  /// Create a copy of TokenPair
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? accessToken = null,
    Object? refreshToken = freezed,
    Object? idToken = freezed,
    Object? expiresAt = freezed,
    Object? tokenType = null,
    Object? scope = freezed,
  }) {
    return _then(_$TokenPairImpl(
      accessToken: null == accessToken
          ? _value.accessToken
          : accessToken // ignore: cast_nullable_to_non_nullable
              as String,
      refreshToken: freezed == refreshToken
          ? _value.refreshToken
          : refreshToken // ignore: cast_nullable_to_non_nullable
              as String?,
      idToken: freezed == idToken
          ? _value.idToken
          : idToken // ignore: cast_nullable_to_non_nullable
              as String?,
      expiresAt: freezed == expiresAt
          ? _value.expiresAt
          : expiresAt // ignore: cast_nullable_to_non_nullable
              as int?,
      tokenType: null == tokenType
          ? _value.tokenType
          : tokenType // ignore: cast_nullable_to_non_nullable
              as String,
      scope: freezed == scope
          ? _value.scope
          : scope // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$TokenPairImpl extends _TokenPair {
  const _$TokenPairImpl(
      {@JsonKey(name: 'access_token') required this.accessToken,
      @JsonKey(name: 'refresh_token') this.refreshToken,
      @JsonKey(name: 'id_token') this.idToken,
      @JsonKey(name: 'expires_at') this.expiresAt,
      @JsonKey(name: 'token_type') this.tokenType = 'Bearer',
      this.scope})
      : super._();

  factory _$TokenPairImpl.fromJson(Map<String, dynamic> json) =>
      _$$TokenPairImplFromJson(json);

  /// Access token (JWT)
  @override
  @JsonKey(name: 'access_token')
  final String accessToken;

  /// Refresh token
  @override
  @JsonKey(name: 'refresh_token')
  final String? refreshToken;

  /// ID token (OIDC)
  @override
  @JsonKey(name: 'id_token')
  final String? idToken;

  /// Access token expiration time (Unix timestamp in milliseconds)
  @override
  @JsonKey(name: 'expires_at')
  final int? expiresAt;

  /// Token type (usually "Bearer")
  @override
  @JsonKey(name: 'token_type')
  final String tokenType;

  /// Scopes
  @override
  final String? scope;

  @override
  String toString() {
    return 'TokenPair(accessToken: $accessToken, refreshToken: $refreshToken, idToken: $idToken, expiresAt: $expiresAt, tokenType: $tokenType, scope: $scope)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$TokenPairImpl &&
            (identical(other.accessToken, accessToken) ||
                other.accessToken == accessToken) &&
            (identical(other.refreshToken, refreshToken) ||
                other.refreshToken == refreshToken) &&
            (identical(other.idToken, idToken) || other.idToken == idToken) &&
            (identical(other.expiresAt, expiresAt) ||
                other.expiresAt == expiresAt) &&
            (identical(other.tokenType, tokenType) ||
                other.tokenType == tokenType) &&
            (identical(other.scope, scope) || other.scope == scope));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, accessToken, refreshToken,
      idToken, expiresAt, tokenType, scope);

  /// Create a copy of TokenPair
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$TokenPairImplCopyWith<_$TokenPairImpl> get copyWith =>
      __$$TokenPairImplCopyWithImpl<_$TokenPairImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$TokenPairImplToJson(
      this,
    );
  }
}

abstract class _TokenPair extends TokenPair {
  const factory _TokenPair(
      {@JsonKey(name: 'access_token') required final String accessToken,
      @JsonKey(name: 'refresh_token') final String? refreshToken,
      @JsonKey(name: 'id_token') final String? idToken,
      @JsonKey(name: 'expires_at') final int? expiresAt,
      @JsonKey(name: 'token_type') final String tokenType,
      final String? scope}) = _$TokenPairImpl;
  const _TokenPair._() : super._();

  factory _TokenPair.fromJson(Map<String, dynamic> json) =
      _$TokenPairImpl.fromJson;

  /// Access token (JWT)
  @override
  @JsonKey(name: 'access_token')
  String get accessToken;

  /// Refresh token
  @override
  @JsonKey(name: 'refresh_token')
  String? get refreshToken;

  /// ID token (OIDC)
  @override
  @JsonKey(name: 'id_token')
  String? get idToken;

  /// Access token expiration time (Unix timestamp in milliseconds)
  @override
  @JsonKey(name: 'expires_at')
  int? get expiresAt;

  /// Token type (usually "Bearer")
  @override
  @JsonKey(name: 'token_type')
  String get tokenType;

  /// Scopes
  @override
  String? get scope;

  /// Create a copy of TokenPair
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$TokenPairImplCopyWith<_$TokenPairImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
