import 'dart:convert';
import 'package:http/http.dart' as http;

import 'client.dart';
import 'types.dart';

// HTTP AI クライアント: AI ゲートウェイと HTTP 通信する実装
class HttpAiClient implements AiClient {
  final String baseUrl;
  final String apiKey;
  final Duration timeout;
  final http.Client _httpClient;

  HttpAiClient({
    required this.baseUrl,
    this.apiKey = '',
    // デフォルトタイムアウトは 30 秒
    this.timeout = const Duration(seconds: 30),
    http.Client? httpClient,
  }) : _httpClient = httpClient ?? http.Client();

  // AI ゲートウェイの /v1/complete エンドポイントを呼び出す
  @override
  Future<CompleteResponse> complete(CompleteRequest req) async {
    final raw = await _post('/v1/complete', req.toJson());
    return CompleteResponse.fromJson(raw);
  }

  // AI ゲートウェイの /v1/embed エンドポイントを呼び出す
  @override
  Future<EmbedResponse> embed(EmbedRequest req) async {
    final raw = await _post('/v1/embed', req.toJson());
    return EmbedResponse.fromJson(raw);
  }

  // AI ゲートウェイの /v1/models エンドポイントを呼び出す
  @override
  Future<List<ModelInfo>> listModels() async {
    final uri = Uri.parse('${baseUrl.replaceAll(RegExp(r'/$'), '')}/v1/models');
    final headers = <String, String>{};
    if (apiKey.isNotEmpty) {
      headers['Authorization'] = 'Bearer $apiKey';
    }
    final response = await _httpClient.get(uri, headers: headers).timeout(timeout);
    if (response.statusCode != 200) {
      throw AiClientError(
        'API error: ${response.statusCode}',
        statusCode: response.statusCode,
      );
    }
    final list = json.decode(response.body) as List;
    return list.map((e) => ModelInfo.fromJson(e as Map<String, dynamic>)).toList();
  }

  // POST リクエストを送信してレスポンス JSON を返す
  Future<Map<String, dynamic>> _post(String path, Map<String, dynamic> body) async {
    final uri = Uri.parse('${baseUrl.replaceAll(RegExp(r'/$'), '')}$path');
    final headers = <String, String>{'Content-Type': 'application/json'};
    if (apiKey.isNotEmpty) {
      headers['Authorization'] = 'Bearer $apiKey';
    }
    final response = await _httpClient
        .post(uri, headers: headers, body: json.encode(body))
        .timeout(timeout);
    if (response.statusCode != 200) {
      throw AiClientError(
        'API error: ${response.statusCode}',
        statusCode: response.statusCode,
      );
    }
    return json.decode(response.body) as Map<String, dynamic>;
  }
}
