import 'graphql_query.dart';

abstract class GraphQlClient {
  Future<GraphQlResponse<T>> execute<T>(
    GraphQlQuery query,
    T Function(Map<String, dynamic>) fromJson,
  );

  Future<GraphQlResponse<T>> executeMutation<T>(
    GraphQlQuery mutation,
    T Function(Map<String, dynamic>) fromJson,
  );
}

class InMemoryGraphQlClient implements GraphQlClient {
  final _responses = <String, Map<String, dynamic>>{};

  void setResponse(String operationName, Map<String, dynamic> response) =>
      _responses[operationName] = response;

  @override
  Future<GraphQlResponse<T>> execute<T>(
    GraphQlQuery query,
    T Function(Map<String, dynamic>) fromJson,
  ) async {
    return _resolve(query, fromJson);
  }

  @override
  Future<GraphQlResponse<T>> executeMutation<T>(
    GraphQlQuery mutation,
    T Function(Map<String, dynamic>) fromJson,
  ) async {
    return _resolve(mutation, fromJson);
  }

  GraphQlResponse<T> _resolve<T>(
    GraphQlQuery query,
    T Function(Map<String, dynamic>) fromJson,
  ) {
    final key = query.operationName ?? query.query;
    final response = _responses[key];
    if (response == null) {
      return GraphQlResponse<T>(
        errors: [GraphQlError(message: 'No response configured for: $key')],
      );
    }

    final errorsRaw = response['errors'] as List<dynamic>?;
    final errors = errorsRaw
        ?.map((e) => GraphQlError(message: e['message'] as String))
        .toList();

    final dataRaw = response['data'] as Map<String, dynamic>?;
    final data = dataRaw != null ? fromJson(dataRaw) : null;

    return GraphQlResponse<T>(data: data, errors: errors);
  }
}
