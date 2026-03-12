import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_app_updater/app_updater.dart';

void main() {
  testWidgets('optional update dialog can be dismissed', (tester) async {
    final updater = InMemoryAppUpdater(
      versionInfo: const AppVersionInfo(
        latestVersion: '2.1.0',
        minimumVersion: '1.0.0',
        releaseNotes: 'Bug fixes',
      ),
      currentVersion: '2.0.0',
      storeOpener: (_) async => true,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              onPressed: () async {
                await UpdateDialog.show(
                  context: context,
                  result: const UpdateCheckResult(
                    type: UpdateType.optional,
                    currentVersion: '2.0.0',
                    versionInfo: AppVersionInfo(
                      latestVersion: '2.1.0',
                      minimumVersion: '1.0.0',
                      releaseNotes: 'Bug fixes',
                    ),
                  ),
                  updater: updater,
                );
              },
              child: const Text('open'),
            );
          },
        ),
      ),
    );

    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();

    expect(find.text('新しいバージョンがあります'), findsOneWidget);
    expect(find.text('後で'), findsOneWidget);
  });

  testWidgets('mandatory update dialog hides dismiss action', (tester) async {
    final updater = InMemoryAppUpdater(
      versionInfo: const AppVersionInfo(
        latestVersion: '3.0.0',
        minimumVersion: '3.0.0',
        mandatory: true,
      ),
      currentVersion: '2.0.0',
      storeOpener: (_) async => true,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              onPressed: () async {
                await UpdateDialog.show(
                  context: context,
                  result: const UpdateCheckResult(
                    type: UpdateType.mandatory,
                    currentVersion: '2.0.0',
                    versionInfo: AppVersionInfo(
                      latestVersion: '3.0.0',
                      minimumVersion: '3.0.0',
                      mandatory: true,
                    ),
                  ),
                  updater: updater,
                );
              },
              child: const Text('open'),
            );
          },
        ),
      ),
    );

    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();

    expect(find.text('更新が必要です'), findsOneWidget);
    expect(find.text('後で'), findsNothing);
  });
}
