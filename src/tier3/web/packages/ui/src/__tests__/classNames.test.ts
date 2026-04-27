// classNames helper のユニットテスト（DOM レンダリング不要）。

import { describe, it, expect } from 'vitest';
import { classNames } from '../classNames';

describe('classNames', () => {
  it('複数 class を空白区切りで連結する', () => {
    expect(classNames('a', 'b', 'c')).toBe('a b c');
  });

  it('false / null / undefined / 空文字を除去する', () => {
    expect(classNames('a', false, 'b', null, undefined, '', 'c')).toBe('a b c');
  });

  it('全部 falsy なら空文字', () => {
    expect(classNames(false, null, undefined, '')).toBe('');
  });
});
