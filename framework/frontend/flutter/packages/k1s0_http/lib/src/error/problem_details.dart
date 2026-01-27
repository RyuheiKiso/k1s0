import 'package:freezed_annotation/freezed_annotation.dart';

part 'problem_details.freezed.dart';
part 'problem_details.g.dart';

/// RFC 7807 Problem Details format for HTTP API errors
///
/// This is the standard error response format used by k1s0 backend services.
/// See: https://tools.ietf.org/html/rfc7807
@freezed
class ProblemDetails with _$ProblemDetails {
  /// Creates a ProblemDetails instance
  const factory ProblemDetails({
    /// A short, human-readable summary of the problem type
    required String title,

    /// The HTTP status code
    required int status,

    /// k1s0 standard: Error code for operational identification
    @JsonKey(name: 'error_code') required String errorCode,

    /// A URI reference that identifies the problem type
    @Default('about:blank') String type,

    /// A human-readable explanation specific to this occurrence
    String? detail,

    /// A URI reference that identifies the specific occurrence
    String? instance,

    /// k1s0 standard: Trace ID for log/trace investigation
    @JsonKey(name: 'trace_id') String? traceId,

    /// k1s0 standard: Field-level validation errors
    List<FieldError>? errors,
  }) = _ProblemDetails;

  /// Creates ProblemDetails from JSON
  factory ProblemDetails.fromJson(Map<String, dynamic> json) =>
      _$ProblemDetailsFromJson(json);
}

/// Field-level validation error
@freezed
class FieldError with _$FieldError {
  /// Creates a FieldError instance
  const factory FieldError({
    /// The field that has the error
    required String field,

    /// Human-readable error message
    required String message,

    /// Error code for this field
    String? code,
  }) = _FieldError;

  /// Creates FieldError from JSON
  factory FieldError.fromJson(Map<String, dynamic> json) =>
      _$FieldErrorFromJson(json);
}
