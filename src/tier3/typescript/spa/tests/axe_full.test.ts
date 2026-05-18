// axe_full.test.ts — @axe-core/react を使った WCAG AA 準拠テスト
// 設計方針 14: axe-core + WCAG 2.1 AA を CI 必須とする
// 強制機構 03_tier3強制機構 層 11: WCAG 2.1 AA axe-core CI

// @testing-library/react をインポートする
import { render } from '@testing-library/react';
// vitest の describe/test/expect をインポートする
import { describe, test, expect } from 'vitest';
// axe-core をインポートする (jsdom + axe の実際の axe.run() 呼び出し)
import axe from 'axe-core';

// BusinessErrorPanel コンポーネントをインポートする
import { BusinessErrorPanel } from '../packages/ui-components/src/BusinessErrorPanel';

describe('WCAG AA axe-core 準拠テスト', () => {
  test('BusinessErrorPanel が WCAG AA 準拠である', async () => {
    // BusinessErrorPanel を DOM にレンダリングする
    const { container } = render(
      <BusinessErrorPanel
        subtype="stale_write"
        fieldDiff={{ field: 'quantity', local: 5, remote: 3 }}
        onResolve={() => {}}
      />
    );

    // axe.run() で WCAG AA 違反を検出する
    const results = await axe.run(container, {
      runOnly: {
        type: 'tag',
        values: ['wcag2a', 'wcag2aa'],
      },
    });

    // WCAG AA 違反が 0 件であることを確認する
    expect(results.violations).toHaveLength(0);
  });

  test('PermissionGate が WCAG AA 準拠である', async () => {
    // PermissionGate コンポーネントをインポートする
    const { PermissionGate } = await import('../packages/ui-components/src/PermissionGate');
    // PermissionGate をレンダリングする
    const { container } = render(
      <PermissionGate roles={['inspector']} userRoles={['inspector']}>
        <div>Content</div>
      </PermissionGate>
    );
    // axe.run() で WCAG AA 違反を検出する
    const results = await axe.run(container, {
      runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa'] },
    });
    // WCAG AA 違反が 0 件であることを確認する
    expect(results.violations).toHaveLength(0);
  });
});
