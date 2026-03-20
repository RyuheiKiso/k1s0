import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/legacy.dart';

import 'config_editor_notifier.dart';
import 'widgets/category_nav.dart';
import 'widgets/config_field_list.dart';

/// 設定エディタページ
/// Riverpod の StateNotifier で状態管理を行い、setState を排除する
class ConfigEditorPage extends ConsumerStatefulWidget {
  const ConfigEditorPage({
    super.key,
    required this.dio,
    required this.serviceName,
  });

  final Dio dio;
  final String serviceName;

  @override
  ConsumerState<ConfigEditorPage> createState() => _ConfigEditorPageState();
}

class _ConfigEditorPageState extends ConsumerState<ConfigEditorPage> {
  /// サービスごとの Provider インスタンスを保持する
  late final StateNotifierProvider<ConfigEditorNotifier, ConfigEditorState>
      _provider;

  @override
  void initState() {
    super.initState();
    /// サービス名と Dio インスタンスに基づいた Provider を生成する
    _provider = configEditorProvider(widget.dio, widget.serviceName);
  }

  @override
  Widget build(BuildContext context) {
    /// Riverpod の ref.watch で状態変更を監視して自動再描画する
    final editorState = ref.watch(_provider);

    /// ローディング中はプログレスインジケーターを表示する
    if (editorState.isLoading) {
      return const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      );
    }

    /// エラー発生時はエラーメッセージを表示する
    if (editorState.error != null) {
      return Scaffold(
        body: Center(child: Text(editorState.error!)),
      );
    }

    final category = editorState.selectedCategory;

    return Scaffold(
      appBar: AppBar(
        title: Text(editorState.data?.service ?? widget.serviceName),
        actions: [
          /// 変更フィールド数のバッジを表示する
          if (editorState.hasDirtyFields)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Center(
                child: Text('${editorState.data?.dirtyCount ?? 0} changes'),
              ),
            ),
          /// 変更を破棄するボタン
          TextButton(
            onPressed: editorState.hasDirtyFields
                ? () => ref.read(_provider.notifier).discard()
                : null,
            child: const Text('Discard'),
          ),
          /// 保存ボタン（バリデーションエラーや保存中は無効化）
          FilledButton(
            onPressed: editorState.hasDirtyFields &&
                    !editorState.isSaving &&
                    !editorState.hasValidationErrors
                ? () => _save()
                : null,
            child: editorState.isSaving
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('Save'),
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: Row(
        children: [
          /// 左側のカテゴリナビゲーション
          SizedBox(
            width: 240,
            child: CategoryNav(
              categories: editorState.data!.categories
                  .map((category) => category.schema)
                  .toList(),
              selectedId: editorState.selectedCategoryId ?? '',
              onSelected: (id) => ref.read(_provider.notifier).selectCategory(id),
            ),
          ),
          const VerticalDivider(width: 1),
          /// 右側のフィールド一覧
          Expanded(
            child: category != null
                ? ConfigFieldList(
                    category: category,
                    onFieldChanged: (key, value) =>
                        ref.read(_provider.notifier).onFieldChanged(key, value),
                    onFieldValidationChanged: (key, error) =>
                        ref.read(_provider.notifier).onFieldValidationChanged(key, error),
                    onResetToDefault: (key) =>
                        ref.read(_provider.notifier).resetFieldToDefault(key),
                  )
                : const Center(child: Text('Select a category')),
          ),
        ],
      ),
      /// コンフリクト発生時に警告バーを表示する
      bottomSheet: editorState.hasConflict
          ? const Material(
              color: Colors.amber,
              child: SizedBox(
                width: double.infinity,
                child: Padding(
                  padding: EdgeInsets.all(12),
                  child: Text(
                      'Conflict detected. Reload and review before saving again.'),
                ),
              ),
            )
          : null,
    );
  }

  /// 保存を実行し、コンフリクト発生時はダイアログを表示する
  Future<void> _save() async {
    final success = await ref.read(_provider.notifier).save();
    if (!success && mounted) {
      final editorState = ref.read(_provider);
      if (editorState.hasConflict) {
        await showDialog<void>(
          context: context,
          builder: (ctx) => AlertDialog(
            title: const Text('Conflict'),
            content: const Text(
              'Another user updated this config. Reload and review before saving again.',
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(ctx).pop(),
                child: const Text('OK'),
              ),
            ],
          ),
        );
      }
    }
  }
}
