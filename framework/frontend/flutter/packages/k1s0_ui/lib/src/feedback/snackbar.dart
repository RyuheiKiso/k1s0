import 'package:flutter/material.dart';

import '../theme/k1s0_colors.dart';
import '../theme/k1s0_spacing.dart';

/// Snackbar type
enum K1s0SnackbarType {
  /// Default/info snackbar
  info,

  /// Success snackbar
  success,

  /// Warning snackbar
  warning,

  /// Error snackbar
  error,
}

/// k1s0 snackbar helper
class K1s0Snackbar {
  K1s0Snackbar._();

  /// Show a snackbar
  static void show(
    BuildContext context, {
    required String message,
    K1s0SnackbarType type = K1s0SnackbarType.info,
    Duration duration = const Duration(seconds: 4),
    String? actionLabel,
    VoidCallback? onAction,
    VoidCallback? onDismissed,
  }) {
    final scheme = Theme.of(context).colorScheme;

    Color backgroundColor;
    Color foregroundColor;
    IconData icon;

    switch (type) {
      case K1s0SnackbarType.info:
        backgroundColor = scheme.inverseSurface;
        foregroundColor = scheme.onInverseSurface;
        icon = Icons.info_outline;
        break;
      case K1s0SnackbarType.success:
        backgroundColor = K1s0Colors.success;
        foregroundColor = K1s0Colors.onSuccess;
        icon = Icons.check_circle_outline;
        break;
      case K1s0SnackbarType.warning:
        backgroundColor = K1s0Colors.warning;
        foregroundColor = K1s0Colors.onWarning;
        icon = Icons.warning_amber_outlined;
        break;
      case K1s0SnackbarType.error:
        backgroundColor = scheme.error;
        foregroundColor = scheme.onError;
        icon = Icons.error_outline;
        break;
    }

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            Icon(icon, color: foregroundColor, size: 20),
            K1s0Spacing.gapHSm,
            Expanded(
              child: Text(
                message,
                style: TextStyle(color: foregroundColor),
              ),
            ),
          ],
        ),
        backgroundColor: backgroundColor,
        duration: duration,
        behavior: SnackBarBehavior.floating,
        action: actionLabel != null
            ? SnackBarAction(
                label: actionLabel,
                textColor: foregroundColor,
                onPressed: onAction ?? () {},
              )
            : null,
        onVisible: () {},
      ),
    );
  }

  /// Show an info snackbar
  static void info(BuildContext context, String message) {
    show(context, message: message, type: K1s0SnackbarType.info);
  }

  /// Show a success snackbar
  static void success(BuildContext context, String message) {
    show(context, message: message, type: K1s0SnackbarType.success);
  }

  /// Show a warning snackbar
  static void warning(BuildContext context, String message) {
    show(context, message: message, type: K1s0SnackbarType.warning);
  }

  /// Show an error snackbar
  static void error(BuildContext context, String message) {
    show(context, message: message, type: K1s0SnackbarType.error);
  }

  /// Hide the current snackbar
  static void hide(BuildContext context) {
    ScaffoldMessenger.of(context).hideCurrentSnackBar();
  }
}
