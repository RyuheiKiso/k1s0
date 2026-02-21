class NotFoundError implements Exception {
  final String resource;

  const NotFoundError(this.resource);

  @override
  String toString() => 'NotFoundError: $resource not found';
}

bool isNotFound(Object? err) => err is NotFoundError;

class SchemaRegistryError implements Exception {
  final int statusCode;
  final String message;

  const SchemaRegistryError(this.statusCode, this.message);

  @override
  String toString() => 'SchemaRegistryError($statusCode): $message';
}
