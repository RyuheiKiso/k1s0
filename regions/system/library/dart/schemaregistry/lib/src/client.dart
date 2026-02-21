import 'dart:convert';

import 'package:http/http.dart' as http;

import 'types.dart';
import 'config.dart';
import 'error.dart';

abstract class SchemaRegistryClient {
  Future<int> registerSchema(
      String subject, String schema, String schemaType);
  Future<RegisteredSchema> getSchemaById(int id);
  Future<RegisteredSchema> getLatestSchema(String subject);
  Future<RegisteredSchema> getSchemaVersion(String subject, int version);
  Future<List<String>> listSubjects();
  Future<bool> checkCompatibility(String subject, String schema);
  Future<void> healthCheck();
}

class HttpSchemaRegistryClient implements SchemaRegistryClient {
  final SchemaRegistryConfig config;
  final http.Client _httpClient;

  HttpSchemaRegistryClient(this.config, {http.Client? httpClient})
      : _httpClient = httpClient ?? http.Client() {
    config.validate();
  }

  Map<String, String> get _headers => {
        'Content-Type': 'application/vnd.schemaregistry.v1+json',
        if (config.username != null)
          'Authorization':
              'Basic ${base64Encode(utf8.encode('${config.username}:${config.password}'))}',
      };

  Future<http.Response> _doRequest(
      String method, String path, Object? body) async {
    final uri = Uri.parse('${config.url}$path');
    final http.Response response;

    switch (method) {
      case 'GET':
        response = await _httpClient.get(uri, headers: _headers);
      case 'POST':
        response = await _httpClient.post(
          uri,
          headers: _headers,
          body: body != null ? jsonEncode(body) : null,
        );
      default:
        throw SchemaRegistryError(0, 'unsupported method: $method');
    }

    return response;
  }

  @override
  Future<int> registerSchema(
      String subject, String schema, String schemaType) async {
    final body = {'schema': schema, 'schemaType': schemaType};
    final response =
        await _doRequest('POST', '/subjects/$subject/versions', body);

    if (response.statusCode == 404) {
      throw NotFoundError(subject);
    }
    if (response.statusCode != 200) {
      throw SchemaRegistryError(response.statusCode, response.body);
    }

    final result = jsonDecode(response.body) as Map<String, dynamic>;
    return result['id'] as int;
  }

  @override
  Future<RegisteredSchema> getSchemaById(int id) async {
    final response = await _doRequest('GET', '/schemas/ids/$id', null);

    if (response.statusCode == 404) {
      throw NotFoundError('schema id=$id');
    }
    if (response.statusCode != 200) {
      throw SchemaRegistryError(response.statusCode, response.body);
    }

    final json = jsonDecode(response.body) as Map<String, dynamic>;
    json['id'] = id;
    return RegisteredSchema.fromJson(json);
  }

  @override
  Future<RegisteredSchema> getLatestSchema(String subject) async {
    return _getSchemaVersion(subject, 'latest');
  }

  @override
  Future<RegisteredSchema> getSchemaVersion(
      String subject, int version) async {
    return _getSchemaVersion(subject, '$version');
  }

  Future<RegisteredSchema> _getSchemaVersion(
      String subject, String version) async {
    final response =
        await _doRequest('GET', '/subjects/$subject/versions/$version', null);

    if (response.statusCode == 404) {
      throw NotFoundError('$subject/versions/$version');
    }
    if (response.statusCode != 200) {
      throw SchemaRegistryError(response.statusCode, response.body);
    }

    final json = jsonDecode(response.body) as Map<String, dynamic>;
    return RegisteredSchema.fromJson(json);
  }

  @override
  Future<List<String>> listSubjects() async {
    final response = await _doRequest('GET', '/subjects', null);

    if (response.statusCode != 200) {
      throw SchemaRegistryError(response.statusCode, response.body);
    }

    final list = jsonDecode(response.body) as List<dynamic>;
    return list.cast<String>();
  }

  @override
  Future<bool> checkCompatibility(String subject, String schema) async {
    final body = {'schema': schema};
    final response = await _doRequest(
        'POST', '/compatibility/subjects/$subject/versions/latest', body);

    if (response.statusCode == 404) {
      throw NotFoundError(subject);
    }
    if (response.statusCode != 200) {
      throw SchemaRegistryError(response.statusCode, response.body);
    }

    final result = jsonDecode(response.body) as Map<String, dynamic>;
    return result['is_compatible'] as bool;
  }

  @override
  Future<void> healthCheck() async {
    final response = await _doRequest('GET', '/', null);

    if (response.statusCode != 200) {
      throw SchemaRegistryError(
          response.statusCode, 'health check failed');
    }
  }
}
