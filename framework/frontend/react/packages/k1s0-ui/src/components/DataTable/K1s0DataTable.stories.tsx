import type { Meta, StoryObj } from '@storybook/react';
import { fn } from '@storybook/test';
import {
  K1s0DataTable,
  createColumns,
  dateColumn,
  numberColumn,
  booleanColumn,
  actionsColumn,
  statusColumn,
} from './index';

// サンプルデータ型
interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
  status: 'active' | 'inactive' | 'pending';
  age: number;
  salary: number;
  isVerified: boolean;
  createdAt: Date;
  lastLogin: Date | null;
}

// サンプルデータ
const sampleUsers: User[] = [
  {
    id: '1',
    name: '山田太郎',
    email: 'yamada@example.com',
    role: 'admin',
    status: 'active',
    age: 35,
    salary: 800000,
    isVerified: true,
    createdAt: new Date('2023-01-15'),
    lastLogin: new Date('2024-01-20'),
  },
  {
    id: '2',
    name: '鈴木花子',
    email: 'suzuki@example.com',
    role: 'user',
    status: 'active',
    age: 28,
    salary: 550000,
    isVerified: true,
    createdAt: new Date('2023-03-22'),
    lastLogin: new Date('2024-01-19'),
  },
  {
    id: '3',
    name: '田中一郎',
    email: 'tanaka@example.com',
    role: 'guest',
    status: 'pending',
    age: 42,
    salary: 0,
    isVerified: false,
    createdAt: new Date('2023-06-10'),
    lastLogin: null,
  },
  {
    id: '4',
    name: '佐藤美咲',
    email: 'sato@example.com',
    role: 'user',
    status: 'inactive',
    age: 31,
    salary: 620000,
    isVerified: true,
    createdAt: new Date('2023-02-28'),
    lastLogin: new Date('2023-12-01'),
  },
  {
    id: '5',
    name: '高橋健太',
    email: 'takahashi@example.com',
    role: 'user',
    status: 'active',
    age: 25,
    salary: 480000,
    isVerified: false,
    createdAt: new Date('2023-08-05'),
    lastLogin: new Date('2024-01-18'),
  },
];

// 大量データ生成
const generateLargeDataset = (count: number): User[] => {
  const roles: User['role'][] = ['admin', 'user', 'guest'];
  const statuses: User['status'][] = ['active', 'inactive', 'pending'];

  return Array.from({ length: count }, (_, i) => ({
    id: String(i + 1),
    name: `ユーザー${i + 1}`,
    email: `user${i + 1}@example.com`,
    role: roles[i % 3],
    status: statuses[i % 3],
    age: 20 + (i % 50),
    salary: 300000 + (i % 10) * 50000,
    isVerified: i % 2 === 0,
    createdAt: new Date(2023, i % 12, (i % 28) + 1),
    lastLogin: i % 3 === 0 ? null : new Date(2024, 0, (i % 28) + 1),
  }));
};

const meta: Meta<typeof K1s0DataTable<User>> = {
  title: 'Components/DataTable/K1s0DataTable',
  component: K1s0DataTable,
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        component: 'MUI DataGrid をベースにした高機能データテーブルコンポーネント',
      },
    },
  },
  tags: ['autodocs'],
};

export default meta;
type Story = StoryObj<typeof K1s0DataTable<User>>;

// 基本カラム
const basicColumns = createColumns<User>([
  { field: 'name', headerName: '氏名', flex: 1, sortable: true },
  { field: 'email', headerName: 'メール', flex: 1 },
  { field: 'role', headerName: '権限', width: 100 },
]);

/**
 * 基本的なテーブル表示
 */
export const Basic: Story = {
  args: {
    rows: sampleUsers,
    columns: basicColumns,
    getRowId: (row) => row.id,
  },
};

/**
 * ソート機能付き
 */
export const WithSorting: Story = {
  args: {
    rows: sampleUsers,
    columns: createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1, sortable: true },
      { field: 'email', headerName: 'メール', flex: 1, sortable: true },
      { field: 'age', headerName: '年齢', width: 100, sortable: true, type: 'number' },
      dateColumn<User>({ field: 'createdAt', headerName: '作成日', sortable: true }),
    ]),
    getRowId: (row) => row.id,
    initialState: {
      sorting: {
        sortModel: [{ field: 'name', sort: 'asc' }],
      },
    },
  },
};

/**
 * フィルタリング機能付き
 */
export const WithFiltering: Story = {
  args: {
    rows: sampleUsers,
    columns: createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1, filterable: true },
      { field: 'email', headerName: 'メール', flex: 1, filterable: true },
      {
        field: 'role',
        headerName: '権限',
        width: 120,
        filterable: true,
        type: 'singleSelect',
        valueOptions: [
          { value: 'admin', label: '管理者' },
          { value: 'user', label: '一般' },
          { value: 'guest', label: 'ゲスト' },
        ],
      },
    ]),
    getRowId: (row) => row.id,
    filterMode: 'client',
  },
};

/**
 * ページネーション付き
 */
export const WithPagination: Story = {
  args: {
    rows: generateLargeDataset(100),
    columns: basicColumns,
    getRowId: (row) => row.id,
    pagination: true,
    pageSizeOptions: [5, 10, 25, 50],
    initialState: {
      pagination: {
        paginationModel: { pageSize: 10, page: 0 },
      },
    },
  },
};

/**
 * チェックボックス選択
 */
export const WithSelection: Story = {
  args: {
    rows: sampleUsers,
    columns: basicColumns,
    getRowId: (row) => row.id,
    checkboxSelection: true,
    onRowSelectionModelChange: fn(),
  },
};

/**
 * 行クリックイベント
 */
export const WithRowClick: Story = {
  args: {
    rows: sampleUsers,
    columns: basicColumns,
    getRowId: (row) => row.id,
    onRowClick: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: '行をクリックすると onRowClick イベントが発火します',
      },
    },
  },
};

/**
 * ローディング状態
 */
export const Loading: Story = {
  args: {
    rows: [],
    columns: basicColumns,
    getRowId: (row) => row.id,
    loading: true,
  },
};

/**
 * 空状態
 */
export const Empty: Story = {
  args: {
    rows: [],
    columns: basicColumns,
    getRowId: (row) => row.id,
    localeText: {
      noRowsLabel: 'データがありません',
    },
  },
};

/**
 * カスタムカラム（日付）
 */
export const WithDateColumn: Story = {
  args: {
    rows: sampleUsers,
    columns: createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      dateColumn<User>({ field: 'createdAt', headerName: '作成日' }),
      dateColumn<User>({
        field: 'lastLogin',
        headerName: '最終ログイン',
        format: 'YYYY/MM/DD HH:mm',
        emptyText: '未ログイン',
      }),
    ]),
    getRowId: (row) => row.id,
  },
};

/**
 * カスタムカラム（数値）
 */
export const WithNumberColumn: Story = {
  args: {
    rows: sampleUsers,
    columns: createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'age', headerName: '年齢', width: 100, type: 'number' },
      numberColumn<User>({
        field: 'salary',
        headerName: '給与',
        format: 'currency',
        currency: 'JPY',
      }),
    ]),
    getRowId: (row) => row.id,
  },
};

/**
 * カスタムカラム（ブール値）
 */
export const WithBooleanColumn: Story = {
  args: {
    rows: sampleUsers,
    columns: createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
      booleanColumn<User>({ field: 'isVerified', headerName: '認証済み' }),
    ]),
    getRowId: (row) => row.id,
  },
};

/**
 * カスタムカラム（ステータス）
 */
export const WithStatusColumn: Story = {
  args: {
    rows: sampleUsers,
    columns: createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
      statusColumn<User>({
        field: 'status',
        headerName: 'ステータス',
        statusConfig: {
          active: { label: '有効', color: 'success' },
          inactive: { label: '無効', color: 'error' },
          pending: { label: '保留中', color: 'warning' },
        },
      }),
    ]),
    getRowId: (row) => row.id,
  },
};

/**
 * アクションカラム付き
 */
export const WithActions: Story = {
  args: {
    rows: sampleUsers,
    columns: createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1 },
      { field: 'email', headerName: 'メール', flex: 1 },
      actionsColumn<User>({
        onEdit: fn(),
        onDelete: fn(),
      }),
    ]),
    getRowId: (row) => row.id,
  },
};

/**
 * ツールバー付き
 */
export const WithToolbar: Story = {
  args: {
    rows: sampleUsers,
    columns: basicColumns,
    getRowId: (row) => row.id,
    slots: {
      toolbar: () => (
        <div style={{ padding: '8px 16px', borderBottom: '1px solid #e0e0e0' }}>
          カスタムツールバー
        </div>
      ),
    },
  },
};

/**
 * 全機能統合
 */
export const FullFeatured: Story = {
  args: {
    rows: generateLargeDataset(50),
    columns: createColumns<User>([
      { field: 'name', headerName: '氏名', flex: 1, sortable: true, filterable: true },
      { field: 'email', headerName: 'メール', flex: 1, filterable: true },
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
      statusColumn<User>({
        field: 'status',
        headerName: 'ステータス',
        statusConfig: {
          active: { label: '有効', color: 'success' },
          inactive: { label: '無効', color: 'error' },
          pending: { label: '保留中', color: 'warning' },
        },
      }),
      numberColumn<User>({
        field: 'salary',
        headerName: '給与',
        format: 'currency',
        currency: 'JPY',
      }),
      booleanColumn<User>({ field: 'isVerified', headerName: '認証済み' }),
      dateColumn<User>({ field: 'createdAt', headerName: '作成日', sortable: true }),
      actionsColumn<User>({
        onEdit: fn(),
        onDelete: fn(),
      }),
    ]),
    getRowId: (row) => row.id,
    checkboxSelection: true,
    pagination: true,
    pageSizeOptions: [10, 25, 50],
    initialState: {
      pagination: {
        paginationModel: { pageSize: 10, page: 0 },
      },
    },
    onRowSelectionModelChange: fn(),
    onRowClick: fn(),
  },
  parameters: {
    docs: {
      description: {
        story:
          'ソート、フィルタ、ページネーション、選択、アクションなど全機能を統合したテーブル',
      },
    },
  },
};
