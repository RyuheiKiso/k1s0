/**
 * DataTable + Form Generator 統合テスト
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { z } from 'zod';
import { K1s0DataTable, createColumns, actionsColumn } from '../components/DataTable';
import { createFormFromSchema } from '../components/FormGenerator';

// テーマプロバイダーラッパー
const theme = createTheme();

const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <ThemeProvider theme={theme}>
    <LocalizationProvider dateAdapter={AdapterDayjs}>
      {children}
    </LocalizationProvider>
  </ThemeProvider>
);

// テストデータ型
interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
}

// テストデータ
const testUsers: User[] = [
  { id: '1', name: '山田太郎', email: 'yamada@example.com', role: 'admin' },
  { id: '2', name: '鈴木花子', email: 'suzuki@example.com', role: 'user' },
  { id: '3', name: '田中一郎', email: 'tanaka@example.com', role: 'guest' },
];

// Zod スキーマ
const userSchema = z.object({
  name: z.string().min(1, '名前は必須です'),
  email: z.string().email('有効なメールアドレスを入力してください'),
  role: z.enum(['admin', 'user', 'guest']),
});

describe('DataTable + Form Generator 統合テスト', () => {
  it('DataTable から編集ボタンをクリックしてフォームに値を渡せる', async () => {
    const user = userEvent.setup();
    const onEdit = vi.fn();

    // カラム定義
    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
      actionsColumn<User>({
        onEdit: (row) => onEdit(row),
      }),
    ]);

    render(
      <TestWrapper>
        <K1s0DataTable
          rows={testUsers}
          columns={columns}
          getRowId={(row) => row.id}
        />
      </TestWrapper>
    );

    // 編集ボタンを探してクリック
    const editButtons = screen.getAllByRole('button', { name: /編集/i });
    expect(editButtons.length).toBeGreaterThan(0);

    await user.click(editButtons[0]);

    expect(onEdit).toHaveBeenCalledWith(testUsers[0]);
  });

  it('フォームで入力した値を DataTable に反映できる', async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();

    // フォームコンポーネント作成
    const UserForm = createFormFromSchema(userSchema, {
      labels: {
        name: '氏名',
        email: 'メールアドレス',
        role: '権限',
      },
      fieldConfig: {
        role: {
          component: 'Select',
          options: [
            { label: '管理者', value: 'admin' },
            { label: '一般', value: 'user' },
            { label: 'ゲスト', value: 'guest' },
          ],
        },
      },
      submitLabel: '保存',
    });

    render(
      <TestWrapper>
        <UserForm
          defaultValues={{ name: '', email: '', role: 'user' }}
          onSubmit={onSubmit}
        />
      </TestWrapper>
    );

    // フォームに入力
    const nameInput = screen.getByLabelText(/氏名/i);
    const emailInput = screen.getByLabelText(/メールアドレス/i);

    await user.type(nameInput, '新規ユーザー');
    await user.type(emailInput, 'newuser@example.com');

    // 送信
    const submitButton = screen.getByRole('button', { name: /保存/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith(
        expect.objectContaining({
          name: '新規ユーザー',
          email: 'newuser@example.com',
          role: 'user',
        })
      );
    });
  });

  it('DataTable の選択状態とフォーム送信を連携できる', async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();
    const onBulkDelete = vi.fn();

    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
    ]);

    const { rerender } = render(
      <TestWrapper>
        <K1s0DataTable
          rows={testUsers}
          columns={columns}
          getRowId={(row) => row.id}
          checkboxSelection
          onRowSelectionModelChange={onSelectionChange}
        />
        <button onClick={() => onBulkDelete(onSelectionChange.mock.calls.at(-1)?.[0])}>
          選択削除
        </button>
      </TestWrapper>
    );

    // チェックボックスを選択
    const checkboxes = screen.getAllByRole('checkbox');
    // 最初のチェックボックスはヘッダーの全選択なので、2番目以降を選択
    if (checkboxes.length > 1) {
      await user.click(checkboxes[1]);
    }

    // 選択削除ボタンをクリック
    const deleteButton = screen.getByRole('button', { name: /選択削除/i });
    await user.click(deleteButton);

    expect(onBulkDelete).toHaveBeenCalled();
  });

  it('フォームバリデーションエラーが正しく表示される', async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();

    const UserForm = createFormFromSchema(userSchema, {
      labels: {
        name: '氏名',
        email: 'メールアドレス',
        role: '権限',
      },
      fieldConfig: {
        role: {
          component: 'Select',
          options: [
            { label: '管理者', value: 'admin' },
            { label: '一般', value: 'user' },
            { label: 'ゲスト', value: 'guest' },
          ],
        },
      },
      submitLabel: '保存',
    });

    render(
      <TestWrapper>
        <UserForm
          defaultValues={{ name: '', email: '', role: 'user' }}
          onSubmit={onSubmit}
        />
      </TestWrapper>
    );

    // 空のまま送信
    const submitButton = screen.getByRole('button', { name: /保存/i });
    await user.click(submitButton);

    // バリデーションエラーが表示される
    await waitFor(() => {
      expect(screen.getByText(/名前は必須です/i)).toBeInTheDocument();
    });

    // onSubmit は呼ばれない
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('編集モードでフォームに初期値が設定される', async () => {
    const existingUser = testUsers[0];

    const UserForm = createFormFromSchema(userSchema, {
      labels: {
        name: '氏名',
        email: 'メールアドレス',
        role: '権限',
      },
      fieldConfig: {
        role: {
          component: 'Select',
          options: [
            { label: '管理者', value: 'admin' },
            { label: '一般', value: 'user' },
            { label: 'ゲスト', value: 'guest' },
          ],
        },
      },
      submitLabel: '更新',
    });

    render(
      <TestWrapper>
        <UserForm
          defaultValues={existingUser}
          onSubmit={vi.fn()}
        />
      </TestWrapper>
    );

    // 初期値が設定されている
    const nameInput = screen.getByLabelText(/氏名/i) as HTMLInputElement;
    const emailInput = screen.getByLabelText(/メールアドレス/i) as HTMLInputElement;

    expect(nameInput.value).toBe(existingUser.name);
    expect(emailInput.value).toBe(existingUser.email);
  });
});

describe('パフォーマンステスト', () => {
  it('大量データ（1000行）でも DataTable がレンダリングできる', () => {
    const largeDataset: User[] = Array.from({ length: 1000 }, (_, i) => ({
      id: String(i + 1),
      name: `ユーザー${i + 1}`,
      email: `user${i + 1}@example.com`,
      role: (['admin', 'user', 'guest'] as const)[i % 3],
    }));

    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
      { field: 'role', headerName: '権限', width: 120 },
    ]);

    const startTime = performance.now();

    render(
      <TestWrapper>
        <K1s0DataTable
          rows={largeDataset}
          columns={columns}
          getRowId={(row) => row.id}
          pagination
          pageSizeOptions={[10, 25, 50, 100]}
        />
      </TestWrapper>
    );

    const endTime = performance.now();
    const renderTime = endTime - startTime;

    // 3秒以内にレンダリング完了
    expect(renderTime).toBeLessThan(3000);

    // ページネーションが表示される
    expect(screen.getByRole('grid')).toBeInTheDocument();
  });

  it('フォームの複数フィールドが効率的にレンダリングされる', () => {
    const largeSchema = z.object({
      field1: z.string(),
      field2: z.string(),
      field3: z.string(),
      field4: z.string(),
      field5: z.string(),
      field6: z.string(),
      field7: z.string(),
      field8: z.string(),
      field9: z.string(),
      field10: z.string(),
    });

    const LargeForm = createFormFromSchema(largeSchema, {
      labels: {
        field1: 'フィールド1',
        field2: 'フィールド2',
        field3: 'フィールド3',
        field4: 'フィールド4',
        field5: 'フィールド5',
        field6: 'フィールド6',
        field7: 'フィールド7',
        field8: 'フィールド8',
        field9: 'フィールド9',
        field10: 'フィールド10',
      },
      columns: 2,
    });

    const startTime = performance.now();

    render(
      <TestWrapper>
        <LargeForm
          defaultValues={{
            field1: '',
            field2: '',
            field3: '',
            field4: '',
            field5: '',
            field6: '',
            field7: '',
            field8: '',
            field9: '',
            field10: '',
          }}
          onSubmit={vi.fn()}
        />
      </TestWrapper>
    );

    const endTime = performance.now();
    const renderTime = endTime - startTime;

    // 1秒以内にレンダリング完了
    expect(renderTime).toBeLessThan(1000);

    // 全フィールドが表示される
    expect(screen.getByLabelText(/フィールド1/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/フィールド10/i)).toBeInTheDocument();
  });
});
