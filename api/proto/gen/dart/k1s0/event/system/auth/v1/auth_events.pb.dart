// This is a generated file - do not edit.
//
// Generated from k1s0/event/system/auth/v1/auth_events.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

import '../../../../system/common/v1/event_metadata.pb.dart' as $0;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

/// LoginEvent はログイン成功/失敗イベント。
/// Kafka トピック: k1s0.system.auth.login.v1
/// パーティションキー: user_id
class LoginEvent extends $pb.GeneratedMessage {
  factory LoginEvent({
    $0.EventMetadata? metadata,
    $core.String? userId,
    $core.String? username,
    $core.String? clientId,
    $core.String? ipAddress,
    $core.String? userAgent,
    $core.String? result,
    $core.String? failureReason,
  }) {
    final result$ = create();
    if (metadata != null) result$.metadata = metadata;
    if (userId != null) result$.userId = userId;
    if (username != null) result$.username = username;
    if (clientId != null) result$.clientId = clientId;
    if (ipAddress != null) result$.ipAddress = ipAddress;
    if (userAgent != null) result$.userAgent = userAgent;
    if (result != null) result$.result = result;
    if (failureReason != null) result$.failureReason = failureReason;
    return result$;
  }

  LoginEvent._();

  factory LoginEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory LoginEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LoginEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.system.auth.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'userId')
    ..aOS(3, _omitFieldNames ? '' : 'username')
    ..aOS(4, _omitFieldNames ? '' : 'clientId')
    ..aOS(5, _omitFieldNames ? '' : 'ipAddress')
    ..aOS(6, _omitFieldNames ? '' : 'userAgent')
    ..aOS(7, _omitFieldNames ? '' : 'result')
    ..aOS(8, _omitFieldNames ? '' : 'failureReason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LoginEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LoginEvent copyWith(void Function(LoginEvent) updates) =>
      super.copyWith((message) => updates(message as LoginEvent)) as LoginEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static LoginEvent create() => LoginEvent._();
  @$core.override
  LoginEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static LoginEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<LoginEvent>(create);
  static LoginEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $0.EventMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata($0.EventMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EventMetadata ensureMetadata() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get userId => $_getSZ(1);
  @$pb.TagNumber(2)
  set userId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUserId() => $_has(1);
  @$pb.TagNumber(2)
  void clearUserId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get username => $_getSZ(2);
  @$pb.TagNumber(3)
  set username($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasUsername() => $_has(2);
  @$pb.TagNumber(3)
  void clearUsername() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get clientId => $_getSZ(3);
  @$pb.TagNumber(4)
  set clientId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasClientId() => $_has(3);
  @$pb.TagNumber(4)
  void clearClientId() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get ipAddress => $_getSZ(4);
  @$pb.TagNumber(5)
  set ipAddress($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIpAddress() => $_has(4);
  @$pb.TagNumber(5)
  void clearIpAddress() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get userAgent => $_getSZ(5);
  @$pb.TagNumber(6)
  set userAgent($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasUserAgent() => $_has(5);
  @$pb.TagNumber(6)
  void clearUserAgent() => $_clearField(6);

  /// SUCCESS / FAILURE
  @$pb.TagNumber(7)
  $core.String get result => $_getSZ(6);
  @$pb.TagNumber(7)
  set result($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasResult() => $_has(6);
  @$pb.TagNumber(7)
  void clearResult() => $_clearField(7);

  /// 失敗時のみ
  @$pb.TagNumber(8)
  $core.String get failureReason => $_getSZ(7);
  @$pb.TagNumber(8)
  set failureReason($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasFailureReason() => $_has(7);
  @$pb.TagNumber(8)
  void clearFailureReason() => $_clearField(8);
}

/// TokenValidationEvent はトークン検証結果イベント。
/// Kafka トピック: k1s0.system.auth.audit.v1
/// パーティションキー: user_id
class TokenValidationEvent extends $pb.GeneratedMessage {
  factory TokenValidationEvent({
    $0.EventMetadata? metadata,
    $core.String? userId,
    $core.String? tokenJti,
    $core.bool? valid,
    $core.String? errorMessage,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (userId != null) result.userId = userId;
    if (tokenJti != null) result.tokenJti = tokenJti;
    if (valid != null) result.valid = valid;
    if (errorMessage != null) result.errorMessage = errorMessage;
    return result;
  }

  TokenValidationEvent._();

  factory TokenValidationEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TokenValidationEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TokenValidationEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.system.auth.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'userId')
    ..aOS(3, _omitFieldNames ? '' : 'tokenJti')
    ..aOB(4, _omitFieldNames ? '' : 'valid')
    ..aOS(5, _omitFieldNames ? '' : 'errorMessage')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenValidationEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenValidationEvent copyWith(void Function(TokenValidationEvent) updates) =>
      super.copyWith((message) => updates(message as TokenValidationEvent))
          as TokenValidationEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TokenValidationEvent create() => TokenValidationEvent._();
  @$core.override
  TokenValidationEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TokenValidationEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TokenValidationEvent>(create);
  static TokenValidationEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $0.EventMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata($0.EventMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EventMetadata ensureMetadata() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get userId => $_getSZ(1);
  @$pb.TagNumber(2)
  set userId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUserId() => $_has(1);
  @$pb.TagNumber(2)
  void clearUserId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get tokenJti => $_getSZ(2);
  @$pb.TagNumber(3)
  set tokenJti($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTokenJti() => $_has(2);
  @$pb.TagNumber(3)
  void clearTokenJti() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get valid => $_getBF(3);
  @$pb.TagNumber(4)
  set valid($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasValid() => $_has(3);
  @$pb.TagNumber(4)
  void clearValid() => $_clearField(4);

  /// 検証失敗時のみ
  @$pb.TagNumber(5)
  $core.String get errorMessage => $_getSZ(4);
  @$pb.TagNumber(5)
  set errorMessage($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasErrorMessage() => $_has(4);
  @$pb.TagNumber(5)
  void clearErrorMessage() => $_clearField(5);
}

/// PermissionCheckEvent はパーミッション確認結果イベント。
/// Kafka トピック: k1s0.system.auth.audit.v1
/// パーティションキー: user_id
class PermissionCheckEvent extends $pb.GeneratedMessage {
  factory PermissionCheckEvent({
    $0.EventMetadata? metadata,
    $core.String? userId,
    $core.String? permission,
    $core.String? resource,
    $core.Iterable<$core.String>? roles,
    $core.bool? allowed,
    $core.String? reason,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (userId != null) result.userId = userId;
    if (permission != null) result.permission = permission;
    if (resource != null) result.resource = resource;
    if (roles != null) result.roles.addAll(roles);
    if (allowed != null) result.allowed = allowed;
    if (reason != null) result.reason = reason;
    return result;
  }

  PermissionCheckEvent._();

  factory PermissionCheckEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PermissionCheckEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PermissionCheckEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.system.auth.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'userId')
    ..aOS(3, _omitFieldNames ? '' : 'permission')
    ..aOS(4, _omitFieldNames ? '' : 'resource')
    ..pPS(5, _omitFieldNames ? '' : 'roles')
    ..aOB(6, _omitFieldNames ? '' : 'allowed')
    ..aOS(7, _omitFieldNames ? '' : 'reason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PermissionCheckEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PermissionCheckEvent copyWith(void Function(PermissionCheckEvent) updates) =>
      super.copyWith((message) => updates(message as PermissionCheckEvent))
          as PermissionCheckEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PermissionCheckEvent create() => PermissionCheckEvent._();
  @$core.override
  PermissionCheckEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PermissionCheckEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PermissionCheckEvent>(create);
  static PermissionCheckEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $0.EventMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata($0.EventMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EventMetadata ensureMetadata() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get userId => $_getSZ(1);
  @$pb.TagNumber(2)
  set userId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUserId() => $_has(1);
  @$pb.TagNumber(2)
  void clearUserId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get permission => $_getSZ(2);
  @$pb.TagNumber(3)
  set permission($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPermission() => $_has(2);
  @$pb.TagNumber(3)
  void clearPermission() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get resource => $_getSZ(3);
  @$pb.TagNumber(4)
  set resource($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasResource() => $_has(3);
  @$pb.TagNumber(4)
  void clearResource() => $_clearField(4);

  @$pb.TagNumber(5)
  $pb.PbList<$core.String> get roles => $_getList(4);

  @$pb.TagNumber(6)
  $core.bool get allowed => $_getBF(5);
  @$pb.TagNumber(6)
  set allowed($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAllowed() => $_has(5);
  @$pb.TagNumber(6)
  void clearAllowed() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get reason => $_getSZ(6);
  @$pb.TagNumber(7)
  set reason($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasReason() => $_has(6);
  @$pb.TagNumber(7)
  void clearReason() => $_clearField(7);
}

/// AuditLogRecordedEvent は監査ログ記録イベント。
/// Kafka トピック: k1s0.system.auth.audit.v1
/// パーティションキー: user_id
class AuditLogRecordedEvent extends $pb.GeneratedMessage {
  factory AuditLogRecordedEvent({
    $0.EventMetadata? metadata,
    $core.String? auditLogId,
    $core.String? eventType,
    $core.String? userId,
    $core.String? ipAddress,
    $core.String? resource,
    $core.String? action,
    $core.String? result,
  }) {
    final result$ = create();
    if (metadata != null) result$.metadata = metadata;
    if (auditLogId != null) result$.auditLogId = auditLogId;
    if (eventType != null) result$.eventType = eventType;
    if (userId != null) result$.userId = userId;
    if (ipAddress != null) result$.ipAddress = ipAddress;
    if (resource != null) result$.resource = resource;
    if (action != null) result$.action = action;
    if (result != null) result$.result = result;
    return result$;
  }

  AuditLogRecordedEvent._();

  factory AuditLogRecordedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AuditLogRecordedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AuditLogRecordedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.system.auth.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'auditLogId')
    ..aOS(3, _omitFieldNames ? '' : 'eventType')
    ..aOS(4, _omitFieldNames ? '' : 'userId')
    ..aOS(5, _omitFieldNames ? '' : 'ipAddress')
    ..aOS(6, _omitFieldNames ? '' : 'resource')
    ..aOS(7, _omitFieldNames ? '' : 'action')
    ..aOS(8, _omitFieldNames ? '' : 'result')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuditLogRecordedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuditLogRecordedEvent copyWith(
          void Function(AuditLogRecordedEvent) updates) =>
      super.copyWith((message) => updates(message as AuditLogRecordedEvent))
          as AuditLogRecordedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AuditLogRecordedEvent create() => AuditLogRecordedEvent._();
  @$core.override
  AuditLogRecordedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AuditLogRecordedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AuditLogRecordedEvent>(create);
  static AuditLogRecordedEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $0.EventMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata($0.EventMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EventMetadata ensureMetadata() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get auditLogId => $_getSZ(1);
  @$pb.TagNumber(2)
  set auditLogId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAuditLogId() => $_has(1);
  @$pb.TagNumber(2)
  void clearAuditLogId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get eventType => $_getSZ(2);
  @$pb.TagNumber(3)
  set eventType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEventType() => $_has(2);
  @$pb.TagNumber(3)
  void clearEventType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get userId => $_getSZ(3);
  @$pb.TagNumber(4)
  set userId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasUserId() => $_has(3);
  @$pb.TagNumber(4)
  void clearUserId() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get ipAddress => $_getSZ(4);
  @$pb.TagNumber(5)
  set ipAddress($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIpAddress() => $_has(4);
  @$pb.TagNumber(5)
  void clearIpAddress() => $_clearField(5);

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

  @$pb.TagNumber(8)
  $core.String get result => $_getSZ(7);
  @$pb.TagNumber(8)
  set result($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasResult() => $_has(7);
  @$pb.TagNumber(8)
  void clearResult() => $_clearField(8);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
