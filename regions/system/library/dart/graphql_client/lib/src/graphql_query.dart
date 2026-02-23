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

class GraphQlResponse<T> {
  final T? data;
  final List<GraphQlError>? errors;

  const GraphQlResponse({this.data, this.errors});

  bool get hasErrors => errors != null && errors!.isNotEmpty;
}
