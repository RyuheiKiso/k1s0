/**
 * createFormFromSchema テスト
 */

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { z } from 'zod';
import { createFormFromSchema } from '../createFormFromSchema.js';

// テスト用のテーマ
const theme = createTheme();

// テスト用のラッパー
function TestWrapper({ children }: { children: React.ReactNode }) {
  return <ThemeProvider theme={theme}>{children}</ThemeProvider>;
}

describe('createFormFromSchema', () => {
  describe('基本レンダリング', () => {
    it('スキーマからフォームが生成される', () => {
      const schema = z.object({
        name: z.string().min(1),
        email: z.string().email(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { name: '氏名', email: 'メールアドレス' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      expect(screen.getByLabelText(/氏名/)).toBeInTheDocument();
      expect(screen.getByLabelText(/メールアドレス/)).toBeInTheDocument();
    });

    it('送信ボタンが表示される', () => {
      const schema = z.object({
        name: z.string(),
      });

      const Form = createFormFromSchema(schema, {
        submitLabel: '保存',
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument();
    });

    it('キャンセルボタンが表示される', () => {
      const schema = z.object({
        name: z.string(),
      });

      const Form = createFormFromSchema(schema, {
        showCancel: true,
        cancelLabel: 'キャンセル',
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} onCancel={vi.fn()} />
        </TestWrapper>
      );

      expect(screen.getByRole('button', { name: 'キャンセル' })).toBeInTheDocument();
    });
  });

  describe('フィールドタイプ', () => {
    it('string フィールドは TextField として表示される', () => {
      const schema = z.object({
        name: z.string(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { name: '氏名' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      const input = screen.getByLabelText(/氏名/) as HTMLInputElement;
      expect(input.type).toBe('text');
    });

    it('email フィールドは type="email" として表示される', () => {
      const schema = z.object({
        email: z.string().email(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { email: 'メール' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      const input = screen.getByLabelText(/メール/) as HTMLInputElement;
      expect(input.type).toBe('email');
    });

    it('number フィールドは type="number" として表示される', () => {
      const schema = z.object({
        age: z.number(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { age: '年齢' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      const input = screen.getByLabelText(/年齢/) as HTMLInputElement;
      expect(input.type).toBe('number');
    });

    it('boolean フィールドは Switch として表示される', () => {
      const schema = z.object({
        active: z.boolean(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { active: '有効' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      expect(screen.getByRole('checkbox', { name: /有効/ })).toBeInTheDocument();
    });

    it('enum フィールドは Select として表示される（5項目以上）', () => {
      const schema = z.object({
        status: z.enum(['a', 'b', 'c', 'd', 'e']),
      });

      const Form = createFormFromSchema(schema, {
        labels: { status: 'ステータス' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      expect(screen.getByLabelText(/ステータス/)).toBeInTheDocument();
    });
  });

  describe('バリデーション', () => {
    it('必須フィールドのエラーが表示される', async () => {
      const schema = z.object({
        name: z.string().min(1, '名前は必須です'),
      });

      const Form = createFormFromSchema(schema, {
        labels: { name: '氏名' },
        submitLabel: '送信',
      });

      const handleSubmit = vi.fn();

      render(
        <TestWrapper>
          <Form onSubmit={handleSubmit} />
        </TestWrapper>
      );

      // 送信ボタンをクリック
      fireEvent.click(screen.getByRole('button', { name: '送信' }));

      // エラーメッセージが表示されることを確認
      await waitFor(() => {
        expect(screen.getByText('名前は必須です')).toBeInTheDocument();
      });

      // onSubmit は呼ばれない
      expect(handleSubmit).not.toHaveBeenCalled();
    });

    it('メール形式のバリデーションエラーが表示される', async () => {
      const schema = z.object({
        email: z.string().email('有効なメールアドレスを入力してください'),
      });

      const Form = createFormFromSchema(schema, {
        labels: { email: 'メール' },
        submitLabel: '送信',
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      // 無効なメールを入力
      const input = screen.getByLabelText(/メール/);
      await userEvent.type(input, 'invalid-email');
      fireEvent.blur(input);

      // エラーメッセージが表示されることを確認
      await waitFor(() => {
        expect(screen.getByText('有効なメールアドレスを入力してください')).toBeInTheDocument();
      });
    });
  });

  describe('送信', () => {
    it('有効なフォームが送信される', async () => {
      const schema = z.object({
        name: z.string().min(1),
        email: z.string().email(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { name: '氏名', email: 'メール' },
        submitLabel: '送信',
      });

      const handleSubmit = vi.fn();

      render(
        <TestWrapper>
          <Form onSubmit={handleSubmit} />
        </TestWrapper>
      );

      // フォームに入力
      await userEvent.type(screen.getByLabelText(/氏名/), '山田太郎');
      await userEvent.type(screen.getByLabelText(/メール/), 'yamada@example.com');

      // 送信
      fireEvent.click(screen.getByRole('button', { name: '送信' }));

      await waitFor(() => {
        expect(handleSubmit).toHaveBeenCalledWith({
          name: '山田太郎',
          email: 'yamada@example.com',
        });
      });
    });
  });

  describe('デフォルト値', () => {
    it('defaultValues が反映される', () => {
      const schema = z.object({
        name: z.string(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { name: '氏名' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} defaultValues={{ name: '初期値' }} />
        </TestWrapper>
      );

      const input = screen.getByLabelText(/氏名/) as HTMLInputElement;
      expect(input.value).toBe('初期値');
    });

    it('スキーマの default() が反映される', () => {
      const schema = z.object({
        active: z.boolean().default(true),
      });

      const Form = createFormFromSchema(schema, {
        labels: { active: '有効' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} />
        </TestWrapper>
      );

      const checkbox = screen.getByRole('checkbox', { name: /有効/ }) as HTMLInputElement;
      expect(checkbox.checked).toBe(true);
    });
  });

  describe('無効化', () => {
    it('disabled=true でフォームが無効化される', () => {
      const schema = z.object({
        name: z.string(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { name: '氏名' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} disabled />
        </TestWrapper>
      );

      const input = screen.getByLabelText(/氏名/) as HTMLInputElement;
      expect(input).toBeDisabled();
    });

    it('loading=true でフォームが無効化される', () => {
      const schema = z.object({
        name: z.string(),
      });

      const Form = createFormFromSchema(schema, {
        labels: { name: '氏名' },
      });

      render(
        <TestWrapper>
          <Form onSubmit={vi.fn()} loading />
        </TestWrapper>
      );

      const input = screen.getByLabelText(/氏名/) as HTMLInputElement;
      expect(input).toBeDisabled();
    });
  });
});
