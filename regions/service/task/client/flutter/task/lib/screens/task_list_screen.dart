import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/task.dart';
import '../providers/task_provider.dart';

/// タスク一覧画面
/// タスクの検索・フィルタリング・一覧表示を行うメイン画面
class TaskListScreen extends ConsumerStatefulWidget {
  const TaskListScreen({super.key});

  @override
  ConsumerState<TaskListScreen> createState() => _TaskListScreenState();
}

class _TaskListScreenState extends ConsumerState<TaskListScreen> {
  /// 現在選択中のステータスフィルタ（nullの場合は全件表示）
  TaskStatus? _selectedStatus;

  @override
  Widget build(BuildContext context) {
    /// タスク一覧の状態を監視する
    final tasksAsync = ref.watch(taskListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('タスク一覧'),
        actions: [
          /// 新規タスク作成画面への遷移ボタン
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: '新規タスク',
            onPressed: () => context.push('/tasks/new'),
          ),
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () => ref.read(taskListProvider.notifier).load(
                  status: _selectedStatus,
                ),
          ),
        ],
      ),
      body: Column(
        children: [
          /// ステータスフィルタ用のチップを表示する
          _buildStatusFilterChips(),

          /// タスク一覧を表示する
          Expanded(
            child: tasksAsync.when(
              /// ローディング中はプログレスインジケーターを表示する
              loading: () =>
                  const Center(child: CircularProgressIndicator()),
              /// エラー時はエラーメッセージとリトライボタンを表示する
              error: (error, stack) => Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text('エラーが発生しました: $error'),
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: () =>
                          ref.read(taskListProvider.notifier).load(
                                status: _selectedStatus,
                              ),
                      child: const Text('再試行'),
                    ),
                  ],
                ),
              ),
              /// データ取得成功時はタスク一覧をリスト表示する
              data: (tasks) {
                if (tasks.isEmpty) {
                  return const Center(child: Text('タスクがありません'));
                }
                return ListView.builder(
                  itemCount: tasks.length,
                  itemBuilder: (context, index) {
                    final task = tasks[index];
                    /// key にタスクIDを指定してリスト再構築時の差分検出を最適化する
                    return _TaskListTile(
                      key: ValueKey(task.id),
                      task: task,
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  /// ステータスフィルタ用のチップ一覧を構築する
  /// 選択状態に応じてフィルタリングを実行する
  Widget _buildStatusFilterChips() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: [
            /// 「すべて」チップ: フィルタを解除する
            FilterChip(
              label: const Text('すべて'),
              selected: _selectedStatus == null,
              onSelected: (selected) {
                setState(() => _selectedStatus = null);
                ref.read(taskListProvider.notifier).load();
              },
            ),
            const SizedBox(width: 8),
            /// 各ステータスのフィルタチップを生成する
            ...TaskStatus.values.map((status) {
              return Padding(
                padding: const EdgeInsets.only(right: 8),
                child: FilterChip(
                  label: Text(status.displayName),
                  selected: _selectedStatus == status,
                  onSelected: (selected) {
                    setState(() {
                      _selectedStatus = selected ? status : null;
                    });
                    ref.read(taskListProvider.notifier).load(
                          status: selected ? status : null,
                        );
                  },
                ),
              );
            }),
          ],
        ),
      ),
    );
  }
}

/// タスク一覧の個別タイルウィジェット
/// タスク情報の概要表示とタップによる詳細遷移を提供する
class _TaskListTile extends StatelessWidget {
  final Task task;

  const _TaskListTile({super.key, required this.task});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        /// タスクタイトルをタイトルに表示する
        title: Text(task.title),
        /// プロジェクトID・担当者・期日をサブタイトルに表示する
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('プロジェクト: ${task.projectId}'),
            Text(
              '担当者: ${task.assigneeId ?? '未割当'} · '
              '期日: ${task.dueDate ?? '未設定'}',
            ),
          ],
        ),
        /// ステータスバッジと優先度バッジを右側に表示する
        trailing: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            _StatusBadge(status: task.status),
            const SizedBox(height: 4),
            _PriorityBadge(priority: task.priority),
          ],
        ),
        /// タップでタスク詳細画面へ遷移する
        onTap: () => context.push('/tasks/${task.id}'),
      ),
    );
  }
}

/// ステータスバッジウィジェット
/// タスクステータスに応じた色分けバッジを表示する
class _StatusBadge extends StatelessWidget {
  final TaskStatus status;

  const _StatusBadge({required this.status});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: _getStatusColor(status),
        borderRadius: BorderRadius.circular(10),
      ),
      child: Text(
        status.displayName,
        style: const TextStyle(
          color: Colors.white,
          fontSize: 11,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  /// ステータスに応じたバッジ背景色を返す
  Color _getStatusColor(TaskStatus status) {
    return switch (status) {
      TaskStatus.open => Colors.teal,
      TaskStatus.inProgress => Colors.blue,
      TaskStatus.review => Colors.purple,
      TaskStatus.done => Colors.green,
      TaskStatus.cancelled => Colors.red,
    };
  }
}

/// 優先度バッジウィジェット
/// タスク優先度に応じた色分けバッジを表示する
class _PriorityBadge extends StatelessWidget {
  final TaskPriority priority;

  const _PriorityBadge({required this.priority});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: _getPriorityColor(priority),
        borderRadius: BorderRadius.circular(10),
      ),
      child: Text(
        priority.displayName,
        style: const TextStyle(
          color: Colors.white,
          fontSize: 11,
        ),
      ),
    );
  }

  /// 優先度に応じたバッジ背景色を返す
  Color _getPriorityColor(TaskPriority priority) {
    return switch (priority) {
      TaskPriority.low => Colors.grey,
      TaskPriority.medium => Colors.orange,
      TaskPriority.high => Colors.deepOrange,
      TaskPriority.critical => Colors.red,
    };
  }
}
