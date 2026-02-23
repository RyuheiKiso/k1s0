import 'config.dart';
import 'error.dart';

Future<T> withRetry<T>(
  RetryConfig config,
  Future<T> Function() operation,
) async {
  Object? lastError;

  for (var attempt = 0; attempt < config.maxAttempts; attempt++) {
    try {
      return await operation();
    } catch (err) {
      lastError = err;
      if (attempt + 1 < config.maxAttempts) {
        final delay = computeDelay(config, attempt);
        await Future<void>.delayed(Duration(milliseconds: delay));
      }
    }
  }
  throw RetryError(config.maxAttempts, lastError!);
}
