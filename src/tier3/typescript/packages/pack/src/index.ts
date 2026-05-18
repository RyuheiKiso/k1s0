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
