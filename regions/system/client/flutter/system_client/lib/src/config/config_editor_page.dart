import 'package:dio/dio.dart';
import 'package:flutter/material.dart';

import 'config_interpreter.dart';
import 'config_types.dart';
import 'widgets/category_nav.dart';
import 'widgets/config_field_list.dart';

class ConfigEditorPage extends StatefulWidget {
  const ConfigEditorPage({
    super.key,
    required this.dio,
    required this.serviceName,
  });

  final Dio dio;
  final String serviceName;

  @override
  State<ConfigEditorPage> createState() => _ConfigEditorPageState();
}

class _ConfigEditorPageState extends State<ConfigEditorPage> {
  ConfigData? _data;
  String? _selectedCategoryId;
  final Map<String, dynamic> _dirtyValues = {};
  bool _isLoading = true;
  String? _error;
  bool _isSaving = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final interpreter = ConfigInterpreter(dio: widget.dio);
      final data = await interpreter.build(widget.serviceName);
      setState(() {
        _data = data;
        _selectedCategoryId = data.schema.categories.firstOrNull?.id;
        _isLoading = false;
      });
    } on DioException catch (e) {
      setState(() {
        _error = e.message ?? 'Failed to load config';
        _isLoading = false;
      });
    }
  }

  ConfigCategorySchema? get _selectedCategory {
    if (_data == null || _selectedCategoryId == null) return null;
    return _data!.schema.categories
        .where((c) => c.id == _selectedCategoryId)
        .firstOrNull;
  }

  Map<String, dynamic> get _mergedValues {
    final merged = Map<String, dynamic>.from(_data?.values ?? {});
    merged.addAll(_dirtyValues);
    return merged;
  }

  void _onFieldChanged(String key, dynamic value) {
    setState(() {
      _dirtyValues[key] = value;
    });
  }

  void _discard() {
    setState(() {
      _dirtyValues.clear();
    });
  }

  Future<void> _save() async {
    if (_dirtyValues.isEmpty || _data == null) return;

    setState(() => _isSaving = true);

    try {
      final prefix = _data!.schema.namespacePrefix;
      for (final entry in _dirtyValues.entries) {
        await widget.dio.put(
          '/api/v1/config/$prefix/${entry.key}',
          data: {'value': entry.value},
        );
      }

      _dirtyValues.clear();
      await _load();
    } on DioException catch (e) {
      if (e.response?.statusCode == 409 && mounted) {
        await showDialog<void>(
          context: context,
          builder: (ctx) => AlertDialog(
            title: const Text('Conflict'),
            content: const Text(
              '別のユーザーにより値が更新されています。リロードしてください。',
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
    } finally {
      if (mounted) setState(() => _isSaving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      );
    }

    if (_error != null) {
      return Scaffold(
        body: Center(child: Text(_error!)),
      );
    }

    final category = _selectedCategory;

    return Scaffold(
      appBar: AppBar(
        title: Text(widget.serviceName),
        actions: [
          if (_dirtyValues.isNotEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Center(
                child: Text('${_dirtyValues.length} 件の変更'),
              ),
            ),
          TextButton(
            onPressed: _dirtyValues.isEmpty ? null : _discard,
            child: const Text('破棄'),
          ),
          FilledButton(
            onPressed: _dirtyValues.isEmpty || _isSaving ? null : _save,
            child: _isSaving
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('保存'),
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: Row(
        children: [
          SizedBox(
            width: 240,
            child: CategoryNav(
              categories: _data!.schema.categories,
              selectedId: _selectedCategoryId ?? '',
              onSelected: (id) =>
                  setState(() => _selectedCategoryId = id),
            ),
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: category != null
                ? ConfigFieldList(
                    fields: category.fields,
                    values: _mergedValues,
                    onFieldChanged: _onFieldChanged,
                  )
                : const Center(child: Text('カテゴリを選択してください')),
          ),
        ],
      ),
    );
  }
}
