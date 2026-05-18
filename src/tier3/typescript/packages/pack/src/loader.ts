// k1s0 tier3 pack config loader（設計方針 27: pack config loader）
// 業種別 pack（manufacturing / retail / logistics 等）の設定を動的ロードする
// pack config は JSON ファイルとして管理し、dynamic import で遅延ロードする
// pack ID は tenant claim から取得する（直接引数で tenant_id を渡すことを禁止する）

// サポートされている pack の識別子（v1 で定義された pack のみ許可する）
export type PackId =
  // 製造業 pack（Plant / Field / Floor / Equipment 管理）
  | "manufacturing"
  // 小売業 pack（Store / Shelf / SKU 管理）
  | "retail"
  // 物流業 pack（Warehouse / Route / Shipment 管理）
  | "logistics";

// pack 設定の型
export interface PackConfig {
  // pack 識別子
  readonly packId: PackId;
  // pack の表示名（i18n キー）
  readonly displayNameKey: string;
  // pack バージョン
  readonly version: string;
  // 有効な画面 ID リスト（routing で参照する）
  readonly enabledScreenIds: readonly string[];
  // pack 固有の feature flags
  readonly featureFlags: Readonly<Record<string, boolean>>;
  // pack 固有の i18n 名前空間
  readonly i18nNamespace: string;
}

// manufacturing pack のデフォルト設定
const MANUFACTURING_PACK_CONFIG: PackConfig = {
  // pack 識別子を設定する
  packId: "manufacturing",
  // pack 表示名の i18n キーを設定する
  displayNameKey: "pack.manufacturing.displayName",
  // pack バージョンを設定する
  version: "1.0.0",
  // 有効な画面 ID リストを設定する
  enabledScreenIds: ["plant", "field", "floor", "equipment"],
  // pack 固有の feature flags を設定する
  featureFlags: {
    // フロア管理機能の有効/無効
    floorManagement: true,
    // 設備モニタリング機能の有効/無効
    equipmentMonitoring: true,
    // 生産計画機能の有効/無効
    productionPlanning: false,
  },
  // pack 固有の i18n 名前空間を設定する
  i18nNamespace: "manufacturing",
};

// retail pack のデフォルト設定
const RETAIL_PACK_CONFIG: PackConfig = {
  // pack 識別子を設定する
  packId: "retail",
  // pack 表示名の i18n キーを設定する
  displayNameKey: "pack.retail.displayName",
  // pack バージョンを設定する
  version: "1.0.0",
  // 有効な画面 ID リストを設定する
  enabledScreenIds: ["store", "shelf", "sku"],
  // pack 固有の feature flags を設定する
  featureFlags: {
    // 在庫管理機能の有効/無効
    inventoryManagement: true,
    // 棚割り管理機能の有効/無効
    shelfPlanning: false,
  },
  // pack 固有の i18n 名前空間を設定する
  i18nNamespace: "retail",
};

// logistics pack のデフォルト設定
const LOGISTICS_PACK_CONFIG: PackConfig = {
  // pack 識別子を設定する
  packId: "logistics",
  // pack 表示名の i18n キーを設定する
  displayNameKey: "pack.logistics.displayName",
  // pack バージョンを設定する
  version: "1.0.0",
  // 有効な画面 ID リストを設定する
  enabledScreenIds: ["warehouse", "route", "shipment"],
  // pack 固有の feature flags を設定する
  featureFlags: {
    // ルート最適化機能の有効/無効
    routeOptimization: true,
    // リアルタイム追跡機能の有効/無効
    realTimeTracking: false,
  },
  // pack 固有の i18n 名前空間を設定する
  i18nNamespace: "logistics",
};

// pack ID → pack 設定のマップ（static registry）
const PACK_REGISTRY: Readonly<Record<PackId, PackConfig>> = {
  // manufacturing pack を登録する
  manufacturing: MANUFACTURING_PACK_CONFIG,
  // retail pack を登録する
  retail: RETAIL_PACK_CONFIG,
  // logistics pack を登録する
  logistics: LOGISTICS_PACK_CONFIG,
};

// pack config キャッシュ（同一 pack ID の重複ロードを防ぐ）
const _packConfigCache = new Map<PackId, PackConfig>();

// pack config をロードする
// packId: ロードする pack の識別子（tenant claim から取得した値）
// オーバーライドコンフィグを渡すことで pack 設定をカスタマイズできる（テスト用）
export function loadPackConfig(packId: PackId, override?: Partial<PackConfig>): PackConfig {
  // キャッシュに存在する場合はキャッシュから返す
  const cached = _packConfigCache.get(packId);
  if (cached !== undefined && override === undefined) {
    // キャッシュヒット: キャッシュから返す
    return cached;
  }
  // registry から pack 設定を取得する
  const baseConfig = PACK_REGISTRY[packId];
  // override が存在する場合はマージする
  const config: PackConfig = override !== undefined
    ? { ...baseConfig, ...override }
    : baseConfig;
  // キャッシュに保存する（override がある場合はキャッシュしない）
  if (override === undefined) {
    _packConfigCache.set(packId, config);
  }
  // pack 設定を返す
  return config;
}

// pack config キャッシュをクリアする（テスト用）
export function clearPackConfigCache(): void {
  // キャッシュをクリアする
  _packConfigCache.clear();
}

// pack ID が有効かどうかを確認する型ガード
export function isValidPackId(value: string): value is PackId {
  // 既知の pack ID に含まれるか確認する
  return Object.keys(PACK_REGISTRY).includes(value);
}

// 画面 ID が指定した pack で有効かどうかを確認する
export function isScreenEnabled(packId: PackId, screenId: string): boolean {
  // pack 設定をロードする
  const config = loadPackConfig(packId);
  // 有効な画面 ID リストに含まれるか確認する
  return config.enabledScreenIds.includes(screenId);
}

// feature flag の値を取得する（未設定の場合は false を返す）
export function getFeatureFlag(packId: PackId, flagKey: string): boolean {
  // pack 設定をロードする
  const config = loadPackConfig(packId);
  // feature flag の値を返す（未設定の場合は false）
  return config.featureFlags[flagKey] ?? false;
}
