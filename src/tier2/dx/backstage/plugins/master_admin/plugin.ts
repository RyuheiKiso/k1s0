// plugin.ts — 業務マスタ管理 Backstage plugin stub 実装
// docs 参照: docs/03_概要設計/03_tier2設計方針/README.md §開発者体験 / Backstage plugin
// Backstage plugin factory utilities をインポートする
import {
  createPlugin,
  createRoutableExtension,
} from '@backstage/core-plugin-api';

// plugin が使用するルートの型定義（Backstage ルーティングに登録する）
export type MasterAdminRouteRef = {
  // マスタ一覧ページのパス
  list: string;
  // マスタ詳細・編集ページのパス（マスタ ID を動的セグメントとして受け取る）
  detail: string;
};

// 業務マスタ管理 plugin のアイデンティティ定義
// id は Backstage catalog の name と対応させる
export const masterAdminPlugin = createPlugin({
  // plugin ID（Backstage の plugin registry に登録される一意キー）
  id: 'k1s0-tier2-master-admin',
  // この plugin が公開する route refs（ページ間ナビゲーションに使用する）
  routes: {},
  // この plugin が外部から参照できる external route refs
  externalRoutes: {},
});

// MasterAdminPage コンポーネント: 業務マスタ一覧・編集 UI のエントリポイント
// lazy load することで初回表示のバンドルサイズを削減する
export const MasterAdminPage = masterAdminPlugin.provide(
  createRoutableExtension({
    // コンポーネント名（Backstage DevTools に表示される）
    name: 'MasterAdminPage',
    // マウントポイント: App.tsx の routes に追加する
    mountPoint: masterAdminPlugin.getId(),
    // lazy import: 実際のページコンポーネントを非同期で読み込む
    component: () =>
      // NOTE: 本実装では stub として MasterAdminPageContent を動的 import する
      // 実装完了後にここを実際のコンポーネントモジュールに置き換えること
      Promise.resolve({
        // デフォルトエクスポート: stub コンポーネント（業務マスタ管理の骨格）
        default: function MasterAdminPageContent(): JSX.Element {
          // stub 実装: 業務マスタ管理 UI のプレースホルダを返す
          // 実装完了時にここをマスタ一覧・フォームコンポーネントに置き換える
          throw new Error(
            'MasterAdminPage は stub 実装です。実装完了後に置き換えてください。',
          );
        },
      }),
  }),
);

// MasterAdminApi の型定義（tier2 REST API との通信インターフェース）
export type MasterAdminApi = {
  // 品目マスタ一覧を取得する（ページネーション対応）
  listItems(params: {
    // ページトークン（カーソルベースページネーション）
    pageToken?: string;
    // 1 ページあたりの取得件数
    pageSize: number;
  }): Promise<MasterItemListResponse>;

  // 品目マスタの詳細を取得する
  getItem(params: {
    // 品目識別子（UUID v7 形式）
    itemId: string;
  }): Promise<MasterItemDetail>;
};

// 品目マスタ一覧レスポンスの型定義
export type MasterItemListResponse = {
  // 品目マスタエントリのリスト
  items: MasterItemSummary[];
  // 次ページのカーソルトークン（最終ページの場合は undefined）
  nextPageToken?: string;
  // 総件数（UI のページング表示に使用する）
  totalCount: number;
};

// 品目マスタサマリの型定義（一覧表示用の軽量型）
export type MasterItemSummary = {
  // 品目識別子（UUID v7 形式）
  itemId: string;
  // 品目コード（人間可読のビジネスキー）
  itemCode: string;
  // 品目名称（業界中立の汎用名称）
  itemName: string;
  // 有効フラグ
  isActive: boolean;
};

// 品目マスタ詳細の型定義（詳細表示・編集用の完全型）
export type MasterItemDetail = MasterItemSummary & {
  // 作成日時（HLC タイムスタンプ文字列）
  createdAt: string;
  // 最終更新日時（HLC タイムスタンプ文字列）
  updatedAt: string;
  // 拡張フィールド（pack が追加するカスタム属性）
  packExtensions: Record<string, string>;
};
