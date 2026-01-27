import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/src/theme/k1s0_colors.dart';
import 'package:k1s0_ui/src/theme/k1s0_spacing.dart';
import 'package:k1s0_ui/src/theme/k1s0_theme.dart';
import 'package:k1s0_ui/src/theme/k1s0_typography.dart';

void main() {
  group('K1s0Theme', () {
    group('light', () {
      test('creates valid ThemeData', () {
        final theme = K1s0Theme.light();

        expect(theme, isA<ThemeData>());
        expect(theme.brightness, Brightness.light);
        expect(theme.useMaterial3, true);
      });

      test('uses default color scheme', () {
        final theme = K1s0Theme.light();

        expect(theme.colorScheme.brightness, Brightness.light);
      });

      test('uses custom color scheme when provided', () {
        final customScheme = ColorScheme.fromSeed(
          seedColor: Colors.purple,
          brightness: Brightness.light,
        );
        final theme = K1s0Theme.light(colorScheme: customScheme);

        expect(theme.colorScheme.primary, customScheme.primary);
      });

      test('applies custom font family', () {
        final theme = K1s0Theme.light(fontFamily: 'Roboto');

        expect(theme.textTheme.bodyLarge?.fontFamily, 'Roboto');
      });

      test('configures AppBar theme', () {
        final theme = K1s0Theme.light();

        expect(theme.appBarTheme.backgroundColor, isNotNull);
        expect(theme.appBarTheme.elevation, K1s0Elevation.none);
      });

      test('configures Card theme', () {
        final theme = K1s0Theme.light();

        expect(theme.cardTheme.elevation, K1s0Elevation.level1);
        expect(theme.cardTheme.shape, isA<RoundedRectangleBorder>());
      });

      test('configures Button themes', () {
        final theme = K1s0Theme.light();

        expect(theme.elevatedButtonTheme.style, isNotNull);
        expect(theme.filledButtonTheme.style, isNotNull);
        expect(theme.outlinedButtonTheme.style, isNotNull);
        expect(theme.textButtonTheme.style, isNotNull);
      });

      test('configures Input decoration theme', () {
        final theme = K1s0Theme.light();

        expect(theme.inputDecorationTheme.filled, true);
        expect(theme.inputDecorationTheme.border, isA<OutlineInputBorder>());
      });

      test('configures Dialog theme', () {
        final theme = K1s0Theme.light();

        expect(theme.dialogTheme.shape, isA<RoundedRectangleBorder>());
        expect(theme.dialogTheme.elevation, K1s0Elevation.level3);
      });

      test('configures SnackBar theme', () {
        final theme = K1s0Theme.light();

        expect(theme.snackBarTheme.behavior, SnackBarBehavior.floating);
      });
    });

    group('dark', () {
      test('creates valid ThemeData', () {
        final theme = K1s0Theme.dark();

        expect(theme, isA<ThemeData>());
        expect(theme.brightness, Brightness.dark);
        expect(theme.useMaterial3, true);
      });

      test('uses dark color scheme', () {
        final theme = K1s0Theme.dark();

        expect(theme.colorScheme.brightness, Brightness.dark);
      });

      test('uses custom color scheme when provided', () {
        final customScheme = ColorScheme.fromSeed(
          seedColor: Colors.purple,
          brightness: Brightness.dark,
        );
        final theme = K1s0Theme.dark(colorScheme: customScheme);

        expect(theme.colorScheme.primary, customScheme.primary);
      });

      test('applies custom font family', () {
        final theme = K1s0Theme.dark(fontFamily: 'Roboto');

        expect(theme.textTheme.bodyLarge?.fontFamily, 'Roboto');
      });
    });
  });

  group('K1s0Colors', () {
    test('lightScheme has light brightness', () {
      expect(K1s0Colors.lightScheme.brightness, Brightness.light);
    });

    test('darkScheme has dark brightness', () {
      expect(K1s0Colors.darkScheme.brightness, Brightness.dark);
    });

    test('schemes have required colors', () {
      expect(K1s0Colors.lightScheme.primary, isNotNull);
      expect(K1s0Colors.lightScheme.secondary, isNotNull);
      expect(K1s0Colors.lightScheme.error, isNotNull);
      expect(K1s0Colors.lightScheme.surface, isNotNull);

      expect(K1s0Colors.darkScheme.primary, isNotNull);
      expect(K1s0Colors.darkScheme.secondary, isNotNull);
      expect(K1s0Colors.darkScheme.error, isNotNull);
      expect(K1s0Colors.darkScheme.surface, isNotNull);
    });
  });

  group('K1s0Spacing', () {
    test('spacing values are correct', () {
      expect(K1s0Spacing.xs, 4.0);
      expect(K1s0Spacing.sm, 8.0);
      expect(K1s0Spacing.md, 16.0);
      expect(K1s0Spacing.lg, 24.0);
      expect(K1s0Spacing.xl, 32.0);
      expect(K1s0Spacing.xxl, 48.0);
    });

    test('gap widgets are SizedBox', () {
      expect(K1s0Spacing.gapXs, isA<SizedBox>());
      expect(K1s0Spacing.gapSm, isA<SizedBox>());
      expect(K1s0Spacing.gapMd, isA<SizedBox>());
      expect(K1s0Spacing.gapLg, isA<SizedBox>());
    });

    test('horizontal gap widgets have correct width', () {
      expect((K1s0Spacing.gapHSm as SizedBox).width, K1s0Spacing.sm);
      expect((K1s0Spacing.gapHMd as SizedBox).width, K1s0Spacing.md);
    });

    test('EdgeInsets constants are correct', () {
      expect(K1s0Spacing.allSm, const EdgeInsets.all(8.0));
      expect(K1s0Spacing.allMd, const EdgeInsets.all(16.0));
      expect(K1s0Spacing.allLg, const EdgeInsets.all(24.0));
    });
  });

  group('K1s0Typography', () {
    test('textTheme has all required styles', () {
      final textTheme = K1s0Typography.textTheme;

      expect(textTheme.displayLarge, isNotNull);
      expect(textTheme.displayMedium, isNotNull);
      expect(textTheme.displaySmall, isNotNull);
      expect(textTheme.headlineLarge, isNotNull);
      expect(textTheme.headlineMedium, isNotNull);
      expect(textTheme.headlineSmall, isNotNull);
      expect(textTheme.titleLarge, isNotNull);
      expect(textTheme.titleMedium, isNotNull);
      expect(textTheme.titleSmall, isNotNull);
      expect(textTheme.bodyLarge, isNotNull);
      expect(textTheme.bodyMedium, isNotNull);
      expect(textTheme.bodySmall, isNotNull);
      expect(textTheme.labelLarge, isNotNull);
      expect(textTheme.labelMedium, isNotNull);
      expect(textTheme.labelSmall, isNotNull);
    });

    test('applyFontFamily applies font to all styles', () {
      final textTheme = K1s0Typography.applyFontFamily(
        K1s0Typography.textTheme,
        'CustomFont',
      );

      expect(textTheme.bodyLarge?.fontFamily, 'CustomFont');
      expect(textTheme.titleLarge?.fontFamily, 'CustomFont');
      expect(textTheme.labelSmall?.fontFamily, 'CustomFont');
    });
  });

  group('K1s0Elevation', () {
    test('elevation values are correct', () {
      expect(K1s0Elevation.none, 0.0);
      expect(K1s0Elevation.level1, 1.0);
      expect(K1s0Elevation.level2, 3.0);
      expect(K1s0Elevation.level3, 6.0);
      expect(K1s0Elevation.level4, 8.0);
      expect(K1s0Elevation.level5, 12.0);
    });
  });

  group('K1s0Radius', () {
    test('border radius values are correct', () {
      expect(K1s0Radius.borderSm.topLeft.x, 4.0);
      expect(K1s0Radius.borderMd.topLeft.x, 8.0);
      expect(K1s0Radius.borderLg.topLeft.x, 12.0);
      expect(K1s0Radius.borderXl.topLeft.x, 16.0);
      expect(K1s0Radius.borderFull.topLeft.x, 9999.0);
    });
  });
}
