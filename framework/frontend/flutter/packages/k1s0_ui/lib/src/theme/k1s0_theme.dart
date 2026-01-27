import 'package:flutter/material.dart';

import 'k1s0_colors.dart';
import 'k1s0_spacing.dart';
import 'k1s0_typography.dart';

/// k1s0 theme data factory
class K1s0Theme {
  /// Private constructor to prevent instantiation.
  K1s0Theme._();

  /// Create the light theme
  static ThemeData light({
    String? fontFamily,
    ColorScheme? colorScheme,
  }) {
    final scheme = colorScheme ?? K1s0Colors.lightScheme;
    final textTheme = fontFamily != null
        ? K1s0Typography.applyFontFamily(K1s0Typography.textTheme, fontFamily)
        : K1s0Typography.textTheme;

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      colorScheme: scheme,
      textTheme: textTheme,

      // AppBar
      appBarTheme: AppBarTheme(
        backgroundColor: scheme.surface,
        foregroundColor: scheme.onSurface,
        elevation: K1s0Elevation.none,
        scrolledUnderElevation: K1s0Elevation.level1,
        centerTitle: false,
        titleTextStyle: textTheme.titleLarge?.copyWith(
          color: scheme.onSurface,
        ),
      ),

      // Card
      cardTheme: const CardThemeData(
        elevation: K1s0Elevation.level1,
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderMd,
        ),
        margin: K1s0Spacing.allSm,
      ),

      // Elevated Button
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          elevation: K1s0Elevation.level1,
          padding: const EdgeInsets.symmetric(
            horizontal: K1s0Spacing.md,
            vertical: K1s0Spacing.sm,
          ),
          shape: const RoundedRectangleBorder(
            borderRadius: K1s0Radius.borderMd,
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),

      // Filled Button
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          padding: const EdgeInsets.symmetric(
            horizontal: K1s0Spacing.md,
            vertical: K1s0Spacing.sm,
          ),
          shape: const RoundedRectangleBorder(
            borderRadius: K1s0Radius.borderMd,
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),

      // Outlined Button
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          padding: const EdgeInsets.symmetric(
            horizontal: K1s0Spacing.md,
            vertical: K1s0Spacing.sm,
          ),
          shape: const RoundedRectangleBorder(
            borderRadius: K1s0Radius.borderMd,
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),

      // Text Button
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          padding: const EdgeInsets.symmetric(
            horizontal: K1s0Spacing.md,
            vertical: K1s0Spacing.sm,
          ),
          shape: const RoundedRectangleBorder(
            borderRadius: K1s0Radius.borderMd,
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),

      // Input Decoration
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: scheme.surfaceContainerHighest.withValues(alpha: 0.5),
        border: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.outline),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.outline),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.primary, width: 2),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.error),
        ),
        focusedErrorBorder: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.error, width: 2),
        ),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: K1s0Spacing.md,
          vertical: K1s0Spacing.sm,
        ),
        labelStyle: textTheme.bodyMedium,
        hintStyle: textTheme.bodyMedium?.copyWith(
          color: scheme.onSurfaceVariant,
        ),
        errorStyle: textTheme.bodySmall?.copyWith(
          color: scheme.error,
        ),
      ),

      // Chip
      chipTheme: const ChipThemeData(
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderSm,
        ),
        padding: EdgeInsets.symmetric(
          horizontal: K1s0Spacing.sm,
          vertical: K1s0Spacing.xs,
        ),
      ),

      // Dialog
      dialogTheme: const DialogThemeData(
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderLg,
        ),
        elevation: K1s0Elevation.level3,
      ),

      // SnackBar
      snackBarTheme: const SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderMd,
        ),
      ),

      // Divider
      dividerTheme: DividerThemeData(
        color: scheme.outlineVariant,
        thickness: 1,
        space: K1s0Spacing.md,
      ),

      // Bottom Navigation Bar
      bottomNavigationBarTheme: BottomNavigationBarThemeData(
        elevation: K1s0Elevation.level2,
        type: BottomNavigationBarType.fixed,
        selectedItemColor: scheme.primary,
        unselectedItemColor: scheme.onSurfaceVariant,
      ),

      // Navigation Rail
      navigationRailTheme: NavigationRailThemeData(
        elevation: K1s0Elevation.level1,
        labelType: NavigationRailLabelType.all,
        selectedIconTheme: IconThemeData(color: scheme.primary),
        unselectedIconTheme: IconThemeData(color: scheme.onSurfaceVariant),
      ),

      // Floating Action Button
      floatingActionButtonTheme: const FloatingActionButtonThemeData(
        elevation: K1s0Elevation.level3,
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderLg,
        ),
      ),
    );
  }

  /// Create the dark theme
  static ThemeData dark({
    String? fontFamily,
    ColorScheme? colorScheme,
  }) {
    final scheme = colorScheme ?? K1s0Colors.darkScheme;
    final textTheme = fontFamily != null
        ? K1s0Typography.applyFontFamily(K1s0Typography.textTheme, fontFamily)
        : K1s0Typography.textTheme;

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      colorScheme: scheme,
      textTheme: textTheme,

      // AppBar
      appBarTheme: AppBarTheme(
        backgroundColor: scheme.surface,
        foregroundColor: scheme.onSurface,
        elevation: K1s0Elevation.none,
        scrolledUnderElevation: K1s0Elevation.level1,
        centerTitle: false,
        titleTextStyle: textTheme.titleLarge?.copyWith(
          color: scheme.onSurface,
        ),
      ),

      // Card
      cardTheme: const CardThemeData(
        elevation: K1s0Elevation.level1,
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderMd,
        ),
        margin: K1s0Spacing.allSm,
      ),

      // Elevated Button
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          elevation: K1s0Elevation.level1,
          padding: const EdgeInsets.symmetric(
            horizontal: K1s0Spacing.md,
            vertical: K1s0Spacing.sm,
          ),
          shape: const RoundedRectangleBorder(
            borderRadius: K1s0Radius.borderMd,
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),

      // Filled Button
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          padding: const EdgeInsets.symmetric(
            horizontal: K1s0Spacing.md,
            vertical: K1s0Spacing.sm,
          ),
          shape: const RoundedRectangleBorder(
            borderRadius: K1s0Radius.borderMd,
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),

      // Outlined Button
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          padding: const EdgeInsets.symmetric(
            horizontal: K1s0Spacing.md,
            vertical: K1s0Spacing.sm,
          ),
          shape: const RoundedRectangleBorder(
            borderRadius: K1s0Radius.borderMd,
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),

      // Text Button
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          padding: const EdgeInsets.symmetric(
            horizontal: K1s0Spacing.md,
            vertical: K1s0Spacing.sm,
          ),
          shape: const RoundedRectangleBorder(
            borderRadius: K1s0Radius.borderMd,
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),

      // Input Decoration
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: scheme.surfaceContainerHighest.withValues(alpha: 0.5),
        border: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.outline),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.outline),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.primary, width: 2),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.error),
        ),
        focusedErrorBorder: OutlineInputBorder(
          borderRadius: K1s0Radius.borderMd,
          borderSide: BorderSide(color: scheme.error, width: 2),
        ),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: K1s0Spacing.md,
          vertical: K1s0Spacing.sm,
        ),
        labelStyle: textTheme.bodyMedium,
        hintStyle: textTheme.bodyMedium?.copyWith(
          color: scheme.onSurfaceVariant,
        ),
        errorStyle: textTheme.bodySmall?.copyWith(
          color: scheme.error,
        ),
      ),

      // Chip
      chipTheme: const ChipThemeData(
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderSm,
        ),
        padding: EdgeInsets.symmetric(
          horizontal: K1s0Spacing.sm,
          vertical: K1s0Spacing.xs,
        ),
      ),

      // Dialog
      dialogTheme: const DialogThemeData(
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderLg,
        ),
        elevation: K1s0Elevation.level3,
      ),

      // SnackBar
      snackBarTheme: const SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderMd,
        ),
      ),

      // Divider
      dividerTheme: DividerThemeData(
        color: scheme.outlineVariant,
        thickness: 1,
        space: K1s0Spacing.md,
      ),

      // Bottom Navigation Bar
      bottomNavigationBarTheme: BottomNavigationBarThemeData(
        elevation: K1s0Elevation.level2,
        type: BottomNavigationBarType.fixed,
        selectedItemColor: scheme.primary,
        unselectedItemColor: scheme.onSurfaceVariant,
      ),

      // Navigation Rail
      navigationRailTheme: NavigationRailThemeData(
        elevation: K1s0Elevation.level1,
        labelType: NavigationRailLabelType.all,
        selectedIconTheme: IconThemeData(color: scheme.primary),
        unselectedIconTheme: IconThemeData(color: scheme.onSurfaceVariant),
      ),

      // Floating Action Button
      floatingActionButtonTheme: const FloatingActionButtonThemeData(
        elevation: K1s0Elevation.level3,
        shape: RoundedRectangleBorder(
          borderRadius: K1s0Radius.borderLg,
        ),
      ),
    );
  }
}
