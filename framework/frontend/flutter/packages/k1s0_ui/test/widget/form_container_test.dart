import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/src/form/form_container.dart';

void main() {
  group('K1s0FormContainer', () {
    testWidgets('renders form with children', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormContainer(
              children: [
                TextField(key: Key('field1')),
                TextField(key: Key('field2')),
              ],
            ),
          ),
        ),
      );

      expect(find.byKey(const Key('field1')), findsOneWidget);
      expect(find.byKey(const Key('field2')), findsOneWidget);
    });

    testWidgets('wraps children in Form widget', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormContainer(
              children: [TextField()],
            ),
          ),
        ),
      );

      expect(find.byType(Form), findsOneWidget);
    });

    testWidgets('uses provided formKey', (tester) async {
      final formKey = GlobalKey<FormState>();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormContainer(
              formKey: formKey,
              children: const [TextField()],
            ),
          ),
        ),
      );

      expect(formKey.currentState, isNotNull);
    });

    testWidgets('adds spacing between children', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormContainer(
              children: [
                TextField(key: Key('field1')),
                TextField(key: Key('field2')),
              ],
            ),
          ),
        ),
      );

      // Should find SizedBox widgets used as spacers
      expect(find.byType(SizedBox), findsWidgets);
    });

    testWidgets('uses custom spacing', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormContainer(
              spacing: 24.0,
              children: [
                TextField(key: Key('field1')),
                TextField(key: Key('field2')),
              ],
            ),
          ),
        ),
      );

      final sizedBoxes = tester.widgetList<SizedBox>(find.byType(SizedBox));
      expect(
        sizedBoxes.any((sb) => sb.height == 24.0),
        true,
      );
    });

    testWidgets('applies custom padding', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormContainer(
              padding: EdgeInsets.all(24.0),
              children: [TextField()],
            ),
          ),
        ),
      );

      final padding = tester.widget<Padding>(find.byType(Padding).first);
      expect(padding.padding, const EdgeInsets.all(24.0));
    });
  });

  group('K1s0FormSection', () {
    testWidgets('renders children', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormSection(
              children: [
                TextField(key: Key('field1')),
              ],
            ),
          ),
        ),
      );

      expect(find.byKey(const Key('field1')), findsOneWidget);
    });

    testWidgets('renders title when provided', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormSection(
              title: 'Personal Information',
              children: [TextField()],
            ),
          ),
        ),
      );

      expect(find.text('Personal Information'), findsOneWidget);
    });

    testWidgets('renders subtitle when provided', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormSection(
              title: 'Personal Information',
              subtitle: 'Enter your details below',
              children: [TextField()],
            ),
          ),
        ),
      );

      expect(find.text('Enter your details below'), findsOneWidget);
    });

    testWidgets('does not render title when not provided', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: K1s0FormSection(
              children: [TextField()],
            ),
          ),
        ),
      );

      // Should only find TextField text widgets, no title
      final textWidgets = tester.widgetList<Text>(find.byType(Text));
      expect(textWidgets.isEmpty, true);
    });
  });

  group('K1s0FormActions', () {
    testWidgets('renders submit button', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormActions(
              onSubmit: () {},
            ),
          ),
        ),
      );

      expect(find.byType(FilledButton), findsOneWidget);
    });

    testWidgets('renders cancel button when onCancel provided', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormActions(
              onSubmit: () {},
              onCancel: () {},
            ),
          ),
        ),
      );

      expect(find.byType(TextButton), findsOneWidget);
      expect(find.byType(FilledButton), findsOneWidget);
    });

    testWidgets('uses custom button labels', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormActions(
              onSubmit: () {},
              onCancel: () {},
              submitLabel: 'Save',
              cancelLabel: 'Discard',
            ),
          ),
        ),
      );

      expect(find.text('Save'), findsOneWidget);
      expect(find.text('Discard'), findsOneWidget);
    });

    testWidgets('calls onSubmit when submit button pressed', (tester) async {
      var submitted = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormActions(
              onSubmit: () => submitted = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(FilledButton));

      expect(submitted, true);
    });

    testWidgets('calls onCancel when cancel button pressed', (tester) async {
      var cancelled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormActions(
              onSubmit: () {},
              onCancel: () => cancelled = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(TextButton));

      expect(cancelled, true);
    });

    testWidgets('shows loading indicator when loading', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormActions(
              onSubmit: () {},
              loading: true,
            ),
          ),
        ),
      );

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('disables submit when submitDisabled is true', (tester) async {
      var submitted = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormActions(
              onSubmit: () => submitted = true,
              submitDisabled: true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(FilledButton));

      expect(submitted, false);
    });

    testWidgets('disables buttons when loading', (tester) async {
      var submitted = false;
      var cancelled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: K1s0FormActions(
              onSubmit: () => submitted = true,
              onCancel: () => cancelled = true,
              loading: true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(FilledButton));
      await tester.tap(find.byType(TextButton));

      expect(submitted, false);
      expect(cancelled, false);
    });
  });
}
