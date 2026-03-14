import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/domain_master_provider.dart';

/// バージョン履歴画面
/// マスタアイテムの変更履歴を時系列で表示する監査証跡画面
class VersionHistoryScreen extends ConsumerWidget {
  /// 対象カテゴリのコード
  final String categoryCode;

  /// 対象アイテムのコード
  final String itemCode;

  const VersionHistoryScreen({
    super.key,
    required this.categoryCode,
    required this.itemCode,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    /// バージョン履歴の状態を監視する
    final versionsAsync = ref.watch(
      versionListProvider(
        (categoryCode: categoryCode, itemCode: itemCode),
      ),
    );

    return Scaffold(
      appBar: AppBar(
        title: Text('バージョン履歴: $itemCode'),
        actions: [
          /// 履歴を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () => ref
                .read(versionListProvider(
                  (categoryCode: categoryCode, itemCode: itemCode),
                ).notifier)
                .load(),
          ),
        ],
      ),
      body: versionsAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () => ref
                    .read(versionListProvider(
                      (categoryCode: categoryCode, itemCode: itemCode),
                    ).notifier)
                    .load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        data: (versions) {
          if (versions.isEmpty) {
            return const Center(child: Text('バージョン履歴がありません'));
          }

          return ListView.builder(
            itemCount: versions.length,
            itemBuilder: (context, index) {
              final version = versions[index];
              return Card(
                margin: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 4,
                ),
                child: ExpansionTile(
                  /// バージョン番号をタイトルに表示する
                  title: Text('バージョン ${version.versionNumber}'),
                  /// 変更者と変更日時をサブタイトルに表示する
                  subtitle: Text(
                    '変更者: ${version.changedBy} · '
                    '${_formatDateTime(version.createdAt)}',
                  ),
                  leading: CircleAvatar(
                    child: Text('${version.versionNumber}'),
                  ),
                  children: [
                    Padding(
                      padding: const EdgeInsets.all(16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          /// 変更理由がある場合は表示する
                          if (version.changeReason != null) ...[
                            Text(
                              '変更理由: ${version.changeReason}',
                              style: const TextStyle(
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                            const SizedBox(height: 8),
                          ],

                          /// 変更前データがある場合は表示する
                          if (version.beforeData != null) ...[
                            const Text(
                              '変更前:',
                              style: TextStyle(
                                fontWeight: FontWeight.bold,
                                color: Colors.red,
                              ),
                            ),
                            const SizedBox(height: 4),
                            Container(
                              width: double.infinity,
                              padding: const EdgeInsets.all(8),
                              decoration: BoxDecoration(
                                color: Colors.red.shade50,
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: Text(
                                _formatJson(version.beforeData!),
                                style: const TextStyle(
                                  fontFamily: 'monospace',
                                  fontSize: 12,
                                ),
                              ),
                            ),
                            const SizedBox(height: 8),
                          ],

                          /// 変更後データを表示する
                          const Text(
                            '変更後:',
                            style: TextStyle(
                              fontWeight: FontWeight.bold,
                              color: Colors.green,
                            ),
                          ),
                          const SizedBox(height: 4),
                          Container(
                            width: double.infinity,
                            padding: const EdgeInsets.all(8),
                            decoration: BoxDecoration(
                              color: Colors.green.shade50,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              _formatJson(version.afterData),
                              style: const TextStyle(
                                fontFamily: 'monospace',
                                fontSize: 12,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              );
            },
          );
        },
      ),
    );
  }

  /// DateTimeを読みやすい日本語形式にフォーマットする
  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}/${dateTime.month.toString().padLeft(2, '0')}/'
        '${dateTime.day.toString().padLeft(2, '0')} '
        '${dateTime.hour.toString().padLeft(2, '0')}:'
        '${dateTime.minute.toString().padLeft(2, '0')}';
  }

  /// JSONマップを整形された文字列に変換する
  String _formatJson(Map<String, dynamic> json) {
    final buffer = StringBuffer();
    json.forEach((key, value) {
      buffer.writeln('$key: $value');
    });
    return buffer.toString().trimRight();
  }
}
