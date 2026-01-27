import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'k1s0_theme.dart';

/// Theme mode state
enum K1s0ThemeMode {
  /// Use system setting
  system,

  /// Light mode
  light,

  /// Dark mode
  dark,
}

/// Theme state
class ThemeState {
  /// Creates a theme state
  const ThemeState({
    this.mode = K1s0ThemeMode.system,
    this.fontFamily,
  });

  /// Theme mode
  final K1s0ThemeMode mode;

  /// Custom font family
  final String? fontFamily;

  /// Create a copy with updated values
  ThemeState copyWith({
    K1s0ThemeMode? mode,
    String? fontFamily,
  }) {
    return ThemeState(
      mode: mode ?? this.mode,
      fontFamily: fontFamily ?? this.fontFamily,
    );
  }

  /// Get the light theme
  ThemeData get lightTheme => K1s0Theme.light(fontFamily: fontFamily);

  /// Get the dark theme
  ThemeData get darkTheme => K1s0Theme.dark(fontFamily: fontFamily);

  /// Get the theme mode for MaterialApp
  ThemeMode get themeMode {
    switch (mode) {
      case K1s0ThemeMode.system:
        return ThemeMode.system;
      case K1s0ThemeMode.light:
        return ThemeMode.light;
      case K1s0ThemeMode.dark:
        return ThemeMode.dark;
    }
  }
}

/// Theme notifier for managing theme state
class ThemeNotifier extends StateNotifier<ThemeState> {
  /// Creates a theme notifier
  ThemeNotifier([ThemeState? initial]) : super(initial ?? const ThemeState());

  /// Set theme mode
  void setMode(K1s0ThemeMode mode) {
    state = state.copyWith(mode: mode);
  }

  /// Set font family
  void setFontFamily(String? fontFamily) {
    state = state.copyWith(fontFamily: fontFamily);
  }

  /// Toggle between light and dark mode
  void toggle() {
    final newMode = state.mode == K1s0ThemeMode.light
        ? K1s0ThemeMode.dark
        : K1s0ThemeMode.light;
    setMode(newMode);
  }
}

/// Theme provider
final themeProvider = StateNotifierProvider<ThemeNotifier, ThemeState>((ref) {
  return ThemeNotifier();
});

/// Provider for light theme
final lightThemeProvider = Provider<ThemeData>((ref) {
  return ref.watch(themeProvider).lightTheme;
});

/// Provider for dark theme
final darkThemeProvider = Provider<ThemeData>((ref) {
  return ref.watch(themeProvider).darkTheme;
});

/// Provider for theme mode
final themeModeProvider = Provider<ThemeMode>((ref) {
  return ref.watch(themeProvider).themeMode;
});

/// k1s0 theme provider widget
class K1s0ThemeProvider extends ConsumerWidget {
  /// Creates a k1s0 theme provider
  const K1s0ThemeProvider({
    required this.child,
    this.initialMode = K1s0ThemeMode.system,
    this.fontFamily,
    super.key,
  });

  /// Child widget
  final Widget child;

  /// Initial theme mode
  final K1s0ThemeMode initialMode;

  /// Custom font family
  final String? fontFamily;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);

    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: themeState.lightTheme,
      darkTheme: themeState.darkTheme,
      themeMode: themeState.themeMode,
      home: child,
    );
  }
}

/// Extension methods for theme access
extension ThemeRef on WidgetRef {
  /// Get the current theme mode
  K1s0ThemeMode get themeMode => watch(themeProvider).mode;

  /// Set the theme mode
  void setThemeMode(K1s0ThemeMode mode) {
    read(themeProvider.notifier).setMode(mode);
  }

  /// Toggle theme mode
  void toggleTheme() {
    read(themeProvider.notifier).toggle();
  }

  /// Check if dark mode is active
  bool get isDarkMode {
    final mode = watch(themeProvider).mode;
    if (mode == K1s0ThemeMode.dark) return true;
    if (mode == K1s0ThemeMode.light) return false;
    // System mode - check platform brightness
    return WidgetsBinding.instance.platformDispatcher.platformBrightness ==
        Brightness.dark;
  }
}
