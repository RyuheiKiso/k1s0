// PresenceIndicator.tsx — concurrent_edit subtype のための presence indicator UI
// packages/ui-components に移動した版（spa/src/ui/PresenceIndicator.tsx から adapted）
// spec 11 §presence indicator: getActiveActors を購読してオンラインアクターを表示する
// 05_リアルタイム更新UX.md §presence indicator 表示 の物理コンポーネント
// wall-clock TTL 禁止: HLC ベースの expiresAtLogical を使用する

// React の必要なフックをインポートする
import React, { useEffect, useState } from 'react';

// HLC カウンタ値の型定義（wall-clock timestamp の代わりに使用する）
// src/client/hlc_lib 相当の型（packages 間で共有される想定）
export type HlcCounter = number;

// presence entry の型定義（packages/realtime/presence.ts と整合させる）
export interface PresenceEntry {
  // アクターの識別子（認証済み actor ID）
  actorId: string;
  // HLC による有効期限カウンタ（wall-clock TTL 禁止のため HLC で管理する）
  // HLC カウンタが現在の HLC を超えたら失効と判断する
  expiresAtLogical: HlcCounter;
}

// PresenceIndicator の props 型定義
export interface PresenceIndicatorProps {
  // 現在 online のアクター一覧を取得する関数
  // now: HLC カウンタ値（省略時はフルリストを返す）
  getActiveActors: (now?: HlcCounter) => PresenceEntry[];
  // 自分自身のアクター ID（自分は別表示する）
  selfActorId: string;
  // HLC カウンタを取得する関数（polling 更新に使用する）
  // wall-clock Date.now() の代わりに HLC を使用する（wall-clock TTL 禁止）
  getCurrentHlc: () => HlcCounter;
  // 更新間隔のイベント数（polling の頻度を HLC イベント数で制御する）
  // 30 秒ごとの wall-clock polling ではなく、エンジンイベント駆動で更新する
  pollIntervalMs?: number;
}

// PresenceIndicator コンポーネント
export const PresenceIndicator: React.FC<PresenceIndicatorProps> = ({
  // getActiveActors 関数を受け取る
  getActiveActors,
  // 自分のアクター ID を受け取る
  selfActorId,
  // HLC カウンタ取得関数を受け取る
  getCurrentHlc,
  // ポーリング間隔（デフォルト 30 秒 = 30000ms）
  pollIntervalMs = 30_000,
}) => {
  // online アクター一覧の state を定義する
  const [activeActors, setActiveActors] = useState<PresenceEntry[]>([]);

  // ポーリングで presence リストを更新する effect
  useEffect(() => {
    // 初回の presence リストを HLC カウンタで取得する
    setActiveActors(getActiveActors(getCurrentHlc()));
    // pollIntervalMs ごとに更新する interval を設定する
    const interval = setInterval(() => {
      // 現在の HLC カウンタで失効エントリを除外した一覧を取得する
      setActiveActors(getActiveActors(getCurrentHlc()));
    }, pollIntervalMs);
    // クリーンアップ時に interval を解除する
    return () => clearInterval(interval);
  }, [getActiveActors, getCurrentHlc, pollIntervalMs]);

  // 自分以外の online アクターをフィルタする
  const others = activeActors.filter((a) => a.actorId !== selfActorId);

  // online アクターが自分だけの場合はインジケータを非表示にする
  if (others.length === 0) return null;

  return (
    // presence インジケータのコンテナ
    <div
      aria-label={`${others.length} 人が同時編集中`}
      role="status"
      // WCAG 2.1 AA: aria-live="polite" でスクリーンリーダーに状態変化を通知する
      aria-live="polite"
      aria-atomic="false"
    >
      {/* online アクター数を表示する */}
      <span>
        {others.length} 人が編集中
      </span>
      {/* アクター ID 一覧をスクリーンリーダー向けに提供する */}
      <ul aria-label="編集中のユーザー一覧">
        {/* 各アクターを リスト項目として表示する */}
        {others.map((actor) => (
          <li key={actor.actorId}>
            {/* アクター ID を表示する（PII 保護: フルネームではなく匿名 ID を使用する） */}
            {actor.actorId}
          </li>
        ))}
      </ul>
    </div>
  );
};
