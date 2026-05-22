// plugin.ts — 監査ログ検索 Backstage plugin 本実装
// docs 参照: docs/03_概要設計/03_tier2設計方針/README.md §開発者体験 / Backstage plugin
// docs 参照: docs/04_詳細設計/01_適合仕様/08_業務エラー監査適合仕様.md
// stub 実装を除去して実 React component に置き換えた本実装
// Backstage plugin factory utilities をインポートする
import {
  createPlugin,
  createRoutableExtension,
} from '@backstage/core-plugin-api';
// React をインポートする（JSX.Element の返却に必要）
import React from 'react';

// 監査ログ検索 plugin のアイデンティティ定義
// id は Backstage catalog の name と対応させる
export const auditSearchPlugin = createPlugin({
  // plugin ID（Backstage の plugin registry に登録される一意キー）
  id: 'k1s0-tier2-audit-search',
  // この plugin が公開する route refs
  routes: {},
  // この plugin が外部から参照できる external route refs
  externalRoutes: {},
});

// AuditSearchPage コンポーネント: 監査ログ検索 UI のエントリポイント
// lazy load することで初回表示のバンドルサイズを削減する
export const AuditSearchPage = auditSearchPlugin.provide(
  createRoutableExtension({
    // コンポーネント名（Backstage DevTools に表示される）
    name: 'AuditSearchPage',
    // マウントポイント: App.tsx の routes に追加する
    mountPoint: auditSearchPlugin.getId(),
    // lazy import: 実際のページコンポーネントを非同期で読み込む
    component: () =>
      // 実 React component を返す Promise を返す
      Promise.resolve({
        // デフォルトエクスポート: 監査ログ検索 React コンポーネント（本実装）
        default: function AuditSearchPageContent(): JSX.Element {
          // 検索クエリの状態を管理する（初期値は空文字列）
          const [query, setQuery] = React.useState('');
          // 検索結果の状態を管理する（初期値は空配列）
          const [results, setResults] = React.useState<AuditEventSummary[]>([]);
          // ローディング状態を管理する（初期値は false）
          const [loading, setLoading] = React.useState(false);
          // エラーメッセージの状態を管理する（初期値は null）
          const [error, setError] = React.useState<string | null>(null);

          // handleSearch: 検索ボタンクリック時に実行する
          const handleSearch = React.useCallback(async () => {
            // ローディング開始を設定する
            setLoading(true);
            // エラーをリセットする
            setError(null);
            // 検索結果をリセットする
            setResults([]);
            try {
              // 監査ログ検索 API を呼び出す（tier2 REST API エンドポイントに POST する）
              const response = await fetch('/api/k1s0/tier2/audit/search', {
                // POST メソッドで検索リクエストを送信する
                method: 'POST',
                // Content-Type を JSON に設定する
                headers: { 'Content-Type': 'application/json' },
                // 検索パラメータを JSON 形式で送信する
                body: JSON.stringify({
                  // 全文検索クエリを含める
                  query,
                  // 1 ページあたりの取得件数を設定する
                  pageSize: 50,
                }),
              });
              // レスポンスが 200 OK でない場合はエラーをスローする
              if (!response.ok) {
                // HTTP エラーレスポンスをエラーメッセージに変換する
                throw new Error(`検索 API エラー: HTTP ${response.status}`);
              }
              // レスポンス JSON を解析する
              const data: AuditSearchResponse = await response.json();
              // 検索結果を更新する
              setResults(data.events);
            } catch (e) {
              // エラーが発生した場合はエラーメッセージを設定する
              setError(
                e instanceof Error ? e.message : '検索中に予期しないエラーが発生しました',
              );
            } finally {
              // ローディング終了を設定する
              setLoading(false);
            }
          }, [query]);

          // 監査ログ検索 UI を返す
          return React.createElement(
            // コンテナ div を生成する
            'div',
            // スタイルとクラス名を設定する
            { style: { padding: '24px', fontFamily: 'system-ui, sans-serif' } },
            // ページタイトルを表示する
            React.createElement('h1', { style: { marginBottom: '16px' } }, '監査ログ検索'),
            // 検索フォームを生成する
            React.createElement(
              'div',
              { style: { display: 'flex', gap: '8px', marginBottom: '16px' } },
              // 検索クエリ入力フィールドを生成する
              React.createElement('input', {
                // 入力フィールドの型を設定する
                type: 'text',
                // プレースホルダを設定する
                placeholder: '監査イベントを検索する（操作種別 / アクター ID 等）',
                // 現在のクエリ値を設定する
                value: query,
                // 入力値の変更を処理する
                onChange: (e: React.ChangeEvent<HTMLInputElement>) =>
                  setQuery(e.target.value),
                // スタイルを設定する
                style: {
                  flex: 1,
                  padding: '8px 12px',
                  fontSize: '14px',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                },
              }),
              // 検索ボタンを生成する
              React.createElement(
                'button',
                {
                  // クリック時に検索を実行する
                  onClick: handleSearch,
                  // ローディング中はボタンを無効にする
                  disabled: loading,
                  // スタイルを設定する
                  style: {
                    padding: '8px 16px',
                    backgroundColor: '#1976d2',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: loading ? 'not-allowed' : 'pointer',
                    fontSize: '14px',
                  },
                },
                // ローディング中は「検索中...」を表示する
                loading ? '検索中...' : '検索',
              ),
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
            // 検索結果テーブルを生成する（results が空でない場合のみ）
            results.length > 0 &&
              React.createElement(
                'table',
                // テーブルのスタイルを設定する
                {
                  style: {
                    width: '100%',
                    borderCollapse: 'collapse',
                    fontSize: '13px',
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
                    ...['イベント ID', 'アクター', 'アクション', 'リソース種別', 'リソース ID', '発生日時'].map(
                      (label) =>
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
                  // 各検索結果行を生成する
                  ...results.map((event) =>
                    React.createElement(
                      'tr',
                      // 行のスタイルを設定する（交互に背景色を変える）
                      {
                        key: event.eventId,
                        style: { borderBottom: '1px solid #e0e0e0' },
                      },
                      // イベント ID セルを生成する
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px', fontFamily: 'monospace' } },
                        // イベント ID の先頭 8 文字を表示する
                        event.eventId.slice(0, 8) + '...',
                      ),
                      // アクター ID セルを生成する
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px' } },
                        event.actorId,
                      ),
                      // アクションセルを生成する
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px' } },
                        event.action,
                      ),
                      // リソース種別セルを生成する
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px' } },
                        event.resourceType,
                      ),
                      // リソース ID セルを生成する
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px', fontFamily: 'monospace' } },
                        // リソース ID の先頭 8 文字を表示する
                        event.resourceId.slice(0, 8) + '...',
                      ),
                      // 発生日時セルを生成する
                      React.createElement(
                        'td',
                        { style: { padding: '8px 12px' } },
                        // 発生日時を表示する
                        event.occurredAt,
                      ),
                    ),
                  ),
                ),
              ),
            // 検索結果が 0 件の場合のメッセージを表示する
            !loading &&
              results.length === 0 &&
              !error &&
              React.createElement(
                'p',
                { style: { color: '#757575', fontSize: '14px' } },
                // 検索前または結果なしのメッセージを表示する
                query
                  ? '検索結果が見つかりませんでした'
                  : '検索クエリを入力して「検索」ボタンを押してください',
              ),
          );
        },
      }),
  }),
);

// AuditSearchApi の型定義（tier2 REST API との通信インターフェース）
export type AuditSearchApi = {
  // 監査イベントを全文検索する（ClickHouse 経由）
  searchAuditEvents(params: AuditSearchParams): Promise<AuditSearchResponse>;

  // ハッシュチェーンの整合性を検証する
  verifyHashChain(params: {
    // テナント識別子（AuthContext から取得する）
    tenantId: string;
    // 検証開始の chain_sequence
    fromSequence: number;
    // 検証終了の chain_sequence
    toSequence: number;
  }): Promise<HashChainVerificationResult>;
};

// 監査ログ検索パラメータの型定義
export type AuditSearchParams = {
  // 全文検索クエリ文字列（空文字列の場合は全件取得）
  query: string;
  // 検索対象の開始日時（HLC タイムスタンプ文字列）
  fromAt?: string;
  // 検索対象の終了日時（HLC タイムスタンプ文字列）
  toAt?: string;
  // 操作者 ID でフィルタリングする（省略時はフィルタなし）
  actorId?: string;
  // リソース種別でフィルタリングする（省略時はフィルタなし）
  resourceType?: string;
  // ページトークン（カーソルベースページネーション）
  pageToken?: string;
  // 1 ページあたりの取得件数
  pageSize: number;
};

// 監査ログ検索レスポンスの型定義
export type AuditSearchResponse = {
  // 検索結果の監査イベントリスト
  events: AuditEventSummary[];
  // 次ページのカーソルトークン（最終ページの場合は undefined）
  nextPageToken?: string;
  // 総ヒット件数
  totalHits: number;
};

// 監査イベントサマリの型定義（検索結果表示用）
export type AuditEventSummary = {
  // イベント識別子（UUID v7 形式）
  eventId: string;
  // 操作者識別子（AuthContext.user_id）
  actorId: string;
  // 実行されたアクション（CREATE / UPDATE / DELETE 等）
  action: string;
  // 操作対象リソース種別
  resourceType: string;
  // 操作対象リソース識別子
  resourceId: string;
  // イベント発生日時（HLC タイムスタンプ文字列）
  occurredAt: string;
  // ハッシュチェーンのシーケンス番号（chain integrity 確認に使用する）
  chainSequence: number;
};

// ハッシュチェーン検証結果の型定義
export type HashChainVerificationResult = {
  // 検証結果（true: チェーン整合 / false: 不整合あり）
  isValid: boolean;
  // 検証したシーケンス範囲
  verifiedRange: { from: number; to: number };
  // 不整合が発見された chain_sequence のリスト（isValid が true の場合は空配列）
  mismatchSequences: number[];
};
