import 'package:flutter/material.dart';

import '../theme/k1s0_spacing.dart';
import '../widgets/text_fields.dart';

/// Form field wrapper with label and error handling
class K1s0FormField extends StatelessWidget {
  /// Creates a form field
  const K1s0FormField({
    required this.child,
    this.label,
    this.required = false,
    this.helperText,
    this.errorText,
    super.key,
  });

  /// Field widget
  final Widget child;

  /// Label text
  final String? label;

  /// Whether the field is required
  final bool required;

  /// Helper text
  final String? helperText;

  /// Error text
  final String? errorText;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    final scheme = Theme.of(context).colorScheme;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (label != null) ...[
          Row(
            children: [
              Text(
                label!,
                style: textTheme.bodyMedium?.copyWith(
                  fontWeight: FontWeight.w500,
                ),
              ),
              if (required) ...[
                K1s0Spacing.gapHXs,
                Text(
                  '*',
                  style: textTheme.bodyMedium?.copyWith(
                    color: scheme.error,
                  ),
                ),
              ],
            ],
          ),
          K1s0Spacing.gapSm,
        ],
        child,
        if (helperText != null && errorText == null) ...[
          K1s0Spacing.gapXs,
          Text(
            helperText!,
            style: textTheme.bodySmall?.copyWith(
              color: scheme.onSurfaceVariant,
            ),
          ),
        ],
        if (errorText != null) ...[
          K1s0Spacing.gapXs,
          Text(
            errorText!,
            style: textTheme.bodySmall?.copyWith(
              color: scheme.error,
            ),
          ),
        ],
      ],
    );
  }
}

/// Pre-configured email field
class K1s0EmailField extends StatelessWidget {
  /// Creates an email field
  const K1s0EmailField({
    this.controller,
    this.label,
    this.required = false,
    this.helperText,
    this.errorText,
    this.enabled = true,
    this.onChanged,
    this.validator,
    super.key,
  });

  /// Text controller
  final TextEditingController? controller;

  /// Label text
  final String? label;

  /// Whether the field is required
  final bool required;

  /// Helper text
  final String? helperText;

  /// Error text
  final String? errorText;

  /// Whether the field is enabled
  final bool enabled;

  /// Callback when text changes
  final ValueChanged<String>? onChanged;

  /// Validator function
  final String? Function(String?)? validator;

  @override
  Widget build(BuildContext context) {
    return K1s0TextField(
      controller: controller,
      label: label ?? 'Email',
      hint: 'example@email.com',
      helperText: helperText,
      errorText: errorText,
      prefixIcon: Icons.email_outlined,
      enabled: enabled,
      onChanged: onChanged,
      validator: validator,
      keyboardType: TextInputType.emailAddress,
      textInputAction: TextInputAction.next,
    );
  }
}

/// Pre-configured phone field
class K1s0PhoneField extends StatelessWidget {
  /// Creates a phone field
  const K1s0PhoneField({
    this.controller,
    this.label,
    this.required = false,
    this.helperText,
    this.errorText,
    this.enabled = true,
    this.onChanged,
    this.validator,
    super.key,
  });

  /// Text controller
  final TextEditingController? controller;

  /// Label text
  final String? label;

  /// Whether the field is required
  final bool required;

  /// Helper text
  final String? helperText;

  /// Error text
  final String? errorText;

  /// Whether the field is enabled
  final bool enabled;

  /// Callback when text changes
  final ValueChanged<String>? onChanged;

  /// Validator function
  final String? Function(String?)? validator;

  @override
  Widget build(BuildContext context) {
    return K1s0TextField(
      controller: controller,
      label: label ?? 'Phone',
      hint: '090-1234-5678',
      helperText: helperText,
      errorText: errorText,
      prefixIcon: Icons.phone_outlined,
      enabled: enabled,
      onChanged: onChanged,
      validator: validator,
      keyboardType: TextInputType.phone,
      textInputAction: TextInputAction.next,
    );
  }
}
