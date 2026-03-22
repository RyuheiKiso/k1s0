import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/board_column.dart';
import '../providers/board_provider.dart';

/// ボード画面
/// 指定プロジェクトのKanbanカラム一覧をWIPゲージ付きで表示するメイン画面
class BoardScreen extends ConsumerWidget {
  /// 表示対象のプロジェクトID
  final String projectId;

  const BoardScreen({super.key, required this.projectId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    /// ボードカラム一覧の状態を監視する（プロジェクトIDをキーとして使用）
    final columnsAsync = ref.watch(boardColumnListProvider(projectId));

    return Scaffold(
      appBar: AppBar(
        title: Text('Kanbanボード: $projectId'),
        actions: [
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () =>
                ref.read(boardColumnListProvider(projectId).notifier).load(projectId),
          ),
        ],
      ),
      body: columnsAsync.when(
        /// ローディング中はプログレスインジケーターを表示する
        loading: () => const Center(child: CircularProgressIndicator()),
        /// エラー時はエラーメッセージとリトライボタンを表示する
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () =>
                    ref.read(boardColumnListProvider(projectId).notifier).load(projectId),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        /// データ取得成功時はカラム一覧を横スクロールで表示する
        data: (columns) {
          if (columns.isEmpty) {
            return const Center(child: Text('カラムが見つかりませんでした'));
          }
          return SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            padding: const EdgeInsets.all(16),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: columns.map((column) {
                return Padding(
                  padding: const EdgeInsets.only(right: 16),
                  child: _BoardColumnCard(
                    key: ValueKey('${column.projectId}-${column.statusCode}'),
                    column: column,
                    onEditWipLimit: () => context.push(
                      '/boards/$projectId/columns/${column.statusCode}/wip-limit',
                    ),
                    onIncrement: () => ref
                        .read(boardColumnListProvider(projectId).notifier)
                        .increment(projectId, column.statusCode),
                    onDecrement: () => ref
                        .read(boardColumnListProvider(projectId).notifier)
                        .decrement(projectId, column.statusCode),
                  ),
                );
              }).toList(),
            ),
          );
        },
      ),
    );
  }
}

/// ボードカラムカードウィジェット
/// 1カラムの情報（WIPゲージ・タスク増減ボタン）を表示する
class _BoardColumnCard extends StatelessWidget {
  final BoardColumn column;
  /// WIP制限編集画面へ遷移するコールバック
  final VoidCallback onEditWipLimit;
  /// タスク数インクリメントのコールバック
  final VoidCallback onIncrement;
  /// タスク数デクリメントのコールバック
  final VoidCallback onDecrement;

  const _BoardColumnCard({
    super.key,
    required this.column,
    required this.onEditWipLimit,
    required this.onIncrement,
    required this.onDecrement,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 220,
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              /// カードヘッダー: ステータスコードとWIP制限編集ボタン
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Expanded(
                    child: Text(
                      column.statusCode,
                      style: const TextStyle(
                        fontWeight: FontWeight.bold,
                        fontSize: 16,
                      ),
                    ),
                  ),
                  TextButton(
                    onPressed: onEditWipLimit,
                    child: const Text('編集'),
                  ),
                ],
              ),
              const SizedBox(height: 8),

              /// WIPゲージ: タスク数 / WIP制限を視覚化
              _WipGauge(column: column),
              const SizedBox(height: 8),

              /// タスク数とWIP制限の数値表示
              Row(
                crossAxisAlignment: CrossAxisAlignment.baseline,
                textBaseline: TextBaseline.alphabetic,
                children: [
                  Text(
                    '${column.taskCount}',
                    style: const TextStyle(
                      fontSize: 36,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  const SizedBox(width: 4),
                  Text(
                    '/ ${column.wipLimit > 0 ? column.wipLimit : '∞'} WIP',
                    style: TextStyle(
                      fontSize: 14,
                      color: Colors.grey.shade600,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),

              /// タスク増減ボタン
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton(
                      onPressed: column.taskCount > 0 ? onDecrement : null,
                      style: OutlinedButton.styleFrom(
                        foregroundColor: Colors.red,
                        side: const BorderSide(color: Colors.red),
                      ),
                      child: const Text('−'),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: OutlinedButton(
                      onPressed: onIncrement,
                      style: OutlinedButton.styleFrom(
                        foregroundColor: Colors.green,
                        side: const BorderSide(color: Colors.green),
                      ),
                      child: const Text('＋'),
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// WIPゲージウィジェット
/// タスク数 / WIP制限の割合をプログレスバーで表示する
class _WipGauge extends StatelessWidget {
  final BoardColumn column;

  const _WipGauge({required this.column});

  /// 使用率に応じたゲージカラーを返す
  Color _getGaugeColor(double ratio) {
    if (ratio >= 1.0) return Colors.red;
    if (ratio >= 0.8) return Colors.orange;
    return Colors.green;
  }

  @override
  Widget build(BuildContext context) {
    final ratio = column.wipUsageRatio;
    final color = _getGaugeColor(ratio);

    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: LinearProgressIndicator(
        value: column.wipLimit > 0 ? ratio : 0,
        minHeight: 12,
        backgroundColor: Colors.grey.shade200,
        valueColor: AlwaysStoppedAnimation<Color>(color),
      ),
    );
  }
}
