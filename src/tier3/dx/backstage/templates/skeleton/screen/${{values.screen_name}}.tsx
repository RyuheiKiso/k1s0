// ${{values.screen_name}}.tsx — tier3 SPA screen component（Backstage template skeleton）
// このファイルは Backstage SoftwareTemplate が scaffold する際に生成されるテンプレート
// ドメイン: ${{values.domain}} のビジネスロジックを表示する screen コンポーネント

// React の必要なフックをインポートする
import React, { useCallback, useEffect, useState } from 'react';
// tier2 SDK 経由で生成された型定義をインポートする（直接 tier1 import 禁止）
// import type { ${{values.screen_name}}Entity } from '@k1s0/tier2-sdk';
// エラーバウンダリーコンポーネントをインポートする
import { ErrorBoundary } from '@k1s0/ui-components';
// ローディングスピナーコンポーネントをインポートする
import { LoadingSpinner } from '@k1s0/ui-components';
// i18n フックをインポートする（未定義 key は CI fail）
import { useTranslation } from 'react-i18next';

// ${{values.screen_name}} Props 型定義
interface ${{values.screen_name}}Props {
  // 表示するエンティティの ID
  entityId: string;
}

// ${{values.screen_name}} コンポーネント（デフォルトエクスポート）
// WCAG 2.1 AA 準拠: axe-core CI で検証される
const ${{values.screen_name}}: React.FC<${{values.screen_name}}Props> = ({
  // 表示するエンティティの ID を受け取る
  entityId,
}) => {
  // i18n hook を初期化する（namespace は domain 名と一致させる）
  const { t } = useTranslation('${{values.domain}}');

  // ローディング状態の state を定義する
  const [loading, setLoading] = useState(true);

  // エラー状態の state を定義する（null = エラーなし）
  const [error, setError] = useState<string | null>(null);

  // データ取得の effect を定義する
  const fetchData = useCallback(async () => {
    // ローディング開始
    setLoading(true);
    // エラー状態をリセットする
    setError(null);
    try {
      // SCAFFOLD: tier2 SDK 経由でデータを取得する（テンプレート生成後に実装する）
      // const data = await tier2Sdk.${{values.domain}}.get(entityId);
      void entityId; // 未使用変数の型チェックを通過させる（実装時に削除する）
    } catch (err) {
      // エラーをキャプチャして state に設定する
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      // ローディング終了
      setLoading(false);
    }
  }, [entityId]);

  // コンポーネントマウント時にデータを取得する
  useEffect(() => {
    // データ取得を実行する（依存配列に fetchData を含める）
    void fetchData();
  }, [fetchData]);

  // ローディング中はスピナーを表示する
  if (loading) {
    // WCAG: aria-live="polite" でスクリーンリーダーにローディング状態を通知する
    return <LoadingSpinner aria-label={t('loading')} />;
  }

  // エラーが発生した場合はエラーバウンダリーに委譲する
  if (error !== null) {
    return (
      // role="alert" でスクリーンリーダーにエラーを通知する
      <div role="alert" aria-live="assertive">
        {/* エラーメッセージを i18n 経由で表示する */}
        <p>{t('error.fetch_failed', { message: error })}</p>
      </div>
    );
  }

  return (
    // main ランドマーク（スクリーンリーダーのナビゲーション用）
    <main aria-labelledby="${{values.screen_name}}-title">
      {/* スクリーンタイトル（h1 要素） */}
      <h1 id="${{values.screen_name}}-title">
        {/* i18n 経由でタイトルを表示する */}
        {t('${{values.screen_name}}.title')}
      </h1>
      {/* メインコンテンツ領域（実装時にビジネスロジックを追加する） */}
      <section aria-label={t('${{values.screen_name}}.content_label')}>
        {/* SCAFFOLD: ビジネスコンテンツを実装する（テンプレート生成後に実装する） */}
      </section>
    </main>
  );
};

// デフォルトエクスポート（Backstage template が import する）
export default ${{values.screen_name}};
