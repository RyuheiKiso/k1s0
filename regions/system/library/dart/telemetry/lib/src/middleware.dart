import 'dart:io';
import 'dart:math';

import 'package:logging/logging.dart';
import 'package:shelf/shelf.dart';

/// HTTP リクエストの分散トレーシングと構造化ログを提供するミドルウェア。
/// リクエストごとに duration を計測し、メソッド・パス・ステータスコード・レイテンシをログに記録する。
class TelemetryMiddleware {
  final Logger _logger;
  final Random _random = Random();

  /// TelemetryMiddleware を生成する。
  /// [logger] はリクエストログの出力先。
  TelemetryMiddleware({required Logger logger}) : _logger = logger;

  /// shelf の Middleware として使用するための関数。
  Middleware get middleware => _createMiddleware;

  Handler _createMiddleware(Handler innerHandler) {
    return (Request request) async {
      final traceId = request.headers['x-trace-id'] ?? _generateTraceId();
      final start = DateTime.now();

      // trace-id をリクエストヘッダーに注入
      final tracedRequest = request.change(
        headers: {'x-trace-id': traceId},
      );

      Response response;
      try {
        response = await innerHandler(tracedRequest);
      } catch (e) {
        final duration = DateTime.now().difference(start);
        _logger.severe(
          'Request failed: ${request.method} ${request.requestedUri.path} '
          '- ${e.toString()} [${duration.inMilliseconds}ms] '
          'trace-id=$traceId',
        );
        response = Response.internalServerError(
          body: 'Internal Server Error',
        );
      }

      final duration = DateTime.now().difference(start);

      // trace-id をレスポンスヘッダーに追加
      response = response.change(
        headers: {'x-trace-id': traceId},
      );

      // ステータスコードに応じてログレベルを変更
      final statusCode = response.statusCode;
      final message =
          '${request.method} ${request.requestedUri.path} '
          '$statusCode [${duration.inMilliseconds}ms] '
          'trace-id=$traceId';

      if (statusCode >= HttpStatus.internalServerError) {
        _logger.warning(message);
      } else {
        _logger.info(message);
      }

      return response;
    };
  }

  /// ランダムな trace-id を生成する。
  String _generateTraceId() {
    final bytes = List<int>.generate(16, (_) => _random.nextInt(256));
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
