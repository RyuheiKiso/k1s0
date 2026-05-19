// ThreeWayMergeUi.tsx — 3way merge UI コンポーネント
// base / local / remote の 3 バージョンを並べて差分をハイライト表示する
// accept-local / accept-remote ボタンで競合を解決する
// 11_クライアント状態適合仕様.md §lost_update → present_3way_merge_ui の物理実装

// React の必要なフックをインポートする
import React, { useCallback, useState } from 'react';

// 3way merge で比較するバージョンを表す型定義
export interface ThreeWayMergeVersions<T extends Record<string, unknown>> {
  // 競合の基点バージョン（共通の祖先）
  base: T;
  // クライアント側の変更バージョン（local の編集）
  local: T;
  // サーバー側の変更バージョン（他ユーザーの変更）
  remote: T;
}

// ThreeWayMergeUi のコールバック型定義
export interface ThreeWayMergeCallbacks {
  // ローカル側の変更を採用するコールバック
  onAcceptLocal: () => void;
  // リモート側の変更を採用するコールバック
  onAcceptRemote: () => void;
  // マージをキャンセルするコールバック（編集を中断する）
  onCancel: () => void;
}

// ThreeWayMergeUi のプロパティ型定義
export interface ThreeWayMergeUiProps<T extends Record<string, unknown>> {
  // 3 バージョンのデータ
  versions: ThreeWayMergeVersions<T>;
  // コールバック関数群
  callbacks: ThreeWayMergeCallbacks;
  // フィールド名のラベルマッピング（表示用の日本語名を提供する）
  fieldLabels?: Partial<Record<keyof T, string>>;
}

// フィールドの競合状態を判定するヘルパー関数
function detectConflictingFields<T extends Record<string, unknown>>(
  versions: ThreeWayMergeVersions<T>,
): Set<keyof T> {
  // 競合フィールドを格納する Set を初期化する
  const conflicting = new Set<keyof T>();
  // base の全フィールドを確認する
  for (const key of Object.keys(versions.base) as Array<keyof T>) {
    // local の変更判定（base と local が異なる場合は local が変更している）
    const localChanged = versions.local[key] !== versions.base[key];
    // remote の変更判定（base と remote が異なる場合は remote が変更している）
    const remoteChanged = versions.remote[key] !== versions.base[key];
    // 両方が変更されている場合は競合フィールドとして追加する
    if (localChanged && remoteChanged) {
      conflicting.add(key);
    }
  }
  // 競合フィールドの Set を返す
  return conflicting;
}

// フィールドの値を文字列に変換するヘルパー関数
function formatFieldValue(value: unknown): string {
  // null / undefined の場合は空文字を返す
  if (value === null || value === undefined) {
    return '(空)';
  }
  // オブジェクトの場合は JSON.stringify で変換する
  if (typeof value === 'object') {
    return JSON.stringify(value, null, 2);
  }
  // それ以外は toString を使用する
  return String(value);
}

// ThreeWayMergeUi コンポーネント（ジェネリクスで任意のオブジェクト型に対応する）
export const ThreeWayMergeUi = <T extends Record<string, unknown>>({
  // 3 バージョンのデータを受け取る
  versions,
  // コールバック関数群を受け取る
  callbacks,
  // フィールドラベルマッピングを受け取る（省略可能）
  fieldLabels = {},
}: ThreeWayMergeUiProps<T>): React.ReactElement => {
  // 現在ハイライト中のフィールド名（null = ハイライトなし）
  const [highlightedField, setHighlightedField] = useState<keyof T | null>(null);

  // 競合フィールドを検出する
  const conflictingFields = detectConflictingFields(versions);

  // フィールドのホバー時にハイライトを設定するコールバック
  const handleFieldHover = useCallback((field: keyof T) => {
    // ハイライトするフィールドを設定する
    setHighlightedField(field);
  }, []);

  // フィールドのホバー解除時にハイライトをクリアするコールバック
  const handleFieldLeave = useCallback(() => {
    // ハイライトをクリアする
    setHighlightedField(null);
  }, []);

  // 表示するフィールドの一覧を取得する（base のキーを使用する）
  const allFields = Object.keys(versions.base) as Array<keyof T>;

  return (
    // ダイアログとして使用する場合はラッパーを role="dialog" にする
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="three-way-merge-title"
      // WCAG 2.1 AA: focus trap のために tabIndex を設定する（親コンポーネントが管理する）
    >
      {/* ダイアログタイトル */}
      <h2 id="three-way-merge-title">
        編集の競合が検出されました
      </h2>
      {/* 競合の説明文 */}
      <p role="note">
        同じフィールドが複数のユーザーによって変更されました。
        どちらの変更を採用するか選択してください。
      </p>

      {/* 3 カラムの比較テーブル */}
      <table
        aria-label="3バージョン比較テーブル"
        // table の summary は aria-label で代替する（summary 属性は deprecated）
      >
        <thead>
          <tr>
            {/* フィールド名のヘッダー */}
            <th scope="col">フィールド</th>
            {/* base バージョンのヘッダー */}
            <th scope="col">基点バージョン</th>
            {/* local バージョンのヘッダー */}
            <th scope="col" aria-label="あなたの変更">
              あなたの変更
            </th>
            {/* remote バージョンのヘッダー */}
            <th scope="col" aria-label="他ユーザーの変更">
              他ユーザーの変更
            </th>
          </tr>
        </thead>
        <tbody>
          {/* 全フィールドを行として表示する */}
          {allFields.map((field) => {
            // このフィールドが競合中かどうかを判定する
            const isConflicting = conflictingFields.has(field);
            // このフィールドがハイライト中かどうかを判定する
            const isHighlighted = highlightedField === field;
            // フィールドのラベル（マッピングがない場合はフィールド名をそのまま使用する）
            const label = (fieldLabels as Record<keyof T, string>)[field] ?? String(field);
            return (
              <tr
                // フィールドキーを row の key に使用する
                key={String(field)}
                // 競合中・ハイライト中のスタイルを aria で示す
                aria-current={isHighlighted ? 'true' : 'false'}
                // マウスイベントを設定する
                onMouseEnter={() => handleFieldHover(field)}
                onMouseLeave={handleFieldLeave}
              >
                {/* フィールド名セル */}
                <th scope="row" aria-label={label}>
                  {/* フィールドラベルを表示する */}
                  {label}
                  {/* 競合フィールドには警告アイコンを表示する */}
                  {isConflicting && (
                    <span
                      aria-label="競合あり"
                      role="img"
                    >
                      {' '}⚠
                    </span>
                  )}
                </th>
                {/* base バージョンのセル */}
                <td lang="ja">
                  {formatFieldValue(versions.base[field])}
                </td>
                {/* local バージョンのセル（変更があれば強調する） */}
                <td
                  aria-label={`あなたの変更: ${label}`}
                  // local が base と異なる場合は変更済みを示す
                >
                  {formatFieldValue(versions.local[field])}
                  {/* local が base と異なる場合はマーカーを表示する */}
                  {versions.local[field] !== versions.base[field] && (
                    <span aria-label="変更あり" role="img"> ✏️</span>
                  )}
                </td>
                {/* remote バージョンのセル（変更があれば強調する） */}
                <td
                  aria-label={`他ユーザーの変更: ${label}`}
                >
                  {formatFieldValue(versions.remote[field])}
                  {/* remote が base と異なる場合はマーカーを表示する */}
                  {versions.remote[field] !== versions.base[field] && (
                    <span aria-label="変更あり" role="img"> ✏️</span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>

      {/* アクションボタンエリア */}
      <div role="group" aria-label="競合解決アクション">
        {/* ローカル側の変更を採用するボタン */}
        <button
          type="button"
          onClick={callbacks.onAcceptLocal}
          aria-label="あなたの変更を採用する"
        >
          あなたの変更を採用
        </button>
        {/* リモート側の変更を採用するボタン */}
        <button
          type="button"
          onClick={callbacks.onAcceptRemote}
          aria-label="他ユーザーの変更を採用する"
        >
          他ユーザーの変更を採用
        </button>
        {/* キャンセルボタン（編集を中断して元の状態に戻る） */}
        <button
          type="button"
          onClick={callbacks.onCancel}
          aria-label="キャンセルして閉じる"
        >
          キャンセル
        </button>
      </div>
    </div>
  );
};
