// BusinessErrorPanel.stories.tsx — BusinessErrorPanel の Storybook stories
// 設計方針 32_開発者体験: UI コンポーネントカタログ

// Storybook の Meta/StoryObj 型をインポートする
import type { Meta, StoryObj } from '@storybook/react';
// BusinessErrorPanel コンポーネントをインポートする
import { BusinessErrorPanel } from './BusinessErrorPanel';

// BusinessErrorPanel の Meta 設定を定義する
const meta: Meta<typeof BusinessErrorPanel> = {
  // Storybook のカテゴリパスを設定する
  title: 'tier3/BusinessErrorPanel',
  // story 対象コンポーネントを設定する
  component: BusinessErrorPanel,
};

// Meta をデフォルトエクスポートする
export default meta;

// Story 型を定義する
type Story = StoryObj<typeof BusinessErrorPanel>;

// stale_write subtype の Story を定義する
export const StaleWrite: Story = {
  // stale_write シナリオの props を設定する
  args: {
    // subtype を stale_write に設定する
    subtype: 'stale_write',
    // フィールド差分を設定する（quantity フィールド: local=5, remote=3）
    fieldDiff: { field: 'quantity', local: 5, remote: 3 },
    // 解決コールバックを空関数で設定する
    onResolve: () => {},
  },
};

// lost_update subtype の Story を定義する
export const LostUpdate: Story = {
  // lost_update シナリオの props を設定する
  args: {
    // subtype を lost_update に設定する
    subtype: 'lost_update',
    // lost_update では差分なし（null）
    fieldDiff: null,
    // 解決コールバックを空関数で設定する
    onResolve: () => {},
  },
};
