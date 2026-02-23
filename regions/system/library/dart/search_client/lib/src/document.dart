class IndexDocument {
  final String id;
  final Map<String, dynamic> fields;

  const IndexDocument({required this.id, required this.fields});
}

class IndexResult {
  final String id;
  final int version;

  const IndexResult({required this.id, required this.version});
}

class BulkFailure {
  final String id;
  final String error;

  const BulkFailure({required this.id, required this.error});
}

class BulkResult {
  final int successCount;
  final int failedCount;
  final List<BulkFailure> failures;

  const BulkResult({
    required this.successCount,
    required this.failedCount,
    required this.failures,
  });
}

class FieldMapping {
  final String fieldType;
  final bool indexed;

  const FieldMapping({required this.fieldType, this.indexed = true});
}

class IndexMapping {
  final Map<String, FieldMapping> fields;

  IndexMapping({Map<String, FieldMapping>? fields})
      : fields = fields ?? {};

  IndexMapping withField(String name, String fieldType) {
    final newFields = Map<String, FieldMapping>.from(fields);
    newFields[name] = FieldMapping(fieldType: fieldType);
    return IndexMapping(fields: newFields);
  }
}
