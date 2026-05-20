// axe_full.test.ts — @axe-core/react を使った WCAG AA 準拠テスト
// 設計方針 14: axe-core + WCAG 2.1 AA を CI 必須とする
// 強制機構 03_tier3強制機構 層 11: WCAG 2.1 AA axe-core CI
// FieldDiff 型は packages/state/src/events.ts の FieldDiff（clientFields / serverFields 形式）に揃える

// @testing-library/react をインポートする
import { render } from '@testing-library/react';
// vitest の describe/test/expect をインポートする
import { describe, test, expect } from 'vitest';
// axe-core をインポートする (jsdom + axe の実際の axe.run() 呼び出し)
import axe from 'axe-core';

// BusinessErrorPanel コンポーネントをインポートする
// BusinessErrorPanel の props は errors（readonly BusinessError[]）/ title / onDismiss を使用する
import { BusinessErrorPanel, type BusinessError } from '../packages/ui-components/src/BusinessErrorPanel';

describe('WCAG AA axe-core 準拠テスト', () => {
  test('BusinessErrorPanel が WCAG AA 準拠である', async () => {
    // BusinessErrorPanel に渡すエラー一覧を用意する
    // stale_write subtype の FieldDiff に対応するエラーメッセージを使用する
    // FieldDiff は packages/state/src/events.ts で { clientFields: string[]; serverFields: string[] } として定義されている
    const errors: readonly BusinessError[] = [
      {
        // エラーコード: stale_write（fieldDiff.clientFields と serverFields の disjoint）
        code: 'stale_write',
        // エラーメッセージ: quantity フィールドの競合（fieldDiff.clientFields: ['quantity'], serverFields: ['quantity']）
        message: 'quantity フィールドが競合しています（clientFields: ["quantity"], serverFields: ["quantity"]）',
        // 重大度: エラー
        severity: 'error',
        // 対象フィールド: quantity
        field: 'quantity',
      },
    ];
    // BusinessErrorPanel を DOM にレンダリングする
    const { container } = render(
      // errors / title / onDismiss の props で正しい型を使用する（subtype/fieldDiff/onResolve は BusinessErrorPanel の props に存在しない）
      <BusinessErrorPanel
        // エラー一覧を渡す（FieldDiff は events.ts の型に揃えた上でエラーメッセージとして表現する）
        errors={errors}
        // タイトルを設定する
        title="競合エラー"
        // クローズハンドラを渡す（アクセシビリティテスト用）
        onDismiss={() => {}}
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
