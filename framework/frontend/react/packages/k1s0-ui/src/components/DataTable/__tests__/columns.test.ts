/**
 * カラムヘルパーテスト
 */

import { describe, it, expect } from 'vitest';
import {
  createColumns,
  dateColumn,
  dateTimeColumn,
  numberColumn,
  currencyColumn,
  percentColumn,
} from '../columns/index.js';

interface TestData {
  id: string;
  name: string;
  price: number;
  rate: number;
  createdAt: Date;
}

describe('createColumns', () => {
  it('カラム定義が正しく作成される', () => {
    const columns = createColumns<TestData>([
      { field: 'name', headerName: '名前' },
      { field: 'price', headerName: '価格' },
    ]);

    expect(columns).toHaveLength(2);
    expect(columns[0].field).toBe('name');
    expect(columns[0].headerName).toBe('名前');
    expect(columns[0].sortable).toBe(true); // デフォルト値
    expect(columns[0].filterable).toBe(true); // デフォルト値
  });

  it('カスタム設定が上書きされる', () => {
    const columns = createColumns<TestData>([
      { field: 'name', headerName: '名前', sortable: false },
    ]);

    expect(columns[0].sortable).toBe(false);
  });
});

describe('dateColumn', () => {
  it('デフォルトフォーマットで日付カラムが作成される', () => {
    const column = dateColumn<TestData>({
      field: 'createdAt',
      headerName: '作成日',
    });

    expect(column.field).toBe('createdAt');
    expect(column.headerName).toBe('作成日');
    expect(column.type).toBe('date');
    expect(column.width).toBe(120);
  });

  it('valueFormatter が正しく動作する', () => {
    const column = dateColumn<TestData>({
      field: 'createdAt',
      headerName: '作成日',
    });

    const formatted = column.valueFormatter!(
      new Date('2024-06-15'),
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('2024/06/15');
  });

  it('カスタムフォーマットが適用される', () => {
    const column = dateColumn<TestData>({
      field: 'createdAt',
      headerName: '作成日',
      format: 'YYYY年MM月DD日',
    });

    const formatted = column.valueFormatter!(
      new Date('2024-06-15'),
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('2024年06月15日');
  });

  it('null 値は空文字を返す', () => {
    const column = dateColumn<TestData>({
      field: 'createdAt',
      headerName: '作成日',
    });

    const formatted = column.valueFormatter!(
      null,
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('');
  });
});

describe('dateTimeColumn', () => {
  it('日時カラムが作成される', () => {
    const column = dateTimeColumn<TestData>({
      field: 'createdAt',
      headerName: '作成日時',
    });

    expect(column.width).toBe(150);

    const formatted = column.valueFormatter!(
      new Date('2024-06-15T14:30:00'),
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('2024/06/15 14:30');
  });
});

describe('numberColumn', () => {
  it('数値カラムが正しく作成される', () => {
    const column = numberColumn<TestData>({
      field: 'price',
      headerName: '価格',
    });

    expect(column.field).toBe('price');
    expect(column.type).toBe('number');
    expect(column.align).toBe('right');
  });

  it('3桁区切りが適用される', () => {
    const column = numberColumn<TestData>({
      field: 'price',
      headerName: '価格',
    });

    const formatted = column.valueFormatter!(
      1234567,
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('1,234,567');
  });

  it('プレフィックスとサフィックスが適用される', () => {
    const column = numberColumn<TestData>({
      field: 'price',
      headerName: '価格',
      prefix: '¥',
      suffix: '円',
    });

    const formatted = column.valueFormatter!(
      1000,
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('¥1,000円');
  });

  it('小数点が適用される', () => {
    const column = numberColumn<TestData>({
      field: 'rate',
      headerName: 'レート',
      decimalPlaces: 2,
      thousandSeparator: false,
    });

    const formatted = column.valueFormatter!(
      3.14159,
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('3.14');
  });
});

describe('currencyColumn', () => {
  it('通貨カラムが正しく作成される', () => {
    const column = currencyColumn<TestData>({
      field: 'price',
      headerName: '価格',
    });

    const formatted = column.valueFormatter!(
      10000,
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('¥10,000');
  });
});

describe('percentColumn', () => {
  it('パーセンテージカラムが正しく作成される', () => {
    const column = percentColumn<TestData>({
      field: 'rate',
      headerName: '割合',
    });

    const formatted = column.valueFormatter!(
      85.5,
      {} as TestData,
      column,
      {} as never
    );
    expect(formatted).toBe('85.5%');
  });
});
