// plugin.ts — 緊急操作 Backstage plugin 本実装
// docs 参照: docs/03_概要設計/03_tier2設計方針/README.md §開発者体験 / Backstage plugin
// docs 参照: docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md §緊急操作
// IMPORTANT: 本 plugin は高権限操作を提供する。全操作は Audit 必須かつ dual sign-off 対象。
// stub 実装を除去して実 React component に置き換えた本実装
// Backstage plugin factory utilities をインポートする
import {
  createPlugin,
  createRoutableExtension,
} from '@backstage/core-plugin-api';
// React をインポートする（JSX.Element の返却に必要）
import React from 'react';

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
      // 実 React component を返す Promise を返す
      Promise.resolve({
        // デフォルトエクスポート: 緊急操作 React コンポーネント（本実装）
        default: function EmergencyOverridePageContent(): JSX.Element {
          // 操作種別の選択状態を管理する（初期値は 'break_glass'）
          const [operationType, setOperationType] = React.useState<
            'break_glass' | 'key_rotation' | 'tenant_suspend'
          >('break_glass');
          // 操作理由の入力状態を管理する（初期値は空文字列）
          const [reason, setReason] = React.useState('');
          // 承認者 ID の入力状態を管理する（初期値は空文字列）
          const [approverId, setApproverId] = React.useState('');
          // 承認トークンの入力状態を管理する（初期値は空文字列）
          const [approvalToken, setApprovalToken] = React.useState('');
          // 処理中フラグを管理する（初期値は false）
          const [submitting, setSubmitting] = React.useState(false);
          // 操作結果メッセージを管理する（初期値は null）
          const [resultMessage, setResultMessage] = React.useState<string | null>(
            null,
          );
          // エラーメッセージを管理する（初期値は null）
          const [error, setError] = React.useState<string | null>(null);

          // handleSubmit: 緊急操作の実行ボタンクリック時に実行する
          const handleSubmit = React.useCallback(async () => {
            // 必須フィールドが空の場合はエラーを表示して中断する
            if (!reason.trim() || !approverId.trim() || !approvalToken.trim()) {
              // バリデーションエラーメッセージを設定する
              setError('操作理由・承認者 ID・承認トークンは全て必須です');
              // 処理を中断する
              return;
            }
            // 処理中フラグを立てる
            setSubmitting(true);
            // エラーと結果メッセージをリセットする
            setError(null);
            // 結果メッセージをリセットする
            setResultMessage(null);
            try {
              // 操作種別に応じた API エンドポイントを決定する
              const endpoint =
                operationType === 'break_glass'
                  ? '/api/k1s0/tier2/emergency/break-glass'
                  : operationType === 'key_rotation'
                    ? '/api/k1s0/tier2/emergency/key-rotation'
                    : '/api/k1s0/tier2/emergency/tenant-suspend';
              // 緊急操作 API を呼び出す（tier2 REST API エンドポイントに POST する）
              const response = await fetch(endpoint, {
                // POST メソッドで操作リクエストを送信する
                method: 'POST',
                // Content-Type を JSON に設定する
                headers: { 'Content-Type': 'application/json' },
                // 操作パラメータを JSON 形式で送信する
                body: JSON.stringify({
                  // 操作理由を含める（Audit ログに記録される）
                  reason,
                  // 承認者 ID を含める（dual sign-off の承認者）
                  approverId,
                  // 承認トークンを含める（dual sign-off トークン）
                  approvalToken,
                }),
              });
              // レスポンスが 200 OK でない場合はエラーをスローする
              if (!response.ok) {
                // HTTP エラーレスポンスをエラーメッセージに変換する
                throw new Error(`緊急操作 API エラー: HTTP ${response.status}`);
              }
              // レスポンス JSON を解析する
              const data = (await response.json()) as { sessionId?: string; success: boolean };
              // 操作成功メッセージを設定する
              setResultMessage(
                data.sessionId
                  ? `緊急操作が完了しました。セッション ID: ${data.sessionId}`
                  : '緊急操作が完了しました。Audit ログを確認してください。',
              );
            } catch (e) {
              // エラーが発生した場合はエラーメッセージを設定する
              setError(
                e instanceof Error
                  ? e.message
                  : '緊急操作中に予期しないエラーが発生しました',
              );
            } finally {
              // 処理中フラグを解除する
              setSubmitting(false);
            }
          }, [operationType, reason, approverId, approvalToken]);

          // 緊急操作 UI を返す
          return React.createElement(
            // コンテナ div を生成する
            'div',
            // スタイルとクラス名を設定する
            {
              style: {
                padding: '24px',
                fontFamily: 'system-ui, sans-serif',
                maxWidth: '640px',
              },
            },
            // ページタイトルを表示する
            React.createElement(
              'h1',
              { style: { marginBottom: '8px', color: '#b71c1c' } },
              '緊急操作',
            ),
            // 警告メッセージを表示する
            React.createElement(
              'p',
              {
                style: {
                  padding: '12px',
                  backgroundColor: '#fff3e0',
                  color: '#e65100',
                  borderRadius: '4px',
                  fontSize: '13px',
                  marginBottom: '24px',
                },
              },
              '⚠ 本操作は全て Audit ログに記録され、dual sign-off が必要です。不正操作は厳重に管理されます。',
            ),
            // 操作種別選択フィールドを生成する
            React.createElement(
              'div',
              { style: { marginBottom: '16px' } },
              React.createElement(
                'label',
                { style: { display: 'block', fontSize: '13px', marginBottom: '4px', fontWeight: 'bold' } },
                '操作種別',
              ),
              React.createElement(
                'select',
                {
                  // 現在の選択値を設定する
                  value: operationType,
                  // 選択値の変更を処理する
                  onChange: (e: React.ChangeEvent<HTMLSelectElement>) =>
                    setOperationType(
                      e.target.value as 'break_glass' | 'key_rotation' | 'tenant_suspend',
                    ),
                  // スタイルを設定する
                  style: {
                    width: '100%',
                    padding: '8px 12px',
                    fontSize: '14px',
                    border: '1px solid #ccc',
                    borderRadius: '4px',
                  },
                },
                // break_glass オプション
                React.createElement('option', { value: 'break_glass' }, 'Break Glass 緊急アクセス'),
                // key_rotation オプション
                React.createElement('option', { value: 'key_rotation' }, '鍵緊急ローテーション（KEK）'),
                // tenant_suspend オプション
                React.createElement('option', { value: 'tenant_suspend' }, 'テナント緊急停止'),
              ),
            ),
            // 操作理由入力フィールドを生成する
            React.createElement(
              'div',
              { style: { marginBottom: '16px' } },
              React.createElement(
                'label',
                { style: { display: 'block', fontSize: '13px', marginBottom: '4px', fontWeight: 'bold' } },
                '操作理由（必須 — Audit ログに記録されます）',
              ),
              React.createElement('textarea', {
                // 現在の操作理由を設定する
                value: reason,
                // 入力値の変更を処理する
                onChange: (e: React.ChangeEvent<HTMLTextAreaElement>) => setReason(e.target.value),
                // プレースホルダを設定する
                placeholder: '操作の理由を具体的に記述してください（例: 不正アクセス検知のため緊急調査が必要）',
                // 行数を設定する
                rows: 3,
                // スタイルを設定する
                style: {
                  width: '100%',
                  padding: '8px 12px',
                  fontSize: '14px',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  boxSizing: 'border-box' as const,
                },
              }),
            ),
            // 承認者 ID 入力フィールドを生成する
            React.createElement(
              'div',
              { style: { marginBottom: '16px' } },
              React.createElement(
                'label',
                { style: { display: 'block', fontSize: '13px', marginBottom: '4px', fontWeight: 'bold' } },
                '承認者 ID（dual sign-off 必須）',
              ),
              React.createElement('input', {
                // 入力フィールドの型を設定する
                type: 'text',
                // 現在の承認者 ID を設定する
                value: approverId,
                // 入力値の変更を処理する
                onChange: (e: React.ChangeEvent<HTMLInputElement>) =>
                  setApproverId(e.target.value),
                // プレースホルダを設定する
                placeholder: 'approver-user-id',
                // スタイルを設定する
                style: {
                  width: '100%',
                  padding: '8px 12px',
                  fontSize: '14px',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  boxSizing: 'border-box' as const,
                },
              }),
            ),
            // 承認トークン入力フィールドを生成する
            React.createElement(
              'div',
              { style: { marginBottom: '24px' } },
              React.createElement(
                'label',
                { style: { display: 'block', fontSize: '13px', marginBottom: '4px', fontWeight: 'bold' } },
                '承認トークン（dual sign-off トークン）',
              ),
              React.createElement('input', {
                // 入力フィールドの型をパスワードに設定する（トークンを隠す）
                type: 'password',
                // 現在の承認トークンを設定する
                value: approvalToken,
                // 入力値の変更を処理する
                onChange: (e: React.ChangeEvent<HTMLInputElement>) =>
                  setApprovalToken(e.target.value),
                // プレースホルダを設定する
                placeholder: '承認トークンを入力してください',
                // スタイルを設定する
                style: {
                  width: '100%',
                  padding: '8px 12px',
                  fontSize: '14px',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  boxSizing: 'border-box' as const,
                },
              }),
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
            // 成功メッセージを表示する（resultMessage が null でない場合のみ）
            resultMessage &&
              React.createElement(
                'div',
                {
                  // 成功メッセージのスタイルを設定する
                  style: {
                    padding: '12px',
                    backgroundColor: '#e8f5e9',
                    color: '#2e7d32',
                    borderRadius: '4px',
                    marginBottom: '16px',
                    fontSize: '14px',
                  },
                },
                // 成功メッセージを表示する
                resultMessage,
              ),
            // 実行ボタンを生成する
            React.createElement(
              'button',
              {
                // クリック時に緊急操作を実行する
                onClick: handleSubmit,
                // 送信中はボタンを無効にする
                disabled: submitting,
                // スタイルを設定する
                style: {
                  padding: '10px 24px',
                  backgroundColor: submitting ? '#9e9e9e' : '#b71c1c',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: submitting ? 'not-allowed' : 'pointer',
                  fontSize: '14px',
                  fontWeight: 'bold',
                },
              },
              // 送信中は「実行中...」を表示する
              submitting ? '実行中...' : '緊急操作を実行する',
            ),
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
