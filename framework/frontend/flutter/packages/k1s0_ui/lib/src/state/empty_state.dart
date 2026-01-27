import 'package:flutter/material.dart';

import '../theme/k1s0_spacing.dart';
import '../widgets/buttons.dart';

/// Empty state widget
class K1s0EmptyState extends StatelessWidget {
  /// Creates an empty state widget
  const K1s0EmptyState({
    required this.message,
    this.title,
    this.icon,
    this.action,
    this.actionLabel,
    this.onAction,
    this.centered = true,
    super.key,
  });

  /// Message to display
  final String message;

  /// Title
  final String? title;

  /// Custom icon
  final IconData? icon;

  /// Custom action widget
  final Widget? action;

  /// Action button label
  final String? actionLabel;

  /// Action callback
  final VoidCallback? onAction;

  /// Whether to center the content
  final bool centered;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    final content = Padding(
      padding: K1s0Spacing.allLg,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            icon ?? Icons.inbox_outlined,
            size: 64,
            color: scheme.onSurfaceVariant,
          ),
          K1s0Spacing.gapMd,
          if (title != null) ...[
            Text(
              title!,
              style: textTheme.titleLarge,
              textAlign: TextAlign.center,
            ),
            K1s0Spacing.gapSm,
          ],
          Text(
            message,
            style: textTheme.bodyMedium?.copyWith(
              color: scheme.onSurfaceVariant,
            ),
            textAlign: TextAlign.center,
          ),
          if (action != null || (actionLabel != null && onAction != null)) ...[
            K1s0Spacing.gapLg,
            action ??
                K1s0PrimaryButton(
                  onPressed: onAction,
                  child: Text(actionLabel!),
                ),
          ],
        ],
      ),
    );

    if (centered) {
      return Center(child: content);
    }

    return content;
  }
}

/// No results state (for search)
class K1s0NoResults extends StatelessWidget {
  /// Creates a no results state
  const K1s0NoResults({
    this.searchQuery,
    this.onClear,
    super.key,
  });

  /// The search query that yielded no results
  final String? searchQuery;

  /// Callback to clear the search
  final VoidCallback? onClear;

  @override
  Widget build(BuildContext context) {
    final message = searchQuery != null
        ? 'No results found for "$searchQuery"'
        : 'No results found';

    return K1s0EmptyState(
      title: 'No Results',
      message: message,
      icon: Icons.search_off,
      actionLabel: onClear != null ? 'Clear Search' : null,
      onAction: onClear,
    );
  }
}

/// No data state (for lists/tables)
class K1s0NoData extends StatelessWidget {
  /// Creates a no data state
  const K1s0NoData({
    this.entityName,
    this.onAdd,
    this.addLabel,
    super.key,
  });

  /// Name of the entity (e.g., "users", "items")
  final String? entityName;

  /// Callback to add new item
  final VoidCallback? onAdd;

  /// Add button label
  final String? addLabel;

  @override
  Widget build(BuildContext context) {
    final entity = entityName ?? 'items';
    final label = addLabel ?? 'Add ${entityName ?? 'Item'}';

    return K1s0EmptyState(
      title: 'No $entity yet',
      message: 'Get started by creating your first ${entity.toLowerCase()}.',
      icon: Icons.add_box_outlined,
      actionLabel: onAdd != null ? label : null,
      onAction: onAdd,
    );
  }
}

/// Coming soon state (for features in development)
class K1s0ComingSoon extends StatelessWidget {
  /// Creates a coming soon state
  const K1s0ComingSoon({
    this.featureName,
    super.key,
  });

  /// Name of the feature
  final String? featureName;

  @override
  Widget build(BuildContext context) {
    final feature = featureName != null ? '"$featureName"' : 'This feature';

    return K1s0EmptyState(
      title: 'Coming Soon',
      message: '$feature is currently under development. Check back later!',
      icon: Icons.construction,
    );
  }
}
