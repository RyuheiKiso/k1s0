# k1s0_ui

Design System for k1s0 Flutter applications with Material 3 theming.

## Features

- Material 3 based theming (light/dark mode)
- Common UI widgets (buttons, cards, text fields)
- Form validation utilities
- Feedback components (snackbars, dialogs)
- Loading/error/empty state widgets
- Riverpod-based theme management

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  k1s0_ui:
    path: ../packages/k1s0_ui
```

## Basic Usage

### Theme Setup

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:k1s0_ui/k1s0_ui.dart';

void main() {
  runApp(
    ProviderScope(
      child: Consumer(
        builder: (context, ref, child) {
          final themeState = ref.watch(themeProvider);

          return MaterialApp(
            theme: themeState.lightTheme,
            darkTheme: themeState.darkTheme,
            themeMode: themeState.themeMode,
            home: const HomePage(),
          );
        },
      ),
    ),
  );
}
```

### Toggle Theme

```dart
class SettingsPage extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDark = ref.isDarkMode;

    return SwitchListTile(
      title: const Text('Dark Mode'),
      value: isDark,
      onChanged: (_) => ref.toggleTheme(),
    );
  }
}
```

### Buttons

```dart
// Primary button
K1s0PrimaryButton(
  onPressed: () {},
  child: const Text('Submit'),
)

// Secondary button
K1s0SecondaryButton(
  onPressed: () {},
  icon: Icons.add,
  child: const Text('Add Item'),
)

// Danger button
K1s0DangerButton(
  onPressed: () {},
  loading: isDeleting,
  child: const Text('Delete'),
)

// Full width button
K1s0PrimaryButton(
  onPressed: () {},
  fullWidth: true,
  child: const Text('Continue'),
)
```

### Text Fields

```dart
// Basic text field
K1s0TextField(
  controller: controller,
  label: 'Name',
  hint: 'Enter your name',
  onChanged: (value) {},
  validator: K1s0Validators.required,
)

// Password field with visibility toggle
K1s0PasswordField(
  controller: passwordController,
  label: 'Password',
  validator: K1s0Validators.passwordStrength,
)

// Email field
K1s0EmailField(
  controller: emailController,
  validator: K1s0Validators.combine([
    K1s0Validators.required,
    K1s0Validators.email,
  ]),
)

// Search field
K1s0SearchField(
  controller: searchController,
  hint: 'Search users...',
  onChanged: (query) {},
  onClear: () {},
)
```

### Form Validation

```dart
// Validators
K1s0Validators.required(value)
K1s0Validators.email(value)
K1s0Validators.phone(value)
K1s0Validators.minLength(8)(value)
K1s0Validators.maxLength(100)(value)
K1s0Validators.numeric(value)
K1s0Validators.url(value)
K1s0Validators.passwordStrength(value)
K1s0Validators.match(otherValue, 'Password')

// Combine validators
K1s0Validators.combine([
  K1s0Validators.required,
  K1s0Validators.minLength(8),
  K1s0Validators.passwordStrength,
])
```

### Form Container

```dart
K1s0FormContainer(
  formKey: _formKey,
  children: [
    K1s0FormSection(
      title: 'Personal Information',
      children: [
        K1s0TextField(
          controller: nameController,
          label: 'Name',
        ),
        K1s0EmailField(
          controller: emailController,
        ),
      ],
    ),
    K1s0FormActions(
      onSubmit: _handleSubmit,
      onCancel: () => Navigator.pop(context),
      loading: isSubmitting,
    ),
  ],
)
```

### Feedback

```dart
// Snackbars
K1s0Snackbar.info(context, 'Information message');
K1s0Snackbar.success(context, 'Operation successful!');
K1s0Snackbar.warning(context, 'Warning message');
K1s0Snackbar.error(context, 'Error occurred');

// Dialogs
final confirmed = await K1s0Dialog.confirm(
  context,
  title: 'Delete Item',
  message: 'Are you sure you want to delete this item?',
  isDanger: true,
);

await K1s0Dialog.alert(
  context,
  title: 'Success',
  message: 'Item has been saved.',
);

// Loading dialog
K1s0Dialog.showLoading(context, message: 'Saving...');
// ... do work
K1s0Dialog.hideLoading(context);
```

### Loading States

```dart
// Simple loading indicator
K1s0Loading(message: 'Loading data...')

// Loading overlay
K1s0LoadingOverlay(
  isLoading: isLoading,
  message: 'Please wait...',
  child: ContentWidget(),
)

// Skeleton loading
K1s0ListSkeleton(itemCount: 5)

// Shimmer placeholder
K1s0ShimmerLoading(height: 200)
```

### Error States

```dart
// Generic error
K1s0ErrorState(
  title: 'Error',
  message: 'Something went wrong',
  onRetry: _retry,
)

// Network error
K1s0NetworkError(onRetry: _retry)

// Server error
K1s0ServerError(
  onRetry: _retry,
  errorCode: 'ERR_500',
)

// Permission denied
K1s0PermissionDenied(
  onGoBack: () => Navigator.pop(context),
)
```

### Empty States

```dart
// Generic empty state
K1s0EmptyState(
  title: 'No items',
  message: 'Start by adding your first item.',
  icon: Icons.inbox_outlined,
  actionLabel: 'Add Item',
  onAction: _addItem,
)

// No search results
K1s0NoResults(
  searchQuery: 'flutter',
  onClear: _clearSearch,
)

// No data
K1s0NoData(
  entityName: 'Users',
  onAdd: _addUser,
)

// Coming soon
K1s0ComingSoon(featureName: 'Analytics')
```

### Cards

```dart
// Basic card
K1s0Card(
  padding: K1s0Spacing.allMd,
  child: Text('Card content'),
)

// Tappable card
K1s0Card(
  onTap: () {},
  child: ListTile(...),
)

// Outlined card
K1s0OutlinedCard(
  child: Text('Outlined content'),
)

// Info card
K1s0InfoCard(
  title: 'Success',
  subtitle: 'Operation completed',
  icon: Icons.check_circle,
  type: K1s0InfoCardType.success,
)
```

### Spacing

```dart
// Padding
K1s0Spacing.allMd  // EdgeInsets.all(16)
K1s0Spacing.horizontalLg  // EdgeInsets.symmetric(horizontal: 24)
K1s0Spacing.verticalSm  // EdgeInsets.symmetric(vertical: 8)

// Gaps (for Column/Row)
K1s0Spacing.gapMd  // SizedBox(height: 16)
K1s0Spacing.gapHMd  // SizedBox(width: 16)

// Border radius
K1s0Radius.borderMd  // BorderRadius.circular(8)
K1s0Radius.borderLg  // BorderRadius.circular(12)

// Elevation
K1s0Elevation.level1  // 1.0
K1s0Elevation.level3  // 6.0
```

## Colors

The color system follows Material 3 guidelines with custom k1s0 colors:

- Primary: Blue (#1976D2)
- Secondary: Teal (#26A69A)
- Tertiary: Amber (#FFA726)
- Error: Red (#D32F2F)
- Success: Green (#388E3C)
- Warning: Orange (#F57C00)
- Info: Blue (#0288D1)

## Typography

Based on Material 3 type scale:

- Display (Large/Medium/Small)
- Headline (Large/Medium/Small)
- Title (Large/Medium/Small)
- Label (Large/Medium/Small)
- Body (Large/Medium/Small)

## Providers

| Provider | Type | Description |
|----------|------|-------------|
| `themeProvider` | `ThemeState` | Theme state and notifier |
| `lightThemeProvider` | `ThemeData` | Light theme data |
| `darkThemeProvider` | `ThemeData` | Dark theme data |
| `themeModeProvider` | `ThemeMode` | Current theme mode |
| `feedbackServiceProvider` | `FeedbackService` | Feedback service |

## License

MIT
