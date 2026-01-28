import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// k1s0 Flutter Integration Test Helpers
///
/// Common utilities for integration testing.

/// Test user credentials
class TestUser {
  final String email;
  final String password;
  final String name;

  const TestUser({
    required this.email,
    required this.password,
    required this.name,
  });

  static const defaultUser = TestUser(
    email: 'test@example.com',
    password: 'Test@Password123',
    name: 'Test User',
  );
}

/// Helper extension for WidgetTester
extension K1s0WidgetTesterExtension on WidgetTester {
  /// Pump the widget tree and settle animations
  Future<void> pumpAndSettleWithTimeout({Duration timeout = const Duration(seconds: 10)}) async {
    await pumpAndSettle(
      const Duration(milliseconds: 100),
      EnginePhase.sendSemanticsUpdate,
      timeout,
    );
  }

  /// Enter text into a TextField by label
  Future<void> enterTextByLabel(String label, String text) async {
    final finder = find.widgetWithText(TextField, label);
    await tap(finder);
    await enterText(finder, text);
    await pumpAndSettle();
  }

  /// Tap a button by text
  Future<void> tapButton(String text) async {
    final finder = find.widgetWithText(ElevatedButton, text);
    await tap(finder);
    await pumpAndSettle();
  }

  /// Verify snackbar message
  Future<void> expectSnackbar(String message) async {
    expect(find.text(message), findsOneWidget);
    await pumpAndSettle();
  }

  /// Wait for page navigation
  Future<void> waitForNavigation(String routeName) async {
    await pumpAndSettleWithTimeout();
    // Verification depends on implementation
  }

  /// Login with test user
  Future<void> login(TestUser user) async {
    await enterTextByLabel('Email', user.email);
    await enterTextByLabel('Password', user.password);
    await tapButton('Login');
    await pumpAndSettleWithTimeout();
  }
}

/// Screen size configurations for responsive testing
class ScreenSizes {
  static const mobile = Size(375, 667);
  static const tablet = Size(768, 1024);
  static const desktop = Size(1920, 1080);
}

/// Test data generators
class TestData {
  static String uniqueEmail() => 'test_${DateTime.now().millisecondsSinceEpoch}@example.com';
  static String uniqueUsername() => 'user_${DateTime.now().millisecondsSinceEpoch}';
}

/// Navigation test helper
class NavigationHelper {
  final WidgetTester tester;

  NavigationHelper(this.tester);

  /// Navigate using bottom navigation bar
  Future<void> tapBottomNavItem(int index) async {
    final bottomNav = find.byType(BottomNavigationBar);
    expect(bottomNav, findsOneWidget);

    final items = find.descendant(
      of: bottomNav,
      matching: find.byType(InkWell),
    );
    await tester.tap(items.at(index));
    await tester.pumpAndSettle();
  }

  /// Open drawer navigation
  Future<void> openDrawer() async {
    final scaffoldState = tester.state<ScaffoldState>(
      find.byType(Scaffold),
    );
    scaffoldState.openDrawer();
    await tester.pumpAndSettle();
  }

  /// Navigate using drawer item
  Future<void> tapDrawerItem(String text) async {
    await openDrawer();
    await tester.tap(find.text(text));
    await tester.pumpAndSettle();
  }
}

/// Scroll helper
class ScrollHelper {
  final WidgetTester tester;

  ScrollHelper(this.tester);

  /// Scroll until widget is visible
  Future<void> scrollUntilVisible(Finder finder, {double offset = 100}) async {
    await tester.scrollUntilVisible(
      finder,
      offset,
      scrollable: find.byType(Scrollable).first,
    );
  }

  /// Scroll to top
  Future<void> scrollToTop() async {
    await tester.drag(find.byType(Scrollable).first, const Offset(0, 500));
    await tester.pumpAndSettle();
  }

  /// Scroll to bottom
  Future<void> scrollToBottom() async {
    await tester.drag(find.byType(Scrollable).first, const Offset(0, -500));
    await tester.pumpAndSettle();
  }
}

/// Form test helper
class FormHelper {
  final WidgetTester tester;

  FormHelper(this.tester);

  /// Clear and enter text
  Future<void> clearAndEnterText(Finder finder, String text) async {
    await tester.tap(finder);
    await tester.enterText(finder, '');
    await tester.enterText(finder, text);
    await tester.pumpAndSettle();
  }

  /// Verify form validation error
  Future<void> expectValidationError(String message) async {
    expect(find.text(message), findsOneWidget);
  }

  /// Verify no validation errors
  Future<void> expectNoValidationErrors() async {
    expect(
      find.byWidgetPredicate(
        (widget) => widget is Text && widget.style?.color == Colors.red,
      ),
      findsNothing,
    );
  }
}
