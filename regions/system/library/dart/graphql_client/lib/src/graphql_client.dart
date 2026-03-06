import 'dart:convert';

import 'package:http/http.dart' as http;

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

class GraphQlHttpClient implements GraphQlClient {
  final String _endpoint;
  final Map<String, String> _headers;
  final http.Client _httpClient;

  GraphQlHttpClient(
    this._endpoint, {
    Map<String, String>? headers,
    http.Client? httpClient,
  })  : _headers = {
          'Content-Type': 'application/json',
          ...?headers,
        },
        _httpClient = httpClient ?? http.Client();

  @override
  Future<GraphQlResponse<T>> execute<T>(
    GraphQlQuery query,
    T Function(Map<String, dynamic>) fromJson,
  ) {
    return _send(query, fromJson);
  }

  @override
  Future<GraphQlResponse<T>> executeMutation<T>(
    GraphQlQuery mutation,
    T Function(Map<String, dynamic>) fromJson,
  ) {
    return _send(mutation, fromJson);
  }

  @override
  Stream<GraphQlResponse<T>> subscribe<T>(
    GraphQlQuery subscription,
    T Function(Map<String, dynamic>) fromJson,
  ) {
    throw ClientError.request(
      'Subscriptions are not supported over HTTP',
    );
  }

  Future<GraphQlResponse<T>> _send<T>(
    GraphQlQuery query,
    T Function(Map<String, dynamic>) fromJson,
  ) async {
    final body = <String, dynamic>{
      'query': query.query,
    };
    if (query.variables != null) {
      body['variables'] = query.variables;
    }
    if (query.operationName != null) {
      body['operationName'] = query.operationName;
    }

    final http.Response response;
    try {
      response = await _httpClient.post(
        Uri.parse(_endpoint),
        headers: _headers,
        body: jsonEncode(body),
      );
    } on Exception catch (e) {
      throw ClientError.request('HTTP request failed: $e');
    }

    if (response.statusCode == 404) {
      throw ClientError.notFound('Resource not found: ${response.statusCode}');
    }

    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw ClientError.request(
        'HTTP ${response.statusCode}: ${response.body}',
      );
    }

    final Map<String, dynamic> json;
    try {
      json = jsonDecode(response.body) as Map<String, dynamic>;
    } on FormatException catch (e) {
      throw ClientError.deserialization(
        'Failed to parse response JSON: $e',
      );
    }

    final errorsRaw = json['errors'] as List<dynamic>?;
    if (errorsRaw != null && errorsRaw.isNotEmpty) {
      final errors = errorsRaw.map((e) {
        final map = e as Map<String, dynamic>;
        final locationsRaw = map['locations'] as List<dynamic>?;
        final locations = locationsRaw?.map((l) {
          final loc = l as Map<String, dynamic>;
          return ErrorLocation(loc['line'] as int, loc['column'] as int);
        }).toList();
        final path = map['path'] as List<dynamic>?;
        return GraphQlError(
          message: map['message'] as String,
          locations: locations,
          path: path,
        );
      }).toList();

      final dataRaw = json['data'] as Map<String, dynamic>?;
      final data = dataRaw != null ? fromJson(dataRaw) : null;
      return GraphQlResponse<T>(data: data, errors: errors);
    }

    final dataRaw = json['data'] as Map<String, dynamic>?;
    if (dataRaw == null) {
      throw ClientError.deserialization('Response missing "data" field');
    }

    return GraphQlResponse<T>(data: fromJson(dataRaw));
  }
}
