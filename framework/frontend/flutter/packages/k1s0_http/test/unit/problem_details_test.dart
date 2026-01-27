import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_http/src/error/problem_details.dart';

void main() {
  group('ProblemDetails', () {
    test('creates with required fields', () {
      const details = ProblemDetails(
        title: 'Bad Request',
        status: 400,
        errorCode: 'VALIDATION_ERROR',
      );

      expect(details.title, 'Bad Request');
      expect(details.status, 400);
      expect(details.errorCode, 'VALIDATION_ERROR');
      expect(details.type, 'about:blank');
      expect(details.detail, isNull);
      expect(details.instance, isNull);
      expect(details.traceId, isNull);
      expect(details.errors, isNull);
    });

    test('creates with all fields', () {
      const details = ProblemDetails(
        title: 'Validation Error',
        status: 400,
        errorCode: 'VALIDATION_ERROR',
        type: 'https://api.example.com/errors/validation',
        detail: 'The request body contains invalid data',
        instance: '/api/users/123',
        traceId: 'trace-123',
        errors: [
          FieldError(field: 'email', message: 'Invalid email format'),
        ],
      );

      expect(details.title, 'Validation Error');
      expect(details.status, 400);
      expect(details.errorCode, 'VALIDATION_ERROR');
      expect(details.type, 'https://api.example.com/errors/validation');
      expect(details.detail, 'The request body contains invalid data');
      expect(details.instance, '/api/users/123');
      expect(details.traceId, 'trace-123');
      expect(details.errors, isNotNull);
      expect(details.errors!.length, 1);
    });

    test('fromJson creates correct instance', () {
      final json = {
        'title': 'Not Found',
        'status': 404,
        'error_code': 'RESOURCE_NOT_FOUND',
        'type': 'https://api.example.com/errors/not-found',
        'detail': 'The requested resource was not found',
        'trace_id': 'trace-456',
      };

      final details = ProblemDetails.fromJson(json);

      expect(details.title, 'Not Found');
      expect(details.status, 404);
      expect(details.errorCode, 'RESOURCE_NOT_FOUND');
      expect(details.type, 'https://api.example.com/errors/not-found');
      expect(details.detail, 'The requested resource was not found');
      expect(details.traceId, 'trace-456');
    });

    test('toJson returns correct map', () {
      const details = ProblemDetails(
        title: 'Unauthorized',
        status: 401,
        errorCode: 'UNAUTHORIZED',
        traceId: 'trace-789',
      );

      final json = details.toJson();

      expect(json['title'], 'Unauthorized');
      expect(json['status'], 401);
      expect(json['error_code'], 'UNAUTHORIZED');
      expect(json['trace_id'], 'trace-789');
    });

    test('fromJson with field errors', () {
      final json = {
        'title': 'Validation Error',
        'status': 400,
        'error_code': 'VALIDATION_ERROR',
        'errors': [
          {'field': 'email', 'message': 'Invalid email', 'code': 'INVALID_EMAIL'},
          {'field': 'password', 'message': 'Too short', 'code': 'TOO_SHORT'},
        ],
      };

      final details = ProblemDetails.fromJson(json);

      expect(details.errors, isNotNull);
      expect(details.errors!.length, 2);
      expect(details.errors![0].field, 'email');
      expect(details.errors![0].message, 'Invalid email');
      expect(details.errors![0].code, 'INVALID_EMAIL');
      expect(details.errors![1].field, 'password');
    });
  });

  group('FieldError', () {
    test('creates with required fields', () {
      const error = FieldError(
        field: 'username',
        message: 'Username is required',
      );

      expect(error.field, 'username');
      expect(error.message, 'Username is required');
      expect(error.code, isNull);
    });

    test('creates with code', () {
      const error = FieldError(
        field: 'email',
        message: 'Invalid email format',
        code: 'INVALID_FORMAT',
      );

      expect(error.field, 'email');
      expect(error.message, 'Invalid email format');
      expect(error.code, 'INVALID_FORMAT');
    });

    test('fromJson creates correct instance', () {
      final json = {
        'field': 'phone',
        'message': 'Invalid phone number',
        'code': 'INVALID_PHONE',
      };

      final error = FieldError.fromJson(json);

      expect(error.field, 'phone');
      expect(error.message, 'Invalid phone number');
      expect(error.code, 'INVALID_PHONE');
    });

    test('toJson returns correct map', () {
      const error = FieldError(
        field: 'age',
        message: 'Must be positive',
        code: 'POSITIVE_NUMBER',
      );

      final json = error.toJson();

      expect(json['field'], 'age');
      expect(json['message'], 'Must be positive');
      expect(json['code'], 'POSITIVE_NUMBER');
    });
  });
}
