import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_observability/src/logging/log_level.dart';

void main() {
  group('LogLevel', () {
    test('contains all expected levels', () {
      expect(LogLevel.values, contains(LogLevel.debug));
      expect(LogLevel.values, contains(LogLevel.info));
      expect(LogLevel.values, contains(LogLevel.warn));
      expect(LogLevel.values, contains(LogLevel.error));
    });
  });

  group('LogLevelExtension', () {
    test('value returns correct string representation', () {
      expect(LogLevel.debug.value, 'DEBUG');
      expect(LogLevel.info.value, 'INFO');
      expect(LogLevel.warn.value, 'WARN');
      expect(LogLevel.error.value, 'ERROR');
    });

    test('priority increases with severity', () {
      expect(LogLevel.debug.priority, lessThan(LogLevel.info.priority));
      expect(LogLevel.info.priority, lessThan(LogLevel.warn.priority));
      expect(LogLevel.warn.priority, lessThan(LogLevel.error.priority));
    });

    test('isAtLeast returns true for same level', () {
      expect(LogLevel.debug.isAtLeast(LogLevel.debug), true);
      expect(LogLevel.info.isAtLeast(LogLevel.info), true);
      expect(LogLevel.warn.isAtLeast(LogLevel.warn), true);
      expect(LogLevel.error.isAtLeast(LogLevel.error), true);
    });

    test('isAtLeast returns true for higher severity', () {
      expect(LogLevel.info.isAtLeast(LogLevel.debug), true);
      expect(LogLevel.warn.isAtLeast(LogLevel.debug), true);
      expect(LogLevel.error.isAtLeast(LogLevel.debug), true);
      expect(LogLevel.error.isAtLeast(LogLevel.warn), true);
    });

    test('isAtLeast returns false for lower severity', () {
      expect(LogLevel.debug.isAtLeast(LogLevel.info), false);
      expect(LogLevel.info.isAtLeast(LogLevel.warn), false);
      expect(LogLevel.warn.isAtLeast(LogLevel.error), false);
    });

    test('fromString parses uppercase values', () {
      expect(LogLevelExtension.fromString('DEBUG'), LogLevel.debug);
      expect(LogLevelExtension.fromString('INFO'), LogLevel.info);
      expect(LogLevelExtension.fromString('WARN'), LogLevel.warn);
      expect(LogLevelExtension.fromString('ERROR'), LogLevel.error);
    });

    test('fromString parses lowercase values', () {
      expect(LogLevelExtension.fromString('debug'), LogLevel.debug);
      expect(LogLevelExtension.fromString('info'), LogLevel.info);
      expect(LogLevelExtension.fromString('warn'), LogLevel.warn);
      expect(LogLevelExtension.fromString('error'), LogLevel.error);
    });

    test('fromString parses WARNING as warn', () {
      expect(LogLevelExtension.fromString('WARNING'), LogLevel.warn);
      expect(LogLevelExtension.fromString('warning'), LogLevel.warn);
    });

    test('fromString returns info for unknown values', () {
      expect(LogLevelExtension.fromString('unknown'), LogLevel.info);
      expect(LogLevelExtension.fromString(''), LogLevel.info);
      expect(LogLevelExtension.fromString('trace'), LogLevel.info);
    });
  });
}
