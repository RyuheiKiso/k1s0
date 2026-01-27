import 'package:flutter/material.dart';

/// k1s0 color palette
class K1s0Colors {
  K1s0Colors._();

  // Primary colors
  /// Primary blue
  static const Color primary = Color(0xFF1976D2);

  /// Primary variant
  static const Color primaryVariant = Color(0xFF1565C0);

  /// On primary (text/icon color on primary background)
  static const Color onPrimary = Colors.white;

  // Secondary colors
  /// Secondary teal
  static const Color secondary = Color(0xFF26A69A);

  /// Secondary variant
  static const Color secondaryVariant = Color(0xFF00897B);

  /// On secondary
  static const Color onSecondary = Colors.white;

  // Tertiary colors
  /// Tertiary amber
  static const Color tertiary = Color(0xFFFFA726);

  /// On tertiary
  static const Color onTertiary = Colors.black;

  // Error colors
  /// Error red
  static const Color error = Color(0xFFD32F2F);

  /// Error variant
  static const Color errorVariant = Color(0xFFC62828);

  /// On error
  static const Color onError = Colors.white;

  // Success colors
  /// Success green
  static const Color success = Color(0xFF388E3C);

  /// On success
  static const Color onSuccess = Colors.white;

  // Warning colors
  /// Warning orange
  static const Color warning = Color(0xFFF57C00);

  /// On warning
  static const Color onWarning = Colors.black;

  // Info colors
  /// Info blue
  static const Color info = Color(0xFF0288D1);

  /// On info
  static const Color onInfo = Colors.white;

  // Surface colors (Light)
  /// Background (light)
  static const Color backgroundLight = Color(0xFFFAFAFA);

  /// Surface (light)
  static const Color surfaceLight = Colors.white;

  /// Surface variant (light)
  static const Color surfaceVariantLight = Color(0xFFF5F5F5);

  /// On surface (light)
  static const Color onSurfaceLight = Color(0xFF212121);

  /// On surface variant (light)
  static const Color onSurfaceVariantLight = Color(0xFF757575);

  /// Outline (light)
  static const Color outlineLight = Color(0xFFBDBDBD);

  /// Outline variant (light)
  static const Color outlineVariantLight = Color(0xFFE0E0E0);

  // Surface colors (Dark)
  /// Background (dark)
  static const Color backgroundDark = Color(0xFF121212);

  /// Surface (dark)
  static const Color surfaceDark = Color(0xFF1E1E1E);

  /// Surface variant (dark)
  static const Color surfaceVariantDark = Color(0xFF2C2C2C);

  /// On surface (dark)
  static const Color onSurfaceDark = Color(0xFFE0E0E0);

  /// On surface variant (dark)
  static const Color onSurfaceVariantDark = Color(0xFF9E9E9E);

  /// Outline (dark)
  static const Color outlineDark = Color(0xFF616161);

  /// Outline variant (dark)
  static const Color outlineVariantDark = Color(0xFF424242);

  /// Create a light color scheme
  static ColorScheme get lightScheme => const ColorScheme.light(
        primary: primary,
        onPrimary: onPrimary,
        primaryContainer: Color(0xFFBBDEFB),
        onPrimaryContainer: Color(0xFF0D47A1),
        secondary: secondary,
        onSecondary: onSecondary,
        secondaryContainer: Color(0xFFB2DFDB),
        onSecondaryContainer: Color(0xFF004D40),
        tertiary: tertiary,
        onTertiary: onTertiary,
        tertiaryContainer: Color(0xFFFFE0B2),
        onTertiaryContainer: Color(0xFFE65100),
        error: error,
        onError: onError,
        errorContainer: Color(0xFFFFCDD2),
        onErrorContainer: Color(0xFFB71C1C),
        surface: surfaceLight,
        onSurface: onSurfaceLight,
        surfaceContainerHighest: surfaceVariantLight,
        onSurfaceVariant: onSurfaceVariantLight,
        outline: outlineLight,
        outlineVariant: outlineVariantLight,
      );

  /// Create a dark color scheme
  static ColorScheme get darkScheme => const ColorScheme.dark(
        primary: Color(0xFF64B5F6),
        onPrimary: Color(0xFF0D47A1),
        primaryContainer: Color(0xFF1565C0),
        onPrimaryContainer: Color(0xFFBBDEFB),
        secondary: Color(0xFF80CBC4),
        onSecondary: Color(0xFF004D40),
        secondaryContainer: Color(0xFF00897B),
        onSecondaryContainer: Color(0xFFB2DFDB),
        tertiary: Color(0xFFFFCC80),
        onTertiary: Color(0xFFE65100),
        tertiaryContainer: Color(0xFFF57C00),
        onTertiaryContainer: Color(0xFFFFE0B2),
        error: Color(0xFFEF9A9A),
        onError: Color(0xFFB71C1C),
        errorContainer: Color(0xFFC62828),
        onErrorContainer: Color(0xFFFFCDD2),
        surface: surfaceDark,
        onSurface: onSurfaceDark,
        surfaceContainerHighest: surfaceVariantDark,
        onSurfaceVariant: onSurfaceVariantDark,
        outline: outlineDark,
        outlineVariant: outlineVariantDark,
      );
}
