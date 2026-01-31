# k1s0_ui (Flutter)

← [Flutter パッケージ一覧](./)

## 目的

k1s0 Design System を提供する。Material 3 ベースの統一されたテーマ、共通ウィジェット、フォームバリデーション、フィードバックコンポーネントを実現。

## モジュール構成

| モジュール | 内容 |
|-----------|------|
| `theme/` | K1s0Theme, K1s0Colors, K1s0Typography, K1s0Spacing, ThemeProvider |
| `widgets/` | K1s0PrimaryButton, K1s0SecondaryButton, K1s0Card, K1s0TextField |
| `form/` | K1s0Validators, K1s0FormContainer, K1s0FormSection, K1s0Form（スキーマ駆動） |
| `feedback/` | K1s0Snackbar, K1s0Dialog |
| `state/` | K1s0Loading, K1s0ErrorState, K1s0EmptyState |
| `data_table/` | K1s0DataTable、K1s0Column、ソート・ページネーション・選択機能 |

## 使用例

```dart
// テーマ設定
MaterialApp(
  theme: ref.watch(themeProvider).lightTheme,
  darkTheme: ref.watch(themeProvider).darkTheme,
  themeMode: ref.watch(themeProvider).themeMode,
)

// ボタン
K1s0PrimaryButton(
  onPressed: () {},
  loading: isSubmitting,
  child: Text('Submit'),
)

// テキストフィールド
K1s0TextField(
  controller: controller,
  label: 'Email',
  validator: K1s0Validators.combine([
    K1s0Validators.required,
    K1s0Validators.email,
  ]),
)

// フィードバック
K1s0Snackbar.success(context, 'Operation completed!');

final confirmed = await K1s0Dialog.confirm(
  context,
  title: 'Delete Item',
  message: 'Are you sure?',
  isDanger: true,
);

// 状態ウィジェット
K1s0Loading(message: 'Loading...')
K1s0ErrorState(message: 'Error occurred', onRetry: _retry)
K1s0EmptyState(title: 'No items', message: 'Add your first item')
```

## DataTable

高機能データテーブルコンポーネント。ソート、ページネーション、行選択、カスタムセルレンダリングをサポート。

### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `K1s0DataTable<T>` | メインデータテーブルウィジェット |
| `K1s0Column<T>` | カラム定義 |
| `K1s0DataTableController` | 状態管理コントローラー |
| `K1s0SortModel` | ソート状態モデル |

### カラムタイプ

| タイプ | 説明 |
|--------|------|
| `K1s0ColumnType.text` | テキスト表示（デフォルト） |
| `K1s0ColumnType.number` | 数値（右寄せ、カンマ区切り） |
| `K1s0ColumnType.date` | 日付フォーマット |
| `K1s0ColumnType.boolean` | チェックアイコン表示 |
| `K1s0ColumnType.chip` | Chip表示 |
| `K1s0ColumnType.actions` | アクションボタン |
| `K1s0ColumnType.custom` | カスタムレンダラー |

### 使用例

```dart
K1s0DataTable<User>(
  rows: users,
  columns: [
    K1s0Column<User>(
      id: 'name',
      label: '氏名',
      sortable: true,
      valueGetter: (user) => user.name,
    ),
    K1s0Column<User>(
      id: 'role',
      label: '権限',
      type: K1s0ColumnType.chip,
      valueGetter: (user) => user.role,
    ),
    K1s0Column<User>(
      id: 'createdAt',
      label: '作成日',
      type: K1s0ColumnType.date,
      valueGetter: (user) => user.createdAt,
    ),
  ],
  getRowId: (user) => user.id,
  pagination: true,
  pageSize: 20,
  selectionMode: K1s0SelectionMode.multiple,
  onRowTap: (user) => _navigateToDetail(user),
);
```

## Form Generator

スキーマ駆動でフォームを自動生成。バリデーション、条件付き表示、グリッドレイアウトをサポート。

### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `K1s0Form<T>` | メインフォームウィジェット |
| `K1s0FormSchema<T>` | フォームスキーマ定義 |
| `K1s0FormFieldSchema` | フィールド定義 |
| `K1s0FormController<T>` | フォーム状態管理 |

### フィールドタイプ

| タイプ | 説明 |
|--------|------|
| `K1s0FieldType.text` | テキスト入力 |
| `K1s0FieldType.email` | メールアドレス入力 |
| `K1s0FieldType.password` | パスワード入力 |
| `K1s0FieldType.number` | 数値入力 |
| `K1s0FieldType.select` | ドロップダウン選択 |
| `K1s0FieldType.radio` | ラジオボタン |
| `K1s0FieldType.checkbox` | チェックボックス |
| `K1s0FieldType.switchField` | スイッチ |
| `K1s0FieldType.date` | 日付選択 |
| `K1s0FieldType.slider` | スライダー |

### 使用例

```dart
final userSchema = K1s0FormSchema<UserInput>(
  fields: [
    K1s0FormFieldSchema(
      name: 'name',
      label: '氏名',
      required: true,
    ),
    K1s0FormFieldSchema(
      name: 'email',
      label: 'メールアドレス',
      type: K1s0FieldType.email,
      required: true,
    ),
    K1s0FormFieldSchema(
      name: 'role',
      label: '権限',
      type: K1s0FieldType.select,
      options: [
        K1s0FieldOption(label: '管理者', value: 'admin'),
        K1s0FieldOption(label: '一般', value: 'user'),
      ],
    ),
  ],
  fromMap: (map) => UserInput.fromMap(map),
  toMap: (user) => user.toMap(),
);

K1s0Form<UserInput>(
  schema: userSchema,
  onSubmit: (values) async {
    await createUser(values);
  },
  submitLabel: '作成',
  columns: 2,
);
```
