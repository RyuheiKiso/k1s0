import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/tenant_extension.dart';
import '../providers/tenant_extension_provider.dart';

/// テナント拡張画面
/// テナント固有のプロジェクトマスタカスタマイズを管理する画面
class TenantExtensionScreen extends ConsumerStatefulWidget {
  /// 対象テナントのID
  final String tenantId;

  const TenantExtensionScreen({super.key, required this.tenantId});

  @override
  ConsumerState<TenantExtensionScreen> createState() =>
      _TenantExtensionScreenState();
}

class _TenantExtensionScreenState
    extends ConsumerState<TenantExtensionScreen> {
  /// ステータス定義ID入力用コントローラー
  final _statusDefIdController = TextEditingController();

  /// 表示名オーバーライド入力用コントローラー
  final _displayNameController = TextEditingController();

  /// 属性オーバーライド入力用コントローラー（JSON形式）
  final _attributesController = TextEditingController();

  /// 現在表示中のステータス定義ID
  String? _currentStatusDefId;

  @override
  void dispose() {
    _statusDefIdController.dispose();
    _displayNameController.dispose();
    _attributesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    /// テナント拡張の状態を監視する
    final extensionAsync = ref.watch(tenantExtensionProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text('テナント拡張: ${widget.tenantId}'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            /// ステータス定義ID入力と検索セクション
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _statusDefIdController,
                    decoration: const InputDecoration(
                      labelText: 'ステータス定義ID',
                      hintText: 'ステータス定義IDを入力して検索',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                FilledButton.icon(
                  onPressed: _loadExtension,
                  icon: const Icon(Icons.search),
                  label: const Text('検索'),
                ),
              ],
            ),
            const SizedBox(height: 24),

            /// テナント拡張データの表示・編集セクション
            Expanded(
              child: extensionAsync.when(
                loading: () =>
                    const Center(child: CircularProgressIndicator()),
                error: (error, stack) => Center(
                  child: Text('エラー: $error'),
                ),
                data: (extension) {
                  if (extension == null && _currentStatusDefId == null) {
                    return const Center(
                      child: Text('ステータス定義IDを入力して検索してください'),
                    );
                  }

                  if (extension == null) {
                    return Center(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          const Text('テナント拡張が見つかりません'),
                          const SizedBox(height: 16),
                          FilledButton(
                            onPressed: _showCreateExtensionForm,
                            child: const Text('新規作成'),
                          ),
                        ],
                      ),
                    );
                  }

                  /// 取得した拡張データをフォームに反映する
                  return _buildExtensionDetail(extension);
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// テナント拡張情報をサーバーから取得する
  void _loadExtension() {
    final statusDefId = _statusDefIdController.text.trim();
    if (statusDefId.isEmpty) return;

    setState(() => _currentStatusDefId = statusDefId);
    ref
        .read(tenantExtensionProvider.notifier)
        .load(widget.tenantId, statusDefId);
  }

  /// テナント拡張の詳細表示・編集フォームを構築する
  Widget _buildExtensionDetail(TenantProjectExtension extension) {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          /// 拡張情報のメタデータを表示するカード
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'テナント拡張情報',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 8),
                  _buildInfoRow('ID', extension.id),
                  _buildInfoRow('テナントID', extension.tenantId),
                  _buildInfoRow('ステータス定義ID', extension.statusDefinitionId),
                  _buildInfoRow(
                    '表示名オーバーライド',
                    extension.displayNameOverride ?? '（未設定）',
                  ),
                  _buildInfoRow(
                    '有効',
                    extension.isEnabled ? 'はい' : 'いいえ',
                  ),
                  _buildInfoRow(
                    '作成日時',
                    _formatDateTime(extension.createdAt),
                  ),
                  _buildInfoRow(
                    '更新日時',
                    _formatDateTime(extension.updatedAt),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),

          /// 拡張データ編集フォーム
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    '拡張データ編集',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: _displayNameController
                      ..text = extension.displayNameOverride ?? '',
                    decoration: const InputDecoration(
                      labelText: '表示名オーバーライド',
                      border: OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: _attributesController
                      ..text =
                          extension.attributesOverride?.toString() ?? '',
                    decoration: const InputDecoration(
                      labelText: '属性オーバーライド（JSON）',
                      border: OutlineInputBorder(),
                      helperText: 'JSON形式で入力してください',
                    ),
                    maxLines: 5,
                  ),
                  const SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      /// テナント拡張を削除するボタン
                      OutlinedButton(
                        style: OutlinedButton.styleFrom(
                          foregroundColor: Colors.red,
                        ),
                        onPressed: () =>
                            _deleteExtension(extension.statusDefinitionId),
                        child: const Text('削除'),
                      ),
                      const SizedBox(width: 8),

                      /// テナント拡張を更新するボタン
                      FilledButton(
                        onPressed: () =>
                            _updateExtension(extension.statusDefinitionId),
                        child: const Text('更新'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
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
            width: 160,
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

  /// テナント拡張情報を更新する
  void _updateExtension(String statusDefinitionId) {
    final input = UpdateTenantExtensionInput(
      displayNameOverride: _displayNameController.text.trim().isNotEmpty
          ? _displayNameController.text.trim()
          : null,
    );

    ref
        .read(tenantExtensionProvider.notifier)
        .upsert(widget.tenantId, statusDefinitionId, input);

    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('更新しました')),
    );
  }

  /// テナント拡張情報を削除する（確認ダイアログ付き）
  void _deleteExtension(String statusDefinitionId) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('テナント拡張削除'),
        content: const Text('このテナント拡張を削除しますか？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          FilledButton(
            style: FilledButton.styleFrom(
              backgroundColor: Colors.red,
            ),
            onPressed: () {
              ref
                  .read(tenantExtensionProvider.notifier)
                  .delete(widget.tenantId, statusDefinitionId);
              Navigator.of(context).pop();
              ScaffoldMessenger.of(this.context).showSnackBar(
                const SnackBar(content: Text('削除しました')),
              );
            },
            child: const Text('削除'),
          ),
        ],
      ),
    );
  }

  /// 新規テナント拡張作成フォームを表示する
  void _showCreateExtensionForm() {
    _displayNameController.clear();
    _attributesController.clear();

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('テナント拡張作成'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: _displayNameController,
                decoration: const InputDecoration(
                  labelText: '表示名オーバーライド',
                ),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: _attributesController,
                decoration: const InputDecoration(
                  labelText: '属性オーバーライド（JSON）',
                ),
                maxLines: 3,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          FilledButton(
            onPressed: () {
              final input = UpdateTenantExtensionInput(
                displayNameOverride:
                    _displayNameController.text.trim().isNotEmpty
                        ? _displayNameController.text.trim()
                        : null,
                isEnabled: true,
              );
              ref.read(tenantExtensionProvider.notifier).upsert(
                    widget.tenantId,
                    _currentStatusDefId!,
                    input,
                  );
              Navigator.of(context).pop();
            },
            child: const Text('作成'),
          ),
        ],
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
}
