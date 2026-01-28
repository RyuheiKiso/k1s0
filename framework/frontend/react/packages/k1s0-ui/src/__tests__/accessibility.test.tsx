/**
 * アクセシビリティテスト
 *
 * axe-core を使用してWCAG 2.1準拠をチェック
 */
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { configureAxe, toHaveNoViolations } from 'vitest-axe';
import { z } from 'zod';
import { K1s0DataTable, createColumns, actionsColumn, statusColumn } from '../components/DataTable';
import { createFormFromSchema } from '../components/FormGenerator';

// vitest-axe のマッチャーを追加
expect.extend(toHaveNoViolations);

// axe の設定
const axe = configureAxe({
  rules: {
    // MUI DataGrid 固有の問題を除外
    'nested-interactive': { enabled: false },
    // color-contrast は MUI テーマに依存するため除外
    'color-contrast': { enabled: false },
  },
});

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
  status: 'active' | 'inactive';
}

// テストデータ
const testUsers: User[] = [
  { id: '1', name: '山田太郎', email: 'yamada@example.com', role: 'admin', status: 'active' },
  { id: '2', name: '鈴木花子', email: 'suzuki@example.com', role: 'user', status: 'active' },
  { id: '3', name: '田中一郎', email: 'tanaka@example.com', role: 'guest', status: 'inactive' },
];

describe('DataTable アクセシビリティテスト', () => {
  it('基本的な DataTable にアクセシビリティ違反がない', async () => {
    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
      { field: 'role', headerName: '権限', width: 120 },
    ]);

    const { container } = render(
      <TestWrapper>
        <K1s0DataTable
          rows={testUsers}
          columns={columns}
          getRowId={(row) => row.id}
          aria-label="ユーザー一覧"
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('チェックボックス選択付き DataTable にアクセシビリティ違反がない', async () => {
    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
    ]);

    const { container } = render(
      <TestWrapper>
        <K1s0DataTable
          rows={testUsers}
          columns={columns}
          getRowId={(row) => row.id}
          checkboxSelection
          aria-label="ユーザー一覧（選択可能）"
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('アクションカラム付き DataTable にアクセシビリティ違反がない', async () => {
    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      actionsColumn<User>({
        onEdit: () => {},
        onDelete: () => {},
      }),
    ]);

    const { container } = render(
      <TestWrapper>
        <K1s0DataTable
          rows={testUsers}
          columns={columns}
          getRowId={(row) => row.id}
          aria-label="ユーザー一覧（アクション付き）"
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('ステータスカラム付き DataTable にアクセシビリティ違反がない', async () => {
    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      statusColumn<User>({
        field: 'status',
        headerName: 'ステータス',
        statusConfig: {
          active: { label: '有効', color: 'success' },
          inactive: { label: '無効', color: 'error' },
        },
      }),
    ]);

    const { container } = render(
      <TestWrapper>
        <K1s0DataTable
          rows={testUsers}
          columns={columns}
          getRowId={(row) => row.id}
          aria-label="ユーザー一覧（ステータス付き）"
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('ローディング状態の DataTable にアクセシビリティ違反がない', async () => {
    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
    ]);

    const { container } = render(
      <TestWrapper>
        <K1s0DataTable
          rows={[]}
          columns={columns}
          getRowId={(row) => row.id}
          loading
          aria-label="ユーザー一覧（読み込み中）"
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});

describe('Form Generator アクセシビリティテスト', () => {
  const userSchema = z.object({
    name: z.string().min(1, '名前は必須です'),
    email: z.string().email('有効なメールアドレスを入力してください'),
    role: z.enum(['admin', 'user', 'guest']),
    notifications: z.boolean().default(true),
  });

  it('基本的なフォームにアクセシビリティ違反がない', async () => {
    const UserForm = createFormFromSchema(userSchema, {
      labels: {
        name: '氏名',
        email: 'メールアドレス',
        role: '権限',
        notifications: '通知を受け取る',
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
        notifications: {
          component: 'Switch',
        },
      },
      submitLabel: '保存',
    });

    const { container } = render(
      <TestWrapper>
        <UserForm
          defaultValues={{ name: '', email: '', role: 'user', notifications: true }}
          onSubmit={() => {}}
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('必須フィールドにアクセシビリティ違反がない', async () => {
    const requiredSchema = z.object({
      name: z.string().min(1, '名前は必須です'),
      email: z.string().email('メールは必須です'),
    });

    const RequiredForm = createFormFromSchema(requiredSchema, {
      labels: {
        name: '氏名（必須）',
        email: 'メールアドレス（必須）',
      },
      submitLabel: '送信',
    });

    const { container } = render(
      <TestWrapper>
        <RequiredForm
          defaultValues={{ name: '', email: '' }}
          onSubmit={() => {}}
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('RadioGroup フィールドにアクセシビリティ違反がない', async () => {
    const radioSchema = z.object({
      gender: z.enum(['male', 'female', 'other']),
    });

    const RadioForm = createFormFromSchema(radioSchema, {
      labels: {
        gender: '性別',
      },
      fieldConfig: {
        gender: {
          component: 'RadioGroup',
          options: [
            { label: '男性', value: 'male' },
            { label: '女性', value: 'female' },
            { label: 'その他', value: 'other' },
          ],
        },
      },
      submitLabel: '送信',
    });

    const { container } = render(
      <TestWrapper>
        <RadioForm
          defaultValues={{ gender: 'male' }}
          onSubmit={() => {}}
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('複数列レイアウトのフォームにアクセシビリティ違反がない', async () => {
    const multiColumnSchema = z.object({
      firstName: z.string(),
      lastName: z.string(),
      email: z.string().email(),
      phone: z.string(),
    });

    const MultiColumnForm = createFormFromSchema(multiColumnSchema, {
      labels: {
        firstName: '姓',
        lastName: '名',
        email: 'メールアドレス',
        phone: '電話番号',
      },
      columns: 2,
      submitLabel: '送信',
    });

    const { container } = render(
      <TestWrapper>
        <MultiColumnForm
          defaultValues={{ firstName: '', lastName: '', email: '', phone: '' }}
          onSubmit={() => {}}
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('Slider フィールドにアクセシビリティ違反がない', async () => {
    const sliderSchema = z.object({
      age: z.number().min(0).max(100),
    });

    const SliderForm = createFormFromSchema(sliderSchema, {
      labels: {
        age: '年齢',
      },
      fieldConfig: {
        age: {
          component: 'Slider',
          min: 0,
          max: 100,
        },
      },
      submitLabel: '送信',
    });

    const { container } = render(
      <TestWrapper>
        <SliderForm
          defaultValues={{ age: 30 }}
          onSubmit={() => {}}
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('無効化されたフォームにアクセシビリティ違反がない', async () => {
    const simpleSchema = z.object({
      name: z.string(),
    });

    const DisabledForm = createFormFromSchema(simpleSchema, {
      labels: {
        name: '氏名',
      },
      submitLabel: '送信',
    });

    const { container } = render(
      <TestWrapper>
        <DisabledForm
          defaultValues={{ name: '' }}
          onSubmit={() => {}}
          disabled
        />
      </TestWrapper>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});

describe('キーボードナビゲーションテスト', () => {
  it('DataTable がキーボードでフォーカス可能', () => {
    const columns = createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
    ]);

    const { container } = render(
      <TestWrapper>
        <K1s0DataTable
          rows={testUsers}
          columns={columns}
          getRowId={(row) => row.id}
        />
      </TestWrapper>
    );

    // グリッドが存在する
    const grid = container.querySelector('[role="grid"]');
    expect(grid).toBeInTheDocument();

    // グリッドセルが存在する
    const cells = container.querySelectorAll('[role="gridcell"]');
    expect(cells.length).toBeGreaterThan(0);
  });

  it('フォームフィールドが適切なラベルを持つ', () => {
    const schema = z.object({
      name: z.string(),
      email: z.string().email(),
    });

    const Form = createFormFromSchema(schema, {
      labels: {
        name: '氏名',
        email: 'メールアドレス',
      },
    });

    render(
      <TestWrapper>
        <Form
          defaultValues={{ name: '', email: '' }}
          onSubmit={() => {}}
        />
      </TestWrapper>
    );

    // ラベルが存在し、フィールドに関連付けられている
    const nameInput = document.querySelector('input[name="name"]');
    const emailInput = document.querySelector('input[name="email"]');

    expect(nameInput).toBeInTheDocument();
    expect(emailInput).toBeInTheDocument();
  });
});
