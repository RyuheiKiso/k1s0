// k1s0 FA tier3 — 設備リモート操作画面
// 製造業 pack 適用例 scenario 1: 設備リモート操作（v1_interactive、bidi 双方向）
// 設備を遠隔から操作し、操作結果をリアルタイムで確認する業務 UI
// WCAG 2.1 AA 準拠: ランドマーク / heading 階層 / aria-label を使用する

// React をインポートする
import React, { useState } from "react";
// tier2 pack の型をインポートする（独自再宣言型禁止ポリシーに準拠する）
import type { ManufacturingPackClient } from "@k1s0/tier3-pack";

// 設備エンティティの型（tier2 生成 stub の型定義に合わせる）
export interface EquipmentSummary {
  // 設備 ID（UUID）
  readonly equipmentId: string;
  // 設備番号（業務キー）
  readonly equipmentNumber: string;
  // 設備名（表示用）
  readonly equipmentName: string;
  // 設備種別（press / conveyor / robot / cnc / other）
  readonly equipmentType: "press" | "conveyor" | "robot" | "cnc" | "other";
  // 稼働ステータス（running / stopped / error / maintenance）
  readonly operationStatus: "running" | "stopped" | "error" | "maintenance";
  // 最終操作日時（ISO 8601 文字列）
  readonly lastOperatedAt: string;
  // 操作可能フラグ（リモート操作が許可されているか）
  readonly isRemoteOperable: boolean;
}

// EquipmentRemoteScreen のプロパティ型
export interface EquipmentRemoteScreenProps {
  // tier2 pack クライアント（undefined の場合は stub を使用する）
  readonly packClient?: ManufacturingPackClient;
  // 設備一覧データ（undefined の場合はローディング中）
  readonly equipments?: readonly EquipmentSummary[];
  // ローディング中フラグ
  readonly isLoading?: boolean;
  // エラーメッセージ（存在する場合は表示する）
  readonly errorMessage?: string;
}

// 稼働ステータスの日本語ラベルを返す
function operationStatusLabel(status: EquipmentSummary["operationStatus"]): string {
  // ステータスに応じたラベルを返す
  switch (status) {
    case "running":
      // 稼働中
      return "稼働中";
    case "stopped":
      // 停止中
      return "停止中";
    case "error":
      // 異常
      return "異常";
    case "maintenance":
      // メンテナンス中
      return "メンテナンス中";
    default: {
      // 網羅性チェック（新ステータス追加時はコンパイルエラーで検知する）
      const _exhaustive: never = status;
      return String(_exhaustive);
    }
  }
}

// 設備種別の日本語ラベルを返す
function equipmentTypeLabel(type: EquipmentSummary["equipmentType"]): string {
  // 設備種別に応じたラベルを返す
  switch (type) {
    case "press":
      // プレス機
      return "プレス機";
    case "conveyor":
      // コンベア
      return "コンベア";
    case "robot":
      // ロボット
      return "ロボット";
    case "cnc":
      // CNC 工作機械
      return "CNC 工作機械";
    case "other":
      // その他
      return "その他";
    default: {
      // 網羅性チェック（新種別追加時はコンパイルエラーで検知する）
      const _exhaustive: never = type;
      return String(_exhaustive);
    }
  }
}

// 設備リモート操作画面コンポーネント
export function EquipmentRemoteScreen({
  equipments,
  isLoading = false,
  errorMessage,
}: EquipmentRemoteScreenProps): React.JSX.Element {
  // 操作中の設備 ID を state として保持する（null の場合は操作パネル非表示）
  const [operatingEquipmentId, setOperatingEquipmentId] = useState<string | null>(null);

  // ローディング中の場合はローディング表示を返す
  if (isLoading) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="設備リモート操作">
        {/* ページ見出し */}
        <h1>設備リモート操作</h1>
        {/* ローディング状態を aria-live で通知する */}
        <div role="status" aria-live="polite" aria-label="設備一覧を読み込み中">
          <p>設備一覧を読み込み中...</p>
        </div>
      </main>
    );
  }

  // エラーがある場合はエラー表示を返す
  if (errorMessage !== undefined) {
    return (
      // main: ページのメインコンテンツランドマーク
      <main id="main-content" role="main" aria-label="設備リモート操作">
        {/* ページ見出し */}
        <h1>設備リモート操作</h1>
        {/* エラーを role="alert" で即時通知する */}
        <div role="alert" aria-label="エラー">
          <p>{errorMessage}</p>
        </div>
      </main>
    );
  }

  // 設備一覧を表示する
  return (
    // main: ページのメインコンテンツランドマーク（WCAG 2.1 ランドマーク要件）
    <main id="main-content" role="main" aria-label="設備リモート操作">
      {/* ページ見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件）*/}
      <h1>設備リモート操作</h1>
      {/* 設備件数サマリ */}
      <p aria-live="polite" aria-label="設備件数">
        {equipments !== undefined ? `${equipments.length} 台の設備` : ""}
      </p>
      {/* 設備一覧または空状態 */}
      {equipments === undefined || equipments.length === 0 ? (
        // 空状態を表示する
        <div role="status" aria-label="設備がありません">
          <p>設備がありません。</p>
        </div>
      ) : (
        // 設備リストを表示する
        <ul aria-label="設備リスト">
          {equipments.map((equipment) => (
            // 設備アイテムを key 付きで展開する
            <li
              key={equipment.equipmentId}
              // aria-label で設備番号と稼働ステータスを説明する
              aria-label={`設備 ${equipment.equipmentNumber}（${operationStatusLabel(equipment.operationStatus)}）`}
            >
              {/* 設備番号（見出し h3）*/}
              <h3>{equipment.equipmentNumber}</h3>
              {/* 設備名 */}
              <p aria-label="設備名">{equipment.equipmentName}</p>
              {/* 設備種別 */}
              <p aria-label="種別">{equipmentTypeLabel(equipment.equipmentType)}</p>
              {/* 稼働ステータス */}
              <p role="status" aria-label="稼働ステータス">
                {operationStatusLabel(equipment.operationStatus)}
              </p>
              {/* 最終操作日時 */}
              <p aria-label="最終操作">{equipment.lastOperatedAt}</p>
              {/* リモート操作ボタン（操作可能な場合のみ有効化する）*/}
              <button
                // 操作可能な場合はクリックで操作パネルを開く
                onClick={() => {
                  if (equipment.isRemoteOperable) {
                    // 操作中の設備 ID を設定する
                    setOperatingEquipmentId(equipment.equipmentId);
                  }
                }}
                // 操作不可の場合は disabled にする
                disabled={!equipment.isRemoteOperable}
                // aria-label でボタンの目的を説明する
                aria-label={`設備 ${equipment.equipmentNumber} をリモート操作する`}
                // type="button" を明示する
                type="button"
              >
                {equipment.isRemoteOperable ? "リモート操作" : "操作不可"}
              </button>
              {/* 操作中の設備の操作パネルを表示する（選択時のみ）*/}
              {operatingEquipmentId === equipment.equipmentId && (
                // 操作パネルを region として表示する
                <div
                  // role="region" で操作パネルを示す
                  aria-label={`設備 ${equipment.equipmentNumber} 操作パネル`}
                >
                  {/* 操作パネル見出し */}
                  <h4>操作パネル: {equipment.equipmentName}</h4>
                  {/* 停止ボタン（tier2 pack の双方向操作 stub を呼ぶ）*/}
                  <button
                    // 停止コマンドを送信する stub（実装は tier2 pack が担当する）
                    onClick={() => { setOperatingEquipmentId(null); }}
                    // aria-label でボタンの目的を説明する
                    aria-label={`設備 ${equipment.equipmentNumber} を停止する`}
                    // type="button" を明示する
                    type="button"
                  >
                    停止
                  </button>
                  {/* 閉じるボタン */}
                  <button
                    // クリック時に操作パネルを閉じる
                    onClick={() => { setOperatingEquipmentId(null); }}
                    // aria-label でボタンの目的を説明する
                    aria-label="操作パネルを閉じる"
                    // type="button" を明示する
                    type="button"
                  >
                    閉じる
                  </button>
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
