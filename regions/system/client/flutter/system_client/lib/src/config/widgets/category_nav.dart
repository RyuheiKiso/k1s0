import 'package:flutter/material.dart';

import '../config_types.dart';

class CategoryNav extends StatelessWidget {
  const CategoryNav({
    super.key,
    required this.categories,
    required this.selectedId,
    required this.onSelected,
  });

  final List<ConfigCategorySchema> categories;
  final String selectedId;
  final ValueChanged<String> onSelected;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return ListView.builder(
      itemCount: categories.length,
      itemBuilder: (context, index) {
        final category = categories[index];
        final isSelected = category.id == selectedId;

        return ListTile(
          title: Text(category.label),
          selected: isSelected,
          selectedTileColor:
              theme.colorScheme.primaryContainer.withAlpha(77),
          onTap: () => onSelected(category.id),
        );
      },
    );
  }
}
