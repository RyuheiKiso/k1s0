import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/src/widgets/buttons.dart';

void main() {
  group('K1s0PrimaryButton', () {
    testWidgets('renders child widget', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0PrimaryButton(
              onPressed: null,
              child: Text('Click Me'),
            ),
          ),
        ),
      );

      expect(find.text('Click Me'), findsOneWidget);
    });

    testWidgets('calls onPressed when tapped', (tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0PrimaryButton(
              onPressed: () => pressed = true,
              child: const Text('Click'),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(K1s0PrimaryButton));

      expect(pressed, true);
    });

    testWidgets('does not call onPressed when disabled', (tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0PrimaryButton(
              onPressed: () => pressed = true,
              disabled: true,
              child: const Text('Click'),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(K1s0PrimaryButton));

      expect(pressed, false);
    });

    testWidgets('does not call onPressed when loading', (tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0PrimaryButton(
              onPressed: () => pressed = true,
              loading: true,
              child: const Text('Click'),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(K1s0PrimaryButton));

      expect(pressed, false);
    });

    testWidgets('shows loading indicator when loading', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0PrimaryButton(
              onPressed: null,
              loading: true,
              child: Text('Click'),
            ),
          ),
        ),
      );

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('shows icon when provided', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0PrimaryButton(
              onPressed: null,
              icon: Icons.add,
              child: Text('Add'),
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.add), findsOneWidget);
    });

    testWidgets('expands to full width when fullWidth is true', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0PrimaryButton(
              onPressed: null,
              fullWidth: true,
              child: Text('Full Width'),
            ),
          ),
        ),
      );

      final sizedBox = tester.widget<SizedBox>(
        find.ancestor(
          of: find.byType(FilledButton),
          matching: find.byType(SizedBox),
        ),
      );
      expect(sizedBox.width, double.infinity);
    });
  });

  group('K1s0SecondaryButton', () {
    testWidgets('renders as OutlinedButton', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0SecondaryButton(
              onPressed: null,
              child: Text('Secondary'),
            ),
          ),
        ),
      );

      expect(find.byType(OutlinedButton), findsOneWidget);
    });

    testWidgets('shows loading indicator when loading', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0SecondaryButton(
              onPressed: null,
              loading: true,
              child: Text('Loading'),
            ),
          ),
        ),
      );

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });
  });

  group('K1s0TextButton', () {
    testWidgets('renders as TextButton', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0TextButton(
              onPressed: null,
              child: Text('Text'),
            ),
          ),
        ),
      );

      expect(find.byType(TextButton), findsOneWidget);
    });

    testWidgets('calls onPressed when tapped', (tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0TextButton(
              onPressed: () => pressed = true,
              child: const Text('Click'),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(K1s0TextButton));

      expect(pressed, true);
    });
  });

  group('K1s0IconButton', () {
    testWidgets('renders icon', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0IconButton(
              icon: Icons.settings,
              onPressed: null,
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.settings), findsOneWidget);
    });

    testWidgets('shows loading indicator when loading', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0IconButton(
              icon: Icons.settings,
              onPressed: null,
              loading: true,
            ),
          ),
        ),
      );

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.byIcon(Icons.settings), findsNothing);
    });

    testWidgets('shows tooltip when provided', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0IconButton(
              icon: Icons.settings,
              onPressed: null,
              tooltip: 'Settings',
            ),
          ),
        ),
      );

      expect(find.byType(Tooltip), findsOneWidget);
    });

    testWidgets('respects custom size', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0IconButton(
              icon: Icons.settings,
              onPressed: null,
              size: 32.0,
            ),
          ),
        ),
      );

      final icon = tester.widget<Icon>(find.byIcon(Icons.settings));
      expect(icon.size, 32.0);
    });
  });

  group('K1s0DangerButton', () {
    testWidgets('renders with error color', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0DangerButton(
              onPressed: null,
              child: Text('Delete'),
            ),
          ),
        ),
      );

      expect(find.byType(FilledButton), findsOneWidget);
    });

    testWidgets('calls onPressed when tapped', (tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0DangerButton(
              onPressed: () => pressed = true,
              child: const Text('Delete'),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(K1s0DangerButton));

      expect(pressed, true);
    });

    testWidgets('shows icon when provided', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0DangerButton(
              onPressed: null,
              icon: Icons.delete,
              child: Text('Delete'),
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.delete), findsOneWidget);
    });
  });
}
