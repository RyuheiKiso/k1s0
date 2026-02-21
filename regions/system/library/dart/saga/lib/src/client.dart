import 'dart:convert';

import 'package:http/http.dart' as http;

import 'error.dart';
import 'types.dart';

/// Saga サーバーへの REST クライアント。
class SagaClient {
  final String _endpoint;
  final http.Client _httpClient;

  SagaClient(String endpoint, {http.Client? httpClient})
      : _endpoint = endpoint.replaceAll(RegExp(r'/$'), ''),
        _httpClient = httpClient ?? http.Client();

  /// Saga を開始する。POST /api/v1/sagas
  Future<StartSagaResponse> startSaga(StartSagaRequest request) async {
    final response = await _httpClient.post(
      Uri.parse('$_endpoint/api/v1/sagas'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode(request.toJson()),
    );

    if (response.statusCode != 200) {
      throw SagaException(
        'start_saga failed: ${response.body}',
        statusCode: response.statusCode,
      );
    }

    return StartSagaResponse.fromJson(
        jsonDecode(response.body) as Map<String, dynamic>);
  }

  /// Saga の状態を取得する。GET /api/v1/sagas/:sagaId
  Future<SagaState> getSaga(String sagaId) async {
    final response =
        await _httpClient.get(Uri.parse('$_endpoint/api/v1/sagas/$sagaId'));

    if (response.statusCode != 200) {
      throw SagaException(
        'get_saga failed: ${response.body}',
        statusCode: response.statusCode,
      );
    }

    final json = jsonDecode(response.body) as Map<String, dynamic>;
    final sagaJson = json['saga'] as Map<String, dynamic>? ?? json;
    return SagaState.fromJson(sagaJson);
  }

  /// Saga をキャンセルする。POST /api/v1/sagas/:sagaId/cancel
  Future<void> cancelSaga(String sagaId) async {
    final response = await _httpClient.post(
      Uri.parse('$_endpoint/api/v1/sagas/$sagaId/cancel'),
      headers: {'Content-Type': 'application/json'},
      body: '{}',
    );

    if (response.statusCode != 200) {
      throw SagaException(
        'cancel_saga failed: ${response.body}',
        statusCode: response.statusCode,
      );
    }
  }
}
