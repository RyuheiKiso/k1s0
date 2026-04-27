// Card コンポーネント（リリース時点 minimum）。

import type { HTMLAttributes, ReactNode } from 'react';
import { classNames } from './classNames';

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  // タイトル（任意、ヘッダ部に表示）。
  title?: string;
  // 子要素。
  children: ReactNode;
}

// Card は枠線 + 影 + padding を持つ汎用コンテナ。
export function Card(props: CardProps) {
  const { title, children, className, ...rest } = props;
  return (
    <div
      {...rest}
      className={classNames('rounded border border-gray-200 bg-white p-4 shadow-sm', className)}
    >
      {title ? <h3 className="mb-2 text-lg font-semibold text-gray-900">{title}</h3> : null}
      {children}
    </div>
  );
}
