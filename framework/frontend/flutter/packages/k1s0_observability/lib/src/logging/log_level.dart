/// Log level enumeration
enum LogLevel {
  /// Debug level - detailed information for debugging
  debug,

  /// Info level - general information
  info,

  /// Warning level - potentially harmful situations
  warn,

  /// Error level - error events
  error,
}

/// Log level utilities
extension LogLevelExtension on LogLevel {
  /// Get the string representation
  String get value {
    switch (this) {
      case LogLevel.debug:
        return 'DEBUG';
      case LogLevel.info:
        return 'INFO';
      case LogLevel.warn:
        return 'WARN';
      case LogLevel.error:
        return 'ERROR';
    }
  }

  /// Get the priority (higher = more severe)
  int get priority {
    switch (this) {
      case LogLevel.debug:
        return 0;
      case LogLevel.info:
        return 1;
      case LogLevel.warn:
        return 2;
      case LogLevel.error:
        return 3;
    }
  }

  /// Check if this level is at least as severe as the other
  bool isAtLeast(LogLevel other) => priority >= other.priority;

  /// Parse from string
  static LogLevel fromString(String value) {
    switch (value.toUpperCase()) {
      case 'DEBUG':
        return LogLevel.debug;
      case 'INFO':
        return LogLevel.info;
      case 'WARN':
      case 'WARNING':
        return LogLevel.warn;
      case 'ERROR':
        return LogLevel.error;
      default:
        return LogLevel.info;
    }
  }
}
