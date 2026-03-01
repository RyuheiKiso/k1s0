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

  Stream<GraphQlResponse<T>> subscribe<T>(
    GraphQlQuery subscription,
    T Function(Map<String, dynamic>) fromJson,
  );
}

class InMemoryGraphQlClient implements GraphQlClient {
  final Map<String, Map<String, dynamic>> _responses = {};
  final Map<String, List<Map<String, dynamic>>> _subscriptionEvents = {};

  void setResponse(String operationName, Map<String, dynamic> response) {
    _responses[operationName] = response;
  }

  void setSubscriptionEvents(
      String operationName, List<Map<String, dynamic>> events) {
    _subscriptionEvents[operationName] = events;
  }

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

  @override
  Stream<GraphQlResponse<T>> subscribe<T>(
    GraphQlQuery subscription,
    T Function(Map<String, dynamic>) fromJson,
  ) {
    final key = subscription.operationName ?? subscription.query;
    final events = _subscriptionEvents[key] ?? [];
    return Stream.fromIterable(
      events.map((e) => GraphQlResponse(data: fromJson(e))),
    );
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
