import 'package:flutter/material.dart';

import 'app_updater.dart';
import 'model.dart';

class UpdateDialog {
  static Future<void> show({
    required BuildContext context,
    required UpdateCheckResult result,
    required AppUpdater updater,
    String? title,
    String? message,
  }) async {
    if (!result.needsUpdate) {
      return;
    }

    await showDialog<void>(
      context: context,
      barrierDismissible: !result.isMandatory,
      builder: (dialogContext) {
        return AlertDialog(
          title: Text(
            title ?? (result.isMandatory ? '更新が必要です' : '新しいバージョンがあります'),
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Text(
                message ??
                    'バージョン ${result.versionInfo.latestVersion} が利用可能です。'
                        '（現在: ${result.currentVersion}）',
              ),
              if (result.versionInfo.releaseNotes != null) ...<Widget>[
                const SizedBox(height: 8),
                Text(result.versionInfo.releaseNotes!),
              ],
            ],
          ),
          actions: <Widget>[
            if (!result.isMandatory)
              TextButton(
                onPressed: () => Navigator.of(dialogContext).pop(),
                child: const Text('後で'),
              ),
            TextButton(
              onPressed: () async {
                await updater.openStore();
                if (dialogContext.mounted && !result.isMandatory) {
                  Navigator.of(dialogContext).pop();
                }
              },
              child: const Text('更新する'),
            ),
          ],
        );
      },
    );
  }
}

extension AppUpdaterPromptExtension on AppUpdater {
  Future<UpdateCheckResult> checkAndPrompt(
    BuildContext context, {
    String? title,
    String? message,
  }) async {
    final result = await checkForUpdate();
    if (result.needsUpdate && context.mounted) {
      await UpdateDialog.show(
        context: context,
        result: result,
        updater: this,
        title: title,
        message: message,
      );
    }
    return result;
  }
}
