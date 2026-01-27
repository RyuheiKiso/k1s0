import 'package:flutter/material.dart';

import '../theme/k1s0_spacing.dart';

/// k1s0 card widget
class K1s0Card extends StatelessWidget {
  /// Creates a card
  const K1s0Card({
    required this.child,
    this.padding,
    this.margin,
    this.elevation,
    this.onTap,
    this.borderRadius,
    super.key,
  });

  /// Card content
  final Widget child;

  /// Card padding
  final EdgeInsets? padding;

  /// Card margin
  final EdgeInsets? margin;

  /// Card elevation
  final double? elevation;

  /// Callback when tapped
  final VoidCallback? onTap;

  /// Custom border radius
  final BorderRadius? borderRadius;

  @override
  Widget build(BuildContext context) {
    final card = Card(
      elevation: elevation,
      margin: margin ?? K1s0Spacing.allSm,
      shape: borderRadius != null
          ? RoundedRectangleBorder(borderRadius: borderRadius!)
          : null,
      child: Padding(
        padding: padding ?? K1s0Spacing.allMd,
        child: child,
      ),
    );

    if (onTap != null) {
      return InkWell(
        onTap: onTap,
        borderRadius: borderRadius ?? BorderRadius.circular(8),
        child: card,
      );
    }

    return card;
  }
}

/// k1s0 outlined card
class K1s0OutlinedCard extends StatelessWidget {
  /// Creates an outlined card
  const K1s0OutlinedCard({
    required this.child,
    this.padding,
    this.margin,
    this.onTap,
    this.borderColor,
    this.borderRadius,
    super.key,
  });

  /// Card content
  final Widget child;

  /// Card padding
  final EdgeInsets? padding;

  /// Card margin
  final EdgeInsets? margin;

  /// Callback when tapped
  final VoidCallback? onTap;

  /// Border color
  final Color? borderColor;

  /// Custom border radius
  final BorderRadius? borderRadius;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final radius = borderRadius ?? BorderRadius.circular(8);

    final card = Container(
      margin: margin ?? K1s0Spacing.allSm,
      decoration: BoxDecoration(
        border: Border.all(
          color: borderColor ?? scheme.outline,
        ),
        borderRadius: radius,
      ),
      child: Padding(
        padding: padding ?? K1s0Spacing.allMd,
        child: child,
      ),
    );

    if (onTap != null) {
      return InkWell(
        onTap: onTap,
        borderRadius: radius,
        child: card,
      );
    }

    return card;
  }
}

/// k1s0 info card for displaying status information
class K1s0InfoCard extends StatelessWidget {
  /// Creates an info card
  const K1s0InfoCard({
    required this.title,
    this.subtitle,
    this.icon,
    this.action,
    this.type = K1s0InfoCardType.info,
    super.key,
  });

  /// Card title
  final String title;

  /// Card subtitle
  final String? subtitle;

  /// Leading icon
  final IconData? icon;

  /// Trailing action widget
  final Widget? action;

  /// Card type (determines color)
  final K1s0InfoCardType type;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    final (backgroundColor, foregroundColor) = switch (type) {
      K1s0InfoCardType.info => (
          scheme.primaryContainer,
          scheme.onPrimaryContainer,
        ),
      K1s0InfoCardType.success => (
          Colors.green.shade100,
          Colors.green.shade900,
        ),
      K1s0InfoCardType.warning => (
          Colors.orange.shade100,
          Colors.orange.shade900,
        ),
      K1s0InfoCardType.error => (
          scheme.errorContainer,
          scheme.onErrorContainer,
        ),
    };

    return Container(
      padding: K1s0Spacing.allMd,
      decoration: BoxDecoration(
        color: backgroundColor,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          if (icon != null) ...[
            Icon(icon, color: foregroundColor),
            K1s0Spacing.gapHMd,
          ],
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: textTheme.titleSmall?.copyWith(
                    color: foregroundColor,
                  ),
                ),
                if (subtitle != null) ...[
                  K1s0Spacing.gapXs,
                  Text(
                    subtitle!,
                    style: textTheme.bodySmall?.copyWith(
                      color: foregroundColor.withValues(alpha: 0.8),
                    ),
                  ),
                ],
              ],
            ),
          ),
          if (action != null) action!,
        ],
      ),
    );
  }
}

/// Info card types
enum K1s0InfoCardType {
  /// Informational (blue)
  info,

  /// Success (green)
  success,

  /// Warning (orange)
  warning,

  /// Error (red)
  error,
}
