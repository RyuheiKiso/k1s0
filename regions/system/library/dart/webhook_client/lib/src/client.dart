import 'dart:convert';
import 'dart:developer' as developer;
import 'dart:math';

import 'package:http/http.dart' as http;

import 'payload.dart';
import 'signature.dart';

/// Configuration for webhook delivery with retry behavior.
class WebhookConfig {
  final int maxRetries;
  final int initialBackoffMs;
  final int maxBackoffMs;

  const WebhookConfig({
    this.maxRetries = 3,
    this.initialBackoffMs = 1000,
    this.maxBackoffMs = 30000,
  });
}

/// Error codes for webhook delivery failures.
enum WebhookErrorCode {
  sendFailed,
  maxRetriesExceeded,
}

/// Error thrown when webhook delivery fails.
class WebhookError implements Exception {
  final String message;
  final WebhookErrorCode code;

  const WebhookError(this.message, this.code);

  @override
  String toString() => 'WebhookError($code): $message';
}

/// Function type for generating UUIDs, to allow injection in tests.
typedef UuidGenerator = String Function();

/// Function type for delaying, to allow injection in tests.
typedef DelayFn = Future<void> Function(Duration duration);

abstract class WebhookClient {
  Future<int> send(String url, WebhookPayload payload);
}

/// HTTP-based webhook client with retry, idempotency, and signature support.
class HttpWebhookClient implements WebhookClient {
  static const _loggerName = 'k1s0.webhook_client.HttpWebhookClient';
  final String? secret;
  final WebhookConfig config;
  final http.Client _httpClient;
  final UuidGenerator _uuidGenerator;
  final DelayFn _delayFn;
  final Random _random;

  HttpWebhookClient({
    this.secret,
    this.config = const WebhookConfig(),
    http.Client? httpClient,
    UuidGenerator? uuidGenerator,
    DelayFn? delayFn,
    Random? random,
  })  : _httpClient = httpClient ?? http.Client(),
        _uuidGenerator = uuidGenerator ?? _defaultUuidV4,
        _delayFn = delayFn ?? _defaultDelay,
        _random = random ?? Random();

  @override
  Future<int> send(String url, WebhookPayload payload) async {
    final body = jsonEncode({
      'eventType': payload.eventType,
      'timestamp': payload.timestamp,
      'data': payload.data,
    });
    final idempotencyKey = _uuidGenerator();

    final headers = <String, String>{
      'Content-Type': 'application/json',
      'Idempotency-Key': idempotencyKey,
    };

    if (secret != null) {
      headers['X-K1s0-Signature'] = generateSignature(secret!, body);
    }

    int lastStatus = 0;
    Object? lastError;

    for (int attempt = 0; attempt <= config.maxRetries; attempt++) {
      if (attempt > 0) {
        final backoff = min(
          config.initialBackoffMs * pow(2, attempt - 1).toInt(),
          config.maxBackoffMs,
        );
        final jitter = (_random.nextDouble() * backoff).toInt();
        final delay = backoff + jitter;
        developer.log(
          '[webhook-client] Retry attempt $attempt/${config.maxRetries} '
          'for $url after ${delay}ms',
          name: _loggerName,
        );
        await _delayFn(Duration(milliseconds: delay));
      }

      try {
        developer.log(
          '[webhook-client] Sending webhook to $url '
          '(attempt ${attempt + 1}/${config.maxRetries + 1}, '
          'idempotency-key=$idempotencyKey)',
          name: _loggerName,
        );

        final response = await _httpClient.post(
          Uri.parse(url),
          headers: headers,
          body: body,
        );

        lastStatus = response.statusCode;

        if (_isRetryable(lastStatus)) {
          developer.log(
            '[webhook-client] Retryable status $lastStatus from $url',
            name: _loggerName,
            level: 900,
          );
          lastError = WebhookError(
            'Webhook request to $url returned status $lastStatus',
            WebhookErrorCode.sendFailed,
          );
          continue;
        }

        return lastStatus;
      } catch (e) {
        lastError = e;
        developer.log(
          '[webhook-client] Network error on attempt '
          '${attempt + 1}/${config.maxRetries + 1} for $url: $e',
          name: _loggerName,
          error: e,
          level: 1000,
        );
      }
    }

    throw WebhookError(
      'Webhook delivery to $url failed after ${config.maxRetries + 1} '
      'attempts: ${lastError ?? 'status $lastStatus'}',
      WebhookErrorCode.maxRetriesExceeded,
    );
  }

  bool _isRetryable(int status) => status == 429 || status >= 500;

  static String _defaultUuidV4() {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    // Set version to 4
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // Set variant to RFC4122
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return '${hex.substring(0, 8)}-${hex.substring(8, 12)}-'
        '${hex.substring(12, 16)}-${hex.substring(16, 20)}-'
        '${hex.substring(20, 32)}';
  }

  static Future<void> _defaultDelay(Duration duration) =>
      Future.delayed(duration);
}

class InMemoryWebhookClient implements WebhookClient {
  final List<(String, WebhookPayload)> _sent = [];

  List<(String, WebhookPayload)> get sent => List.unmodifiable(_sent);

  @override
  Future<int> send(String url, WebhookPayload payload) async {
    _sent.add((url, payload));
    return 200;
  }
}
