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

  /// === 会計系イベント (300-399) ===
  /// 仕訳エントリ作成イベント
  static const EventType EVENT_TYPE_ACCOUNTING_ENTRY_CREATED = EventType._(
      300, _omitEnumNames ? '' : 'EVENT_TYPE_ACCOUNTING_ENTRY_CREATED');

  /// 仕訳エントリ承認イベント
  static const EventType EVENT_TYPE_ACCOUNTING_ENTRY_APPROVED = EventType._(
      301, _omitEnumNames ? '' : 'EVENT_TYPE_ACCOUNTING_ENTRY_APPROVED');

  /// === 注文系イベント (400-499) ===
  /// 注文作成イベント
  static const EventType EVENT_TYPE_ORDER_CREATED =
      EventType._(400, _omitEnumNames ? '' : 'EVENT_TYPE_ORDER_CREATED');

  /// 注文更新イベント
  static const EventType EVENT_TYPE_ORDER_UPDATED =
      EventType._(401, _omitEnumNames ? '' : 'EVENT_TYPE_ORDER_UPDATED');

  /// 注文キャンセルイベント
  static const EventType EVENT_TYPE_ORDER_CANCELLED =
      EventType._(402, _omitEnumNames ? '' : 'EVENT_TYPE_ORDER_CANCELLED');

  /// === 在庫系イベント (500-599) ===
  /// 在庫予約イベント
  static const EventType EVENT_TYPE_INVENTORY_RESERVED =
      EventType._(500, _omitEnumNames ? '' : 'EVENT_TYPE_INVENTORY_RESERVED');

  /// 在庫解放イベント
  static const EventType EVENT_TYPE_INVENTORY_RELEASED =
      EventType._(501, _omitEnumNames ? '' : 'EVENT_TYPE_INVENTORY_RELEASED');

  /// === 決済系イベント (600-699) ===
  /// 決済開始イベント
  static const EventType EVENT_TYPE_PAYMENT_INITIATED =
      EventType._(600, _omitEnumNames ? '' : 'EVENT_TYPE_PAYMENT_INITIATED');

  /// 決済完了イベント
  static const EventType EVENT_TYPE_PAYMENT_COMPLETED =
      EventType._(601, _omitEnumNames ? '' : 'EVENT_TYPE_PAYMENT_COMPLETED');

  /// 決済失敗イベント
  static const EventType EVENT_TYPE_PAYMENT_FAILED =
      EventType._(602, _omitEnumNames ? '' : 'EVENT_TYPE_PAYMENT_FAILED');

  /// 返金イベント
  static const EventType EVENT_TYPE_PAYMENT_REFUNDED =
      EventType._(603, _omitEnumNames ? '' : 'EVENT_TYPE_PAYMENT_REFUNDED');

  static const $core.List<EventType> values = <EventType>[
    EVENT_TYPE_UNSPECIFIED,
    EVENT_TYPE_AUTH_LOGIN,
    EVENT_TYPE_AUTH_TOKEN_VALIDATION,
    EVENT_TYPE_AUTH_PERMISSION_CHECK,
    EVENT_TYPE_AUTH_AUDIT_LOG_RECORDED,
    EVENT_TYPE_CONFIG_CHANGED,
    EVENT_TYPE_ACCOUNTING_ENTRY_CREATED,
    EVENT_TYPE_ACCOUNTING_ENTRY_APPROVED,
    EVENT_TYPE_ORDER_CREATED,
    EVENT_TYPE_ORDER_UPDATED,
    EVENT_TYPE_ORDER_CANCELLED,
    EVENT_TYPE_INVENTORY_RESERVED,
    EVENT_TYPE_INVENTORY_RELEASED,
    EVENT_TYPE_PAYMENT_INITIATED,
    EVENT_TYPE_PAYMENT_COMPLETED,
    EVENT_TYPE_PAYMENT_FAILED,
    EVENT_TYPE_PAYMENT_REFUNDED,
  ];

  static final $core.Map<$core.int, EventType> _byValue =
      $pb.ProtobufEnum.initByValue(values);
  static EventType? valueOf($core.int value) => _byValue[value];

  const EventType._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
