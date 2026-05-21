// plugin.ts — 緊急操作 Backstage plugin stub 実装
// docs 参照: docs/03_概要設計/03_tier2設計方針/README.md §開発者体験 / Backstage plugin
// docs 参照: docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md §緊急操作
// IMPORTANT: 本 plugin は高権限操作を提供する。全操作は Audit 必須かつ dual sign-off 対象。
// Backstage plugin factory utilities をインポートする
import {
  createPlugin,
  createRoutableExtension,
} from '@backstage/core-plugin-api';

// 緊急操作 plugin のアイデンティティ定義
// id は Backstage catalog の name と対応させる
export const emergencyOverridePlugin = createPlugin({
  // plugin ID（Backstage の plugin registry に登録される一意キー）
  id: 'k1s0-tier2-emergency-override',
  // この plugin が公開する route refs
  routes: {},
  // この plugin が外部から参照できる external route refs
  externalRoutes: {},
});

// EmergencyOverridePage コンポーネント: 緊急操作 UI のエントリポイント
// lazy load することで初回表示のバンドルサイズを削減する
export const EmergencyOverridePage = emergencyOverridePlugin.provide(
  createRoutableExtension({
    // コンポーネント名（Backstage DevTools に表示される）
    name: 'EmergencyOverridePage',
    // マウントポイント: App.tsx の routes に追加する
    mountPoint: emergencyOverridePlugin.getId(),
    // lazy import: 実際のページコンポーネントを非同期で読み込む
    component: () =>
      // NOTE: 本実装では stub として EmergencyOverridePageContent を動的 import する
      Promise.resolve({
        // デフォルトエクスポート: stub コンポーネント（緊急操作 UI の骨格）
        default: function EmergencyOverridePageContent(): JSX.Element {
          // stub 実装: 緊急操作 UI のプレースホルダを返す
          throw new Error(
            'EmergencyOverridePage は stub 実装です。実装完了後に置き換えてください。',
          );
        },
      }),
  }),
);

// EmergencyOverrideApi の型定義（tier2 REST API との通信インターフェース）
// NOTE: 全操作は Audit ログが必須であり、呼び出し元の identity を AuthContext から取得する
export type EmergencyOverrideApi = {
  // break-glass 緊急アクセスを起動する
  // 操作完了後は全 Audit イベントが Outbox 経由で emit される
  activateBreakGlass(params: BreakGlassActivationParams): Promise<BreakGlassActivationResult>;

  // 鍵緊急ローテーションを開始する（OpenBao Transit KEK を対象とする）
  initiateEmergencyKeyRotation(params: EmergencyKeyRotationParams): Promise<EmergencyKeyRotationResult>;

  // テナントを緊急停止する（RLS + quota を強制 block モードにする）
  suspendTenantEmergency(params: TenantSuspensionParams): Promise<TenantSuspensionResult>;
};

// break-glass 緊急アクセス起動パラメータの型定義
export type BreakGlassActivationParams = {
  // 緊急アクセス理由（Audit ログに記録される）
  reason: string;
  // 承認者識別子（dual sign-off の承認者 ID）
  approverId: string;
  // 承認トークン（dual sign-off トークン）
  approvalToken: string;
  // アクセス有効期間（秒単位）
  ttlSeconds: number;
};

// break-glass 緊急アクセス起動結果の型定義
export type BreakGlassActivationResult = {
  // 起動が成功したかどうか
  success: boolean;
  // 緊急アクセスセッション識別子
  sessionId: string;
  // セッション有効期限（HLC タイムスタンプ文字列）
  expiresAt: string;
  // 監査イベント識別子（Audit ログの追跡に使用する）
  auditEventId: string;
};

// 鍵緊急ローテーションパラメータの型定義
export type EmergencyKeyRotationParams = {
  // ローテーション対象の KEK 識別子
  kekId: string;
  // ローテーション理由（Audit ログに記録される）
  reason: string;
  // 承認者識別子（dual sign-off の承認者 ID）
  approverId: string;
  // 承認トークン（dual sign-off トークン）
  approvalToken: string;
};

// 鍵緊急ローテーション結果の型定義
export type EmergencyKeyRotationResult = {
  // ローテーションが成功したかどうか
  success: boolean;
  // 新しい KEK バージョン番号
  newKekVersion: number;
  // ローテーション完了日時（HLC タイムスタンプ文字列）
  rotatedAt: string;
  // 監査イベント識別子（Audit ログの追跡に使用する）
  auditEventId: string;
};

// テナント緊急停止パラメータの型定義
export type TenantSuspensionParams = {
  // 停止対象テナント識別子（AuthContext から取得した値のみ受け付ける）
  tenantId: string;
  // 停止理由（Audit ログに記録される）
  reason: string;
  // 承認者識別子（dual sign-off の承認者 ID）
  approverId: string;
  // 承認トークン（dual sign-off トークン）
  approvalToken: string;
};

// テナント緊急停止結果の型定義
export type TenantSuspensionResult = {
  // 停止が成功したかどうか
  success: boolean;
  // 停止適用日時（HLC タイムスタンプ文字列）
  suspendedAt: string;
  // 監査イベント識別子（Audit ログの追跡に使用する）
  auditEventId: string;
};
