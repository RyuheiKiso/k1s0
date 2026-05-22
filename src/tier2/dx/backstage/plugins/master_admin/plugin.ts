// plugin.ts — 業務マスタ管理 Backstage plugin 本実装
// docs 参照: docs/03_概要設計/03_tier2設計方針/README.md §開発者体験 / Backstage plugin
// stub 実装を除去して実 React component に置き換えた本実装
// Backstage plugin factory utilities をインポートする
import {
  createPlugin,
  createRoutableExtension,
} from '@backstage/core-plugin-api';
// React をインポートする（JSX.Element の返却に必要）
import React from 'react';

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
      // 実 React component を返す Promise を返す
      Promise.resolve({
        // デフォルトエクスポート: 業務マスタ管理 React コンポーネント（本実装）
        default: function MasterAdminPageContent(): JSX.Element {
          // マスタアイテムリストの状態を管理する（初期値は空配列）
          const [items, setItems] = React.useState<MasterItemSummary[]>([]);
          // ローディング状態を管理する（初期値は true に設定して初回ロードを示す）
          const [loading, setLoading] = React.useState(true);
          // エラーメッセージの状態を管理する（初期値は null）
          const [error, setError] = React.useState<string | null>(null);
          // 次ページトークンの状態を管理する（初期値は undefined）
          const [nextPageToken, setNextPageToken] = React.useState<
            string | undefined
          >(undefined);
          // 総件数の状態を管理する（初期値は 0）
          const [totalCount, setTotalCount] = React.useState(0);
          // 選択中のアイテム詳細を管理する（初期値は null）
          const [selectedItem, setSelectedItem] =
            React.useState<MasterItemDetail | null>(null);

          // fetchItems: マスタアイテム一覧を取得する
          const fetchItems = React.useCallback(
            async (pageToken?: string) => {
              // ローディング開始を設定する
              setLoading(true);
              // エラーをリセットする
              setError(null);
              try {
                // マスタ一覧 API を呼び出す（tier2 REST API エンドポイントに GET する）
                const url = new URL('/api/k1s0/tier2/master-admin/items', window.location.origin);
                // ページサイズを設定する
                url.searchParams.set('pageSize', '20');
                // ページトークンが指定されている場合はクエリパラメータに追加する
                if (pageToken) {
                  // ページトークンをクエリパラメータに追加する
                  url.searchParams.set('pageToken', pageToken);
                }
                // API を呼び出す
                const response = await fetch(url.toString(), {
                  // GET メソッドでリクエストを送信する
                  method: 'GET',
                  // Content-Type を JSON に設定する
                  headers: { 'Accept': 'application/json' },
                });
                // レスポンスが 200 OK でない場合はエラーをスローする
                if (!response.ok) {
                  // HTTP エラーレスポンスをエラーメッセージに変換する
                  throw new Error(
                    `マスタ一覧 API エラー: HTTP ${response.status}`,
                  );
                }
                // レスポンス JSON を解析する
                const data: MasterItemListResponse = await response.json();
                // 取得結果をアイテムリストに追加する（ページネーション対応）
                setItems((prev) =>
                  pageToken ? [...prev, ...data.items] : data.items,
                );
                // 次ページトークンを更新する
                setNextPageToken(data.nextPageToken);
                // 総件数を更新する
                setTotalCount(data.totalCount);
              } catch (e) {
                // エラーが発生した場合はエラーメッセージを設定する
                setError(
                  e instanceof Error
                    ? e.message
                    : 'マスタ一覧取得中に予期しないエラーが発生しました',
                );
              } finally {
                // ローディング終了を設定する
                setLoading(false);
              }
            },
            [],
          );

          // fetchItemDetail: マスタアイテム詳細を取得する
          const fetchItemDetail = React.useCallback(async (itemId: string) => {
            // ローディング開始を設定する
            setLoading(true);
            // エラーをリセットする
            setError(null);
            try {
              // マスタ詳細 API を呼び出す（tier2 REST API エンドポイントに GET する）
              const response = await fetch(
                `/api/k1s0/tier2/master-admin/items/${itemId}`,
                {
                  // GET メソッドでリクエストを送信する
                  method: 'GET',
                  // Accept ヘッダを設定する
                  headers: { Accept: 'application/json' },
                },
              );
              // レスポンスが 200 OK でない場合はエラーをスローする
              if (!response.ok) {
                // HTTP エラーレスポンスをエラーメッセージに変換する
                throw new Error(
                  `マスタ詳細 API エラー: HTTP ${response.status}`,
                );
              }
              // レスポンス JSON を解析する
              const detail: MasterItemDetail = await response.json();
              // 選択中のアイテム詳細を更新する
              setSelectedItem(detail);
            } catch (e) {
              // エラーが発生した場合はエラーメッセージを設定する
              setError(
                e instanceof Error
                  ? e.message
                  : 'マスタ詳細取得中に予期しないエラーが発生しました',
              );
            } finally {
              // ローディング終了を設定する
              setLoading(false);
            }
          }, []);

          // 初回マウント時にマスタ一覧を取得する
          React.useEffect(() => {
            // 初回ロードを実行する
            void fetchItems();
          }, [fetchItems]);

          // 業務マスタ管理 UI を返す
          return React.createElement(
            // コンテナ div を生成する
            'div',
            // スタイルとクラス名を設定する
            { style: { padding: '24px', fontFamily: 'system-ui, sans-serif' } },
            // ページタイトルを表示する
            React.createElement(
              'h1',
              { style: { marginBottom: '8px' } },
              '業務マスタ管理',
            ),
            // 総件数を表示する
            React.createElement(
              'p',
              { style: { color: '#757575', fontSize: '13px', marginBottom: '16px' } },
              `全 ${totalCount} 件`,
            ),
            // エラーメッセージを表示する（error が null でない場合のみ）
            error &&
              React.createElement(
                'div',
                {
                  // エラーメッセージのスタイルを設定する
                  style: {
                    padding: '12px',
                    backgroundColor: '#ffebee',
                    color: '#c62828',
                    borderRadius: '4px',
                    marginBottom: '16px',
                    fontSize: '14px',
                  },
                },
                // エラーメッセージを表示する
                error,
              ),
            // ローディング中は「読み込み中...」を表示する
            loading &&
              React.createElement(
                'p',
                { style: { color: '#757575', fontSize: '14px' } },
                '読み込み中...',
              ),
            // アイテムリストが空でない場合はテーブルを表示する
            !loading &&
              items.length > 0 &&
              React.createElement(
                'table',
                // テーブルのスタイルを設定する
                {
                  style: {
                    width: '100%',
                    borderCollapse: 'collapse',
                    fontSize: '13px',
                    marginBottom: '16px',
                  },
                },
                // テーブルヘッダを生成する
                React.createElement(
                  'thead',
                  null,
                  React.createElement(
                    'tr',
                    { style: { backgroundColor: '#f5f5f5' } },
                    // 各列ヘッダを生成する
                    ...['品目コード', '品目名称', '有効', '操作'].map((label) =>
                      React.createElement(
                        'th',
                        // ヘッダセルのスタイルを設定する
                        {
                          key: label,
                          style: {
                            padding: '8px 12px',
                            textAlign: 'left',
                            borderBottom: '2px solid #e0e0e0',
                          },
                        },
                        // ヘッダラベルを表示する
                        label,
                      ),
                    ),
                  ),
                ),
                // テーブルボディを生成する
                React.createElement(
                  'tbody',
                  null,
                  // 各アイテム行を生成する
                  ...items.map((item) =>
                    React.createElement(
                      'tr',
                      // 行のスタイルを設定する
                      {
                        key: item.itemId,
                        style: { borderBottom: '1px solid #e0e0e0' },
                      },
                      // 品目コードセルを生成する
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px', fontFamily: 'monospace' } },
                        item.itemCode,
                      ),
                      // 品目名称セルを生成する
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px' } },
                        item.itemName,
                      ),
                      // 有効フラグセルを生成する
                      React.createElement(
                        'td',
                        {
                          style: {
                            padding: '8px 12px',
                            color: item.isActive ? '#2e7d32' : '#c62828',
                          },
                        },
                        // 有効フラグを表示する
                        item.isActive ? '有効' : '無効',
                      ),
                      // 操作セルを生成する（詳細ボタン）
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px' } },
                        React.createElement(
                          'button',
                          {
                            // クリック時にアイテム詳細を取得する
                            onClick: () => void fetchItemDetail(item.itemId),
                            // スタイルを設定する
                            style: {
                              padding: '4px 8px',
                              fontSize: '12px',
                              backgroundColor: '#1976d2',
                              color: 'white',
                              border: 'none',
                              borderRadius: '3px',
                              cursor: 'pointer',
                            },
                          },
                          // ボタンラベルを表示する
                          '詳細',
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            // 次ページボタンを表示する（nextPageToken が存在する場合のみ）
            nextPageToken &&
              React.createElement(
                'button',
                {
                  // クリック時に次ページを取得する
                  onClick: () => void fetchItems(nextPageToken),
                  // スタイルを設定する
                  style: {
                    padding: '8px 16px',
                    backgroundColor: '#757575',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '13px',
                    marginBottom: '16px',
                  },
                },
                // ボタンラベルを表示する
                '次のページを読み込む',
              ),
            // 選択中のアイテム詳細を表示する（selectedItem が null でない場合のみ）
            selectedItem &&
              React.createElement(
                'div',
                {
                  // 詳細パネルのスタイルを設定する
                  style: {
                    marginTop: '24px',
                    padding: '16px',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    backgroundColor: '#fafafa',
                  },
                },
                // 詳細パネルタイトルを表示する
                React.createElement(
                  'h2',
                  { style: { fontSize: '16px', marginBottom: '12px' } },
                  `マスタ詳細: ${selectedItem.itemCode}`,
                ),
                // 品目名称を表示する
                React.createElement(
                  'p',
                  { style: { fontSize: '13px', marginBottom: '4px' } },
                  `品目名称: ${selectedItem.itemName}`,
                ),
                // 有効フラグを表示する
                React.createElement(
                  'p',
                  {
                    style: {
                      fontSize: '13px',
                      marginBottom: '4px',
                      color: selectedItem.isActive ? '#2e7d32' : '#c62828',
                    },
                  },
                  `有効: ${selectedItem.isActive ? '有効' : '無効'}`,
                ),
                // 作成日時を表示する
                React.createElement(
                  'p',
                  { style: { fontSize: '13px', marginBottom: '4px' } },
                  `作成日時: ${selectedItem.createdAt}`,
                ),
                // 最終更新日時を表示する
                React.createElement(
                  'p',
                  { style: { fontSize: '13px', marginBottom: '4px' } },
                  `最終更新日時: ${selectedItem.updatedAt}`,
                ),
                // 閉じるボタンを表示する
                React.createElement(
                  'button',
                  {
                    // クリック時に詳細パネルを閉じる
                    onClick: () => setSelectedItem(null),
                    // スタイルを設定する
                    style: {
                      marginTop: '8px',
                      padding: '4px 12px',
                      fontSize: '12px',
                      backgroundColor: '#757575',
                      color: 'white',
                      border: 'none',
                      borderRadius: '3px',
                      cursor: 'pointer',
                    },
                  },
                  // ボタンラベルを表示する
                  '閉じる',
                ),
              ),
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
