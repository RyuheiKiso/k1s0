import 'package:dio/dio.dart';
import 'package:flutter/material.dart';

import 'config_interpreter.dart';
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
  bool _isLoading = true;
  String? _error;
  bool _isSaving = false;
  bool _hasConflict = false;

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
        _selectedCategoryId = data.categories.firstOrNull?.schema.id;
        _isLoading = false;
      });
    } on DioException catch (e) {
      setState(() {
        _error = e.message ?? 'Failed to load config';
        _isLoading = false;
      });
    }
  }

  ConfigCategoryState? get _selectedCategory {
    if (_data == null || _selectedCategoryId == null) return null;
    return _data!.categories
        .where((category) => category.schema.id == _selectedCategoryId)
        .firstOrNull;
  }

  bool get _hasDirtyFields => (_data?.dirtyCount ?? 0) > 0;

  bool get _hasValidationErrors =>
      _data?.categories.any(
        (category) => category.fields.any((field) => field.error != null),
      ) ??
      false;

  void _onFieldValidationChanged(String key, String? error) {
    if (_data == null || _selectedCategoryId == null) return;

    setState(() {
      final categories = _data!.categories.map((category) {
        if (category.schema.id != _selectedCategoryId) {
          return category;
        }

        return category.copyWith(
          fields: category.fields.map((field) {
            if (field.key != key) return field;
            if (error == null) {
              return field.copyWith(clearError: true);
            }
            return field.copyWith(error: error);
          }).toList(),
        );
      }).toList();

      _data = _data!.copyWith(categories: categories);
      _hasConflict = false;
    });
  }

  void _onFieldChanged(String key, dynamic value) {
    if (_data == null || _selectedCategoryId == null) return;

    setState(() {
      final categories = _data!.categories.map((category) {
        if (category.schema.id != _selectedCategoryId) {
          return category;
        }

        return category.copyWith(
          fields: category.fields.map((field) {
            if (field.key != key) return field;
            return updateFieldState(field, value);
          }).toList(),
        );
      }).toList();

      _data = _data!.copyWith(
        categories: categories,
        dirtyCount: countDirtyFields(categories),
      );
      _hasConflict = false;
    });
  }

  void _resetFieldToDefault(String key) {
    if (_data == null || _selectedCategoryId == null) return;

    setState(() {
      final categories = _data!.categories.map((category) {
        if (category.schema.id != _selectedCategoryId) {
          return category;
        }

        return category.copyWith(
          fields: category.fields.map((field) {
            if (field.key != key) return field;
            return updateFieldState(field, field.schema.defaultValue);
          }).toList(),
        );
      }).toList();

      _data = _data!.copyWith(
        categories: categories,
        dirtyCount: countDirtyFields(categories),
      );
    });
  }

  void _discard() {
    if (_data == null) return;
    setState(() {
      _data = resetConfigData(_data!);
      _hasConflict = false;
    });
  }

  Future<void> _save() async {
    if (_data == null || !_hasDirtyFields || _hasValidationErrors) return;

    final dirtyFields = _data!.categories
        .expand((category) => category.fields)
        .where((field) => field.isDirty)
        .toList();

    setState(() => _isSaving = true);

    try {
      for (final field in dirtyFields) {
        await widget.dio.put(
          '/api/v1/config/${Uri.encodeComponent(field.namespace)}/${Uri.encodeComponent(field.key)}',
          data: {
            'value': field.value,
            'version': field.version,
          },
        );
      }

      await _load();
      if (mounted) {
        setState(() => _hasConflict = false);
      }
    } on DioException catch (e) {
      if (e.response?.statusCode == 409 && mounted) {
        setState(() => _hasConflict = true);
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
      } else if (mounted) {
        setState(() => _error = e.message ?? 'Failed to save config');
      }
    } finally {
      if (mounted) {
        setState(() => _isSaving = false);
      }
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
        title: Text(_data?.service ?? widget.serviceName),
        actions: [
          if (_hasDirtyFields)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Center(
                child: Text('${_data?.dirtyCount ?? 0} changes'),
              ),
            ),
          TextButton(
            onPressed: _hasDirtyFields ? _discard : null,
            child: const Text('Discard'),
          ),
          FilledButton(
            onPressed: _hasDirtyFields && !_isSaving && !_hasValidationErrors
                ? _save
                : null,
            child: _isSaving
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
          SizedBox(
            width: 240,
            child: CategoryNav(
              categories:
                  _data!.categories.map((category) => category.schema).toList(),
              selectedId: _selectedCategoryId ?? '',
              onSelected: (id) => setState(() => _selectedCategoryId = id),
            ),
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: category != null
                ? ConfigFieldList(
                    category: category,
                    onFieldChanged: _onFieldChanged,
                    onFieldValidationChanged: _onFieldValidationChanged,
                    onResetToDefault: _resetFieldToDefault,
                  )
                : const Center(child: Text('Select a category')),
          ),
        ],
      ),
      bottomSheet: _hasConflict
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
}
