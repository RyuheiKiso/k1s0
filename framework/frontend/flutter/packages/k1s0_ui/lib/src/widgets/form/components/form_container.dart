/// K1s0 Form コンテナ
library;

import 'package:flutter/material.dart';

/// フォームコンテナウィジェット
class K1s0FormContainer extends StatelessWidget {
  /// 子ウィジェット
  final Widget child;

  /// フォームキー
  final GlobalKey<FormState>? formKey;

  /// パディング
  final EdgeInsets? padding;

  const K1s0FormContainer({
    super.key,
    required this.child,
    this.formKey,
    this.padding,
  });

  @override
  Widget build(BuildContext context) {
    return Form(
      key: formKey,
      child: Padding(
        padding: padding ?? const EdgeInsets.all(16),
        child: child,
      ),
    );
  }
}
