import 'package:flutter/material.dart';

/// k1s0 spacing constants
class K1s0Spacing {
  /// Private constructor to prevent instantiation.
  K1s0Spacing._();

  /// Base spacing unit (4px)
  static const double unit = 4;

  /// Extra small spacing (4px)
  static const double xs = 4;

  /// Small spacing (8px)
  static const double sm = 8;

  /// Medium spacing (16px)
  static const double md = 16;

  /// Large spacing (24px)
  static const double lg = 24;

  /// Extra large spacing (32px)
  static const double xl = 32;

  /// 2x extra large spacing (48px)
  static const double xxl = 48;

  /// 3x extra large spacing (64px)
  static const double xxxl = 64;

  // Edge insets helpers

  /// All sides - extra small
  static const EdgeInsets allXs = EdgeInsets.all(xs);

  /// All sides - small
  static const EdgeInsets allSm = EdgeInsets.all(sm);

  /// All sides - medium
  static const EdgeInsets allMd = EdgeInsets.all(md);

  /// All sides - large
  static const EdgeInsets allLg = EdgeInsets.all(lg);

  /// All sides - extra large
  static const EdgeInsets allXl = EdgeInsets.all(xl);

  /// Horizontal - small
  static const EdgeInsets horizontalSm = EdgeInsets.symmetric(horizontal: sm);

  /// Horizontal - medium
  static const EdgeInsets horizontalMd = EdgeInsets.symmetric(horizontal: md);

  /// Horizontal - large
  static const EdgeInsets horizontalLg = EdgeInsets.symmetric(horizontal: lg);

  /// Vertical - small
  static const EdgeInsets verticalSm = EdgeInsets.symmetric(vertical: sm);

  /// Vertical - medium
  static const EdgeInsets verticalMd = EdgeInsets.symmetric(vertical: md);

  /// Vertical - large
  static const EdgeInsets verticalLg = EdgeInsets.symmetric(vertical: lg);

  // Gap helpers (for Column/Row)

  /// Vertical gap - extra small
  static const SizedBox gapXs = SizedBox(height: xs);

  /// Vertical gap - small
  static const SizedBox gapSm = SizedBox(height: sm);

  /// Vertical gap - medium
  static const SizedBox gapMd = SizedBox(height: md);

  /// Vertical gap - large
  static const SizedBox gapLg = SizedBox(height: lg);

  /// Vertical gap - extra large
  static const SizedBox gapXl = SizedBox(height: xl);

  /// Horizontal gap - extra small
  static const SizedBox gapHXs = SizedBox(width: xs);

  /// Horizontal gap - small
  static const SizedBox gapHSm = SizedBox(width: sm);

  /// Horizontal gap - medium
  static const SizedBox gapHMd = SizedBox(width: md);

  /// Horizontal gap - large
  static const SizedBox gapHLg = SizedBox(width: lg);

  /// Horizontal gap - extra large
  static const SizedBox gapHXl = SizedBox(width: xl);
}

/// k1s0 border radius constants
class K1s0Radius {
  /// Private constructor to prevent instantiation.
  K1s0Radius._();

  /// No radius
  static const double none = 0;

  /// Small radius (4px)
  static const double sm = 4;

  /// Medium radius (8px)
  static const double md = 8;

  /// Large radius (12px)
  static const double lg = 12;

  /// Extra large radius (16px)
  static const double xl = 16;

  /// Full/pill radius (9999px)
  static const double full = 9999;

  /// Small border radius
  static const BorderRadius borderSm = BorderRadius.all(Radius.circular(sm));

  /// Medium border radius
  static const BorderRadius borderMd = BorderRadius.all(Radius.circular(md));

  /// Large border radius
  static const BorderRadius borderLg = BorderRadius.all(Radius.circular(lg));

  /// Extra large border radius
  static const BorderRadius borderXl = BorderRadius.all(Radius.circular(xl));

  /// Full/pill border radius
  static const BorderRadius borderFull = BorderRadius.all(Radius.circular(full));
}

/// k1s0 elevation constants
class K1s0Elevation {
  /// Private constructor to prevent instantiation.
  K1s0Elevation._();

  /// No elevation
  static const double none = 0;

  /// Level 1 elevation (1dp)
  static const double level1 = 1;

  /// Level 2 elevation (3dp)
  static const double level2 = 3;

  /// Level 3 elevation (6dp)
  static const double level3 = 6;

  /// Level 4 elevation (8dp)
  static const double level4 = 8;

  /// Level 5 elevation (12dp)
  static const double level5 = 12;
}
