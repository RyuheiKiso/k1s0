/**
 * K1s0DataTable テスト
 */

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { K1s0DataTable } from '../K1s0DataTable.js';
import { createColumns, dateColumn, actionsColumn } from '../columns/index.js';

// テスト用のテーマ
const theme = createTheme();

// テスト用のデータ型
interface TestUser {
  id: string;
  name: string;
  email: string;
  age: number;
  isActive: boolean;
  createdAt: Date;
}

// テスト用のデータ
const testUsers: TestUser[] = [
  {
    id: '1',
    name: '山田太郎',
    email: 'yamada@example.com',
    age: 30,
    isActive: true,
    createdAt: new Date('2024-01-01'),
  },
  {
    id: '2',
    name: '佐藤花子',
    email: 'sato@example.com',
    age: 25,
    isActive: false,
    createdAt: new Date('2024-02-15'),
  },
  {
    id: '3',
    name: '鈴木一郎',
    email: 'suzuki@example.com',
    age: 35,
    isActive: true,
    createdAt: new Date('2024-03-20'),
  },
];

// テスト用のカラム
const testColumns = createColumns<TestUser>([
  { field: 'name', headerName: '氏名', flex: 1 },
  { field: 'email', headerName: 'メール', flex: 1 },
  { field: 'age', headerName: '年齢', width: 100 },
]);

// テスト用のラッパー
function TestWrapper({ children }: { children: React.ReactNode }) {
  return <ThemeProvider theme={theme}>{children}</ThemeProvider>;
}

describe('K1s0DataTable', () => {
  describe('基本レンダリング', () => {
    it('データが正しく表示される', async () => {
      render(
        <TestWrapper>
          <K1s0DataTable rows={testUsers} columns={testColumns} />
        </TestWrapper>
      );

      // データが表示されるまで待機
      await waitFor(() => {
        expect(screen.getByText('山田太郎')).toBeInTheDocument();
      });

      expect(screen.getByText('佐藤花子')).toBeInTheDocument();
      expect(screen.getByText('鈴木一郎')).toBeInTheDocument();
    });

    it('カラムヘッダーが正しく表示される', async () => {
      render(
        <TestWrapper>
          <K1s0DataTable rows={testUsers} columns={testColumns} />
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByText('氏名')).toBeInTheDocument();
      });

      expect(screen.getByText('メール')).toBeInTheDocument();
      expect(screen.getByText('年齢')).toBeInTheDocument();
    });

    it('空のデータで「データがありません」が表示される', async () => {
      render(
        <TestWrapper>
          <K1s0DataTable rows={[]} columns={testColumns} />
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByText('データがありません')).toBeInTheDocument();
      });
    });
  });

  describe('ローディング', () => {
    it('loading=true でローディング表示される', async () => {
      render(
        <TestWrapper>
          <K1s0DataTable rows={[]} columns={testColumns} loading={true} />
        </TestWrapper>
      );

      // MUI DataGrid のローディングインジケーターを確認
      await waitFor(() => {
        expect(
          document.querySelector('.MuiDataGrid-overlay')
        ).toBeInTheDocument();
      });
    });
  });

  describe('選択', () => {
    it('チェックボックス選択が動作する', async () => {
      const handleSelectionChange = vi.fn();

      render(
        <TestWrapper>
          <K1s0DataTable
            rows={testUsers}
            columns={testColumns}
            checkboxSelection
            onRowSelectionModelChange={handleSelectionChange}
          />
        </TestWrapper>
      );

      // データが表示されるまで待機
      await waitFor(() => {
        expect(screen.getByText('山田太郎')).toBeInTheDocument();
      });

      // チェックボックスをクリック
      const checkboxes = screen.getAllByRole('checkbox');
      fireEvent.click(checkboxes[1]); // 最初の行のチェックボックス

      expect(handleSelectionChange).toHaveBeenCalled();
    });
  });

  describe('ページネーション', () => {
    it('ページネーションコントロールが表示される', async () => {
      render(
        <TestWrapper>
          <K1s0DataTable
            rows={testUsers}
            columns={testColumns}
            pagination
            pageSize={1}
            pageSizeOptions={[1, 5, 10]}
          />
        </TestWrapper>
      );

      await waitFor(() => {
        // ページネーションのラベルが表示されることを確認
        expect(screen.getByText(/表示件数/)).toBeInTheDocument();
      });
    });
  });

  describe('行クリック', () => {
    it('行クリックイベントが発火する', async () => {
      const handleRowClick = vi.fn();

      render(
        <TestWrapper>
          <K1s0DataTable
            rows={testUsers}
            columns={testColumns}
            onRowClick={handleRowClick}
          />
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByText('山田太郎')).toBeInTheDocument();
      });

      // 行をクリック
      fireEvent.click(screen.getByText('山田太郎'));

      expect(handleRowClick).toHaveBeenCalledWith(
        expect.objectContaining({ id: '1', name: '山田太郎' })
      );
    });
  });

  describe('ツールバー', () => {
    it('toolbar=true でツールバーが表示される', async () => {
      render(
        <TestWrapper>
          <K1s0DataTable rows={testUsers} columns={testColumns} toolbar />
        </TestWrapper>
      );

      await waitFor(() => {
        // ツールバーボタンが表示されることを確認
        expect(screen.getByText('列')).toBeInTheDocument();
        expect(screen.getByText('フィルタ')).toBeInTheDocument();
      });
    });

    it('CSV エクスポートボタンが表示される', async () => {
      render(
        <TestWrapper>
          <K1s0DataTable
            rows={testUsers}
            columns={testColumns}
            toolbar
            exportOptions={{ csv: true }}
          />
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByText('CSV')).toBeInTheDocument();
      });
    });
  });
});

describe('カラムヘルパー', () => {
  describe('dateColumn', () => {
    it('日付が正しくフォーマットされる', async () => {
      const columnsWithDate = [
        ...testColumns,
        dateColumn<TestUser>({ field: 'createdAt', headerName: '作成日' }),
      ];

      render(
        <TestWrapper>
          <K1s0DataTable rows={testUsers} columns={columnsWithDate} />
        </TestWrapper>
      );

      await waitFor(() => {
        expect(screen.getByText('2024/01/01')).toBeInTheDocument();
      });
    });
  });

  describe('actionsColumn', () => {
    it('編集・削除ボタンが表示される', async () => {
      const handleEdit = vi.fn();
      const handleDelete = vi.fn();

      const columnsWithActions = [
        ...testColumns,
        actionsColumn<TestUser>({
          onEdit: handleEdit,
          onDelete: handleDelete,
        }),
      ];

      render(
        <TestWrapper>
          <K1s0DataTable rows={testUsers} columns={columnsWithActions} />
        </TestWrapper>
      );

      await waitFor(() => {
        // 編集ボタンが存在することを確認
        const editButtons = screen.getAllByLabelText('編集');
        expect(editButtons.length).toBeGreaterThan(0);
      });

      // 削除ボタンが存在することを確認
      const deleteButtons = screen.getAllByLabelText('削除');
      expect(deleteButtons.length).toBeGreaterThan(0);
    });

    it('編集ボタンクリックでコールバックが呼ばれる', async () => {
      const handleEdit = vi.fn();

      const columnsWithActions = [
        ...testColumns,
        actionsColumn<TestUser>({ onEdit: handleEdit }),
      ];

      render(
        <TestWrapper>
          <K1s0DataTable rows={testUsers} columns={columnsWithActions} />
        </TestWrapper>
      );

      await waitFor(() => {
        const editButtons = screen.getAllByLabelText('編集');
        expect(editButtons.length).toBeGreaterThan(0);
      });

      const editButtons = screen.getAllByLabelText('編集');
      fireEvent.click(editButtons[0]);

      expect(handleEdit).toHaveBeenCalledWith(
        expect.objectContaining({ id: '1' })
      );
    });
  });
});
