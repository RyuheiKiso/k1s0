// This is a generated file - do not edit.
//
// Generated from k1s0/system/common/v1/event_types.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// EventType はシステム全体で使用されるイベント種別の列挙型。
/// 各ドメインのイベントを一元的に管理し、型安全なイベントルーティングを実現する。
class EventType extends $pb.ProtobufEnum {
  /// 未指定（デフォルト値）
  static const EventType EVENT_TYPE_UNSPECIFIED =
      EventType._(0, _omitEnumNames ? '' : 'EVENT_TYPE_UNSPECIFIED');

  /// === 認証系イベント (100-199) ===
  /// ログイン成功/失敗イベント
  static const EventType EVENT_TYPE_AUTH_LOGIN =
      EventType._(100, _omitEnumNames ? '' : 'EVENT_TYPE_AUTH_LOGIN');

  /// トークン検証結果イベント
  static const EventType EVENT_TYPE_AUTH_TOKEN_VALIDATION = EventType._(
      101, _omitEnumNames ? '' : 'EVENT_TYPE_AUTH_TOKEN_VALIDATION');

  /// パーミッション確認結果イベント
  static const EventType EVENT_TYPE_AUTH_PERMISSION_CHECK = EventType._(
      102, _omitEnumNames ? '' : 'EVENT_TYPE_AUTH_PERMISSION_CHECK');

  /// 監査ログ記録イベント
  static const EventType EVENT_TYPE_AUTH_AUDIT_LOG_RECORDED = EventType._(
      103, _omitEnumNames ? '' : 'EVENT_TYPE_AUTH_AUDIT_LOG_RECORDED');

  /// === 設定管理系イベント (200-299) ===
  /// 設定値変更イベント
  static const EventType EVENT_TYPE_CONFIG_CHANGED =
      EventType._(200, _omitEnumNames ? '' : 'EVENT_TYPE_CONFIG_CHANGED');

  /// === タスク管理系イベント (300-399) ===
  /// プロジェクトタイプ変更イベント
  static const EventType EVENT_TYPE_TASKMANAGEMENT_PROJECT_TYPE_CHANGED = EventType._(
      300, _omitEnumNames ? '' : 'EVENT_TYPE_TASKMANAGEMENT_PROJECT_TYPE_CHANGED');

  /// ステータス定義変更イベント
  static const EventType EVENT_TYPE_TASKMANAGEMENT_STATUS_DEFINITION_CHANGED = EventType._(
      301, _omitEnumNames ? '' : 'EVENT_TYPE_TASKMANAGEMENT_STATUS_DEFINITION_CHANGED');

  /// === タスク系イベント (400-499) ===
  /// タスク作成イベント
  static const EventType EVENT_TYPE_TASK_CREATED =
      EventType._(400, _omitEnumNames ? '' : 'EVENT_TYPE_TASK_CREATED');

  /// タスクステータス変更イベント
  static const EventType EVENT_TYPE_TASK_STATUS_CHANGED =
      EventType._(401, _omitEnumNames ? '' : 'EVENT_TYPE_TASK_STATUS_CHANGED');

  /// タスクキャンセルイベント
  static const EventType EVENT_TYPE_TASK_CANCELLED =
      EventType._(402, _omitEnumNames ? '' : 'EVENT_TYPE_TASK_CANCELLED');

  /// === ボード系イベント (500-599) ===
  /// ボードカラムインクリメントイベント
  static const EventType EVENT_TYPE_BOARD_COLUMN_INCREMENTED =
      EventType._(500, _omitEnumNames ? '' : 'EVENT_TYPE_BOARD_COLUMN_INCREMENTED');

  /// ボードカラムデクリメントイベント
  static const EventType EVENT_TYPE_BOARD_COLUMN_DECREMENTED =
      EventType._(501, _omitEnumNames ? '' : 'EVENT_TYPE_BOARD_COLUMN_DECREMENTED');

  /// === アクティビティ系イベント (600-699) ===
  /// アクティビティ作成イベント
  static const EventType EVENT_TYPE_ACTIVITY_CREATED =
      EventType._(600, _omitEnumNames ? '' : 'EVENT_TYPE_ACTIVITY_CREATED');

  /// アクティビティ承認イベント
  static const EventType EVENT_TYPE_ACTIVITY_APPROVED =
      EventType._(601, _omitEnumNames ? '' : 'EVENT_TYPE_ACTIVITY_APPROVED');

  /// アクティビティ却下イベント
  static const EventType EVENT_TYPE_ACTIVITY_REJECTED =
      EventType._(602, _omitEnumNames ? '' : 'EVENT_TYPE_ACTIVITY_REJECTED');

  /// アクティビティ削除イベント
  static const EventType EVENT_TYPE_ACTIVITY_DELETED =
      EventType._(603, _omitEnumNames ? '' : 'EVENT_TYPE_ACTIVITY_DELETED');

  static const $core.List<EventType> values = <EventType>[
    EVENT_TYPE_UNSPECIFIED,
    EVENT_TYPE_AUTH_LOGIN,
    EVENT_TYPE_AUTH_TOKEN_VALIDATION,
    EVENT_TYPE_AUTH_PERMISSION_CHECK,
    EVENT_TYPE_AUTH_AUDIT_LOG_RECORDED,
    EVENT_TYPE_CONFIG_CHANGED,
    EVENT_TYPE_TASKMANAGEMENT_PROJECT_TYPE_CHANGED,
    EVENT_TYPE_TASKMANAGEMENT_STATUS_DEFINITION_CHANGED,
    EVENT_TYPE_TASK_CREATED,
    EVENT_TYPE_TASK_STATUS_CHANGED,
    EVENT_TYPE_TASK_CANCELLED,
    EVENT_TYPE_BOARD_COLUMN_INCREMENTED,
    EVENT_TYPE_BOARD_COLUMN_DECREMENTED,
    EVENT_TYPE_ACTIVITY_CREATED,
    EVENT_TYPE_ACTIVITY_APPROVED,
    EVENT_TYPE_ACTIVITY_REJECTED,
    EVENT_TYPE_ACTIVITY_DELETED,
  ];

  static final $core.Map<$core.int, EventType> _byValue =
      $pb.ProtobufEnum.initByValue(values);
  static EventType? valueOf($core.int value) => _byValue[value];

  const EventType._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
