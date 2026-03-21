// This is a generated file - do not edit.
//
// Generated from k1s0/system/ratelimit/v1/ratelimit.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// RateLimitAlgorithm はレートリミットのアルゴリズム種別。
class RateLimitAlgorithm extends $pb.ProtobufEnum {
  /// RATE_LIMIT_ALGORITHM_UNSPECIFIED は未指定（デフォルト値）。
  static const RateLimitAlgorithm RATE_LIMIT_ALGORITHM_UNSPECIFIED =
      RateLimitAlgorithm._(
          0, _omitEnumNames ? '' : 'RATE_LIMIT_ALGORITHM_UNSPECIFIED');

  /// RATE_LIMIT_ALGORITHM_SLIDING_WINDOW はスライディングウィンドウアルゴリズム。
  static const RateLimitAlgorithm RATE_LIMIT_ALGORITHM_SLIDING_WINDOW =
      RateLimitAlgorithm._(
          1, _omitEnumNames ? '' : 'RATE_LIMIT_ALGORITHM_SLIDING_WINDOW');

  /// RATE_LIMIT_ALGORITHM_TOKEN_BUCKET はトークンバケットアルゴリズム。
  static const RateLimitAlgorithm RATE_LIMIT_ALGORITHM_TOKEN_BUCKET =
      RateLimitAlgorithm._(
          2, _omitEnumNames ? '' : 'RATE_LIMIT_ALGORITHM_TOKEN_BUCKET');

  /// RATE_LIMIT_ALGORITHM_FIXED_WINDOW は固定ウィンドウアルゴリズム。
  static const RateLimitAlgorithm RATE_LIMIT_ALGORITHM_FIXED_WINDOW =
      RateLimitAlgorithm._(
          3, _omitEnumNames ? '' : 'RATE_LIMIT_ALGORITHM_FIXED_WINDOW');

  /// RATE_LIMIT_ALGORITHM_LEAKY_BUCKET はリーキーバケットアルゴリズム。
  static const RateLimitAlgorithm RATE_LIMIT_ALGORITHM_LEAKY_BUCKET =
      RateLimitAlgorithm._(
          4, _omitEnumNames ? '' : 'RATE_LIMIT_ALGORITHM_LEAKY_BUCKET');

  static const $core.List<RateLimitAlgorithm> values = <RateLimitAlgorithm>[
    RATE_LIMIT_ALGORITHM_UNSPECIFIED,
    RATE_LIMIT_ALGORITHM_SLIDING_WINDOW,
    RATE_LIMIT_ALGORITHM_TOKEN_BUCKET,
    RATE_LIMIT_ALGORITHM_FIXED_WINDOW,
    RATE_LIMIT_ALGORITHM_LEAKY_BUCKET,
  ];

  static final $core.List<RateLimitAlgorithm?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 4);
  static RateLimitAlgorithm? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const RateLimitAlgorithm._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
