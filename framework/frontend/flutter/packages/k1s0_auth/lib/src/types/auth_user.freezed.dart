// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'auth_user.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$AuthUser {
  /// User ID
  String get id => throw _privateConstructorUsedError;

  /// User roles
  List<String> get roles => throw _privateConstructorUsedError;

  /// User permissions
  List<String> get permissions => throw _privateConstructorUsedError;

  /// Tenant ID
  String? get tenantId => throw _privateConstructorUsedError;

  /// JWT claims
  Claims get claims => throw _privateConstructorUsedError;

  /// Create a copy of AuthUser
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AuthUserCopyWith<AuthUser> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AuthUserCopyWith<$Res> {
  factory $AuthUserCopyWith(AuthUser value, $Res Function(AuthUser) then) =
      _$AuthUserCopyWithImpl<$Res, AuthUser>;
  @useResult
  $Res call(
      {String id,
      List<String> roles,
      List<String> permissions,
      String? tenantId,
      Claims claims});

  $ClaimsCopyWith<$Res> get claims;
}

/// @nodoc
class _$AuthUserCopyWithImpl<$Res, $Val extends AuthUser>
    implements $AuthUserCopyWith<$Res> {
  _$AuthUserCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AuthUser
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? roles = null,
    Object? permissions = null,
    Object? tenantId = freezed,
    Object? claims = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
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
      claims: null == claims
          ? _value.claims
          : claims // ignore: cast_nullable_to_non_nullable
              as Claims,
    ) as $Val);
  }

  /// Create a copy of AuthUser
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ClaimsCopyWith<$Res> get claims {
    return $ClaimsCopyWith<$Res>(_value.claims, (value) {
      return _then(_value.copyWith(claims: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$AuthUserImplCopyWith<$Res>
    implements $AuthUserCopyWith<$Res> {
  factory _$$AuthUserImplCopyWith(
          _$AuthUserImpl value, $Res Function(_$AuthUserImpl) then) =
      __$$AuthUserImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      List<String> roles,
      List<String> permissions,
      String? tenantId,
      Claims claims});

  @override
  $ClaimsCopyWith<$Res> get claims;
}

/// @nodoc
class __$$AuthUserImplCopyWithImpl<$Res>
    extends _$AuthUserCopyWithImpl<$Res, _$AuthUserImpl>
    implements _$$AuthUserImplCopyWith<$Res> {
  __$$AuthUserImplCopyWithImpl(
      _$AuthUserImpl _value, $Res Function(_$AuthUserImpl) _then)
      : super(_value, _then);

  /// Create a copy of AuthUser
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? roles = null,
    Object? permissions = null,
    Object? tenantId = freezed,
    Object? claims = null,
  }) {
    return _then(_$AuthUserImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
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
      claims: null == claims
          ? _value.claims
          : claims // ignore: cast_nullable_to_non_nullable
              as Claims,
    ));
  }
}

/// @nodoc

class _$AuthUserImpl extends _AuthUser {
  const _$AuthUserImpl(
      {required this.id,
      required final List<String> roles,
      required final List<String> permissions,
      this.tenantId,
      required this.claims})
      : _roles = roles,
        _permissions = permissions,
        super._();

  /// User ID
  @override
  final String id;

  /// User roles
  final List<String> _roles;

  /// User roles
  @override
  List<String> get roles {
    if (_roles is EqualUnmodifiableListView) return _roles;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_roles);
  }

  /// User permissions
  final List<String> _permissions;

  /// User permissions
  @override
  List<String> get permissions {
    if (_permissions is EqualUnmodifiableListView) return _permissions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_permissions);
  }

  /// Tenant ID
  @override
  final String? tenantId;

  /// JWT claims
  @override
  final Claims claims;

  @override
  String toString() {
    return 'AuthUser(id: $id, roles: $roles, permissions: $permissions, tenantId: $tenantId, claims: $claims)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AuthUserImpl &&
            (identical(other.id, id) || other.id == id) &&
            const DeepCollectionEquality().equals(other._roles, _roles) &&
            const DeepCollectionEquality()
                .equals(other._permissions, _permissions) &&
            (identical(other.tenantId, tenantId) ||
                other.tenantId == tenantId) &&
            (identical(other.claims, claims) || other.claims == claims));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      const DeepCollectionEquality().hash(_roles),
      const DeepCollectionEquality().hash(_permissions),
      tenantId,
      claims);

  /// Create a copy of AuthUser
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AuthUserImplCopyWith<_$AuthUserImpl> get copyWith =>
      __$$AuthUserImplCopyWithImpl<_$AuthUserImpl>(this, _$identity);
}

abstract class _AuthUser extends AuthUser {
  const factory _AuthUser(
      {required final String id,
      required final List<String> roles,
      required final List<String> permissions,
      final String? tenantId,
      required final Claims claims}) = _$AuthUserImpl;
  const _AuthUser._() : super._();

  /// User ID
  @override
  String get id;

  /// User roles
  @override
  List<String> get roles;

  /// User permissions
  @override
  List<String> get permissions;

  /// Tenant ID
  @override
  String? get tenantId;

  /// JWT claims
  @override
  Claims get claims;

  /// Create a copy of AuthUser
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AuthUserImplCopyWith<_$AuthUserImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
