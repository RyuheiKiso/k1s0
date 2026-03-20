/* ヘルプボタンコンポーネント — 「?」ボタンでP3テーマのダイアログを表示 */

import * as Dialog from '@radix-ui/react-dialog';
import { helpContent } from '../lib/help-content';

interface HelpButtonProps {
  /** ヘルプコンテンツのキー（例: 'init', 'init.baseDir'） */
  helpKey: string;
  /** ボタンのサイズ — 'sm' はフィールド横用、'md' はセクション見出し横用 */
  size?: 'sm' | 'md';
}

/**
 * ヘルプボタン — クリックでヘルプダイアログを表示
 * helpContent マップに登録されたキーの内容を表示する
 */
export default function HelpButton({ helpKey, size = 'sm' }: HelpButtonProps) {
  const entry = helpContent[helpKey];

  /* コンテンツが未登録の場合はレンダリングしない */
  if (!entry) {
    return null;
  }

  /* サイズに応じたスタイル */
  const buttonSize = size === 'md' ? 'h-7 w-7 text-sm' : 'h-5 w-5 text-[10px]';

  return (
    <Dialog.Root>
      {/* トリガーボタン — 「?」アイコン */}
      <Dialog.Trigger asChild>
        <button
          type="button"
          className={`p3-help-trigger inline-flex items-center justify-center border border-[rgba(0,200,255,0.25)] bg-[rgba(0,200,255,0.06)] font-bold text-cyan-200/70 ${buttonSize}`}
          aria-label={`${entry.title}のヘルプを表示`}
          data-testid={`help-btn-${helpKey}`}
        >
          ?
        </button>
      </Dialog.Trigger>

      {/* ダイアログポータル — body直下にレンダリング */}
      <Dialog.Portal>
        {/* オーバーレイ — 半透明ブラー背景 */}
        <Dialog.Overlay className="p3-help-overlay" />

        {/* ダイアログ本体 — P3テーマのパネル */}
        <Dialog.Content
          className="p3-help-dialog"
          data-testid={`help-dialog-${helpKey}`}
        >
          {/* 上辺シマーライン */}
          <div className="p3-help-shimmer" aria-hidden="true" />

          {/* ヘッダー — タイトルと閉じるボタン */}
          <div className="flex items-start justify-between gap-4">
            <div className="space-y-1">
              <Dialog.Title className="text-lg font-semibold text-white">
                {entry.title}
              </Dialog.Title>
              <p className="text-xs uppercase tracking-[0.28em] text-cyan-100/50">
                ヘルプ
              </p>
            </div>
            <Dialog.Close asChild>
              <button
                type="button"
                className="p3-help-close flex h-8 w-8 items-center justify-center border border-[rgba(0,200,255,0.2)] bg-[rgba(0,200,255,0.04)] text-sm text-cyan-200/60"
                aria-label="閉じる"
              >
                ✕
              </button>
            </Dialog.Close>
          </div>

          {/* 区切り線 */}
          <div className="my-4 h-px bg-gradient-to-r from-transparent via-[rgba(0,200,255,0.3)] to-transparent" />

          {/* 本文 — 説明テキスト */}
          <div className="space-y-3">
            {entry.body.map((paragraph, index) => (
              <p
                key={index}
                className="text-sm leading-7 text-slate-200/80"
              >
                {paragraph}
              </p>
            ))}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
