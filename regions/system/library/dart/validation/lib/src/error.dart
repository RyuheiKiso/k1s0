class ValidationError implements Exception {
  final String field;
  final String message;
  final String code;

  const ValidationError(this.field, this.message, {String? code})
      : code = code ?? 'INVALID_${field.toUpperCase()}';

  @override
  String toString() => 'ValidationError($field, $code): $message';
}

class ValidationErrors {
  final List<ValidationError> _errors = [];

  bool hasErrors() => _errors.isNotEmpty;

  List<ValidationError> getErrors() => List.unmodifiable(_errors);

  void add(ValidationError error) => _errors.add(error);
}
