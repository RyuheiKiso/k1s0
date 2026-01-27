import 'package:flutter/material.dart';

import '../theme/k1s0_spacing.dart';

/// Form container with common form patterns
class K1s0FormContainer extends StatelessWidget {
  /// Creates a form container
  const K1s0FormContainer({
    required this.children,
    this.formKey,
    this.onSubmit,
    this.autovalidateMode,
    this.padding,
    this.spacing,
    super.key,
  });

  /// Form children
  final List<Widget> children;

  /// Form key for validation
  final GlobalKey<FormState>? formKey;

  /// Callback when form is submitted
  final VoidCallback? onSubmit;

  /// Auto-validate mode
  final AutovalidateMode? autovalidateMode;

  /// Form padding
  final EdgeInsets? padding;

  /// Spacing between fields
  final double? spacing;

  @override
  Widget build(BuildContext context) => Form(
        key: formKey,
        autovalidateMode: autovalidateMode,
        canPop: true,
        child: Padding(
          padding: padding ?? K1s0Spacing.allMd,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: _buildSpacedChildren(),
          ),
        ),
      );

  List<Widget> _buildSpacedChildren() {
    final spacer = SizedBox(height: spacing ?? K1s0Spacing.md);
    final result = <Widget>[];

    for (var i = 0; i < children.length; i++) {
      result.add(children[i]);
      if (i < children.length - 1) {
        result.add(spacer);
      }
    }

    return result;
  }
}

/// Form section with title
class K1s0FormSection extends StatelessWidget {
  /// Creates a form section
  const K1s0FormSection({
    required this.children,
    this.title,
    this.subtitle,
    this.padding,
    this.spacing,
    super.key,
  });

  /// Section children
  final List<Widget> children;

  /// Section title
  final String? title;

  /// Section subtitle
  final String? subtitle;

  /// Section padding
  final EdgeInsets? padding;

  /// Spacing between fields
  final double? spacing;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    final scheme = Theme.of(context).colorScheme;

    return Padding(
      padding: padding ?? EdgeInsets.zero,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (title != null) ...[
            Text(
              title!,
              style: textTheme.titleMedium,
            ),
            if (subtitle != null) ...[
              K1s0Spacing.gapXs,
              Text(
                subtitle!,
                style: textTheme.bodySmall?.copyWith(
                  color: scheme.onSurfaceVariant,
                ),
              ),
            ],
            K1s0Spacing.gapMd,
          ],
          ..._buildSpacedChildren(),
        ],
      ),
    );
  }

  List<Widget> _buildSpacedChildren() {
    final spacer = SizedBox(height: spacing ?? K1s0Spacing.md);
    final result = <Widget>[];

    for (var i = 0; i < children.length; i++) {
      result.add(children[i]);
      if (i < children.length - 1) {
        result.add(spacer);
      }
    }

    return result;
  }
}

/// Form actions row (submit/cancel buttons)
class K1s0FormActions extends StatelessWidget {
  /// Creates form actions
  const K1s0FormActions({
    required this.onSubmit,
    this.onCancel,
    this.submitLabel,
    this.cancelLabel,
    this.loading = false,
    this.submitDisabled = false,
    this.alignment = MainAxisAlignment.end,
    super.key,
  });

  /// Callback when submit is pressed
  final VoidCallback onSubmit;

  /// Callback when cancel is pressed
  final VoidCallback? onCancel;

  /// Submit button label
  final String? submitLabel;

  /// Cancel button label
  final String? cancelLabel;

  /// Whether submit is loading
  final bool loading;

  /// Whether submit is disabled
  final bool submitDisabled;

  /// Button alignment
  final MainAxisAlignment alignment;

  @override
  Widget build(BuildContext context) => Row(
        mainAxisAlignment: alignment,
        children: [
          if (onCancel != null) ...[
            TextButton(
              onPressed: loading ? null : onCancel,
              child: Text(cancelLabel ?? 'Cancel'),
            ),
            K1s0Spacing.gapHMd,
          ],
          FilledButton(
            onPressed: (loading || submitDisabled) ? null : onSubmit,
            child: loading
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: Colors.white,
                    ),
                  )
                : Text(submitLabel ?? 'Submit'),
          ),
        ],
      );
}
