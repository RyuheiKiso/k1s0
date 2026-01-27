import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// k1s0 text field
class K1s0TextField extends StatelessWidget {
  /// Creates a text field
  const K1s0TextField({
    this.controller,
    this.label,
    this.hint,
    this.helperText,
    this.errorText,
    this.prefixIcon,
    this.suffixIcon,
    this.obscureText = false,
    this.enabled = true,
    this.readOnly = false,
    this.maxLines = 1,
    this.minLines,
    this.maxLength,
    this.keyboardType,
    this.textInputAction,
    this.inputFormatters,
    this.onChanged,
    this.onSubmitted,
    this.onTap,
    this.validator,
    this.autofocus = false,
    this.focusNode,
    super.key,
  });

  /// Text controller
  final TextEditingController? controller;

  /// Label text
  final String? label;

  /// Hint text
  final String? hint;

  /// Helper text
  final String? helperText;

  /// Error text
  final String? errorText;

  /// Prefix icon
  final IconData? prefixIcon;

  /// Suffix icon or widget
  final Widget? suffixIcon;

  /// Whether to obscure text
  final bool obscureText;

  /// Whether the field is enabled
  final bool enabled;

  /// Whether the field is read-only
  final bool readOnly;

  /// Maximum number of lines
  final int? maxLines;

  /// Minimum number of lines
  final int? minLines;

  /// Maximum character length
  final int? maxLength;

  /// Keyboard type
  final TextInputType? keyboardType;

  /// Text input action
  final TextInputAction? textInputAction;

  /// Input formatters
  final List<TextInputFormatter>? inputFormatters;

  /// Callback when text changes
  final ValueChanged<String>? onChanged;

  /// Callback when submitted
  final ValueChanged<String>? onSubmitted;

  /// Callback when tapped
  final VoidCallback? onTap;

  /// Validator function
  final String? Function(String?)? validator;

  /// Whether to autofocus
  final bool autofocus;

  /// Focus node
  final FocusNode? focusNode;

  @override
  Widget build(BuildContext context) => TextFormField(
        controller: controller,
        decoration: InputDecoration(
          labelText: label,
          hintText: hint,
          helperText: helperText,
          errorText: errorText,
          prefixIcon: prefixIcon != null ? Icon(prefixIcon) : null,
          suffixIcon: suffixIcon,
        ),
        obscureText: obscureText,
        enabled: enabled,
        readOnly: readOnly,
        maxLines: obscureText ? 1 : maxLines,
        minLines: minLines,
        maxLength: maxLength,
        keyboardType: keyboardType,
        textInputAction: textInputAction,
        inputFormatters: inputFormatters,
        onChanged: onChanged,
        onFieldSubmitted: onSubmitted,
        onTap: onTap,
        validator: validator,
        autofocus: autofocus,
        focusNode: focusNode,
      );
}

/// k1s0 password field with visibility toggle
class K1s0PasswordField extends StatefulWidget {
  /// Creates a password field
  const K1s0PasswordField({
    this.controller,
    this.label,
    this.hint,
    this.helperText,
    this.errorText,
    this.enabled = true,
    this.onChanged,
    this.onSubmitted,
    this.validator,
    this.autofocus = false,
    this.focusNode,
    this.textInputAction,
    super.key,
  });

  /// Text controller
  final TextEditingController? controller;

  /// Label text
  final String? label;

  /// Hint text
  final String? hint;

  /// Helper text
  final String? helperText;

  /// Error text
  final String? errorText;

  /// Whether the field is enabled
  final bool enabled;

  /// Callback when text changes
  final ValueChanged<String>? onChanged;

  /// Callback when submitted
  final ValueChanged<String>? onSubmitted;

  /// Validator function
  final String? Function(String?)? validator;

  /// Whether to autofocus
  final bool autofocus;

  /// Focus node
  final FocusNode? focusNode;

  /// Text input action
  final TextInputAction? textInputAction;

  @override
  State<K1s0PasswordField> createState() => _K1s0PasswordFieldState();
}

class _K1s0PasswordFieldState extends State<K1s0PasswordField> {
  bool _obscureText = true;

  @override
  Widget build(BuildContext context) => K1s0TextField(
        controller: widget.controller,
        label: widget.label ?? 'Password',
        hint: widget.hint,
        helperText: widget.helperText,
        errorText: widget.errorText,
        prefixIcon: Icons.lock_outline,
        suffixIcon: IconButton(
          icon: Icon(
            _obscureText
                ? Icons.visibility_outlined
                : Icons.visibility_off_outlined,
          ),
          onPressed: () {
            setState(() {
              _obscureText = !_obscureText;
            });
          },
        ),
        obscureText: _obscureText,
        enabled: widget.enabled,
        onChanged: widget.onChanged,
        onSubmitted: widget.onSubmitted,
        validator: widget.validator,
        autofocus: widget.autofocus,
        focusNode: widget.focusNode,
        textInputAction: widget.textInputAction,
        keyboardType: TextInputType.visiblePassword,
      );
}

/// k1s0 search field
class K1s0SearchField extends StatelessWidget {
  /// Creates a search field
  const K1s0SearchField({
    this.controller,
    this.hint,
    this.onChanged,
    this.onSubmitted,
    this.onClear,
    this.autofocus = false,
    this.enabled = true,
    super.key,
  });

  /// Text controller
  final TextEditingController? controller;

  /// Hint text
  final String? hint;

  /// Callback when text changes
  final ValueChanged<String>? onChanged;

  /// Callback when submitted
  final ValueChanged<String>? onSubmitted;

  /// Callback when cleared
  final VoidCallback? onClear;

  /// Whether to autofocus
  final bool autofocus;

  /// Whether the field is enabled
  final bool enabled;

  @override
  Widget build(BuildContext context) => TextField(
        controller: controller,
        decoration: InputDecoration(
          hintText: hint ?? 'Search...',
          prefixIcon: const Icon(Icons.search),
          suffixIcon: (controller?.text.isNotEmpty ?? false)
              ? IconButton(
                  icon: const Icon(Icons.clear),
                  onPressed: () {
                    controller?.clear();
                    onClear?.call();
                    onChanged?.call('');
                  },
                )
              : null,
        ),
        onChanged: onChanged,
        onSubmitted: onSubmitted,
        autofocus: autofocus,
        enabled: enabled,
        textInputAction: TextInputAction.search,
      );
}
