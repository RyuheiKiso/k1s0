import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { DataTable, type DataTableColumn } from './DataTable';

// テスト用の行データ型
interface Row {
  id: string;
  name: string;
  score: number;
}

// サンプルカラム定義
const columns: DataTableColumn<Row>[] = [
  {
    key: 'name',
    header: '名前',
    accessor: (row) => row.name,
    sortValue: (row) => row.name,
    filterValue: (row) => row.name,
  },
  {
    key: 'score',
    header: 'スコア',
    accessor: (row) => row.score,
    sortValue: (row) => row.score,
  },
];

// テスト用データ
const data: Row[] = [
  { id: '1', name: 'Alice', score: 80 },
  { id: '2', name: 'Bob', score: 60 },
  { id: '3', name: 'Carol', score: 90 },
];

describe('DataTable', () => {
  it('ヘッダーが表示される', () => {
    render(<DataTable data={data} columns={columns} rowKey={(r) => r.id} />);
    expect(screen.getByText('名前')).toBeInTheDocument();
    expect(screen.getByText('スコア')).toBeInTheDocument();
  });

  it('すべての行データが表示される', () => {
    render(<DataTable data={data} columns={columns} rowKey={(r) => r.id} />);
    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.getByText('Bob')).toBeInTheDocument();
    expect(screen.getByText('Carol')).toBeInTheDocument();
  });

  it('データが空の場合は emptyMessage を表示する', () => {
    render(
      <DataTable
        data={[]}
        columns={columns}
        rowKey={(r) => r.id}
        emptyMessage="件数ゼロです"
      />
    );
    expect(screen.getByText('件数ゼロです')).toBeInTheDocument();
  });

  it('filterValue がある場合はフィルター入力欄が表示される', () => {
    render(
      <DataTable
        data={data}
        columns={columns}
        rowKey={(r) => r.id}
        filterPlaceholder="名前で検索"
      />
    );
    expect(screen.getByPlaceholderText('名前で検索')).toBeInTheDocument();
  });

  it('フィルター入力でデータが絞り込まれる', () => {
    render(<DataTable data={data} columns={columns} rowKey={(r) => r.id} />);
    const input = screen.getByPlaceholderText('検索...');
    fireEvent.change(input, { target: { value: 'ali' } });
    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.queryByText('Bob')).not.toBeInTheDocument();
  });

  it('filterValue のないカラムのみの場合はフィルター入力欄が非表示', () => {
    const noFilterColumns: DataTableColumn<Row>[] = [
      { key: 'score', header: 'スコア', accessor: (r) => r.score },
    ];
    render(<DataTable data={data} columns={noFilterColumns} rowKey={(r) => r.id} />);
    expect(screen.queryByPlaceholderText('検索...')).not.toBeInTheDocument();
  });

  it('ソート可能なヘッダークリックで昇順ソートされる', () => {
    render(<DataTable data={data} columns={columns} rowKey={(r) => r.id} />);
    fireEvent.click(screen.getByText('名前'));
    const cells = screen.getAllByRole('cell').filter((c) => ['Alice', 'Bob', 'Carol'].includes(c.textContent ?? ''));
    expect(cells[0]).toHaveTextContent('Alice');
    expect(cells[1]).toHaveTextContent('Bob');
    expect(cells[2]).toHaveTextContent('Carol');
  });

  it('同じヘッダーを2回クリックすると降順になる', () => {
    render(<DataTable data={data} columns={columns} rowKey={(r) => r.id} />);
    const nameHeader = screen.getByText('名前');
    fireEvent.click(nameHeader);
    fireEvent.click(nameHeader);
    const cells = screen.getAllByRole('cell').filter((c) => ['Alice', 'Bob', 'Carol'].includes(c.textContent ?? ''));
    expect(cells[0]).toHaveTextContent('Carol');
  });

  it('行クリックで onRowClick が呼ばれる', () => {
    const onRowClick = vi.fn();
    render(
      <DataTable data={data} columns={columns} rowKey={(r) => r.id} onRowClick={onRowClick} />
    );
    fireEvent.click(screen.getByText('Alice'));
    expect(onRowClick).toHaveBeenCalledWith(data[0]);
  });

  it('ariaLabel が テーブルに設定される', () => {
    render(
      <DataTable data={data} columns={columns} rowKey={(r) => r.id} ariaLabel="ユーザー一覧" />
    );
    expect(screen.getByRole('table', { name: 'ユーザー一覧' })).toBeInTheDocument();
  });
});
