# @k1s0/ui

k1s0 Design/UX 標準コンポーネントライブラリ。

画面ごとの"独自流儀"を減らし、実装とレビューの基準を固定するための共通 UI コンポーネントを提供する。

## 目的

- 新規画面の UX が"最初から揃っている"状態を実現
- 実装コストとレビュー観点を固定
- 一貫したデザインシステムの適用

## インストール

```bash
pnpm add @k1s0/ui
```

### Peer Dependencies

```bash
pnpm add @mui/material @emotion/react @emotion/styled react react-dom
```

## 使い方

### テーマの適用

```tsx
import { K1s0ThemeProvider } from '@k1s0/ui/theme';

function App() {
  return (
    <K1s0ThemeProvider>
      <MyApp />
    </K1s0ThemeProvider>
  );
}
```

ダークモードの切り替え:

```tsx
import { useK1s0Theme } from '@k1s0/ui/theme';

function DarkModeToggle() {
  const { darkMode, toggleDarkMode } = useK1s0Theme();

  return (
    <Switch checked={darkMode} onChange={toggleDarkMode} />
  );
}
```

カスタムテーマの作成:

```tsx
import { createK1s0Theme } from '@k1s0/ui/theme';

const customTheme = createK1s0Theme({
  overrides: {
    palette: {
      primary: {
        main: '#your-color',
      },
    },
  },
});
```

### フォームコンポーネント

バリデーション連携付きフォームフィールド:

```tsx
import { FormTextField, FormSelect, validationRules } from '@k1s0/ui/form';

function LoginForm() {
  return (
    <>
      <FormTextField
        name="email"
        label="メールアドレス"
        required
        validation={validationRules.email}
      />
      <FormTextField
        name="password"
        label="パスワード"
        type="password"
        required
        validation={{ minLength: 8 }}
      />
    </>
  );
}
```

FormContainer による状態管理:

```tsx
import { FormContainer, FormTextField } from '@k1s0/ui/form';

function ContactForm() {
  return (
    <FormContainer
      initialValues={{ name: '', email: '', message: '' }}
      validations={{
        name: { required: true },
        email: { required: true, pattern: emailPattern },
        message: { required: true, minLength: 10 },
      }}
      onSubmit={async (values) => {
        await submitContact(values);
      }}
    >
      {({ values, errors, isSubmitting, setValue, submit }) => (
        <>
          <FormTextField
            name="name"
            label="お名前"
            value={values.name}
            error={errors.fields.name}
            onChange={(e) => setValue('name', e.target.value)}
          />
          {/* ... */}
          <Button onClick={submit} disabled={isSubmitting}>
            送信
          </Button>
        </>
      )}
    </FormContainer>
  );
}
```

### 通知（Toast）

```tsx
import { FeedbackProvider, useToast } from '@k1s0/ui/feedback';

// ルートで Provider を設定
function App() {
  return (
    <FeedbackProvider>
      <MyApp />
    </FeedbackProvider>
  );
}

// コンポーネント内で使用
function SaveButton() {
  const toast = useToast();

  const handleSave = async () => {
    try {
      await save();
      toast.success('保存しました');
    } catch (error) {
      toast.error('保存に失敗しました');
    }
  };

  return <Button onClick={handleSave}>保存</Button>;
}
```

### 確認ダイアログ

```tsx
import { useConfirmDialog } from '@k1s0/ui/feedback';

function DeleteButton() {
  const { confirm } = useConfirmDialog();

  const handleDelete = async () => {
    const confirmed = await confirm({
      title: '削除の確認',
      message: 'このアイテムを削除しますか？この操作は取り消せません。',
      confirmLabel: '削除',
      dangerous: true,
    });

    if (confirmed) {
      await deleteItem();
    }
  };

  return <Button onClick={handleDelete}>削除</Button>;
}
```

### ローディング表示

```tsx
import { LoadingSpinner, PageLoading, SkeletonLoader } from '@k1s0/ui/state';

// スピナー
<LoadingSpinner message="読み込み中..." />

// ページ全体
<PageLoading />

// スケルトン
<SkeletonLoader lines={3} avatar />
<SkeletonLoader card />
```

### 空状態

```tsx
import { EmptyState, NoSearchResults, ErrorState } from '@k1s0/ui/state';

// 基本的な空状態
<EmptyState
  title="アイテムがありません"
  description="新しいアイテムを追加してください。"
  actionLabel="アイテムを追加"
  onAction={() => navigate('/items/new')}
/>

// 検索結果なし
<NoSearchResults
  query={searchQuery}
  onReset={() => setSearchQuery('')}
/>

// エラー状態
<ErrorState
  message="データの取得に失敗しました"
  onRetry={() => refetch()}
/>
```

### DataTable（MUI DataGrid ベース）

高機能なデータテーブルコンポーネント:

```tsx
import { K1s0DataTable, createColumns, dateColumn, actionsColumn } from '@k1s0/ui';

interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
  createdAt: Date;
}

const columns = createColumns<User>([
  { field: 'name', headerName: '氏名', flex: 1, sortable: true },
  { field: 'email', headerName: 'メール', flex: 1 },
  {
    field: 'role',
    headerName: '権限',
    width: 120,
    type: 'singleSelect',
    valueOptions: [
      { value: 'admin', label: '管理者' },
      { value: 'user', label: '一般' },
      { value: 'guest', label: 'ゲスト' },
    ],
  },
  dateColumn({ field: 'createdAt', headerName: '作成日' }),
  actionsColumn({
    onEdit: (row) => navigate(`/users/${row.id}/edit`),
    onDelete: (row) => handleDelete(row.id),
  }),
]);

function UserList() {
  const { data: users, isLoading } = useUsers();

  return (
    <K1s0DataTable
      rows={users ?? []}
      columns={columns}
      loading={isLoading}
      checkboxSelection
      pagination
      pageSize={20}
      toolbar
      exportOptions={{ csv: true }}
      onRowClick={(user) => navigate(`/users/${user.id}`)}
    />
  );
}
```

サーバーサイドページネーション:

```tsx
import { K1s0DataTable, useServerSidePagination } from '@k1s0/ui';

function ServerSideUserList() {
  const {
    rows,
    rowCount,
    loading,
    paginationModel,
    setPaginationModel,
    sortModel,
    setSortModel,
  } = useServerSidePagination<User>({
    fetchFn: (params) => api.getUsers(params),
  });

  return (
    <K1s0DataTable
      rows={rows}
      columns={columns}
      loading={loading}
      paginationMode="server"
      sortingMode="server"
      rowCount={rowCount}
      paginationModel={paginationModel}
      onPaginationModelChange={setPaginationModel}
      sortModel={sortModel}
      onSortModelChange={setSortModel}
    />
  );
}
```

### Form Generator（Zod + MUI）

Zod スキーマから MUI フォームを自動生成:

```tsx
import { createFormFromSchema } from '@k1s0/ui';
import { z } from 'zod';

const userSchema = z.object({
  name: z.string().min(1, '名前は必須です'),
  email: z.string().email('有効なメールアドレスを入力してください'),
  age: z.number().min(0).max(120).optional(),
  role: z.enum(['admin', 'user', 'guest']),
  notifications: z.boolean().default(true),
  bio: z.string().max(500).optional(),
});

const UserForm = createFormFromSchema(userSchema, {
  labels: {
    name: '氏名',
    email: 'メールアドレス',
    age: '年齢',
    role: '権限',
    notifications: '通知を受け取る',
    bio: '自己紹介',
  },
  fieldConfig: {
    role: {
      component: 'Select',
      options: [
        { label: '管理者', value: 'admin' },
        { label: '一般ユーザー', value: 'user' },
        { label: 'ゲスト', value: 'guest' },
      ],
    },
    bio: { component: 'TextField', multiline: true, rows: 4 },
    notifications: { component: 'Switch' },
  },
  variant: 'outlined',
  columns: 2,
  submitLabel: '保存',
  showCancel: true,
});

function CreateUserPage() {
  return (
    <UserForm
      defaultValues={{ role: 'user', notifications: true }}
      onSubmit={async (values) => {
        await createUser(values);
        navigate('/users');
      }}
      onCancel={() => navigate('/users')}
    />
  );
}
```

## パッケージ構成

```
@k1s0/ui
├── theme/          # 共通テーマ（色/タイポ/spacing）
├── form/           # フォームコンポーネント（バリデーション連携）
├── feedback/       # 通知・ダイアログ（toast/snackbar/confirm）
├── state/          # 状態表示（loading/empty/error）
├── data-table/     # DataTable（MUI DataGrid ベース）
└── form-generator/ # Form Generator（Zod + react-hook-form + MUI）
```

## デザイン方針

### カラーパレット

- **Primary**: 信頼性・専門性を表すブルー系（#1976d2）
- **Secondary**: アクセントとして使用するティール系（#009688）
- **Semantic Colors**: error/warning/info/success は MUI 標準に準拠

### タイポグラフィ

- 日本語対応を考慮したフォントファミリー
- 読みやすさを重視したラインハイト
- ボタンは大文字変換しない（`textTransform: 'none'`）

### スペーシング

- 8px グリッドベース
- セマンティックな値: xs(4px), sm(8px), md(16px), lg(24px), xl(32px)

### コンポーネントスタイル

- フラットなボタン（elevation なし）
- 角丸: 8px（ボタン/入力）, 12px（カード/ダイアログ）
- ボーダー: 薄いグレー線で区切り

## ライセンス

MIT
