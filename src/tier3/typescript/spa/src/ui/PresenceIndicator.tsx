// PresenceIndicator.tsx — concurrent_edit subtype のための presence indicator UI
// spec 11 §presence indicator: getActiveActors を購読してオンラインアクターを表示する
// 05_リアルタイム更新UX.md §presence indicator 表示 の物理コンポーネント

// React の必要なフックをインポートする
import React, { useEffect, useState } from 'react';

// presence entry の型定義（packages/realtime/presence.ts と整合させる）
interface PresenceEntry {
  // アクターの識別子
  actorId: string;
  // presence の有効期限（UNIX ミリ秒）
  expiresAtMs: number;
}

// PresenceIndicator の props 型定義
interface PresenceIndicatorProps {
  // 現在 online のアクター一覧を取得する関数
  getActiveActors: (now?: number) => PresenceEntry[];
  // 自分自身のアクター ID（自分は別表示する）
  selfActorId: string;
}

// PresenceIndicator コンポーネント
export const PresenceIndicator: React.FC<PresenceIndicatorProps> = ({
  // getActiveActors 関数を受け取る
  getActiveActors,
  // 自分のアクター ID を受け取る
  selfActorId,
}) => {
  // online アクター一覧の state を定義する
  const [activeActors, setActiveActors] = useState<PresenceEntry[]>([]);

  // 30 秒ごとに presence リストを更新する effect
  useEffect(() => {
    // 初回の presence リストを取得する
    setActiveActors(getActiveActors());
    // 30 秒ごとに更新する interval を設定する
    const interval = setInterval(() => {
      // 現在時刻で TTL 切れを除外した一覧を取得する
      setActiveActors(getActiveActors(Date.now()));
    }, 30_000);
    // クリーンアップ時に interval を解除する
    return () => clearInterval(interval);
  }, [getActiveActors]);

  // 自分以外の online アクターをフィルタする
  const others = activeActors.filter((a) => a.actorId !== selfActorId);

  // online アクターが自分だけの場合はインジケータを非表示にする
  if (others.length === 0) return null;

  return (
    // presence インジケータのコンテナ
    <div aria-label={`${others.length} 人が同時編集中`} role="status">
      {/* online アクター数を表示する */}
      <span>{others.length} 人が編集中</span>
    </div>
  );
};
