// Button コンポーネント（リリース時点 minimum）。

import type { ButtonHTMLAttributes, ReactNode } from 'react';
import { classNames } from './classNames';

// Button の variant 種別。
export type ButtonVariant = 'primary' | 'secondary' | 'danger';

// Button のサイズ種別。
export type ButtonSize = 'sm' | 'md' | 'lg';

// Button のプロパティ（HTMLButton の標準属性 + 独自 variant / size）。
export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  // 色テーマ。
  variant?: ButtonVariant;
  // サイズ。
  size?: ButtonSize;
  // 子要素（ボタンラベル）。
  children: ReactNode;
}

// variant ごとの class 文字列（Tailwind 想定、リリース時点 で shadcn/ui スタイルに揃える）。
const VARIANT_CLASS: Record<ButtonVariant, string> = {
  primary: 'bg-blue-600 text-white hover:bg-blue-700',
  secondary: 'bg-gray-200 text-gray-900 hover:bg-gray-300',
  danger: 'bg-red-600 text-white hover:bg-red-700',
};

// size ごとの padding 設定。
const SIZE_CLASS: Record<ButtonSize, string> = {
  sm: 'px-2 py-1 text-sm',
  md: 'px-3 py-2 text-base',
  lg: 'px-4 py-3 text-lg',
};

// Button は variant + size を受ける標準ボタン。
export function Button(props: ButtonProps) {
  // 既定値を当てる。
  const { variant = 'primary', size = 'md', className, children, ...rest } = props;
  // class を組み立てる。
  const merged = classNames(
    'inline-flex items-center justify-center rounded font-medium transition-colors',
    'disabled:cursor-not-allowed disabled:opacity-50',
    VARIANT_CLASS[variant],
    SIZE_CLASS[size],
    className,
  );
  return (
    <button {...rest} className={merged}>
      {children}
    </button>
  );
}
