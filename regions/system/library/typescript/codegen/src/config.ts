import type { ValidationResult } from './types.js';

/** サーバーが属するティア（階層）を表す型 */
export type Tier = 'system' | 'business' | 'service';

/** API スタイルを表す型。REST・gRPC・両方のいずれか */
export type ApiStyle = 'rest' | 'grpc' | 'both';

/** データベースの種類を表す型 */
export type DatabaseType = 'postgres' | 'none';

/**
 * スキャフォールド生成の設定を表すインターフェース。
 * CLI から受け取った入力をもとにコード生成を制御する。
 */
export interface ScaffoldConfig {
  /** サーバー名（例: "auth", "config"） */
  name: string;
  /** 属するティア */
  tier: Tier;
  /** API スタイル */
  apiStyle: ApiStyle;
  /** データベースの種類 */
  database: DatabaseType;
  /** サーバーの説明 */
  description: string;
  /** proto ファイルのパス（gRPC 使用時） */
  protoPath?: string;
  /** クライアント SDK を生成するかどうか */
  generateClient: boolean;
}

/**
 * ScaffoldConfig のバリデーションを行う。
 * 名前が空でないこと、gRPC 使用時に protoPath が指定されていることなどを検証する。
 */
export function validateConfig(config: ScaffoldConfig): ValidationResult {
  const errors: string[] = [];

  // 名前が空でないことを検証する
  if (!config.name || config.name.trim().length === 0) {
    errors.push('name is required');
  }

  // 名前がアルファベット小文字・数字・ハイフンのみであることを検証する
  if (config.name && !/^[a-z][a-z0-9-]*$/.test(config.name)) {
    errors.push(
      'name must start with a lowercase letter and contain only lowercase letters, numbers, and hyphens',
    );
  }

  // 説明が空でないことを検証する
  if (!config.description || config.description.trim().length === 0) {
    errors.push('description is required');
  }

  // gRPC を使用する場合、protoPath が必須であることを検証する
  if (hasGrpc(config.apiStyle) && !config.protoPath) {
    errors.push('protoPath is required when apiStyle includes grpc');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * API スタイルに gRPC が含まれるかどうかを判定する。
 */
export function hasGrpc(style: ApiStyle): boolean {
  return style === 'grpc' || style === 'both';
}

/**
 * API スタイルに REST が含まれるかどうかを判定する。
 */
export function hasRest(style: ApiStyle): boolean {
  return style === 'rest' || style === 'both';
}

/**
 * データベースが設定されているかどうかを判定する。
 */
export function hasDatabase(db: DatabaseType): boolean {
  return db !== 'none';
}
