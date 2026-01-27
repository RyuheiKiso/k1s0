import 'package:flutter/material.dart';

import '../theme/k1s0_spacing.dart';
import '../widgets/buttons.dart';

/// k1s0 dialog helpers
class K1s0Dialog {
  K1s0Dialog._();

  /// Show a confirmation dialog
  static Future<bool> confirm(
    BuildContext context, {
    required String title,
    required String message,
    String? confirmLabel,
    String? cancelLabel,
    bool isDanger = false,
  }) async {
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => K1s0ConfirmDialog(
        title: title,
        message: message,
        confirmLabel: confirmLabel,
        cancelLabel: cancelLabel,
        isDanger: isDanger,
      ),
    );

    return result ?? false;
  }

  /// Show an alert dialog
  static Future<void> alert(
    BuildContext context, {
    required String title,
    required String message,
    String? okLabel,
  }) async {
    await showDialog<void>(
      context: context,
      builder: (context) => K1s0AlertDialog(
        title: title,
        message: message,
        okLabel: okLabel,
      ),
    );
  }

  /// Show a loading dialog
  static void showLoading(
    BuildContext context, {
    String? message,
  }) {
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (context) => K1s0LoadingDialog(
        message: message,
      ),
    );
  }

  /// Hide the loading dialog
  static void hideLoading(BuildContext context) {
    Navigator.of(context, rootNavigator: true).pop();
  }
}

/// Confirmation dialog widget
class K1s0ConfirmDialog extends StatelessWidget {
  /// Creates a confirmation dialog
  const K1s0ConfirmDialog({
    required this.title,
    required this.message,
    this.confirmLabel,
    this.cancelLabel,
    this.isDanger = false,
    super.key,
  });

  /// Dialog title
  final String title;

  /// Dialog message
  final String message;

  /// Confirm button label
  final String? confirmLabel;

  /// Cancel button label
  final String? cancelLabel;

  /// Whether this is a dangerous action
  final bool isDanger;

  @override
  Widget build(BuildContext context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: Text(cancelLabel ?? 'Cancel'),
          ),
          if (isDanger)
            K1s0DangerButton(
              onPressed: () => Navigator.of(context).pop(true),
              child: Text(confirmLabel ?? 'Confirm'),
            )
          else
            FilledButton(
              onPressed: () => Navigator.of(context).pop(true),
              child: Text(confirmLabel ?? 'Confirm'),
            ),
        ],
      );
}

/// Alert dialog widget
class K1s0AlertDialog extends StatelessWidget {
  /// Creates an alert dialog
  const K1s0AlertDialog({
    required this.title,
    required this.message,
    this.okLabel,
    super.key,
  });

  /// Dialog title
  final String title;

  /// Dialog message
  final String message;

  /// OK button label
  final String? okLabel;

  @override
  Widget build(BuildContext context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          FilledButton(
            onPressed: () => Navigator.of(context).pop(),
            child: Text(okLabel ?? 'OK'),
          ),
        ],
      );
}

/// Loading dialog widget
class K1s0LoadingDialog extends StatelessWidget {
  /// Creates a loading dialog
  const K1s0LoadingDialog({
    this.message,
    super.key,
  });

  /// Loading message
  final String? message;

  @override
  Widget build(BuildContext context) => AlertDialog(
        content: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const CircularProgressIndicator(),
            K1s0Spacing.gapHMd,
            Flexible(
              child: Text(message ?? 'Loading...'),
            ),
          ],
        ),
      );
}

/// Custom dialog widget
class K1s0CustomDialog extends StatelessWidget {
  /// Creates a custom dialog
  const K1s0CustomDialog({
    required this.child,
    this.title,
    this.actions,
    this.padding,
    super.key,
  });

  /// Dialog content
  final Widget child;

  /// Dialog title
  final String? title;

  /// Dialog actions
  final List<Widget>? actions;

  /// Content padding
  final EdgeInsets? padding;

  @override
  Widget build(BuildContext context) => AlertDialog(
        title: title != null ? Text(title!) : null,
        content: Padding(
          padding: padding ?? EdgeInsets.zero,
          child: child,
        ),
        actions: actions,
      );
}
