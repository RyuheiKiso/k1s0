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
  /// Private constructor to prevent instantiation.
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

    final (backgroundColor, foregroundColor, icon) = switch (type) {
      K1s0SnackbarType.info => (
          scheme.inverseSurface,
          scheme.onInverseSurface,
          Icons.info_outline,
        ),
      K1s0SnackbarType.success => (
          K1s0Colors.success,
          K1s0Colors.onSuccess,
          Icons.check_circle_outline,
        ),
      K1s0SnackbarType.warning => (
          K1s0Colors.warning,
          K1s0Colors.onWarning,
          Icons.warning_amber_outlined,
        ),
      K1s0SnackbarType.error => (
          scheme.error,
          scheme.onError,
          Icons.error_outline,
        ),
    };

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
      ),
    );
  }

  /// Show an info snackbar
  static void info(BuildContext context, String message) {
    show(context, message: message);
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
