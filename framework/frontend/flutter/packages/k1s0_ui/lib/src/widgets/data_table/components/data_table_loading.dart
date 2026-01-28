/// K1s0 DataTable ローディング
library;

import 'package:flutter/material.dart';

/// DataTable ローディングウィジェット（シマー効果）
class K1s0DataTableLoading extends StatelessWidget {
  /// 表示する行数
  final int rowCount;

  /// カラム数
  final int columnCount;

  /// 行高さ
  final double rowHeight;

  /// チェックボックス表示
  final bool showCheckbox;

  const K1s0DataTableLoading({
    super.key,
    this.rowCount = 5,
    this.columnCount = 4,
    this.rowHeight = 52,
    this.showCheckbox = false,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      children: List.generate(rowCount, (index) {
        return Container(
          height: rowHeight,
          decoration: BoxDecoration(
            border: Border(
              bottom: BorderSide(
                color: theme.dividerColor,
                width: 0.5,
              ),
            ),
          ),
          child: Row(
            children: [
              if (showCheckbox)
                const SizedBox(
                  width: 56,
                  child: _ShimmerBox(width: 24, height: 24),
                ),
              ...List.generate(columnCount, (colIndex) {
                return Expanded(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: _ShimmerBox(
                      height: 16,
                      width: colIndex == 0 ? 150 : 100,
                    ),
                  ),
                );
              }),
            ],
          ),
        );
      }),
    );
  }
}

/// シマーボックス
class _ShimmerBox extends StatefulWidget {
  final double width;
  final double height;

  const _ShimmerBox({
    required this.width,
    required this.height,
  });

  @override
  State<_ShimmerBox> createState() => _ShimmerBoxState();
}

class _ShimmerBoxState extends State<_ShimmerBox>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _animation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 1500),
      vsync: this,
    )..repeat();
    _animation = Tween<double>(begin: -1, end: 2).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeInOutSine),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final baseColor = theme.colorScheme.surfaceContainerHighest;
    final highlightColor = theme.colorScheme.surface;

    return AnimatedBuilder(
      animation: _animation,
      builder: (context, child) {
        return Container(
          width: widget.width,
          height: widget.height,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(4),
            gradient: LinearGradient(
              begin: Alignment(_animation.value - 1, 0),
              end: Alignment(_animation.value, 0),
              colors: [
                baseColor,
                highlightColor,
                baseColor,
              ],
            ),
          ),
        );
      },
    );
  }
}
