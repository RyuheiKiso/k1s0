import 'dart:convert';

import 'package:http/http.dart' as http;

import 'error.dart';
import 'types.dart';

/// DLQ 管理サーバーへの REST クライアント。
class DlqClient {
  final String _endpoint;
  final http.Client _httpClient;

  DlqClient(String endpoint, {http.Client? httpClient})
      : _endpoint = endpoint.replaceAll(RegExp(r'/$'), ''),
        _httpClient = httpClient ?? http.Client();

  /// DLQ メッセージ一覧を取得する。GET /api/v1/dlq/:topic
  Future<ListDlqMessagesResponse> listMessages(
      String topic, int page, int pageSize) async {
    final uri = Uri.parse(
        '$_endpoint/api/v1/dlq/$topic?page=$page&page_size=$pageSize');
    final response = await _httpClient.get(uri);

    if (response.statusCode != 200) {
      throw DlqException(
        'list_messages failed: ${response.body}',
        statusCode: response.statusCode,
      );
    }

    return ListDlqMessagesResponse.fromJson(
        jsonDecode(response.body) as Map<String, dynamic>);
  }

  /// DLQ メッセージの詳細を取得する。GET /api/v1/dlq/messages/:id
  Future<DlqMessage> getMessage(String messageId) async {
    final response = await _httpClient
        .get(Uri.parse('$_endpoint/api/v1/dlq/messages/$messageId'));

    if (response.statusCode != 200) {
      throw DlqException(
        'get_message failed: ${response.body}',
        statusCode: response.statusCode,
      );
    }

    return DlqMessage.fromJson(
        jsonDecode(response.body) as Map<String, dynamic>);
  }

  /// DLQ メッセージを再処理する。POST /api/v1/dlq/messages/:id/retry
  Future<RetryDlqMessageResponse> retryMessage(String messageId) async {
    final response = await _httpClient.post(
      Uri.parse('$_endpoint/api/v1/dlq/messages/$messageId/retry'),
      headers: {'Content-Type': 'application/json'},
      body: '{}',
    );

    if (response.statusCode != 200) {
      throw DlqException(
        'retry_message failed: ${response.body}',
        statusCode: response.statusCode,
      );
    }

    return RetryDlqMessageResponse.fromJson(
        jsonDecode(response.body) as Map<String, dynamic>);
  }

  /// DLQ メッセージを削除する。DELETE /api/v1/dlq/messages/:id
  Future<void> deleteMessage(String messageId) async {
    final response = await _httpClient
        .delete(Uri.parse('$_endpoint/api/v1/dlq/messages/$messageId'));

    if (response.statusCode != 200 && response.statusCode != 204) {
      throw DlqException(
        'delete_message failed: ${response.body}',
        statusCode: response.statusCode,
      );
    }
  }

  /// トピック内全メッセージを一括再処理する。POST /api/v1/dlq/:topic/retry-all
  Future<void> retryAll(String topic) async {
    final response = await _httpClient.post(
      Uri.parse('$_endpoint/api/v1/dlq/$topic/retry-all'),
      headers: {'Content-Type': 'application/json'},
      body: '{}',
    );

    if (response.statusCode != 200 && response.statusCode != 204) {
      throw DlqException(
        'retry_all failed: ${response.body}',
        statusCode: response.statusCode,
      );
    }
  }
}
