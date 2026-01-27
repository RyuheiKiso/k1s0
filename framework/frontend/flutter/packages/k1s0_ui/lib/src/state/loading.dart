import 'package:flutter/material.dart';

import '../theme/k1s0_spacing.dart';

/// Loading indicator widget
class K1s0Loading extends StatelessWidget {
  /// Creates a loading indicator
  const K1s0Loading({
    this.message,
    this.size = 40.0,
    this.strokeWidth = 4.0,
    this.color,
    this.centered = true,
    super.key,
  });

  /// Optional loading message
  final String? message;

  /// Indicator size
  final double size;

  /// Stroke width
  final double strokeWidth;

  /// Indicator color
  final Color? color;

  /// Whether to center the indicator
  final bool centered;

  @override
  Widget build(BuildContext context) {
    final indicator = Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          width: size,
          height: size,
          child: CircularProgressIndicator(
            strokeWidth: strokeWidth,
            color: color,
          ),
        ),
        if (message != null) ...[
          K1s0Spacing.gapMd,
          Text(
            message!,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
            textAlign: TextAlign.center,
          ),
        ],
      ],
    );

    if (centered) {
      return Center(child: indicator);
    }

    return indicator;
  }
}

/// Full page loading overlay
class K1s0LoadingOverlay extends StatelessWidget {
  /// Creates a loading overlay
  const K1s0LoadingOverlay({
    required this.child,
    required this.isLoading,
    this.message,
    this.barrierColor,
    super.key,
  });

  /// Child widget
  final Widget child;

  /// Whether loading is active
  final bool isLoading;

  /// Loading message
  final String? message;

  /// Barrier color
  final Color? barrierColor;

  @override
  Widget build(BuildContext context) => Stack(
        children: [
          child,
          if (isLoading)
            ColoredBox(
              color: barrierColor ??
                  Theme.of(context).colorScheme.surface.withValues(alpha: 0.7),
              child: K1s0Loading(
                message: message,
              ),
            ),
        ],
      );
}

/// Shimmer loading placeholder
class K1s0ShimmerLoading extends StatefulWidget {
  /// Creates a shimmer loading placeholder
  const K1s0ShimmerLoading({
    this.width,
    this.height,
    this.borderRadius,
    super.key,
  });

  /// Width of the placeholder
  final double? width;

  /// Height of the placeholder
  final double? height;

  /// Border radius
  final BorderRadius? borderRadius;

  @override
  State<K1s0ShimmerLoading> createState() => _K1s0ShimmerLoadingState();
}

class _K1s0ShimmerLoadingState extends State<K1s0ShimmerLoading>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _animation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat();

    _animation = Tween<double>(begin: -2, end: 2).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final baseColor = scheme.surfaceContainerHighest;
    final highlightColor = scheme.surface;

    return AnimatedBuilder(
      animation: _animation,
      builder: (context, child) => Container(
        width: widget.width ?? double.infinity,
        height: widget.height ?? 20,
        decoration: BoxDecoration(
          borderRadius: widget.borderRadius ?? BorderRadius.circular(4),
          gradient: LinearGradient(
            begin: Alignment(_animation.value - 1, 0),
            end: Alignment(_animation.value + 1, 0),
            colors: [
              baseColor,
              highlightColor,
              baseColor,
            ],
          ),
        ),
      ),
    );
  }
}

/// Skeleton loading for lists
class K1s0ListSkeleton extends StatelessWidget {
  /// Creates a list skeleton
  const K1s0ListSkeleton({
    this.itemCount = 5,
    this.itemHeight = 72.0,
    this.padding,
    super.key,
  });

  /// Number of skeleton items
  final int itemCount;

  /// Height of each item
  final double itemHeight;

  /// Padding around the list
  final EdgeInsets? padding;

  @override
  Widget build(BuildContext context) => Padding(
        padding: padding ?? K1s0Spacing.allMd,
        child: Column(
          children: List.generate(
            itemCount,
            (index) => Padding(
              padding: const EdgeInsets.only(bottom: K1s0Spacing.sm),
              child: K1s0ShimmerLoading(
                height: itemHeight,
                borderRadius: BorderRadius.circular(8),
              ),
            ),
          ),
        ),
      );
}
