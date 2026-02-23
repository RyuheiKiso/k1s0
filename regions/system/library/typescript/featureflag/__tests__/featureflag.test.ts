import { describe, it, expect } from 'vitest';
import {
  InMemoryFeatureFlagClient,
  FeatureFlagError,
  type FeatureFlag,
  type EvaluationContext,
} from '../src/index.js';

describe('InMemoryFeatureFlagClient', () => {
  const enabledFlag: FeatureFlag = {
    id: 'flag-1',
    flagKey: 'dark-mode',
    description: 'ダークモード機能',
    enabled: true,
    variants: [{ name: 'dark', value: 'true', weight: 100 }],
  };

  const disabledFlag: FeatureFlag = {
    id: 'flag-2',
    flagKey: 'beta-feature',
    description: 'ベータ機能',
    enabled: false,
    variants: [],
  };

  const ctx: EvaluationContext = { userId: 'user-1' };

  it('有効フラグのevaluateでenabled=trueを返す', async () => {
    const client = new InMemoryFeatureFlagClient();
    client.setFlag(enabledFlag);

    const result = await client.evaluate('dark-mode', ctx);
    expect(result.enabled).toBe(true);
    expect(result.flagKey).toBe('dark-mode');
    expect(result.reason).toBe('FLAG_ENABLED');
    expect(result.variant).toBe('dark');
  });

  it('無効フラグのevaluateでenabled=falseを返す', async () => {
    const client = new InMemoryFeatureFlagClient();
    client.setFlag(disabledFlag);

    const result = await client.evaluate('beta-feature', ctx);
    expect(result.enabled).toBe(false);
    expect(result.reason).toBe('FLAG_DISABLED');
    expect(result.variant).toBeUndefined();
  });

  it('存在しないフラグでFeatureFlagErrorを投げる', async () => {
    const client = new InMemoryFeatureFlagClient();
    await expect(client.evaluate('nonexistent', ctx)).rejects.toThrow(FeatureFlagError);
  });

  it('getFlagでフラグ情報を取得できる', async () => {
    const client = new InMemoryFeatureFlagClient();
    client.setFlag(enabledFlag);

    const flag = await client.getFlag('dark-mode');
    expect(flag.id).toBe('flag-1');
    expect(flag.description).toBe('ダークモード機能');
    expect(flag.variants).toHaveLength(1);
  });

  it('存在しないフラグのgetFlagでエラーを投げる', async () => {
    const client = new InMemoryFeatureFlagClient();
    await expect(client.getFlag('nonexistent')).rejects.toThrow(FeatureFlagError);
  });

  it('isEnabledで有効フラグはtrueを返す', async () => {
    const client = new InMemoryFeatureFlagClient();
    client.setFlag(enabledFlag);
    expect(await client.isEnabled('dark-mode', ctx)).toBe(true);
  });

  it('isEnabledで無効フラグはfalseを返す', async () => {
    const client = new InMemoryFeatureFlagClient();
    client.setFlag(disabledFlag);
    expect(await client.isEnabled('beta-feature', ctx)).toBe(false);
  });

  it('setFlagでフラグを上書きできる', async () => {
    const client = new InMemoryFeatureFlagClient();
    client.setFlag(enabledFlag);
    expect(await client.isEnabled('dark-mode', ctx)).toBe(true);

    client.setFlag({ ...enabledFlag, enabled: false });
    expect(await client.isEnabled('dark-mode', ctx)).toBe(false);
  });

  it('contextにattributesを渡せる', async () => {
    const client = new InMemoryFeatureFlagClient();
    client.setFlag(enabledFlag);

    const ctxWithAttrs: EvaluationContext = {
      userId: 'user-1',
      tenantId: 'tenant-1',
      attributes: { region: 'ap-northeast-1' },
    };
    const result = await client.evaluate('dark-mode', ctxWithAttrs);
    expect(result.enabled).toBe(true);
  });
});
