// This is a generated file - do not edit.
//
// Generated from k1s0/event/business/accounting/v1/accounting_events.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class EntryType extends $pb.ProtobufEnum {
  static const EntryType ENTRY_TYPE_UNSPECIFIED =
      EntryType._(0, _omitEnumNames ? '' : 'ENTRY_TYPE_UNSPECIFIED');
  static const EntryType ENTRY_TYPE_DEBIT =
      EntryType._(1, _omitEnumNames ? '' : 'ENTRY_TYPE_DEBIT');
  static const EntryType ENTRY_TYPE_CREDIT =
      EntryType._(2, _omitEnumNames ? '' : 'ENTRY_TYPE_CREDIT');

  static const $core.List<EntryType> values = <EntryType>[
    ENTRY_TYPE_UNSPECIFIED,
    ENTRY_TYPE_DEBIT,
    ENTRY_TYPE_CREDIT,
  ];

  static final $core.List<EntryType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 2);
  static EntryType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const EntryType._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
