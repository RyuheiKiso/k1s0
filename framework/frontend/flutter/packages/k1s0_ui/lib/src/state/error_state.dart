import 'package:flutter/material.dart';

import '../theme/k1s0_spacing.dart';
import '../widgets/buttons.dart';

/// Error state widget
class K1s0ErrorState extends StatelessWidget {
  /// Creates an error state widget
  const K1s0ErrorState({
    required this.message,
    this.title,
    this.icon,
    this.onRetry,
    this.retryLabel,
    this.details,
    this.centered = true,
    super.key,
  });

  /// Error message
  final String message;

  /// Error title
  final String? title;

  /// Custom icon
  final IconData? icon;

  /// Retry callback
  final VoidCallback? onRetry;

  /// Retry button label
  final String? retryLabel;

  /// Error details (for debugging)
  final String? details;

  /// Whether to center the content
  final bool centered;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    final content = Padding(
      padding: K1s0Spacing.allLg,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            icon ?? Icons.error_outline,
            size: 64,
            color: scheme.error,
          ),
          K1s0Spacing.gapMd,
          if (title != null) ...[
            Text(
              title!,
              style: textTheme.titleLarge,
              textAlign: TextAlign.center,
            ),
            K1s0Spacing.gapSm,
          ],
          Text(
            message,
            style: textTheme.bodyMedium?.copyWith(
              color: scheme.onSurfaceVariant,
            ),
            textAlign: TextAlign.center,
          ),
          if (details != null) ...[
            K1s0Spacing.gapMd,
            Container(
              padding: K1s0Spacing.allSm,
              decoration: BoxDecoration(
                color: scheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                details!,
                style: textTheme.bodySmall?.copyWith(
                  fontFamily: 'monospace',
                ),
              ),
            ),
          ],
          if (onRetry != null) ...[
            K1s0Spacing.gapLg,
            K1s0PrimaryButton(
              onPressed: onRetry,
              icon: Icons.refresh,
              child: Text(retryLabel ?? 'Retry'),
            ),
          ],
        ],
      ),
    );

    if (centered) {
      return Center(child: content);
    }

    return content;
  }
}

/// Network error state
class K1s0NetworkError extends StatelessWidget {
  /// Creates a network error state
  const K1s0NetworkError({
    this.onRetry,
    this.message,
    super.key,
  });

  /// Retry callback
  final VoidCallback? onRetry;

  /// Custom message
  final String? message;

  @override
  Widget build(BuildContext context) {
    return K1s0ErrorState(
      title: 'Connection Error',
      message: message ?? 'Unable to connect to the server. Please check your internet connection.',
      icon: Icons.wifi_off,
      onRetry: onRetry,
    );
  }
}

/// Server error state
class K1s0ServerError extends StatelessWidget {
  /// Creates a server error state
  const K1s0ServerError({
    this.onRetry,
    this.message,
    this.errorCode,
    super.key,
  });

  /// Retry callback
  final VoidCallback? onRetry;

  /// Custom message
  final String? message;

  /// Error code
  final String? errorCode;

  @override
  Widget build(BuildContext context) {
    return K1s0ErrorState(
      title: 'Server Error',
      message: message ?? 'Something went wrong on our end. Please try again later.',
      icon: Icons.cloud_off,
      onRetry: onRetry,
      details: errorCode,
    );
  }
}

/// Permission denied error state
class K1s0PermissionDenied extends StatelessWidget {
  /// Creates a permission denied state
  const K1s0PermissionDenied({
    this.message,
    this.onGoBack,
    super.key,
  });

  /// Custom message
  final String? message;

  /// Go back callback
  final VoidCallback? onGoBack;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    return Center(
      child: Padding(
        padding: K1s0Spacing.allLg,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.lock_outline,
              size: 64,
              color: scheme.error,
            ),
            K1s0Spacing.gapMd,
            Text(
              'Access Denied',
              style: textTheme.titleLarge,
              textAlign: TextAlign.center,
            ),
            K1s0Spacing.gapSm,
            Text(
              message ?? 'You do not have permission to view this content.',
              style: textTheme.bodyMedium?.copyWith(
                color: scheme.onSurfaceVariant,
              ),
              textAlign: TextAlign.center,
            ),
            if (onGoBack != null) ...[
              K1s0Spacing.gapLg,
              K1s0SecondaryButton(
                onPressed: onGoBack,
                icon: Icons.arrow_back,
                child: const Text('Go Back'),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
