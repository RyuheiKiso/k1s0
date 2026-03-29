import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/activity.dart';
import '../providers/activity_provider.dart';

/// アクティビティ作成画面
/// 新規アクティビティの入力フォームを表示し、アクティビティを作成する
class ActivityFormScreen extends ConsumerStatefulWidget {
  const ActivityFormScreen({super.key});

  @override
  ConsumerState<ActivityFormScreen> createState() => _ActivityFormScreenState();
}

class _ActivityFormScreenState extends ConsumerState<ActivityFormScreen> {
  /// フォームバリデーション用のキー
  final _formKey = GlobalKey<FormState>();

  /// タスクID入力用コントローラー
  final _taskIdController = TextEditingController();

  /// アクターID入力用コントローラー
  final _actorIdController = TextEditingController();

  /// 内容入力用コントローラー
  final _contentController = TextEditingController();

  /// 作業時間入力用コントローラー
  final _durationController = TextEditingController();

  /// 冪等性キー入力用コントローラー
  final _idempotencyKeyController = TextEditingController();

  /// 選択中のアクティビティ種別（デフォルト: comment）
  ActivityType _selectedType = ActivityType.comment;

  @override
  void dispose() {
    _taskIdController.dispose();
    _actorIdController.dispose();
    _contentController.dispose();
    _durationController.dispose();
    _idempotencyKeyController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('新規アクティビティ'),
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

              /// 内容セクション
              _buildContentSection(context),
              const SizedBox(height: 24),

              /// 詳細オプションセクション
              _buildOptionsSection(context),
              const SizedBox(height: 24),

              /// 送信ボタンセクション
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
            /// タスクID入力フィールド
            TextFormField(
              controller: _taskIdController,
              decoration: const InputDecoration(
                labelText: 'タスクID',
                hintText: 'タスクIDを入力してください',
                border: OutlineInputBorder(),
              ),
              validator: (value) {
                if (value == null || value.trim().isEmpty) {
                  return 'タスクIDは必須です';
                }
                return null;
              },
            ),
            const SizedBox(height: 16),
            /// アクターID入力フィールド
            TextFormField(
              controller: _actorIdController,
              decoration: const InputDecoration(
                labelText: 'アクターID',
                hintText: 'アクターIDを入力してください',
                border: OutlineInputBorder(),
              ),
              validator: (value) {
                if (value == null || value.trim().isEmpty) {
                  return 'アクターIDは必須です';
                }
                return null;
              },
            ),
            const SizedBox(height: 16),
            /// アクティビティ種別選択ドロップダウン
            DropdownButtonFormField<ActivityType>(
              initialValue: _selectedType,
              decoration: const InputDecoration(
                labelText: '種別',
                border: OutlineInputBorder(),
              ),
              items: ActivityType.values.map((type) {
                return DropdownMenuItem(
                  value: type,
                  child: Text(type.displayName),
                );
              }).toList(),
              onChanged: (value) {
                if (value != null) setState(() => _selectedType = value);
              },
            ),
          ],
        ),
      ),
    );
  }

  /// 内容入力セクションを構築する
  Widget _buildContentSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '内容',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            /// 内容入力フィールド（複数行対応）
            TextFormField(
              controller: _contentController,
              decoration: const InputDecoration(
                labelText: '内容（任意）',
                hintText: 'アクティビティの内容を入力してください',
                border: OutlineInputBorder(),
              ),
              maxLines: 4,
            ),
            /// time_entry 種別の場合のみ作業時間入力フィールドを表示する
            if (_selectedType == ActivityType.time_entry) ...[
              const SizedBox(height: 16),
              TextFormField(
                controller: _durationController,
                decoration: const InputDecoration(
                  labelText: '作業時間（分）',
                  hintText: '作業時間を分単位で入力してください',
                  border: OutlineInputBorder(),
                ),
                keyboardType: TextInputType.number,
                validator: (value) {
                  if (value == null || value.trim().isEmpty) return null;
                  final duration = int.tryParse(value);
                  if (duration == null || duration < 0) {
                    return '0以上の整数を入力してください';
                  }
                  return null;
                },
              ),
            ],
          ],
        ),
      ),
    );
  }

  /// 詳細オプション入力セクションを構築する
  Widget _buildOptionsSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'オプション',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            /// 冪等性キー入力フィールド（重複登録防止）
            TextFormField(
              controller: _idempotencyKeyController,
              decoration: const InputDecoration(
                labelText: '冪等性キー（任意）',
                hintText: '重複登録防止のためのユニークキー',
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
            /// アクティビティ作成ボタン
            FilledButton.icon(
              onPressed: _submitActivity,
              icon: const Icon(Icons.send),
              label: const Text('アクティビティを作成'),
            ),
          ],
        ),
      ),
    );
  }

  /// アクティビティをサーバーに送信する
  /// フォームバリデーション後、アクティビティデータを構築してAPIに送信する
  void _submitActivity() {
    if (!_formKey.currentState!.validate()) return;

    /// 作業時間を解析する（time_entry 種別かつ入力がある場合のみ）
    final durationText = _durationController.text.trim();
    final durationMinutes = (_selectedType == ActivityType.time_entry &&
            durationText.isNotEmpty)
        ? int.tryParse(durationText)
        : null;

    /// アクティビティ作成入力データを構築する
    final input = CreateActivityInput(
      taskId: _taskIdController.text.trim(),
      actorId: _actorIdController.text.trim(),
      activityType: _selectedType,
      content: _contentController.text.trim().isNotEmpty
          ? _contentController.text.trim()
          : null,
      durationMinutes: durationMinutes,
      idempotencyKey: _idempotencyKeyController.text.trim().isNotEmpty
          ? _idempotencyKeyController.text.trim()
          : null,
    );

    /// アクティビティを作成してリストを更新する
    ref.read(activityListProvider.notifier).create(input);

    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('アクティビティを作成しました')),
    );

    /// アクティビティ一覧画面に戻る
    context.go('/');
  }
}
