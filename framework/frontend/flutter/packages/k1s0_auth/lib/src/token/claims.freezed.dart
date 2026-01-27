// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'claims.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

Claims _$ClaimsFromJson(Map<String, dynamic> json) {
  return _Claims.fromJson(json);
}

/// @nodoc
mixin _$Claims {
  /// Subject (user ID)
  String get sub => throw _privateConstructorUsedError;

  /// Issuer
  String get iss => throw _privateConstructorUsedError;

  /// Audience (may be string or list)
  @_AudienceConverter()
  List<String>? get aud => throw _privateConstructorUsedError;

  /// Expiration time (Unix timestamp in seconds)
  int get exp => throw _privateConstructorUsedError;

  /// Issued at (Unix timestamp in seconds)
  int get iat => throw _privateConstructorUsedError;

  /// Not before (Unix timestamp in seconds)
  int? get nbf => throw _privateConstructorUsedError;

  /// JWT ID
  String? get jti => throw _privateConstructorUsedError;

  /// User roles
  List<String> get roles => throw _privateConstructorUsedError;

  /// User permissions
  List<String> get permissions => throw _privateConstructorUsedError;

  /// Tenant ID
  @JsonKey(name: 'tenant_id')
  String? get tenantId => throw _privateConstructorUsedError;

  /// Scope
  String? get scope => throw _privateConstructorUsedError;

  /// Serializes this Claims to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Claims
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ClaimsCopyWith<Claims> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ClaimsCopyWith<$Res> {
  factory $ClaimsCopyWith(Claims value, $Res Function(Claims) then) =
      _$ClaimsCopyWithImpl<$Res, Claims>;
  @useResult
  $Res call(
      {String sub,
      String iss,
      @_AudienceConverter() List<String>? aud,
      int exp,
      int iat,
      int? nbf,
      String? jti,
      List<String> roles,
      List<String> permissions,
      @JsonKey(name: 'tenant_id') String? tenantId,
      String? scope});
}

/// @nodoc
class _$ClaimsCopyWithImpl<$Res, $Val extends Claims>
    implements $ClaimsCopyWith<$Res> {
  _$ClaimsCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Claims
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sub = null,
    Object? iss = null,
    Object? aud = freezed,
    Object? exp = null,
    Object? iat = null,
    Object? nbf = freezed,
    Object? jti = freezed,
    Object? roles = null,
    Object? permissions = null,
    Object? tenantId = freezed,
    Object? scope = freezed,
  }) {
    return _then(_value.copyWith(
      sub: null == sub
          ? _value.sub
          : sub // ignore: cast_nullable_to_non_nullable
              as String,
      iss: null == iss
          ? _value.iss
          : iss // ignore: cast_nullable_to_non_nullable
              as String,
      aud: freezed == aud
          ? _value.aud
          : aud // ignore: cast_nullable_to_non_nullable
              as List<String>?,
      exp: null == exp
          ? _value.exp
          : exp // ignore: cast_nullable_to_non_nullable
              as int,
      iat: null == iat
          ? _value.iat
          : iat // ignore: cast_nullable_to_non_nullable
              as int,
      nbf: freezed == nbf
          ? _value.nbf
          : nbf // ignore: cast_nullable_to_non_nullable
              as int?,
      jti: freezed == jti
          ? _value.jti
          : jti // ignore: cast_nullable_to_non_nullable
              as String?,
      roles: null == roles
          ? _value.roles
          : roles // ignore: cast_nullable_to_non_nullable
              as List<String>,
      permissions: null == permissions
          ? _value.permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
      tenantId: freezed == tenantId
          ? _value.tenantId
          : tenantId // ignore: cast_nullable_to_non_nullable
              as String?,
      scope: freezed == scope
          ? _value.scope
          : scope // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ClaimsImplCopyWith<$Res> implements $ClaimsCopyWith<$Res> {
  factory _$$ClaimsImplCopyWith(
          _$ClaimsImpl value, $Res Function(_$ClaimsImpl) then) =
      __$$ClaimsImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String sub,
      String iss,
      @_AudienceConverter() List<String>? aud,
      int exp,
      int iat,
      int? nbf,
      String? jti,
      List<String> roles,
      List<String> permissions,
      @JsonKey(name: 'tenant_id') String? tenantId,
      String? scope});
}

/// @nodoc
class __$$ClaimsImplCopyWithImpl<$Res>
    extends _$ClaimsCopyWithImpl<$Res, _$ClaimsImpl>
    implements _$$ClaimsImplCopyWith<$Res> {
  __$$ClaimsImplCopyWithImpl(
      _$ClaimsImpl _value, $Res Function(_$ClaimsImpl) _then)
      : super(_value, _then);

  /// Create a copy of Claims
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sub = null,
    Object? iss = null,
    Object? aud = freezed,
    Object? exp = null,
    Object? iat = null,
    Object? nbf = freezed,
    Object? jti = freezed,
    Object? roles = null,
    Object? permissions = null,
    Object? tenantId = freezed,
    Object? scope = freezed,
  }) {
    return _then(_$ClaimsImpl(
      sub: null == sub
          ? _value.sub
          : sub // ignore: cast_nullable_to_non_nullable
              as String,
      iss: null == iss
          ? _value.iss
          : iss // ignore: cast_nullable_to_non_nullable
              as String,
      aud: freezed == aud
          ? _value._aud
          : aud // ignore: cast_nullable_to_non_nullable
              as List<String>?,
      exp: null == exp
          ? _value.exp
          : exp // ignore: cast_nullable_to_non_nullable
              as int,
      iat: null == iat
          ? _value.iat
          : iat // ignore: cast_nullable_to_non_nullable
              as int,
      nbf: freezed == nbf
          ? _value.nbf
          : nbf // ignore: cast_nullable_to_non_nullable
              as int?,
      jti: freezed == jti
          ? _value.jti
          : jti // ignore: cast_nullable_to_non_nullable
              as String?,
      roles: null == roles
          ? _value._roles
          : roles // ignore: cast_nullable_to_non_nullable
              as List<String>,
      permissions: null == permissions
          ? _value._permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
      tenantId: freezed == tenantId
          ? _value.tenantId
          : tenantId // ignore: cast_nullable_to_non_nullable
              as String?,
      scope: freezed == scope
          ? _value.scope
          : scope // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ClaimsImpl extends _Claims {
  const _$ClaimsImpl(
      {required this.sub,
      required this.iss,
      @_AudienceConverter() final List<String>? aud,
      required this.exp,
      required this.iat,
      this.nbf,
      this.jti,
      final List<String> roles = const [],
      final List<String> permissions = const [],
      @JsonKey(name: 'tenant_id') this.tenantId,
      this.scope})
      : _aud = aud,
        _roles = roles,
        _permissions = permissions,
        super._();

  factory _$ClaimsImpl.fromJson(Map<String, dynamic> json) =>
      _$$ClaimsImplFromJson(json);

  /// Subject (user ID)
  @override
  final String sub;

  /// Issuer
  @override
  final String iss;

  /// Audience (may be string or list)
  final List<String>? _aud;

  /// Audience (may be string or list)
  @override
  @_AudienceConverter()
  List<String>? get aud {
    final value = _aud;
    if (value == null) return null;
    if (_aud is EqualUnmodifiableListView) return _aud;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  /// Expiration time (Unix timestamp in seconds)
  @override
  final int exp;

  /// Issued at (Unix timestamp in seconds)
  @override
  final int iat;

  /// Not before (Unix timestamp in seconds)
  @override
  final int? nbf;

  /// JWT ID
  @override
  final String? jti;

  /// User roles
  final List<String> _roles;

  /// User roles
  @override
  @JsonKey()
  List<String> get roles {
    if (_roles is EqualUnmodifiableListView) return _roles;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_roles);
  }

  /// User permissions
  final List<String> _permissions;

  /// User permissions
  @override
  @JsonKey()
  List<String> get permissions {
    if (_permissions is EqualUnmodifiableListView) return _permissions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_permissions);
  }

  /// Tenant ID
  @override
  @JsonKey(name: 'tenant_id')
  final String? tenantId;

  /// Scope
  @override
  final String? scope;

  @override
  String toString() {
    return 'Claims(sub: $sub, iss: $iss, aud: $aud, exp: $exp, iat: $iat, nbf: $nbf, jti: $jti, roles: $roles, permissions: $permissions, tenantId: $tenantId, scope: $scope)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ClaimsImpl &&
            (identical(other.sub, sub) || other.sub == sub) &&
            (identical(other.iss, iss) || other.iss == iss) &&
            const DeepCollectionEquality().equals(other._aud, _aud) &&
            (identical(other.exp, exp) || other.exp == exp) &&
            (identical(other.iat, iat) || other.iat == iat) &&
            (identical(other.nbf, nbf) || other.nbf == nbf) &&
            (identical(other.jti, jti) || other.jti == jti) &&
            const DeepCollectionEquality().equals(other._roles, _roles) &&
            const DeepCollectionEquality()
                .equals(other._permissions, _permissions) &&
            (identical(other.tenantId, tenantId) ||
                other.tenantId == tenantId) &&
            (identical(other.scope, scope) || other.scope == scope));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      sub,
      iss,
      const DeepCollectionEquality().hash(_aud),
      exp,
      iat,
      nbf,
      jti,
      const DeepCollectionEquality().hash(_roles),
      const DeepCollectionEquality().hash(_permissions),
      tenantId,
      scope);

  /// Create a copy of Claims
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ClaimsImplCopyWith<_$ClaimsImpl> get copyWith =>
      __$$ClaimsImplCopyWithImpl<_$ClaimsImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ClaimsImplToJson(
      this,
    );
  }
}

abstract class _Claims extends Claims {
  const factory _Claims(
      {required final String sub,
      required final String iss,
      @_AudienceConverter() final List<String>? aud,
      required final int exp,
      required final int iat,
      final int? nbf,
      final String? jti,
      final List<String> roles,
      final List<String> permissions,
      @JsonKey(name: 'tenant_id') final String? tenantId,
      final String? scope}) = _$ClaimsImpl;
  const _Claims._() : super._();

  factory _Claims.fromJson(Map<String, dynamic> json) = _$ClaimsImpl.fromJson;

  /// Subject (user ID)
  @override
  String get sub;

  /// Issuer
  @override
  String get iss;

  /// Audience (may be string or list)
  @override
  @_AudienceConverter()
  List<String>? get aud;

  /// Expiration time (Unix timestamp in seconds)
  @override
  int get exp;

  /// Issued at (Unix timestamp in seconds)
  @override
  int get iat;

  /// Not before (Unix timestamp in seconds)
  @override
  int? get nbf;

  /// JWT ID
  @override
  String? get jti;

  /// User roles
  @override
  List<String> get roles;

  /// User permissions
  @override
  List<String> get permissions;

  /// Tenant ID
  @override
  @JsonKey(name: 'tenant_id')
  String? get tenantId;

  /// Scope
  @override
  String? get scope;

  /// Create a copy of Claims
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ClaimsImplCopyWith<_$ClaimsImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
