// plugin.ts — 監査ログ検索 Backstage plugin stub 実装
// docs 参照: docs/03_概要設計/03_tier2設計方針/README.md §開発者体験 / Backstage plugin
// docs 参照: docs/04_詳細設計/01_適合仕様/08_業務エラー監査適合仕様.md
// Backstage plugin factory utilities をインポートする
import {
  createPlugin,
  createRoutableExtension,
} from '@backstage/core-plugin-api';

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
      // NOTE: 本実装では stub として AuditSearchPageContent を動的 import する
      Promise.resolve({
        // デフォルトエクスポート: stub コンポーネント（監査ログ検索の骨格）
        default: function AuditSearchPageContent(): JSX.Element {
          // stub 実装: 監査ログ検索 UI のプレースホルダを返す
          throw new Error(
            'AuditSearchPage は stub 実装です。実装完了後に置き換えてください。',
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
