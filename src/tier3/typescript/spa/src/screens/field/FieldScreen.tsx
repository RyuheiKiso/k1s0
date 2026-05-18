// k1s0 tier3 現場一覧 UI（製造業 pack — 現場（field）管理画面）
// 製造業 pack における「現場」（フロアや設備が属する作業エリア）の一覧を表示する
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

import React from "react";

// 現場エンティティの型（tier2 生成 stub の代替として最小定義）
export interface FieldSummary {
  // 現場 ID（UUID）
  readonly fieldId: string;
  // 現場名
  readonly name: string;
  // 所属工場 ID
  readonly plantId: string;
  // 稼働状態
  readonly status: "active" | "inactive" | "maintenance";
  // 担当者表示名（PII ではなく表示名のみ）
  readonly supervisorDisplayName: string | null;
}

// FieldScreen のプロパティ型
export interface FieldScreenProps {
  // 現場一覧データ（undefined の場合はローディング中）
  readonly fields?: readonly FieldSummary[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ
  readonly errorMessage?: string;
  // 工場 ID フィルタ（undefined の場合は全工場を表示する）
  readonly filterByPlantId?: string;
}

// 現場ステータスの日本語ラベルを返す
function fieldStatusLabel(status: FieldSummary["status"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "active":
      // 稼働中
      return "稼働中";
    case "inactive":
      // 停止中
      return "停止中";
    case "maintenance":
      // メンテナンス中
      return "メンテナンス中";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラー）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 現場リストアイテムコンポーネント
function FieldListItem({ field }: { readonly field: FieldSummary }): React.JSX.Element {
  // 現場アイテムを li 要素として返す
  return (
    // li: リストアイテム
    <li aria-label={`現場: ${field.name}（${fieldStatusLabel(field.status)}）`}>
      {/* 現場名（見出し h3）*/}
      <h3>{field.name}</h3>
      {/* 稼働ステータス */}
      <p role="status" aria-label="稼働ステータス">{fieldStatusLabel(field.status)}</p>
      {/* 担当者（null の場合は「未設定」と表示する）*/}
      <p aria-label="担当者">
        {field.supervisorDisplayName !== null ? field.supervisorDisplayName : "未設定"}
      </p>
    </li>
  );
}

// 現場一覧画面コンポーネント
export function FieldScreen({
  fields,
  isLoading = false,
  errorMessage,
}: FieldScreenProps): React.JSX.Element {
  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="現場一覧">
        {/* ページ見出し */}
        <h1>現場一覧</h1>
        {/* ローディング状態を通知する */}
        <div role="status" aria-live="polite" aria-label="現場一覧を読み込み中">
          <p>現場一覧を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="現場一覧">
        {/* ページ見出し */}
        <h1>現場一覧</h1>
        {/* エラーを role="alert" で通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 現場一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="現場一覧">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>現場一覧</h1>
      {/* 現場数サマリ */}
      <p aria-live="polite" aria-label="現場数">
        {fields !== undefined ? `${fields.length} 件の現場` : ""}
      </p>
      {/* 現場リストまたは空状態 */}
      {fields === undefined || fields.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="現場が登録されていません">
          <p>現場が登録されていません。</p>
        </div>
      ) : (
        // 現場リストを表示する
        <ul aria-label="現場リスト">
          {fields.map((field) => (
            // 現場アイテムを key 付きで直接展開する（インライン化で TS2322 を回避する）
            <li
              key={field.fieldId}
              aria-label={`現場: ${field.name}（${fieldStatusLabel(field.status)}）`}
            >
              {/* 現場名（見出し h3）*/}
              <h3>{field.name}</h3>
              {/* 稼働ステータス */}
              <p role="status" aria-label="稼働ステータス">{fieldStatusLabel(field.status)}</p>
              {/* 担当者（null の場合は「未設定」と表示する）*/}
              <p aria-label="担当者">
                {field.supervisorDisplayName !== null ? field.supervisorDisplayName : "未設定"}
              </p>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
