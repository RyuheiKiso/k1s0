/**
 * 生成ウィザードの純粋関数群
 * ステップ制御・ティア判定・言語オプション取得などのビジネスロジックを提供する
 */

import type { Kind, Language, Tier } from './tauri-commands';

/** ステップ名のラベル一覧 */
export const STEP_LABELS = ['種別', 'ティア', '配置', '言語', '詳細', '確認'] as const;

/** サーバーのデータベース接続モード */
export type ServerDatabaseMode = 'none' | 'existing' | 'new';

/** APIスタイルの選択肢 */
export const API_STYLE_VALUES = ['Rest', 'Grpc', 'GraphQL'] as const;

/** BFF言語の選択肢 */
export const BFF_LANGUAGE_VALUES = ['Go', 'Rust'] as const;

/**
 * 種別に応じて利用可能なティア一覧を返す
 * Server/Databaseは全ティア、Clientはビジネス以上、Libraryはシステムとビジネスのみ
 */
export function getAvailableTiers(kind: Kind): Tier[] {
  switch (kind) {
    case 'Server':
      return ['System', 'Business', 'Service'];
    case 'Client':
      return ['Business', 'Service'];
    case 'Library':
      return ['System', 'Business'];
    case 'Database':
      return ['System', 'Business', 'Service'];
  }
}

/**
 * 種別に応じて選択可能な言語一覧を返す
 * ServerはGo/Rustのみ、それ以外は全言語
 */
export function getLanguageOptions(kind: Kind): Language[] {
  switch (kind) {
    case 'Server':
      return ['Go', 'Rust'];
    case 'Library':
      return ['Go', 'Rust', 'TypeScript', 'Dart'];
    default:
      return ['Go', 'Rust', 'TypeScript', 'Dart'];
  }
}

/**
 * 配置ステップをスキップすべきかどうかを判定する
 * Systemティアの場合はスキップ（配置の概念がない）
 */
export function shouldSkipPlacement(tier: Tier): boolean {
  return tier === 'System';
}

/**
 * 詳細オプションステップをスキップすべきかどうかを判定する
 * Databaseモジュール、またはClient+Serviceの組み合わせではスキップ
 */
export function shouldSkipDetail(kind: Kind, tier: Tier): boolean {
  return kind === 'Database' || (kind === 'Client' && tier === 'Service');
}

/**
 * 現在のステップから次のステップを計算する
 * スキップ対象のステップを飛ばして最大5（確認ステップ）まで進む
 */
export function getNextStep(currentStep: number, kind: Kind, tier: Tier): number {
  let nextStep = currentStep + 1;

  if (nextStep === 2 && shouldSkipPlacement(tier)) {
    nextStep += 1;
  }

  if (nextStep === 4 && shouldSkipDetail(kind, tier)) {
    nextStep += 1;
  }

  return Math.min(nextStep, 5);
}

/**
 * 現在のステップから前のステップを計算する
 * スキップ対象のステップを飛ばして最小0（種別選択）まで戻る
 */
export function getPreviousStep(currentStep: number, kind: Kind, tier: Tier): number {
  let previousStep = currentStep - 1;

  if (previousStep === 4 && shouldSkipDetail(kind, tier)) {
    previousStep -= 1;
  }

  if (previousStep === 2 && shouldSkipPlacement(tier)) {
    previousStep -= 1;
  }

  return Math.max(previousStep, 0);
}

/**
 * 種別に応じたデフォルトの詳細名を返す
 * 各種別で慣例的に使われる名前を提供する
 */
export function getDefaultDetailName(kind: Kind): string {
  switch (kind) {
    case 'Client':
      return 'app';
    case 'Library':
      return 'shared';
    case 'Database':
      return 'main';
    case 'Server':
      return 'service';
  }
}
