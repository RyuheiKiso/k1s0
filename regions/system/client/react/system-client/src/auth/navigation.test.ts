import { describe, it, expect, vi, beforeEach } from 'vitest';
import { navigateTo, setNavigateImpl, resetNavigateImpl } from './navigation';

describe('navigation', () => {
  beforeEach(() => {
    // 各テスト後にデフォルト実装に戻す
    resetNavigateImpl();
  });

  it('setNavigateImpl でカスタム実装に差し替えられる', () => {
    const customNav = vi.fn();
    setNavigateImpl(customNav);
    navigateTo('/dashboard');
    expect(customNav).toHaveBeenCalledWith('/dashboard');
  });

  it('resetNavigateImpl でデフォルト実装に戻る', () => {
    const customNav = vi.fn();
    setNavigateImpl(customNav);
    resetNavigateImpl();

    // リセット後はカスタム実装が呼ばれない
    Object.defineProperty(window, 'location', {
      value: { href: '' },
      writable: true,
    });
    navigateTo('/home');
    expect(customNav).not.toHaveBeenCalled();
  });

  it('navigateTo がカスタム実装に URL を渡す', () => {
    const spy = vi.fn();
    setNavigateImpl(spy);
    navigateTo('/settings/profile');
    expect(spy).toHaveBeenCalledWith('/settings/profile');
  });

  it('異なる URL で複数回 navigateTo を呼ぶと順番通りに実行される', () => {
    const calls: string[] = [];
    setNavigateImpl((url) => calls.push(url));
    navigateTo('/page1');
    navigateTo('/page2');
    navigateTo('/page3');
    expect(calls).toEqual(['/page1', '/page2', '/page3']);
  });
});
