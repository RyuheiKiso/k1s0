// This is a generated file - do not edit.
//
// Generated from k1s0/system/auth/v1/auth.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/struct.pb.dart' as $2;

import '../../common/v1/types.pb.dart' as $1;
import 'auth.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'auth.pbenum.dart';

/// ValidateTokenRequest はトークン検証リクエスト。
class ValidateTokenRequest extends $pb.GeneratedMessage {
  factory ValidateTokenRequest({
    $core.String? token,
  }) {
    final result = create();
    if (token != null) result.token = token;
    return result;
  }

  ValidateTokenRequest._();

  factory ValidateTokenRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ValidateTokenRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ValidateTokenRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'token')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ValidateTokenRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ValidateTokenRequest copyWith(void Function(ValidateTokenRequest) updates) =>
      super.copyWith((message) => updates(message as ValidateTokenRequest))
          as ValidateTokenRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ValidateTokenRequest create() => ValidateTokenRequest._();
  @$core.override
  ValidateTokenRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ValidateTokenRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ValidateTokenRequest>(create);
  static ValidateTokenRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get token => $_getSZ(0);
  @$pb.TagNumber(1)
  set token($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasToken() => $_has(0);
  @$pb.TagNumber(1)
  void clearToken() => $_clearField(1);
}

/// ValidateTokenResponse はトークン検証レスポンス。
class ValidateTokenResponse extends $pb.GeneratedMessage {
  factory ValidateTokenResponse({
    $core.bool? valid,
    TokenClaims? claims,
    $core.String? errorMessage,
  }) {
    final result = create();
    if (valid != null) result.valid = valid;
    if (claims != null) result.claims = claims;
    if (errorMessage != null) result.errorMessage = errorMessage;
    return result;
  }

  ValidateTokenResponse._();

  factory ValidateTokenResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ValidateTokenResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ValidateTokenResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'valid')
    ..aOM<TokenClaims>(2, _omitFieldNames ? '' : 'claims',
        subBuilder: TokenClaims.create)
    ..aOS(3, _omitFieldNames ? '' : 'errorMessage')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ValidateTokenResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ValidateTokenResponse copyWith(
          void Function(ValidateTokenResponse) updates) =>
      super.copyWith((message) => updates(message as ValidateTokenResponse))
          as ValidateTokenResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ValidateTokenResponse create() => ValidateTokenResponse._();
  @$core.override
  ValidateTokenResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ValidateTokenResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ValidateTokenResponse>(create);
  static ValidateTokenResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get valid => $_getBF(0);
  @$pb.TagNumber(1)
  set valid($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasValid() => $_has(0);
  @$pb.TagNumber(1)
  void clearValid() => $_clearField(1);

  @$pb.TagNumber(2)
  TokenClaims get claims => $_getN(1);
  @$pb.TagNumber(2)
  set claims(TokenClaims value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasClaims() => $_has(1);
  @$pb.TagNumber(2)
  void clearClaims() => $_clearField(2);
  @$pb.TagNumber(2)
  TokenClaims ensureClaims() => $_ensure(1);

  /// valid == false の場合のエラー理由
  @$pb.TagNumber(3)
  $core.String get errorMessage => $_getSZ(2);
  @$pb.TagNumber(3)
  set errorMessage($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasErrorMessage() => $_has(2);
  @$pb.TagNumber(3)
  void clearErrorMessage() => $_clearField(3);
}

/// TokenClaims は JWT トークンのクレーム情報。
class TokenClaims extends $pb.GeneratedMessage {
  factory TokenClaims({
    $core.String? sub,
    $core.String? iss,
    $core.Iterable<$core.String>? aud,
    $fixnum.Int64? exp,
    $fixnum.Int64? iat,
    $core.String? jti,
    $core.String? preferredUsername,
    $core.String? email,
    RealmAccess? realmAccess,
    $core.Iterable<$core.MapEntry<$core.String, ClientRoles>>? resourceAccess,
    $core.Iterable<$core.String>? tierAccess,
    $core.String? scope,
    $core.String? typ,
    $core.String? azp,
  }) {
    final result = create();
    if (sub != null) result.sub = sub;
    if (iss != null) result.iss = iss;
    if (aud != null) result.aud.addAll(aud);
    if (exp != null) result.exp = exp;
    if (iat != null) result.iat = iat;
    if (jti != null) result.jti = jti;
    if (preferredUsername != null) result.preferredUsername = preferredUsername;
    if (email != null) result.email = email;
    if (realmAccess != null) result.realmAccess = realmAccess;
    if (resourceAccess != null)
      result.resourceAccess.addEntries(resourceAccess);
    if (tierAccess != null) result.tierAccess.addAll(tierAccess);
    if (scope != null) result.scope = scope;
    if (typ != null) result.typ = typ;
    if (azp != null) result.azp = azp;
    return result;
  }

  TokenClaims._();

  factory TokenClaims.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TokenClaims.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TokenClaims',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'sub')
    ..aOS(2, _omitFieldNames ? '' : 'iss')
    ..pPS(3, _omitFieldNames ? '' : 'aud')
    ..aInt64(4, _omitFieldNames ? '' : 'exp')
    ..aInt64(5, _omitFieldNames ? '' : 'iat')
    ..aOS(6, _omitFieldNames ? '' : 'jti')
    ..aOS(7, _omitFieldNames ? '' : 'preferredUsername')
    ..aOS(8, _omitFieldNames ? '' : 'email')
    ..aOM<RealmAccess>(9, _omitFieldNames ? '' : 'realmAccess',
        subBuilder: RealmAccess.create)
    ..m<$core.String, ClientRoles>(10, _omitFieldNames ? '' : 'resourceAccess',
        entryClassName: 'TokenClaims.ResourceAccessEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: ClientRoles.create,
        valueDefaultOrMaker: ClientRoles.getDefault,
        packageName: const $pb.PackageName('k1s0.system.auth.v1'))
    ..pPS(11, _omitFieldNames ? '' : 'tierAccess')
    ..aOS(12, _omitFieldNames ? '' : 'scope')
    ..aOS(13, _omitFieldNames ? '' : 'typ')
    ..aOS(14, _omitFieldNames ? '' : 'azp')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenClaims clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenClaims copyWith(void Function(TokenClaims) updates) =>
      super.copyWith((message) => updates(message as TokenClaims))
          as TokenClaims;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TokenClaims create() => TokenClaims._();
  @$core.override
  TokenClaims createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TokenClaims getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TokenClaims>(create);
  static TokenClaims? _defaultInstance;

  /// ユーザー UUID
  @$pb.TagNumber(1)
  $core.String get sub => $_getSZ(0);
  @$pb.TagNumber(1)
  set sub($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSub() => $_has(0);
  @$pb.TagNumber(1)
  void clearSub() => $_clearField(1);

  /// Issuer
  @$pb.TagNumber(2)
  $core.String get iss => $_getSZ(1);
  @$pb.TagNumber(2)
  set iss($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIss() => $_has(1);
  @$pb.TagNumber(2)
  void clearIss() => $_clearField(2);

  /// Audience（JWT spec では配列型。複数 audience に対応するため repeated を使用する）
  @$pb.TagNumber(3)
  $core.List<$core.String> get aud => $_getList(2);

  /// 有効期限（Unix epoch）
  @$pb.TagNumber(4)
  $fixnum.Int64 get exp => $_getI64(3);
  @$pb.TagNumber(4)
  set exp($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasExp() => $_has(3);
  @$pb.TagNumber(4)
  void clearExp() => $_clearField(4);

  /// 発行日時（Unix epoch）
  @$pb.TagNumber(5)
  $fixnum.Int64 get iat => $_getI64(4);
  @$pb.TagNumber(5)
  set iat($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIat() => $_has(4);
  @$pb.TagNumber(5)
  void clearIat() => $_clearField(5);

  /// Token ID
  @$pb.TagNumber(6)
  $core.String get jti => $_getSZ(5);
  @$pb.TagNumber(6)
  set jti($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasJti() => $_has(5);
  @$pb.TagNumber(6)
  void clearJti() => $_clearField(6);

  /// ユーザー名
  @$pb.TagNumber(7)
  $core.String get preferredUsername => $_getSZ(6);
  @$pb.TagNumber(7)
  set preferredUsername($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasPreferredUsername() => $_has(6);
  @$pb.TagNumber(7)
  void clearPreferredUsername() => $_clearField(7);

  /// メールアドレス
  @$pb.TagNumber(8)
  $core.String get email => $_getSZ(7);
  @$pb.TagNumber(8)
  set email($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasEmail() => $_has(7);
  @$pb.TagNumber(8)
  void clearEmail() => $_clearField(8);

  /// グローバルロール
  @$pb.TagNumber(9)
  RealmAccess get realmAccess => $_getN(8);
  @$pb.TagNumber(9)
  set realmAccess(RealmAccess value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasRealmAccess() => $_has(8);
  @$pb.TagNumber(9)
  void clearRealmAccess() => $_clearField(9);
  @$pb.TagNumber(9)
  RealmAccess ensureRealmAccess() => $_ensure(8);

  /// サービス固有ロール
  @$pb.TagNumber(10)
  $pb.PbMap<$core.String, ClientRoles> get resourceAccess => $_getMap(9);

  /// アクセス可能 Tier
  @$pb.TagNumber(11)
  $pb.PbList<$core.String> get tierAccess => $_getList(10);

  /// OAuth2 scope（スペース区切り）
  @$pb.TagNumber(12)
  $core.String get scope => $_getSZ(11);
  @$pb.TagNumber(12)
  set scope($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasScope() => $_has(11);
  @$pb.TagNumber(12)
  void clearScope() => $_clearField(12);

  /// トークン種別（例: Bearer）
  @$pb.TagNumber(13)
  $core.String get typ => $_getSZ(12);
  @$pb.TagNumber(13)
  set typ($core.String value) => $_setString(12, value);
  @$pb.TagNumber(13)
  $core.bool hasTyp() => $_has(12);
  @$pb.TagNumber(13)
  void clearTyp() => $_clearField(13);

  /// 認可されたクライアント ID
  @$pb.TagNumber(14)
  $core.String get azp => $_getSZ(13);
  @$pb.TagNumber(14)
  set azp($core.String value) => $_setString(13, value);
  @$pb.TagNumber(14)
  $core.bool hasAzp() => $_has(13);
  @$pb.TagNumber(14)
  void clearAzp() => $_clearField(14);
}

/// RealmAccess はグローバルロール一覧。
class RealmAccess extends $pb.GeneratedMessage {
  factory RealmAccess({
    $core.Iterable<$core.String>? roles,
  }) {
    final result = create();
    if (roles != null) result.roles.addAll(roles);
    return result;
  }

  RealmAccess._();

  factory RealmAccess.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RealmAccess.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RealmAccess',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'roles')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RealmAccess clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RealmAccess copyWith(void Function(RealmAccess) updates) =>
      super.copyWith((message) => updates(message as RealmAccess))
          as RealmAccess;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RealmAccess create() => RealmAccess._();
  @$core.override
  RealmAccess createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RealmAccess getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RealmAccess>(create);
  static RealmAccess? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get roles => $_getList(0);
}

/// ClientRoles はクライアント固有ロール一覧。
class ClientRoles extends $pb.GeneratedMessage {
  factory ClientRoles({
    $core.Iterable<$core.String>? roles,
  }) {
    final result = create();
    if (roles != null) result.roles.addAll(roles);
    return result;
  }

  ClientRoles._();

  factory ClientRoles.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ClientRoles.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientRoles',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'roles')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientRoles clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientRoles copyWith(void Function(ClientRoles) updates) =>
      super.copyWith((message) => updates(message as ClientRoles))
          as ClientRoles;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ClientRoles create() => ClientRoles._();
  @$core.override
  ClientRoles createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ClientRoles getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientRoles>(create);
  static ClientRoles? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get roles => $_getList(0);
}

/// GetUserRequest はユーザー情報取得リクエスト。
class GetUserRequest extends $pb.GeneratedMessage {
  factory GetUserRequest({
    $core.String? userId,
  }) {
    final result = create();
    if (userId != null) result.userId = userId;
    return result;
  }

  GetUserRequest._();

  factory GetUserRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetUserRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetUserRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'userId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUserRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUserRequest copyWith(void Function(GetUserRequest) updates) =>
      super.copyWith((message) => updates(message as GetUserRequest))
          as GetUserRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetUserRequest create() => GetUserRequest._();
  @$core.override
  GetUserRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetUserRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetUserRequest>(create);
  static GetUserRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get userId => $_getSZ(0);
  @$pb.TagNumber(1)
  set userId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasUserId() => $_has(0);
  @$pb.TagNumber(1)
  void clearUserId() => $_clearField(1);
}

/// GetUserResponse はユーザー情報取得レスポンス。
class GetUserResponse extends $pb.GeneratedMessage {
  factory GetUserResponse({
    User? user,
  }) {
    final result = create();
    if (user != null) result.user = user;
    return result;
  }

  GetUserResponse._();

  factory GetUserResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetUserResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetUserResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOM<User>(1, _omitFieldNames ? '' : 'user', subBuilder: User.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUserResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUserResponse copyWith(void Function(GetUserResponse) updates) =>
      super.copyWith((message) => updates(message as GetUserResponse))
          as GetUserResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetUserResponse create() => GetUserResponse._();
  @$core.override
  GetUserResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetUserResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetUserResponse>(create);
  static GetUserResponse? _defaultInstance;

  @$pb.TagNumber(1)
  User get user => $_getN(0);
  @$pb.TagNumber(1)
  set user(User value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasUser() => $_has(0);
  @$pb.TagNumber(1)
  void clearUser() => $_clearField(1);
  @$pb.TagNumber(1)
  User ensureUser() => $_ensure(0);
}

/// ListUsersRequest はユーザー一覧取得リクエスト。
class ListUsersRequest extends $pb.GeneratedMessage {
  factory ListUsersRequest({
    $1.Pagination? pagination,
    $core.String? search,
    $core.bool? enabled,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (search != null) result.search = search;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  ListUsersRequest._();

  factory ListUsersRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListUsersRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListUsersRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'search')
    ..aOB(3, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListUsersRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListUsersRequest copyWith(void Function(ListUsersRequest) updates) =>
      super.copyWith((message) => updates(message as ListUsersRequest))
          as ListUsersRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListUsersRequest create() => ListUsersRequest._();
  @$core.override
  ListUsersRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListUsersRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListUsersRequest>(create);
  static ListUsersRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Pagination get pagination => $_getN(0);
  @$pb.TagNumber(1)
  set pagination($1.Pagination value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPagination() => $_has(0);
  @$pb.TagNumber(1)
  void clearPagination() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Pagination ensurePagination() => $_ensure(0);

  /// ユーザー名・メールで部分一致検索
  @$pb.TagNumber(2)
  $core.String get search => $_getSZ(1);
  @$pb.TagNumber(2)
  set search($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSearch() => $_has(1);
  @$pb.TagNumber(2)
  void clearSearch() => $_clearField(2);

  /// 有効/無効フィルタ
  @$pb.TagNumber(3)
  $core.bool get enabled => $_getBF(2);
  @$pb.TagNumber(3)
  set enabled($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEnabled() => $_has(2);
  @$pb.TagNumber(3)
  void clearEnabled() => $_clearField(3);
}

/// ListUsersResponse はユーザー一覧取得レスポンス。
class ListUsersResponse extends $pb.GeneratedMessage {
  factory ListUsersResponse({
    $core.Iterable<User>? users,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (users != null) result.users.addAll(users);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListUsersResponse._();

  factory ListUsersResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListUsersResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListUsersResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..pPM<User>(1, _omitFieldNames ? '' : 'users', subBuilder: User.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListUsersResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListUsersResponse copyWith(void Function(ListUsersResponse) updates) =>
      super.copyWith((message) => updates(message as ListUsersResponse))
          as ListUsersResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListUsersResponse create() => ListUsersResponse._();
  @$core.override
  ListUsersResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListUsersResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListUsersResponse>(create);
  static ListUsersResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<User> get users => $_getList(0);

  @$pb.TagNumber(2)
  $1.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($1.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.PaginationResult ensurePagination() => $_ensure(1);
}

/// User はユーザー情報。
class User extends $pb.GeneratedMessage {
  factory User({
    $core.String? id,
    $core.String? username,
    $core.String? email,
    $core.String? firstName,
    $core.String? lastName,
    $core.bool? enabled,
    $core.bool? emailVerified,
    $1.Timestamp? createdAt,
    $core.Iterable<$core.MapEntry<$core.String, StringList>>? attributes,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (username != null) result.username = username;
    if (email != null) result.email = email;
    if (firstName != null) result.firstName = firstName;
    if (lastName != null) result.lastName = lastName;
    if (enabled != null) result.enabled = enabled;
    if (emailVerified != null) result.emailVerified = emailVerified;
    if (createdAt != null) result.createdAt = createdAt;
    if (attributes != null) result.attributes.addEntries(attributes);
    return result;
  }

  User._();

  factory User.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory User.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'User',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'username')
    ..aOS(3, _omitFieldNames ? '' : 'email')
    ..aOS(4, _omitFieldNames ? '' : 'firstName')
    ..aOS(5, _omitFieldNames ? '' : 'lastName')
    ..aOB(6, _omitFieldNames ? '' : 'enabled')
    ..aOB(7, _omitFieldNames ? '' : 'emailVerified')
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..m<$core.String, StringList>(9, _omitFieldNames ? '' : 'attributes',
        entryClassName: 'User.AttributesEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: StringList.create,
        valueDefaultOrMaker: StringList.getDefault,
        packageName: const $pb.PackageName('k1s0.system.auth.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  User clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  User copyWith(void Function(User) updates) =>
      super.copyWith((message) => updates(message as User)) as User;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static User create() => User._();
  @$core.override
  User createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static User getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<User>(create);
  static User? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get username => $_getSZ(1);
  @$pb.TagNumber(2)
  set username($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUsername() => $_has(1);
  @$pb.TagNumber(2)
  void clearUsername() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get email => $_getSZ(2);
  @$pb.TagNumber(3)
  set email($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEmail() => $_has(2);
  @$pb.TagNumber(3)
  void clearEmail() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get firstName => $_getSZ(3);
  @$pb.TagNumber(4)
  set firstName($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasFirstName() => $_has(3);
  @$pb.TagNumber(4)
  void clearFirstName() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get lastName => $_getSZ(4);
  @$pb.TagNumber(5)
  set lastName($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLastName() => $_has(4);
  @$pb.TagNumber(5)
  void clearLastName() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get enabled => $_getBF(5);
  @$pb.TagNumber(6)
  set enabled($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasEnabled() => $_has(5);
  @$pb.TagNumber(6)
  void clearEnabled() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get emailVerified => $_getBF(6);
  @$pb.TagNumber(7)
  set emailVerified($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasEmailVerified() => $_has(6);
  @$pb.TagNumber(7)
  void clearEmailVerified() => $_clearField(7);

  @$pb.TagNumber(8)
  $1.Timestamp get createdAt => $_getN(7);
  @$pb.TagNumber(8)
  set createdAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureCreatedAt() => $_ensure(7);

  /// カスタム属性（部署, 社員番号等）
  @$pb.TagNumber(9)
  $pb.PbMap<$core.String, StringList> get attributes => $_getMap(8);
}

/// StringList は文字列のリスト（map の値型として使用）。
class StringList extends $pb.GeneratedMessage {
  factory StringList({
    $core.Iterable<$core.String>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  StringList._();

  factory StringList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StringList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StringList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'values')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StringList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StringList copyWith(void Function(StringList) updates) =>
      super.copyWith((message) => updates(message as StringList)) as StringList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StringList create() => StringList._();
  @$core.override
  StringList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StringList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StringList>(create);
  static StringList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get values => $_getList(0);
}

/// GetUserRolesRequest はユーザーロール取得リクエスト。
class GetUserRolesRequest extends $pb.GeneratedMessage {
  factory GetUserRolesRequest({
    $core.String? userId,
  }) {
    final result = create();
    if (userId != null) result.userId = userId;
    return result;
  }

  GetUserRolesRequest._();

  factory GetUserRolesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetUserRolesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetUserRolesRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'userId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUserRolesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUserRolesRequest copyWith(void Function(GetUserRolesRequest) updates) =>
      super.copyWith((message) => updates(message as GetUserRolesRequest))
          as GetUserRolesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetUserRolesRequest create() => GetUserRolesRequest._();
  @$core.override
  GetUserRolesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetUserRolesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetUserRolesRequest>(create);
  static GetUserRolesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get userId => $_getSZ(0);
  @$pb.TagNumber(1)
  set userId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasUserId() => $_has(0);
  @$pb.TagNumber(1)
  void clearUserId() => $_clearField(1);
}

/// GetUserRolesResponse はユーザーロール取得レスポンス。
class GetUserRolesResponse extends $pb.GeneratedMessage {
  factory GetUserRolesResponse({
    $core.String? userId,
    $core.Iterable<Role>? realmRoles,
    $core.Iterable<$core.MapEntry<$core.String, RoleList>>? clientRoles,
  }) {
    final result = create();
    if (userId != null) result.userId = userId;
    if (realmRoles != null) result.realmRoles.addAll(realmRoles);
    if (clientRoles != null) result.clientRoles.addEntries(clientRoles);
    return result;
  }

  GetUserRolesResponse._();

  factory GetUserRolesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetUserRolesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetUserRolesResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'userId')
    ..pPM<Role>(2, _omitFieldNames ? '' : 'realmRoles', subBuilder: Role.create)
    ..m<$core.String, RoleList>(3, _omitFieldNames ? '' : 'clientRoles',
        entryClassName: 'GetUserRolesResponse.ClientRolesEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: RoleList.create,
        valueDefaultOrMaker: RoleList.getDefault,
        packageName: const $pb.PackageName('k1s0.system.auth.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUserRolesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUserRolesResponse copyWith(void Function(GetUserRolesResponse) updates) =>
      super.copyWith((message) => updates(message as GetUserRolesResponse))
          as GetUserRolesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetUserRolesResponse create() => GetUserRolesResponse._();
  @$core.override
  GetUserRolesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetUserRolesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetUserRolesResponse>(create);
  static GetUserRolesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get userId => $_getSZ(0);
  @$pb.TagNumber(1)
  set userId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasUserId() => $_has(0);
  @$pb.TagNumber(1)
  void clearUserId() => $_clearField(1);

  /// グローバルロール一覧
  @$pb.TagNumber(2)
  $pb.PbList<Role> get realmRoles => $_getList(1);

  /// クライアント別ロール
  @$pb.TagNumber(3)
  $pb.PbMap<$core.String, RoleList> get clientRoles => $_getMap(2);
}

/// Role はロール情報。
class Role extends $pb.GeneratedMessage {
  factory Role({
    $core.String? id,
    $core.String? name,
    $core.String? description,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    return result;
  }

  Role._();

  factory Role.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Role.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Role',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Role clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Role copyWith(void Function(Role) updates) =>
      super.copyWith((message) => updates(message as Role)) as Role;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Role create() => Role._();
  @$core.override
  Role createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Role getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Role>(create);
  static Role? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);
}

/// RoleList はロールのリスト（map の値型として使用）。
class RoleList extends $pb.GeneratedMessage {
  factory RoleList({
    $core.Iterable<Role>? roles,
  }) {
    final result = create();
    if (roles != null) result.roles.addAll(roles);
    return result;
  }

  RoleList._();

  factory RoleList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RoleList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RoleList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..pPM<Role>(1, _omitFieldNames ? '' : 'roles', subBuilder: Role.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RoleList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RoleList copyWith(void Function(RoleList) updates) =>
      super.copyWith((message) => updates(message as RoleList)) as RoleList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RoleList create() => RoleList._();
  @$core.override
  RoleList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RoleList getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RoleList>(create);
  static RoleList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Role> get roles => $_getList(0);
}

/// CheckPermissionRequest はパーミッション確認リクエスト。
class CheckPermissionRequest extends $pb.GeneratedMessage {
  factory CheckPermissionRequest({
    $core.String? userId,
    $core.String? permission,
    $core.String? resource,
    $core.Iterable<$core.String>? roles,
  }) {
    final result = create();
    if (userId != null) result.userId = userId;
    if (permission != null) result.permission = permission;
    if (resource != null) result.resource = resource;
    if (roles != null) result.roles.addAll(roles);
    return result;
  }

  CheckPermissionRequest._();

  factory CheckPermissionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckPermissionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckPermissionRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'userId')
    ..aOS(2, _omitFieldNames ? '' : 'permission')
    ..aOS(3, _omitFieldNames ? '' : 'resource')
    ..pPS(4, _omitFieldNames ? '' : 'roles')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckPermissionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckPermissionRequest copyWith(
          void Function(CheckPermissionRequest) updates) =>
      super.copyWith((message) => updates(message as CheckPermissionRequest))
          as CheckPermissionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckPermissionRequest create() => CheckPermissionRequest._();
  @$core.override
  CheckPermissionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckPermissionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckPermissionRequest>(create);
  static CheckPermissionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get userId => $_getSZ(0);
  @$pb.TagNumber(1)
  set userId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasUserId() => $_has(0);
  @$pb.TagNumber(1)
  void clearUserId() => $_clearField(1);

  /// read, write, delete, admin
  @$pb.TagNumber(2)
  $core.String get permission => $_getSZ(1);
  @$pb.TagNumber(2)
  set permission($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPermission() => $_has(1);
  @$pb.TagNumber(2)
  void clearPermission() => $_clearField(2);

  /// users, auth_config, audit_logs, etc.
  @$pb.TagNumber(3)
  $core.String get resource => $_getSZ(2);
  @$pb.TagNumber(3)
  set resource($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasResource() => $_has(2);
  @$pb.TagNumber(3)
  void clearResource() => $_clearField(3);

  /// JWT Claims から取得したロール一覧
  @$pb.TagNumber(4)
  $pb.PbList<$core.String> get roles => $_getList(3);
}

/// CheckPermissionResponse はパーミッション確認レスポンス。
class CheckPermissionResponse extends $pb.GeneratedMessage {
  factory CheckPermissionResponse({
    $core.bool? allowed,
    $core.String? reason,
  }) {
    final result = create();
    if (allowed != null) result.allowed = allowed;
    if (reason != null) result.reason = reason;
    return result;
  }

  CheckPermissionResponse._();

  factory CheckPermissionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckPermissionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckPermissionResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'allowed')
    ..aOS(2, _omitFieldNames ? '' : 'reason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckPermissionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckPermissionResponse copyWith(
          void Function(CheckPermissionResponse) updates) =>
      super.copyWith((message) => updates(message as CheckPermissionResponse))
          as CheckPermissionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckPermissionResponse create() => CheckPermissionResponse._();
  @$core.override
  CheckPermissionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckPermissionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckPermissionResponse>(create);
  static CheckPermissionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get allowed => $_getBF(0);
  @$pb.TagNumber(1)
  set allowed($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAllowed() => $_has(0);
  @$pb.TagNumber(1)
  void clearAllowed() => $_clearField(1);

  /// 拒否理由（allowed == false の場合）
  @$pb.TagNumber(2)
  $core.String get reason => $_getSZ(1);
  @$pb.TagNumber(2)
  set reason($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasReason() => $_has(1);
  @$pb.TagNumber(2)
  void clearReason() => $_clearField(2);
}

/// RecordAuditLogRequest は監査ログ記録リクエスト。
class RecordAuditLogRequest extends $pb.GeneratedMessage {
  factory RecordAuditLogRequest({
    $core.String? eventType,
    $core.String? userId,
    $core.String? ipAddress,
    $core.String? userAgent,
    $core.String? resource,
    $core.String? action,
    $core.String? result,
    $2.Struct? detail,
    $core.String? resourceId,
    $core.String? traceId,
    AuditEventType? eventTypeEnum,
    AuditResult? resultEnum,
  }) {
    final result$ = create();
    if (eventType != null) result$.eventType = eventType;
    if (userId != null) result$.userId = userId;
    if (ipAddress != null) result$.ipAddress = ipAddress;
    if (userAgent != null) result$.userAgent = userAgent;
    if (resource != null) result$.resource = resource;
    if (action != null) result$.action = action;
    if (result != null) result$.result = result;
    if (detail != null) result$.detail = detail;
    if (resourceId != null) result$.resourceId = resourceId;
    if (traceId != null) result$.traceId = traceId;
    if (eventTypeEnum != null) result$.eventTypeEnum = eventTypeEnum;
    if (resultEnum != null) result$.resultEnum = resultEnum;
    return result$;
  }

  RecordAuditLogRequest._();

  factory RecordAuditLogRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RecordAuditLogRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RecordAuditLogRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'eventType')
    ..aOS(2, _omitFieldNames ? '' : 'userId')
    ..aOS(3, _omitFieldNames ? '' : 'ipAddress')
    ..aOS(4, _omitFieldNames ? '' : 'userAgent')
    ..aOS(5, _omitFieldNames ? '' : 'resource')
    ..aOS(6, _omitFieldNames ? '' : 'action')
    ..aOS(7, _omitFieldNames ? '' : 'result')
    ..aOM<$2.Struct>(8, _omitFieldNames ? '' : 'detail',
        subBuilder: $2.Struct.create)
    ..aOS(9, _omitFieldNames ? '' : 'resourceId')
    ..aOS(10, _omitFieldNames ? '' : 'traceId')
    ..aE<AuditEventType>(11, _omitFieldNames ? '' : 'eventTypeEnum',
        enumValues: AuditEventType.values)
    ..aE<AuditResult>(12, _omitFieldNames ? '' : 'resultEnum',
        enumValues: AuditResult.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RecordAuditLogRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RecordAuditLogRequest copyWith(
          void Function(RecordAuditLogRequest) updates) =>
      super.copyWith((message) => updates(message as RecordAuditLogRequest))
          as RecordAuditLogRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RecordAuditLogRequest create() => RecordAuditLogRequest._();
  @$core.override
  RecordAuditLogRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RecordAuditLogRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RecordAuditLogRequest>(create);
  static RecordAuditLogRequest? _defaultInstance;

  /// LOGIN_SUCCESS, LOGIN_FAILURE, TOKEN_VALIDATE, PERMISSION_DENIED 等
  /// Deprecated: event_type_enum を使用すること。
  @$pb.TagNumber(1)
  $core.String get eventType => $_getSZ(0);
  @$pb.TagNumber(1)
  set eventType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEventType() => $_has(0);
  @$pb.TagNumber(1)
  void clearEventType() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get userId => $_getSZ(1);
  @$pb.TagNumber(2)
  set userId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUserId() => $_has(1);
  @$pb.TagNumber(2)
  void clearUserId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get ipAddress => $_getSZ(2);
  @$pb.TagNumber(3)
  set ipAddress($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIpAddress() => $_has(2);
  @$pb.TagNumber(3)
  void clearIpAddress() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get userAgent => $_getSZ(3);
  @$pb.TagNumber(4)
  set userAgent($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasUserAgent() => $_has(3);
  @$pb.TagNumber(4)
  void clearUserAgent() => $_clearField(4);

  /// アクセス対象リソース
  @$pb.TagNumber(5)
  $core.String get resource => $_getSZ(4);
  @$pb.TagNumber(5)
  set resource($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasResource() => $_has(4);
  @$pb.TagNumber(5)
  void clearResource() => $_clearField(5);

  /// HTTP メソッドまたは gRPC メソッド名
  @$pb.TagNumber(6)
  $core.String get action => $_getSZ(5);
  @$pb.TagNumber(6)
  set action($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAction() => $_has(5);
  @$pb.TagNumber(6)
  void clearAction() => $_clearField(6);

  /// SUCCESS / FAILURE
  /// Deprecated: result_enum を使用すること。
  @$pb.TagNumber(7)
  $core.String get result => $_getSZ(6);
  @$pb.TagNumber(7)
  set result($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasResult() => $_has(6);
  @$pb.TagNumber(7)
  void clearResult() => $_clearField(7);

  /// 操作の詳細情報（client_id, grant_type 等）
  @$pb.TagNumber(8)
  $2.Struct get detail => $_getN(7);
  @$pb.TagNumber(8)
  set detail($2.Struct value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasDetail() => $_has(7);
  @$pb.TagNumber(8)
  void clearDetail() => $_clearField(8);
  @$pb.TagNumber(8)
  $2.Struct ensureDetail() => $_ensure(7);

  /// 操作対象リソースの ID
  @$pb.TagNumber(9)
  $core.String get resourceId => $_getSZ(8);
  @$pb.TagNumber(9)
  set resourceId($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasResourceId() => $_has(8);
  @$pb.TagNumber(9)
  void clearResourceId() => $_clearField(9);

  /// OpenTelemetry トレース ID
  @$pb.TagNumber(10)
  $core.String get traceId => $_getSZ(9);
  @$pb.TagNumber(10)
  set traceId($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasTraceId() => $_has(9);
  @$pb.TagNumber(10)
  void clearTraceId() => $_clearField(10);

  /// 監査イベント種別（enum）
  @$pb.TagNumber(11)
  AuditEventType get eventTypeEnum => $_getN(10);
  @$pb.TagNumber(11)
  set eventTypeEnum(AuditEventType value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasEventTypeEnum() => $_has(10);
  @$pb.TagNumber(11)
  void clearEventTypeEnum() => $_clearField(11);

  /// 監査イベント結果（enum）
  @$pb.TagNumber(12)
  AuditResult get resultEnum => $_getN(11);
  @$pb.TagNumber(12)
  set resultEnum(AuditResult value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasResultEnum() => $_has(11);
  @$pb.TagNumber(12)
  void clearResultEnum() => $_clearField(12);
}

/// RecordAuditLogResponse は監査ログ記録レスポンス。
class RecordAuditLogResponse extends $pb.GeneratedMessage {
  factory RecordAuditLogResponse({
    $core.String? id,
    $1.Timestamp? createdAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (createdAt != null) result.createdAt = createdAt;
    return result;
  }

  RecordAuditLogResponse._();

  factory RecordAuditLogResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RecordAuditLogResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RecordAuditLogResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOM<$1.Timestamp>(2, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RecordAuditLogResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RecordAuditLogResponse copyWith(
          void Function(RecordAuditLogResponse) updates) =>
      super.copyWith((message) => updates(message as RecordAuditLogResponse))
          as RecordAuditLogResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RecordAuditLogResponse create() => RecordAuditLogResponse._();
  @$core.override
  RecordAuditLogResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RecordAuditLogResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RecordAuditLogResponse>(create);
  static RecordAuditLogResponse? _defaultInstance;

  /// 監査ログ UUID
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Timestamp get createdAt => $_getN(1);
  @$pb.TagNumber(2)
  set createdAt($1.Timestamp value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasCreatedAt() => $_has(1);
  @$pb.TagNumber(2)
  void clearCreatedAt() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Timestamp ensureCreatedAt() => $_ensure(1);
}

/// SearchAuditLogsRequest は監査ログ検索リクエスト。
class SearchAuditLogsRequest extends $pb.GeneratedMessage {
  factory SearchAuditLogsRequest({
    $1.Pagination? pagination,
    $core.String? userId,
    $core.String? eventType,
    $1.Timestamp? from,
    $1.Timestamp? to,
    $core.String? result,
    AuditEventType? eventTypeEnum,
    AuditResult? resultEnum,
  }) {
    final result$ = create();
    if (pagination != null) result$.pagination = pagination;
    if (userId != null) result$.userId = userId;
    if (eventType != null) result$.eventType = eventType;
    if (from != null) result$.from = from;
    if (to != null) result$.to = to;
    if (result != null) result$.result = result;
    if (eventTypeEnum != null) result$.eventTypeEnum = eventTypeEnum;
    if (resultEnum != null) result$.resultEnum = resultEnum;
    return result$;
  }

  SearchAuditLogsRequest._();

  factory SearchAuditLogsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SearchAuditLogsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SearchAuditLogsRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'userId')
    ..aOS(3, _omitFieldNames ? '' : 'eventType')
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'from',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(5, _omitFieldNames ? '' : 'to',
        subBuilder: $1.Timestamp.create)
    ..aOS(6, _omitFieldNames ? '' : 'result')
    ..aE<AuditEventType>(7, _omitFieldNames ? '' : 'eventTypeEnum',
        enumValues: AuditEventType.values)
    ..aE<AuditResult>(8, _omitFieldNames ? '' : 'resultEnum',
        enumValues: AuditResult.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchAuditLogsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchAuditLogsRequest copyWith(
          void Function(SearchAuditLogsRequest) updates) =>
      super.copyWith((message) => updates(message as SearchAuditLogsRequest))
          as SearchAuditLogsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SearchAuditLogsRequest create() => SearchAuditLogsRequest._();
  @$core.override
  SearchAuditLogsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SearchAuditLogsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SearchAuditLogsRequest>(create);
  static SearchAuditLogsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Pagination get pagination => $_getN(0);
  @$pb.TagNumber(1)
  set pagination($1.Pagination value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPagination() => $_has(0);
  @$pb.TagNumber(1)
  void clearPagination() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Pagination ensurePagination() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get userId => $_getSZ(1);
  @$pb.TagNumber(2)
  set userId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUserId() => $_has(1);
  @$pb.TagNumber(2)
  void clearUserId() => $_clearField(2);

  /// Deprecated: event_type_enum を使用すること。
  @$pb.TagNumber(3)
  $core.String get eventType => $_getSZ(2);
  @$pb.TagNumber(3)
  set eventType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEventType() => $_has(2);
  @$pb.TagNumber(3)
  void clearEventType() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Timestamp get from => $_getN(3);
  @$pb.TagNumber(4)
  set from($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasFrom() => $_has(3);
  @$pb.TagNumber(4)
  void clearFrom() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureFrom() => $_ensure(3);

  @$pb.TagNumber(5)
  $1.Timestamp get to => $_getN(4);
  @$pb.TagNumber(5)
  set to($1.Timestamp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasTo() => $_has(4);
  @$pb.TagNumber(5)
  void clearTo() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Timestamp ensureTo() => $_ensure(4);

  /// SUCCESS / FAILURE
  /// Deprecated: result_enum を使用すること。
  @$pb.TagNumber(6)
  $core.String get result => $_getSZ(5);
  @$pb.TagNumber(6)
  set result($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasResult() => $_has(5);
  @$pb.TagNumber(6)
  void clearResult() => $_clearField(6);

  /// 監査イベント種別フィルタ（enum）
  @$pb.TagNumber(7)
  AuditEventType get eventTypeEnum => $_getN(6);
  @$pb.TagNumber(7)
  set eventTypeEnum(AuditEventType value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasEventTypeEnum() => $_has(6);
  @$pb.TagNumber(7)
  void clearEventTypeEnum() => $_clearField(7);

  /// 監査イベント結果フィルタ（enum）
  @$pb.TagNumber(8)
  AuditResult get resultEnum => $_getN(7);
  @$pb.TagNumber(8)
  set resultEnum(AuditResult value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasResultEnum() => $_has(7);
  @$pb.TagNumber(8)
  void clearResultEnum() => $_clearField(8);
}

/// SearchAuditLogsResponse は監査ログ検索レスポンス。
class SearchAuditLogsResponse extends $pb.GeneratedMessage {
  factory SearchAuditLogsResponse({
    $core.Iterable<AuditLog>? logs,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (logs != null) result.logs.addAll(logs);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  SearchAuditLogsResponse._();

  factory SearchAuditLogsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SearchAuditLogsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SearchAuditLogsResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..pPM<AuditLog>(1, _omitFieldNames ? '' : 'logs',
        subBuilder: AuditLog.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchAuditLogsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchAuditLogsResponse copyWith(
          void Function(SearchAuditLogsResponse) updates) =>
      super.copyWith((message) => updates(message as SearchAuditLogsResponse))
          as SearchAuditLogsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SearchAuditLogsResponse create() => SearchAuditLogsResponse._();
  @$core.override
  SearchAuditLogsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SearchAuditLogsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SearchAuditLogsResponse>(create);
  static SearchAuditLogsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<AuditLog> get logs => $_getList(0);

  @$pb.TagNumber(2)
  $1.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($1.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.PaginationResult ensurePagination() => $_ensure(1);
}

/// AuditLog は監査ログエントリ。
class AuditLog extends $pb.GeneratedMessage {
  factory AuditLog({
    $core.String? id,
    $core.String? eventType,
    $core.String? userId,
    $core.String? ipAddress,
    $core.String? userAgent,
    $core.String? resource,
    $core.String? action,
    $core.String? result,
    $2.Struct? detail,
    $1.Timestamp? createdAt,
    $core.String? resourceId,
    $core.String? traceId,
    AuditEventType? eventTypeEnum,
    AuditResult? resultEnum,
  }) {
    final result$ = create();
    if (id != null) result$.id = id;
    if (eventType != null) result$.eventType = eventType;
    if (userId != null) result$.userId = userId;
    if (ipAddress != null) result$.ipAddress = ipAddress;
    if (userAgent != null) result$.userAgent = userAgent;
    if (resource != null) result$.resource = resource;
    if (action != null) result$.action = action;
    if (result != null) result$.result = result;
    if (detail != null) result$.detail = detail;
    if (createdAt != null) result$.createdAt = createdAt;
    if (resourceId != null) result$.resourceId = resourceId;
    if (traceId != null) result$.traceId = traceId;
    if (eventTypeEnum != null) result$.eventTypeEnum = eventTypeEnum;
    if (resultEnum != null) result$.resultEnum = resultEnum;
    return result$;
  }

  AuditLog._();

  factory AuditLog.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AuditLog.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AuditLog',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.auth.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'eventType')
    ..aOS(3, _omitFieldNames ? '' : 'userId')
    ..aOS(4, _omitFieldNames ? '' : 'ipAddress')
    ..aOS(5, _omitFieldNames ? '' : 'userAgent')
    ..aOS(6, _omitFieldNames ? '' : 'resource')
    ..aOS(7, _omitFieldNames ? '' : 'action')
    ..aOS(8, _omitFieldNames ? '' : 'result')
    ..aOM<$2.Struct>(9, _omitFieldNames ? '' : 'detail',
        subBuilder: $2.Struct.create)
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOS(11, _omitFieldNames ? '' : 'resourceId')
    ..aOS(12, _omitFieldNames ? '' : 'traceId')
    ..aE<AuditEventType>(13, _omitFieldNames ? '' : 'eventTypeEnum',
        enumValues: AuditEventType.values)
    ..aE<AuditResult>(14, _omitFieldNames ? '' : 'resultEnum',
        enumValues: AuditResult.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuditLog clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuditLog copyWith(void Function(AuditLog) updates) =>
      super.copyWith((message) => updates(message as AuditLog)) as AuditLog;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AuditLog create() => AuditLog._();
  @$core.override
  AuditLog createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AuditLog getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<AuditLog>(create);
  static AuditLog? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// Deprecated: event_type_enum を使用すること。
  @$pb.TagNumber(2)
  $core.String get eventType => $_getSZ(1);
  @$pb.TagNumber(2)
  set eventType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEventType() => $_has(1);
  @$pb.TagNumber(2)
  void clearEventType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get userId => $_getSZ(2);
  @$pb.TagNumber(3)
  set userId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasUserId() => $_has(2);
  @$pb.TagNumber(3)
  void clearUserId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get ipAddress => $_getSZ(3);
  @$pb.TagNumber(4)
  set ipAddress($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasIpAddress() => $_has(3);
  @$pb.TagNumber(4)
  void clearIpAddress() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get userAgent => $_getSZ(4);
  @$pb.TagNumber(5)
  set userAgent($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasUserAgent() => $_has(4);
  @$pb.TagNumber(5)
  void clearUserAgent() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get resource => $_getSZ(5);
  @$pb.TagNumber(6)
  set resource($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasResource() => $_has(5);
  @$pb.TagNumber(6)
  void clearResource() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get action => $_getSZ(6);
  @$pb.TagNumber(7)
  set action($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasAction() => $_has(6);
  @$pb.TagNumber(7)
  void clearAction() => $_clearField(7);

  /// Deprecated: result_enum を使用すること。
  @$pb.TagNumber(8)
  $core.String get result => $_getSZ(7);
  @$pb.TagNumber(8)
  set result($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasResult() => $_has(7);
  @$pb.TagNumber(8)
  void clearResult() => $_clearField(8);

  /// 操作の詳細情報（変更前後の値等）
  @$pb.TagNumber(9)
  $2.Struct get detail => $_getN(8);
  @$pb.TagNumber(9)
  set detail($2.Struct value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasDetail() => $_has(8);
  @$pb.TagNumber(9)
  void clearDetail() => $_clearField(9);
  @$pb.TagNumber(9)
  $2.Struct ensureDetail() => $_ensure(8);

  @$pb.TagNumber(10)
  $1.Timestamp get createdAt => $_getN(9);
  @$pb.TagNumber(10)
  set createdAt($1.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasCreatedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearCreatedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.Timestamp ensureCreatedAt() => $_ensure(9);

  /// 操作対象リソースの ID
  @$pb.TagNumber(11)
  $core.String get resourceId => $_getSZ(10);
  @$pb.TagNumber(11)
  set resourceId($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasResourceId() => $_has(10);
  @$pb.TagNumber(11)
  void clearResourceId() => $_clearField(11);

  /// OpenTelemetry トレース ID
  @$pb.TagNumber(12)
  $core.String get traceId => $_getSZ(11);
  @$pb.TagNumber(12)
  set traceId($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasTraceId() => $_has(11);
  @$pb.TagNumber(12)
  void clearTraceId() => $_clearField(12);

  /// 監査イベント種別（enum）
  @$pb.TagNumber(13)
  AuditEventType get eventTypeEnum => $_getN(12);
  @$pb.TagNumber(13)
  set eventTypeEnum(AuditEventType value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasEventTypeEnum() => $_has(12);
  @$pb.TagNumber(13)
  void clearEventTypeEnum() => $_clearField(13);

  /// 監査イベント結果（enum）
  @$pb.TagNumber(14)
  AuditResult get resultEnum => $_getN(13);
  @$pb.TagNumber(14)
  set resultEnum(AuditResult value) => $_setField(14, value);
  @$pb.TagNumber(14)
  $core.bool hasResultEnum() => $_has(13);
  @$pb.TagNumber(14)
  void clearResultEnum() => $_clearField(14);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
