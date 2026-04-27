// @k1s0/i18n の単体テスト。

import { describe, it, expect } from 'vitest';
import { createI18n, availableLocales } from '../index';

describe('createI18n', () => {
  it('ja でキーを解決する', () => {
    const i18n = createI18n('ja');
    expect(i18n.t('common.welcome')).toBe('ようこそ');
    expect(i18n.locale).toBe('ja');
  });

  it('en でキーを解決する', () => {
    const i18n = createI18n('en');
    expect(i18n.t('common.welcome')).toBe('Welcome');
  });

  it('未存在キーは key 自体を返す（debug fallback）', () => {
    const i18n = createI18n('ja');
    expect(i18n.t('does.not.exist')).toBe('does.not.exist');
  });

  it('vars で placeholder を補間する', () => {
    const i18n = createI18n('ja');
    // 一時的にテスト用の翻訳を作る代わりに、未存在キー + 補間を検証する（key 自体に補間がかかる）。
    expect(i18n.t('hello {name}', { name: '世界' })).toBe('hello 世界');
  });
});

describe('availableLocales', () => {
  it('ja / en の 2 ロケールを返す', () => {
    const locales = availableLocales();
    expect(locales).toContain('ja');
    expect(locales).toContain('en');
  });
});
