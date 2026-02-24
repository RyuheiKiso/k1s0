import 'dart:async';
import 'dart:io';

import 'checker.dart';

/// HTTP GET リクエストでヘルスを確認する HealthCheck 実装。
class HttpHealthCheck implements HealthCheck {
  @override
  final String name;

  final String url;
  final Duration timeout;

  HttpHealthCheck({
    required this.url,
    this.timeout = const Duration(seconds: 5),
    String? name,
  }) : name = name ?? 'http';

  @override
  Future<void> check() async {
    final client = HttpClient()..connectionTimeout = timeout;
    try {
      final request = await client.getUrl(Uri.parse(url)).timeout(timeout);
      final response = await request.close().timeout(timeout);

      if (response.statusCode < 200 || response.statusCode >= 300) {
        throw Exception('HTTP $url returned status ${response.statusCode}');
      }
    } on TimeoutException {
      throw Exception('HTTP check timeout: $url');
    } finally {
      client.close();
    }
  }
}
