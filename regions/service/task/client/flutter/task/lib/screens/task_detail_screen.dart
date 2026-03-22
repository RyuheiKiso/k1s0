import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/task.dart';
import '../providers/task_provider.dart';

/// タスク詳細画面
/// 個別タスクの詳細情報表示とステータス更新を行う画面
class TaskDetailScreen extends ConsumerStatefulWidget {
  /// 対象タスクのID
  final String taskId;

  const TaskDetailScreen({super.key, required this.taskId});

  @override
  ConsumerState<TaskDetailScreen> createState() => _TaskDetailScreenState();
}

class _TaskDetailScreenState extends ConsumerState<TaskDetailScreen> {
  /// ステータス更新用の選択値
  TaskStatus? _selectedStatus;

  @override
  Widget build(BuildContext context) {
    /// タスク一覧の状態を監視し、対象タスクを抽出する
    final tasksAsync = ref.watch(taskListProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text('タスク詳細: ${widget.taskId}'),
        actions: [
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () => ref.read(taskListProvider.notifier).load(),
          ),
        ],
      ),
      body: tasksAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () => ref.read(taskListProvider.notifier).load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        data: (tasks) {
          /// タスク一覧から対象IDのタスクを検索する
          final task = tasks.where((t) => t.id == widget.taskId).firstOrNull;
          if (task == null) {
            return const Center(child: Text('タスクが見つかりません'));
          }
          return _buildTaskDetail(context, task);
        },
      ),
    );
  }

  /// タスク詳細の全体レイアウトを構築する
  Widget _buildTaskDetail(BuildContext context, Task task) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          /// タスク基本情報カード
          _buildTaskInfoCard(context, task),
          const SizedBox(height: 16),

          /// ステータス更新カード
          _buildStatusUpdateCard(context, task),
          const SizedBox(height: 16),

          /// ラベル一覧カード（ラベルがある場合のみ表示）
          if (task.labels.isNotEmpty) _buildLabelsCard(context, task),
        ],
      ),
    );
  }

  /// タスク基本情報を表示するカードを構築する
  Widget _buildTaskInfoCard(BuildContext context, Task task) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'タスク情報',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            _buildInfoRow('タスクID', task.id),
            _buildInfoRow('プロジェクトID', task.projectId),
            _buildInfoRow('タイトル', task.title),
            if (task.description != null && task.description!.isNotEmpty)
              _buildInfoRow('説明', task.description!),
            _buildInfoRow('ステータス', task.status.displayName),
            _buildInfoRow('優先度', task.priority.displayName),
            _buildInfoRow('担当者ID', task.assigneeId ?? '未割当'),
            _buildInfoRow('報告者ID', task.reporterId),
            _buildInfoRow('期日', task.dueDate ?? '未設定'),
            _buildInfoRow('バージョン', task.version.toString()),
            _buildInfoRow('作成日時', _formatDateTime(task.createdAt)),
            _buildInfoRow('更新日時', _formatDateTime(task.updatedAt)),
          ],
        ),
      ),
    );
  }

  /// ステータス更新操作を行うカードを構築する
  Widget _buildStatusUpdateCard(BuildContext context, Task task) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'ステータス更新',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                /// ステータス選択用のドロップダウン
                Expanded(
                  child: DropdownButtonFormField<TaskStatus>(
                    initialValue: _selectedStatus ?? task.status,
                    decoration: const InputDecoration(
                      labelText: '新しいステータス',
                      border: OutlineInputBorder(),
                    ),
                    items: TaskStatus.values.map((status) {
                      return DropdownMenuItem(
                        value: status,
                        child: Text(status.displayName),
                      );
                    }).toList(),
                    onChanged: (value) {
                      setState(() => _selectedStatus = value);
                    },
                  ),
                ),
                const SizedBox(width: 16),
                /// ステータス更新実行ボタン
                FilledButton(
                  onPressed: () => _updateStatus(task),
                  child: const Text('更新'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  /// ラベル一覧を表示するカードを構築する
  Widget _buildLabelsCard(BuildContext context, Task task) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'ラベル',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            /// ラベルをチップとして表示する
            Wrap(
              spacing: 8,
              runSpacing: 4,
              children: task.labels.map((label) {
                return Chip(
                  label: Text(label),
                );
              }).toList(),
            ),
          ],
        ),
      ),
    );
  }

  /// 情報表示用の行ウィジェットを構築する
  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }

  /// タスクステータスを更新する
  void _updateStatus(Task task) {
    final newStatus = _selectedStatus ?? task.status;
    if (newStatus == task.status) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('ステータスが変更されていません')),
      );
      return;
    }

    final input = UpdateTaskStatusInput(status: newStatus);
    ref.read(taskListProvider.notifier).updateStatus(task.id, input);

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('ステータスを「${newStatus.displayName}」に更新しました')),
    );
  }

  /// DateTimeを読みやすい日本語形式にフォーマットする
  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}/${dateTime.month.toString().padLeft(2, '0')}/'
        '${dateTime.day.toString().padLeft(2, '0')} '
        '${dateTime.hour.toString().padLeft(2, '0')}:'
        '${dateTime.minute.toString().padLeft(2, '0')}';
  }
}
