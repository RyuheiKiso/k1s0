/// K1s0 Form グリッド
library;

import 'package:flutter/material.dart';

/// フォームフィールドのグリッドレイアウト
class K1s0FormGrid extends StatelessWidget {
  /// 子ウィジェット
  final List<Widget> children;

  /// カラム数
  final int columns;

  /// 水平間隔
  final double horizontalSpacing;

  /// 垂直間隔
  final double verticalSpacing;

  const K1s0FormGrid({
    super.key,
    required this.children,
    this.columns = 1,
    this.horizontalSpacing = 16,
    this.verticalSpacing = 0,
  });

  @override
  Widget build(BuildContext context) {
    if (columns == 1) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: children,
      );
    }

    final rows = <Widget>[];
    for (var i = 0; i < children.length; i += columns) {
      final rowChildren = <Widget>[];
      for (var j = 0; j < columns && i + j < children.length; j++) {
        if (j > 0) {
          rowChildren.add(SizedBox(width: horizontalSpacing));
        }
        rowChildren.add(Expanded(child: children[i + j]));
      }
      // 残りのカラムを埋める
      while (rowChildren.length < columns * 2 - 1) {
        rowChildren.add(SizedBox(width: horizontalSpacing));
        rowChildren.add(const Expanded(child: SizedBox.shrink()));
      }
      rows.add(
        Padding(
          padding: EdgeInsets.only(bottom: verticalSpacing),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: rowChildren,
          ),
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: rows,
    );
  }
}
