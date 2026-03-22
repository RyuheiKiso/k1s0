import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/task.dart';
import '../providers/task_provider.dart';

/// タスク作成画面
/// 新規タスクの入力フォームを表示し、タスクを作成する
class TaskFormScreen extends ConsumerStatefulWidget {
  const TaskFormScreen({super.key});

  @override
  ConsumerState<TaskFormScreen> createState() => _TaskFormScreenState();
}

class _TaskFormScreenState extends ConsumerState<TaskFormScreen> {
  /// フォームバリデーション用のキー
  final _formKey = GlobalKey<FormState>();

  /// プロジェクトID入力用コントローラー
  final _projectIdController = TextEditingController();

  /// タイトル入力用コントローラー
  final _titleController = TextEditingController();

  /// 説明入力用コントローラー
  final _descriptionController = TextEditingController();

  /// 担当者ID入力用コントローラー
  final _assigneeIdController = TextEditingController();

  /// 期日入力用コントローラー
  final _dueDateController = TextEditingController();

  /// ラベル入力用コントローラー（カンマ区切り）
  final _labelsController = TextEditingController();

  /// 優先度の選択値（デフォルト: medium）
  TaskPriority _selectedPriority = TaskPriority.medium;

  @override
  void dispose() {
    _projectIdController.dispose();
    _titleController.dispose();
    _descriptionController.dispose();
    _assigneeIdController.dispose();
    _dueDateController.dispose();
    _labelsController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('新規タスク'),
      ),
      body: Form(
        key: _formKey,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              /// 基本情報セクション
              _buildBasicSection(context),
              const SizedBox(height: 24),

              /// 詳細設定セクション
              _buildDetailSection(context),
              const SizedBox(height: 24),

              /// 送信ボタン
              _buildFooterSection(context),
            ],
          ),
        ),
      ),
    );
  }

  /// 基本情報入力セクションを構築する
  Widget _buildBasicSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '基本情報',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),

            /// プロジェクトID入力フィールド
            TextFormField(
              controller: _projectIdController,
              decoration: const InputDecoration(
                labelText: 'プロジェクトID',
                hintText: 'プロジェクトIDを入力してください',
                border: OutlineInputBorder(),
              ),
              validator: (value) {
                if (value == null || value.trim().isEmpty) {
                  return 'プロジェクトIDは必須です';
                }
                return null;
              },
            ),
            const SizedBox(height: 16),

            /// タイトル入力フィールド
            TextFormField(
              controller: _titleController,
              decoration: const InputDecoration(
                labelText: 'タイトル',
                hintText: 'タスクのタイトルを入力してください',
                border: OutlineInputBorder(),
              ),
              validator: (value) {
                if (value == null || value.trim().isEmpty) {
                  return 'タイトルは必須です';
                }
                return null;
              },
            ),
            const SizedBox(height: 16),

            /// 説明入力フィールド（任意）
            TextFormField(
              controller: _descriptionController,
              decoration: const InputDecoration(
                labelText: '説明（任意）',
                hintText: 'タスクの詳細説明を入力してください',
                border: OutlineInputBorder(),
              ),
              maxLines: 3,
            ),
          ],
        ),
      ),
    );
  }

  /// 詳細設定入力セクションを構築する
  Widget _buildDetailSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '詳細設定',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),

            /// 優先度選択ドロップダウン
            DropdownButtonFormField<TaskPriority>(
              value: _selectedPriority,
              decoration: const InputDecoration(
                labelText: '優先度',
                border: OutlineInputBorder(),
              ),
              items: TaskPriority.values.map((priority) {
                return DropdownMenuItem(
                  value: priority,
                  child: Text(priority.displayName),
                );
              }).toList(),
              onChanged: (value) {
                if (value != null) {
                  setState(() => _selectedPriority = value);
                }
              },
            ),
            const SizedBox(height: 16),

            /// 担当者ID入力フィールド（任意）
            TextFormField(
              controller: _assigneeIdController,
              decoration: const InputDecoration(
                labelText: '担当者ID（任意）',
                hintText: '担当者のIDを入力してください',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 16),

            /// 期日入力フィールド（任意）
            TextFormField(
              controller: _dueDateController,
              decoration: const InputDecoration(
                labelText: '期日（任意）',
                hintText: 'YYYY-MM-DD形式で入力してください',
                border: OutlineInputBorder(),
              ),
              keyboardType: TextInputType.datetime,
            ),
            const SizedBox(height: 16),

            /// ラベル入力フィールド（カンマ区切り、任意）
            TextFormField(
              controller: _labelsController,
              decoration: const InputDecoration(
                labelText: 'ラベル（カンマ区切り、任意）',
                hintText: '例: bug, frontend, urgent',
                border: OutlineInputBorder(),
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// 送信ボタンのフッターセクションを構築する
  Widget _buildFooterSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            /// タスク作成ボタン
            FilledButton.icon(
              onPressed: _submitTask,
              icon: const Icon(Icons.add_task),
              label: const Text('タスクを作成'),
            ),
          ],
        ),
      ),
    );
  }

  /// タスクをサーバーに送信する
  /// フォームバリデーション後、タスクデータを構築してAPIに送信する
  void _submitTask() {
    if (!_formKey.currentState!.validate()) return;

    /// ラベルをカンマ区切りから配列に変換する
    final labelsText = _labelsController.text.trim();
    final labels = labelsText.isNotEmpty
        ? labelsText
            .split(',')
            .map((l) => l.trim())
            .where((l) => l.isNotEmpty)
            .toList()
        : null;

    /// タスク作成入力データを構築する
    final input = CreateTaskInput(
      projectId: _projectIdController.text.trim(),
      title: _titleController.text.trim(),
      description: _descriptionController.text.trim().isNotEmpty
          ? _descriptionController.text.trim()
          : null,
      priority: _selectedPriority,
      assigneeId: _assigneeIdController.text.trim().isNotEmpty
          ? _assigneeIdController.text.trim()
          : null,
      dueDate: _dueDateController.text.trim().isNotEmpty
          ? _dueDateController.text.trim()
          : null,
      labels: labels,
    );

    /// タスクを作成してリストを更新する
    ref.read(taskListProvider.notifier).create(input);

    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('タスクを作成しました')),
    );

    /// タスク一覧画面に戻る
    context.go('/');
  }
}
