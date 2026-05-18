// tier3 pack package のエントリポイント
// 業務 pack loader の公開 API をエクスポートする

// loader.ts の各 API を再エクスポートする
export {
  // pack 設定をロードする関数をエクスポートする
  loadPackConfig,
  // pack config キャッシュをクリアする関数をエクスポートする
  clearPackConfigCache,
  // pack ID が有効かどうかを確認する型ガードをエクスポートする
  isValidPackId,
  // 画面 ID が指定した pack で有効かどうかを確認する関数をエクスポートする
  isScreenEnabled,
  // feature flag の値を取得する関数をエクスポートする
  getFeatureFlag,
} from './loader.js';
// PackId 型と PackConfig 型をエクスポートする
export type { PackId, PackConfig } from './loader.js';

// --------- manufacturing pack 業務エンティティ stub 型 ---------
// 実際の実装では tier2 生成 stub（protoc / OpenAPI codegen）を使用する
// 現フェーズでは pack package が tier2 stub の代替ファサードとなる

// 工場（Plant）エンティティの型（manufacturing pack stub）
// tier2 生成 stub の代替ファサードとして @k1s0/pack から export する
export interface PlantSummary {
  // 工場 ID（UUID）
  readonly plantId: string;
  // 工場名
  readonly name: string;
  // 所在地（都市名）
  readonly location: string;
  // アクティブかどうか
  readonly active: boolean;
}
