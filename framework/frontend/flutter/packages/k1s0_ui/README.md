# k1s0_ui

Design System for k1s0 Flutter applications with Material 3 theming.

## Features

- Material 3 based theming (light/dark mode)
- Common UI widgets (buttons, cards, text fields)
- Form validation utilities
- Feedback components (snackbars, dialogs)
- Loading/error/empty state widgets
- Riverpod-based theme management
- **DataTable** - 高機能データテーブル（ソート、ページネーション、選択）
- **Form Generator** - スキーマ駆動フォーム生成

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

## DataTable

高機能データテーブルコンポーネント。ソート、ページネーション、行選択、カスタムセルレンダリングをサポート。

### 基本的な使い方

```dart
import 'package:k1s0_ui/k1s0_ui.dart';

class User {
  final String id;
  final String name;
  final String email;
  final String role;
  final DateTime createdAt;

  User({
    required this.id,
    required this.name,
    required this.email,
    required this.role,
    required this.createdAt,
  });
}

class UserListPage extends StatelessWidget {
  final List<User> users;

  const UserListPage({required this.users});

  @override
  Widget build(BuildContext context) {
    return K1s0DataTable<User>(
      rows: users,
      columns: [
        K1s0Column<User>(
          id: 'name',
          label: '氏名',
          sortable: true,
          valueGetter: (user) => user.name,
        ),
        K1s0Column<User>(
          id: 'email',
          label: 'メール',
          flex: 2,
          valueGetter: (user) => user.email,
        ),
        K1s0Column<User>(
          id: 'role',
          label: '権限',
          width: 120,
          type: K1s0ColumnType.chip,
          valueGetter: (user) => user.role,
        ),
        K1s0Column<User>(
          id: 'createdAt',
          label: '作成日',
          type: K1s0ColumnType.date,
          sortable: true,
          valueGetter: (user) => user.createdAt,
        ),
      ],
      getRowId: (user) => user.id,
      onRowTap: (user) => _navigateToDetail(user),
    );
  }
}
```

### ページネーション

```dart
K1s0DataTable<User>(
  rows: users,
  columns: columns,
  getRowId: (user) => user.id,
  // ページネーション設定
  pagination: true,
  pageSize: 20,
  pageSizeOptions: [10, 20, 50, 100],
  onPageChange: (page) {
    print('Page changed to: $page');
  },
  onPageSizeChange: (size) {
    print('Page size changed to: $size');
  },
);
```

### 行選択

```dart
K1s0DataTable<User>(
  rows: users,
  columns: columns,
  getRowId: (user) => user.id,
  // 選択設定
  selectionMode: K1s0SelectionMode.multiple,
  selectedIds: selectedUserIds,
  onSelectionChange: (ids) {
    setState(() => selectedUserIds = ids);
  },
);
```

### サーバーサイドページネーション

```dart
class ServerSideUserList extends StatefulWidget {
  @override
  State<ServerSideUserList> createState() => _ServerSideUserListState();
}

class _ServerSideUserListState extends State<ServerSideUserList> {
  late K1s0DataTableController<User> _controller;
  List<User> _users = [];
  int _totalCount = 0;
  bool _loading = false;

  @override
  void initState() {
    super.initState();
    _controller = K1s0DataTableController<User>();
    _controller.addListener(_fetchData);
    _fetchData();
  }

  Future<void> _fetchData() async {
    setState(() => _loading = true);

    final response = await api.getUsers(
      page: _controller.currentPage,
      pageSize: _controller.pageSize,
      sortColumn: _controller.sortModel?.column,
      sortOrder: _controller.sortModel?.order.name,
    );

    setState(() {
      _users = response.data;
      _totalCount = response.total;
      _loading = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return K1s0DataTable<User>(
      rows: _users,
      columns: columns,
      getRowId: (user) => user.id,
      controller: _controller,
      loading: _loading,
      totalRowCount: _totalCount,
      pagination: true,
    );
  }
}
```

### カラムタイプ

| タイプ | 説明 |
|--------|------|
| `K1s0ColumnType.text` | テキスト表示（デフォルト） |
| `K1s0ColumnType.number` | 数値（右寄せ、カンマ区切り） |
| `K1s0ColumnType.date` | 日付（フォーマット可能） |
| `K1s0ColumnType.dateTime` | 日時 |
| `K1s0ColumnType.boolean` | チェックアイコン表示 |
| `K1s0ColumnType.chip` | Chip表示 |
| `K1s0ColumnType.avatar` | アバター表示 |
| `K1s0ColumnType.actions` | アクションボタン |
| `K1s0ColumnType.custom` | カスタムレンダラー |

### カスタムセルレンダリング

```dart
K1s0Column<User>(
  id: 'status',
  label: 'ステータス',
  type: K1s0ColumnType.custom,
  valueGetter: (user) => user.status,
  cellBuilder: (context, user, value) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: _getStatusColor(value as String),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        _getStatusLabel(value),
        style: const TextStyle(color: Colors.white, fontSize: 12),
      ),
    );
  },
),
```

### アクションカラム

```dart
K1s0Column<User>(
  id: 'actions',
  label: '',
  width: 100,
  type: K1s0ColumnType.actions,
  valueGetter: (user) => user,
  actions: [
    K1s0RowAction(
      icon: Icons.edit,
      tooltip: '編集',
      onPressed: (user) => _editUser(user),
    ),
    K1s0RowAction(
      icon: Icons.delete,
      tooltip: '削除',
      color: Colors.red,
      onPressed: (user) => _deleteUser(user),
    ),
  ],
),
```

## Form Generator

スキーマ駆動でフォームを自動生成。バリデーション、条件付き表示、グリッドレイアウトをサポート。

### 基本的な使い方

```dart
import 'package:k1s0_ui/k1s0_ui.dart';

// スキーマ定義
final userSchema = K1s0FormSchema<UserInput>(
  fields: [
    K1s0FormFieldSchema(
      name: 'name',
      label: '氏名',
      required: true,
      placeholder: '山田太郎',
    ),
    K1s0FormFieldSchema(
      name: 'email',
      label: 'メールアドレス',
      type: K1s0FieldType.email,
      required: true,
    ),
    K1s0FormFieldSchema(
      name: 'age',
      label: '年齢',
      type: K1s0FieldType.number,
      min: 0,
      max: 120,
    ),
    K1s0FormFieldSchema(
      name: 'role',
      label: '権限',
      type: K1s0FieldType.select,
      required: true,
      options: [
        K1s0FieldOption(label: '管理者', value: 'admin'),
        K1s0FieldOption(label: '一般ユーザー', value: 'user'),
        K1s0FieldOption(label: 'ゲスト', value: 'guest'),
      ],
    ),
    K1s0FormFieldSchema(
      name: 'notifications',
      label: '通知を受け取る',
      type: K1s0FieldType.switchField,
      defaultValue: true,
    ),
    K1s0FormFieldSchema(
      name: 'bio',
      label: '自己紹介',
      type: K1s0FieldType.textarea,
      maxLength: 500,
      rows: 4,
    ),
  ],
  fromMap: (map) => UserInput.fromMap(map),
  toMap: (user) => user.toMap(),
);

// フォームウィジェット
class CreateUserPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('ユーザー作成')),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: K1s0Form<UserInput>(
          schema: userSchema,
          onSubmit: (values) async {
            await createUser(values);
            Navigator.pop(context);
          },
          submitLabel: '作成',
          showCancel: true,
          onCancel: () => Navigator.pop(context),
        ),
      ),
    );
  }
}
```

### フィールドタイプ

| タイプ | 説明 |
|--------|------|
| `K1s0FieldType.text` | テキスト入力 |
| `K1s0FieldType.email` | メールアドレス入力 |
| `K1s0FieldType.password` | パスワード入力 |
| `K1s0FieldType.number` | 数値入力 |
| `K1s0FieldType.textarea` | 複数行テキスト |
| `K1s0FieldType.select` | ドロップダウン選択 |
| `K1s0FieldType.radio` | ラジオボタン |
| `K1s0FieldType.checkbox` | チェックボックス |
| `K1s0FieldType.switchField` | スイッチ |
| `K1s0FieldType.date` | 日付選択 |
| `K1s0FieldType.dateTime` | 日時選択 |
| `K1s0FieldType.time` | 時刻選択 |
| `K1s0FieldType.slider` | スライダー |
| `K1s0FieldType.rating` | 評価（星） |

### グリッドレイアウト

```dart
K1s0Form<UserInput>(
  schema: userSchema,
  onSubmit: (values) async {},
  columns: 2,  // 2列レイアウト
  spacing: 16,
);
```

### 初期値の設定

```dart
K1s0Form<UserInput>(
  schema: userSchema,
  initialValues: existingUser,  // 編集時の初期値
  onSubmit: (values) async {
    await updateUser(values);
  },
  submitLabel: '更新',
);
```

### 条件付きフィールド表示

```dart
K1s0FormFieldSchema(
  name: 'companyName',
  label: '会社名',
  // userType が 'business' の場合のみ表示
  visibleWhen: (values) => values['userType'] == 'business',
),
```

### カスタムバリデーション

```dart
K1s0FormFieldSchema(
  name: 'password',
  label: 'パスワード',
  type: K1s0FieldType.password,
  required: true,
  validators: [
    MinLengthValidator(8, message: 'パスワードは8文字以上必要です'),
    PatternValidator(
      RegExp(r'[A-Z]'),
      message: '大文字を含める必要があります',
    ),
    PatternValidator(
      RegExp(r'[0-9]'),
      message: '数字を含める必要があります',
    ),
  ],
),
```

### 組み込みバリデーター

| バリデーター | 説明 |
|--------------|------|
| `RequiredValidator` | 必須チェック |
| `EmailValidator` | メールアドレス形式 |
| `MinLengthValidator(n)` | 最小文字数 |
| `MaxLengthValidator(n)` | 最大文字数 |
| `PatternValidator(regex)` | 正規表現パターン |
| `RangeValidator(min, max)` | 数値範囲 |
| `CompositeValidator([...])` | 複数バリデーター結合 |

### 読み取り専用モード

```dart
K1s0Form<UserInput>(
  schema: userSchema,
  initialValues: user,
  readOnly: true,  // 全フィールドを読み取り専用に
  onSubmit: (_) async {},
);
```

### ローディング状態

```dart
K1s0Form<UserInput>(
  schema: userSchema,
  onSubmit: (values) async {
    // 送信中は自動的にローディング表示
    await createUser(values);
  },
  loading: isExternalLoading,  // 外部からのローディング状態
);
```

## License

MIT
