import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'dialog.dart';
import 'snackbar.dart';

/// Feedback service for showing snackbars and dialogs
class FeedbackService {
  /// Creates a feedback service
  FeedbackService(this._context);

  final BuildContext _context;

  // Snackbar methods

  /// Show an info snackbar
  void showInfo(String message) {
    K1s0Snackbar.info(_context, message);
  }

  /// Show a success snackbar
  void showSuccess(String message) {
    K1s0Snackbar.success(_context, message);
  }

  /// Show a warning snackbar
  void showWarning(String message) {
    K1s0Snackbar.warning(_context, message);
  }

  /// Show an error snackbar
  void showError(String message) {
    K1s0Snackbar.error(_context, message);
  }

  /// Hide the current snackbar
  void hideSnackbar() {
    K1s0Snackbar.hide(_context);
  }

  // Dialog methods

  /// Show a confirmation dialog
  Future<bool> confirm({
    required String title,
    required String message,
    String? confirmLabel,
    String? cancelLabel,
    bool isDanger = false,
  }) {
    return K1s0Dialog.confirm(
      _context,
      title: title,
      message: message,
      confirmLabel: confirmLabel,
      cancelLabel: cancelLabel,
      isDanger: isDanger,
    );
  }

  /// Show an alert dialog
  Future<void> alert({
    required String title,
    required String message,
    String? okLabel,
  }) {
    return K1s0Dialog.alert(
      _context,
      title: title,
      message: message,
      okLabel: okLabel,
    );
  }

  /// Show a loading dialog
  void showLoading({String? message}) {
    K1s0Dialog.showLoading(_context, message: message);
  }

  /// Hide the loading dialog
  void hideLoading() {
    K1s0Dialog.hideLoading(_context);
  }
}

/// Provider for feedback service
/// Must be used with a BuildContext
final feedbackServiceProvider = Provider.family<FeedbackService, BuildContext>(
  (ref, context) => FeedbackService(context),
);

/// Extension methods for using feedback with WidgetRef
extension FeedbackRef on WidgetRef {
  /// Get the feedback service for a context
  FeedbackService feedback(BuildContext context) {
    return read(feedbackServiceProvider(context));
  }
}

/// Mixin for stateful widgets that need feedback capabilities
mixin FeedbackMixin<T extends StatefulWidget> on State<T> {
  /// Show an info snackbar
  void showInfo(String message) {
    K1s0Snackbar.info(context, message);
  }

  /// Show a success snackbar
  void showSuccess(String message) {
    K1s0Snackbar.success(context, message);
  }

  /// Show a warning snackbar
  void showWarning(String message) {
    K1s0Snackbar.warning(context, message);
  }

  /// Show an error snackbar
  void showError(String message) {
    K1s0Snackbar.error(context, message);
  }

  /// Show a confirmation dialog
  Future<bool> confirm({
    required String title,
    required String message,
    String? confirmLabel,
    String? cancelLabel,
    bool isDanger = false,
  }) {
    return K1s0Dialog.confirm(
      context,
      title: title,
      message: message,
      confirmLabel: confirmLabel,
      cancelLabel: cancelLabel,
      isDanger: isDanger,
    );
  }

  /// Show an alert dialog
  Future<void> alert({
    required String title,
    required String message,
    String? okLabel,
  }) {
    return K1s0Dialog.alert(
      context,
      title: title,
      message: message,
      okLabel: okLabel,
    );
  }

  /// Show a loading dialog
  void showLoading({String? message}) {
    K1s0Dialog.showLoading(context, message: message);
  }

  /// Hide the loading dialog
  void hideLoading() {
    K1s0Dialog.hideLoading(context);
  }
}
