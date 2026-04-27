// Spinner コンポーネント（読み込み表示）。

import type { HTMLAttributes } from 'react';
import { classNames } from './classNames';

export interface SpinnerProps extends HTMLAttributes<HTMLDivElement> {
  // サイズ（px）。
  size?: number;
  // 色 class（Tailwind の text-*-* を渡す前提）。
  colorClass?: string;
}

// Spinner は CSS アニメーションでくるくる回るローディングインジケータ。
export function Spinner(props: SpinnerProps) {
  const { size = 24, colorClass = 'text-blue-600', className, style, ...rest } = props;
  return (
    <div
      {...rest}
      role="status"
      aria-label="loading"
      className={classNames('inline-block animate-spin', colorClass, className)}
      style={{ width: size, height: size, ...style }}
    >
      <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="12" cy="12" r="10" stroke="currentColor" strokeOpacity="0.25" strokeWidth="4" />
        <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" strokeWidth="4" strokeLinecap="round" />
      </svg>
    </div>
  );
}
