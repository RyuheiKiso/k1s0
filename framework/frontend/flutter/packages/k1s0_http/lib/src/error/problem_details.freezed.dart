// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'problem_details.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ProblemDetails _$ProblemDetailsFromJson(Map<String, dynamic> json) {
  return _ProblemDetails.fromJson(json);
}

/// @nodoc
mixin _$ProblemDetails {
  /// A URI reference that identifies the problem type
  String get type => throw _privateConstructorUsedError;

  /// A short, human-readable summary of the problem type
  String get title => throw _privateConstructorUsedError;

  /// The HTTP status code
  int get status => throw _privateConstructorUsedError;

  /// A human-readable explanation specific to this occurrence
  String? get detail => throw _privateConstructorUsedError;

  /// A URI reference that identifies the specific occurrence
  String? get instance => throw _privateConstructorUsedError;

  /// k1s0 standard: Error code for operational identification
  @JsonKey(name: 'error_code')
  String get errorCode => throw _privateConstructorUsedError;

  /// k1s0 standard: Trace ID for log/trace investigation
  @JsonKey(name: 'trace_id')
  String? get traceId => throw _privateConstructorUsedError;

  /// k1s0 standard: Field-level validation errors
  List<FieldError>? get errors => throw _privateConstructorUsedError;

  /// Serializes this ProblemDetails to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ProblemDetails
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ProblemDetailsCopyWith<ProblemDetails> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ProblemDetailsCopyWith<$Res> {
  factory $ProblemDetailsCopyWith(
          ProblemDetails value, $Res Function(ProblemDetails) then) =
      _$ProblemDetailsCopyWithImpl<$Res, ProblemDetails>;
  @useResult
  $Res call(
      {String type,
      String title,
      int status,
      String? detail,
      String? instance,
      @JsonKey(name: 'error_code') String errorCode,
      @JsonKey(name: 'trace_id') String? traceId,
      List<FieldError>? errors});
}

/// @nodoc
class _$ProblemDetailsCopyWithImpl<$Res, $Val extends ProblemDetails>
    implements $ProblemDetailsCopyWith<$Res> {
  _$ProblemDetailsCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ProblemDetails
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? type = null,
    Object? title = null,
    Object? status = null,
    Object? detail = freezed,
    Object? instance = freezed,
    Object? errorCode = null,
    Object? traceId = freezed,
    Object? errors = freezed,
  }) {
    return _then(_value.copyWith(
      type: null == type
          ? _value.type
          : type // ignore: cast_nullable_to_non_nullable
              as String,
      title: null == title
          ? _value.title
          : title // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as int,
      detail: freezed == detail
          ? _value.detail
          : detail // ignore: cast_nullable_to_non_nullable
              as String?,
      instance: freezed == instance
          ? _value.instance
          : instance // ignore: cast_nullable_to_non_nullable
              as String?,
      errorCode: null == errorCode
          ? _value.errorCode
          : errorCode // ignore: cast_nullable_to_non_nullable
              as String,
      traceId: freezed == traceId
          ? _value.traceId
          : traceId // ignore: cast_nullable_to_non_nullable
              as String?,
      errors: freezed == errors
          ? _value.errors
          : errors // ignore: cast_nullable_to_non_nullable
              as List<FieldError>?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ProblemDetailsImplCopyWith<$Res>
    implements $ProblemDetailsCopyWith<$Res> {
  factory _$$ProblemDetailsImplCopyWith(_$ProblemDetailsImpl value,
          $Res Function(_$ProblemDetailsImpl) then) =
      __$$ProblemDetailsImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String type,
      String title,
      int status,
      String? detail,
      String? instance,
      @JsonKey(name: 'error_code') String errorCode,
      @JsonKey(name: 'trace_id') String? traceId,
      List<FieldError>? errors});
}

/// @nodoc
class __$$ProblemDetailsImplCopyWithImpl<$Res>
    extends _$ProblemDetailsCopyWithImpl<$Res, _$ProblemDetailsImpl>
    implements _$$ProblemDetailsImplCopyWith<$Res> {
  __$$ProblemDetailsImplCopyWithImpl(
      _$ProblemDetailsImpl _value, $Res Function(_$ProblemDetailsImpl) _then)
      : super(_value, _then);

  /// Create a copy of ProblemDetails
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? type = null,
    Object? title = null,
    Object? status = null,
    Object? detail = freezed,
    Object? instance = freezed,
    Object? errorCode = null,
    Object? traceId = freezed,
    Object? errors = freezed,
  }) {
    return _then(_$ProblemDetailsImpl(
      type: null == type
          ? _value.type
          : type // ignore: cast_nullable_to_non_nullable
              as String,
      title: null == title
          ? _value.title
          : title // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as int,
      detail: freezed == detail
          ? _value.detail
          : detail // ignore: cast_nullable_to_non_nullable
              as String?,
      instance: freezed == instance
          ? _value.instance
          : instance // ignore: cast_nullable_to_non_nullable
              as String?,
      errorCode: null == errorCode
          ? _value.errorCode
          : errorCode // ignore: cast_nullable_to_non_nullable
              as String,
      traceId: freezed == traceId
          ? _value.traceId
          : traceId // ignore: cast_nullable_to_non_nullable
              as String?,
      errors: freezed == errors
          ? _value._errors
          : errors // ignore: cast_nullable_to_non_nullable
              as List<FieldError>?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ProblemDetailsImpl implements _ProblemDetails {
  const _$ProblemDetailsImpl(
      {this.type = 'about:blank',
      required this.title,
      required this.status,
      this.detail,
      this.instance,
      @JsonKey(name: 'error_code') required this.errorCode,
      @JsonKey(name: 'trace_id') this.traceId,
      final List<FieldError>? errors})
      : _errors = errors;

  factory _$ProblemDetailsImpl.fromJson(Map<String, dynamic> json) =>
      _$$ProblemDetailsImplFromJson(json);

  /// A URI reference that identifies the problem type
  @override
  @JsonKey()
  final String type;

  /// A short, human-readable summary of the problem type
  @override
  final String title;

  /// The HTTP status code
  @override
  final int status;

  /// A human-readable explanation specific to this occurrence
  @override
  final String? detail;

  /// A URI reference that identifies the specific occurrence
  @override
  final String? instance;

  /// k1s0 standard: Error code for operational identification
  @override
  @JsonKey(name: 'error_code')
  final String errorCode;

  /// k1s0 standard: Trace ID for log/trace investigation
  @override
  @JsonKey(name: 'trace_id')
  final String? traceId;

  /// k1s0 standard: Field-level validation errors
  final List<FieldError>? _errors;

  /// k1s0 standard: Field-level validation errors
  @override
  List<FieldError>? get errors {
    final value = _errors;
    if (value == null) return null;
    if (_errors is EqualUnmodifiableListView) return _errors;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  @override
  String toString() {
    return 'ProblemDetails(type: $type, title: $title, status: $status, detail: $detail, instance: $instance, errorCode: $errorCode, traceId: $traceId, errors: $errors)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ProblemDetailsImpl &&
            (identical(other.type, type) || other.type == type) &&
            (identical(other.title, title) || other.title == title) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.detail, detail) || other.detail == detail) &&
            (identical(other.instance, instance) ||
                other.instance == instance) &&
            (identical(other.errorCode, errorCode) ||
                other.errorCode == errorCode) &&
            (identical(other.traceId, traceId) || other.traceId == traceId) &&
            const DeepCollectionEquality().equals(other._errors, _errors));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      type,
      title,
      status,
      detail,
      instance,
      errorCode,
      traceId,
      const DeepCollectionEquality().hash(_errors));

  /// Create a copy of ProblemDetails
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ProblemDetailsImplCopyWith<_$ProblemDetailsImpl> get copyWith =>
      __$$ProblemDetailsImplCopyWithImpl<_$ProblemDetailsImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ProblemDetailsImplToJson(
      this,
    );
  }
}

abstract class _ProblemDetails implements ProblemDetails {
  const factory _ProblemDetails(
      {final String type,
      required final String title,
      required final int status,
      final String? detail,
      final String? instance,
      @JsonKey(name: 'error_code') required final String errorCode,
      @JsonKey(name: 'trace_id') final String? traceId,
      final List<FieldError>? errors}) = _$ProblemDetailsImpl;

  factory _ProblemDetails.fromJson(Map<String, dynamic> json) =
      _$ProblemDetailsImpl.fromJson;

  /// A URI reference that identifies the problem type
  @override
  String get type;

  /// A short, human-readable summary of the problem type
  @override
  String get title;

  /// The HTTP status code
  @override
  int get status;

  /// A human-readable explanation specific to this occurrence
  @override
  String? get detail;

  /// A URI reference that identifies the specific occurrence
  @override
  String? get instance;

  /// k1s0 standard: Error code for operational identification
  @override
  @JsonKey(name: 'error_code')
  String get errorCode;

  /// k1s0 standard: Trace ID for log/trace investigation
  @override
  @JsonKey(name: 'trace_id')
  String? get traceId;

  /// k1s0 standard: Field-level validation errors
  @override
  List<FieldError>? get errors;

  /// Create a copy of ProblemDetails
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ProblemDetailsImplCopyWith<_$ProblemDetailsImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

FieldError _$FieldErrorFromJson(Map<String, dynamic> json) {
  return _FieldError.fromJson(json);
}

/// @nodoc
mixin _$FieldError {
  /// The field that has the error
  String get field => throw _privateConstructorUsedError;

  /// Human-readable error message
  String get message => throw _privateConstructorUsedError;

  /// Error code for this field
  String? get code => throw _privateConstructorUsedError;

  /// Serializes this FieldError to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of FieldError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FieldErrorCopyWith<FieldError> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FieldErrorCopyWith<$Res> {
  factory $FieldErrorCopyWith(
          FieldError value, $Res Function(FieldError) then) =
      _$FieldErrorCopyWithImpl<$Res, FieldError>;
  @useResult
  $Res call({String field, String message, String? code});
}

/// @nodoc
class _$FieldErrorCopyWithImpl<$Res, $Val extends FieldError>
    implements $FieldErrorCopyWith<$Res> {
  _$FieldErrorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FieldError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field = null,
    Object? message = null,
    Object? code = freezed,
  }) {
    return _then(_value.copyWith(
      field: null == field
          ? _value.field
          : field // ignore: cast_nullable_to_non_nullable
              as String,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      code: freezed == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$FieldErrorImplCopyWith<$Res>
    implements $FieldErrorCopyWith<$Res> {
  factory _$$FieldErrorImplCopyWith(
          _$FieldErrorImpl value, $Res Function(_$FieldErrorImpl) then) =
      __$$FieldErrorImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String field, String message, String? code});
}

/// @nodoc
class __$$FieldErrorImplCopyWithImpl<$Res>
    extends _$FieldErrorCopyWithImpl<$Res, _$FieldErrorImpl>
    implements _$$FieldErrorImplCopyWith<$Res> {
  __$$FieldErrorImplCopyWithImpl(
      _$FieldErrorImpl _value, $Res Function(_$FieldErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of FieldError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field = null,
    Object? message = null,
    Object? code = freezed,
  }) {
    return _then(_$FieldErrorImpl(
      field: null == field
          ? _value.field
          : field // ignore: cast_nullable_to_non_nullable
              as String,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      code: freezed == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$FieldErrorImpl implements _FieldError {
  const _$FieldErrorImpl(
      {required this.field, required this.message, this.code});

  factory _$FieldErrorImpl.fromJson(Map<String, dynamic> json) =>
      _$$FieldErrorImplFromJson(json);

  /// The field that has the error
  @override
  final String field;

  /// Human-readable error message
  @override
  final String message;

  /// Error code for this field
  @override
  final String? code;

  @override
  String toString() {
    return 'FieldError(field: $field, message: $message, code: $code)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FieldErrorImpl &&
            (identical(other.field, field) || other.field == field) &&
            (identical(other.message, message) || other.message == message) &&
            (identical(other.code, code) || other.code == code));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, field, message, code);

  /// Create a copy of FieldError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FieldErrorImplCopyWith<_$FieldErrorImpl> get copyWith =>
      __$$FieldErrorImplCopyWithImpl<_$FieldErrorImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$FieldErrorImplToJson(
      this,
    );
  }
}

abstract class _FieldError implements FieldError {
  const factory _FieldError(
      {required final String field,
      required final String message,
      final String? code}) = _$FieldErrorImpl;

  factory _FieldError.fromJson(Map<String, dynamic> json) =
      _$FieldErrorImpl.fromJson;

  /// The field that has the error
  @override
  String get field;

  /// Human-readable error message
  @override
  String get message;

  /// Error code for this field
  @override
  String? get code;

  /// Create a copy of FieldError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FieldErrorImplCopyWith<_$FieldErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
