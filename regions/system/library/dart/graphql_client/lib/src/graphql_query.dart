class GraphQlQuery {
  final String query;
  final Map<String, dynamic>? variables;
  final String? operationName;

  const GraphQlQuery({
    required this.query,
    this.variables,
    this.operationName,
  });
}

class GraphQlError {
  final String message;
  final List<ErrorLocation>? locations;
  final List<dynamic>? path;

  const GraphQlError({required this.message, this.locations, this.path});
}

class ErrorLocation {
  final int line;
  final int column;

  const ErrorLocation(this.line, this.column);
}

enum ClientErrorKind { request, deserialization, graphQl, notFound }

class ClientError implements Exception {
  final ClientErrorKind kind;
  final String message;

  ClientError.request(this.message) : kind = ClientErrorKind.request;
  ClientError.deserialization(this.message)
      : kind = ClientErrorKind.deserialization;
  ClientError.graphQl(this.message) : kind = ClientErrorKind.graphQl;
  ClientError.notFound(this.message) : kind = ClientErrorKind.notFound;

  @override
  String toString() => '${_kindPrefix(kind)}: $message';

  static String _kindPrefix(ClientErrorKind kind) {
    switch (kind) {
      case ClientErrorKind.request:
        return 'RequestError';
      case ClientErrorKind.deserialization:
        return 'DeserializationError';
      case ClientErrorKind.graphQl:
        return 'GraphQlError';
      case ClientErrorKind.notFound:
        return 'NotFoundError';
    }
  }
}

class GraphQlResponse<T> {
  final T? data;
  final List<GraphQlError>? errors;

  const GraphQlResponse({this.data, this.errors});

  bool get hasErrors => errors != null && errors!.isNotEmpty;
}
