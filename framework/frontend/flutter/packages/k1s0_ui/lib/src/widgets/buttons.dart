import 'package:flutter/material.dart';

import '../theme/k1s0_spacing.dart';

/// k1s0 primary button
class K1s0PrimaryButton extends StatelessWidget {
  /// Creates a primary button
  const K1s0PrimaryButton({
    required this.onPressed,
    required this.child,
    this.loading = false,
    this.disabled = false,
    this.icon,
    this.fullWidth = false,
    super.key,
  });

  /// Button text or widget
  final Widget child;

  /// Callback when pressed
  final VoidCallback? onPressed;

  /// Whether the button is loading
  final bool loading;

  /// Whether the button is disabled
  final bool disabled;

  /// Optional leading icon
  final IconData? icon;

  /// Whether to expand to full width
  final bool fullWidth;

  @override
  Widget build(BuildContext context) {
    final button = FilledButton(
      onPressed: (loading || disabled) ? null : onPressed,
      child: Row(
        mainAxisSize: fullWidth ? MainAxisSize.max : MainAxisSize.min,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          if (loading) ...[
            const SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: Colors.white,
              ),
            ),
            K1s0Spacing.gapHSm,
          ] else if (icon != null) ...[
            Icon(icon, size: 18),
            K1s0Spacing.gapHSm,
          ],
          child,
        ],
      ),
    );

    if (fullWidth) {
      return SizedBox(
        width: double.infinity,
        child: button,
      );
    }

    return button;
  }
}

/// k1s0 secondary button
class K1s0SecondaryButton extends StatelessWidget {
  /// Creates a secondary button
  const K1s0SecondaryButton({
    required this.onPressed,
    required this.child,
    this.loading = false,
    this.disabled = false,
    this.icon,
    this.fullWidth = false,
    super.key,
  });

  /// Button text or widget
  final Widget child;

  /// Callback when pressed
  final VoidCallback? onPressed;

  /// Whether the button is loading
  final bool loading;

  /// Whether the button is disabled
  final bool disabled;

  /// Optional leading icon
  final IconData? icon;

  /// Whether to expand to full width
  final bool fullWidth;

  @override
  Widget build(BuildContext context) {
    final button = OutlinedButton(
      onPressed: (loading || disabled) ? null : onPressed,
      child: Row(
        mainAxisSize: fullWidth ? MainAxisSize.max : MainAxisSize.min,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          if (loading) ...[
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: Theme.of(context).colorScheme.primary,
              ),
            ),
            K1s0Spacing.gapHSm,
          ] else if (icon != null) ...[
            Icon(icon, size: 18),
            K1s0Spacing.gapHSm,
          ],
          child,
        ],
      ),
    );

    if (fullWidth) {
      return SizedBox(
        width: double.infinity,
        child: button,
      );
    }

    return button;
  }
}

/// k1s0 text button
class K1s0TextButton extends StatelessWidget {
  /// Creates a text button
  const K1s0TextButton({
    required this.onPressed,
    required this.child,
    this.loading = false,
    this.disabled = false,
    this.icon,
    super.key,
  });

  /// Button text or widget
  final Widget child;

  /// Callback when pressed
  final VoidCallback? onPressed;

  /// Whether the button is loading
  final bool loading;

  /// Whether the button is disabled
  final bool disabled;

  /// Optional leading icon
  final IconData? icon;

  @override
  Widget build(BuildContext context) {
    return TextButton(
      onPressed: (loading || disabled) ? null : onPressed,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (loading) ...[
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: Theme.of(context).colorScheme.primary,
              ),
            ),
            K1s0Spacing.gapHSm,
          ] else if (icon != null) ...[
            Icon(icon, size: 18),
            K1s0Spacing.gapHSm,
          ],
          child,
        ],
      ),
    );
  }
}

/// k1s0 icon button
class K1s0IconButton extends StatelessWidget {
  /// Creates an icon button
  const K1s0IconButton({
    required this.icon,
    required this.onPressed,
    this.tooltip,
    this.loading = false,
    this.disabled = false,
    this.size = 24.0,
    this.color,
    super.key,
  });

  /// Icon
  final IconData icon;

  /// Callback when pressed
  final VoidCallback? onPressed;

  /// Tooltip text
  final String? tooltip;

  /// Whether the button is loading
  final bool loading;

  /// Whether the button is disabled
  final bool disabled;

  /// Icon size
  final double size;

  /// Icon color
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final button = IconButton(
      onPressed: (loading || disabled) ? null : onPressed,
      icon: loading
          ? SizedBox(
              width: size,
              height: size,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: color ?? Theme.of(context).colorScheme.primary,
              ),
            )
          : Icon(icon, size: size, color: color),
    );

    if (tooltip != null) {
      return Tooltip(
        message: tooltip!,
        child: button,
      );
    }

    return button;
  }
}

/// k1s0 danger button (for destructive actions)
class K1s0DangerButton extends StatelessWidget {
  /// Creates a danger button
  const K1s0DangerButton({
    required this.onPressed,
    required this.child,
    this.loading = false,
    this.disabled = false,
    this.icon,
    this.fullWidth = false,
    super.key,
  });

  /// Button text or widget
  final Widget child;

  /// Callback when pressed
  final VoidCallback? onPressed;

  /// Whether the button is loading
  final bool loading;

  /// Whether the button is disabled
  final bool disabled;

  /// Optional leading icon
  final IconData? icon;

  /// Whether to expand to full width
  final bool fullWidth;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;

    final button = FilledButton(
      onPressed: (loading || disabled) ? null : onPressed,
      style: FilledButton.styleFrom(
        backgroundColor: scheme.error,
        foregroundColor: scheme.onError,
      ),
      child: Row(
        mainAxisSize: fullWidth ? MainAxisSize.max : MainAxisSize.min,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          if (loading) ...[
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: scheme.onError,
              ),
            ),
            K1s0Spacing.gapHSm,
          ] else if (icon != null) ...[
            Icon(icon, size: 18),
            K1s0Spacing.gapHSm,
          ],
          child,
        ],
      ),
    );

    if (fullWidth) {
      return SizedBox(
        width: double.infinity,
        child: button,
      );
    }

    return button;
  }
}
