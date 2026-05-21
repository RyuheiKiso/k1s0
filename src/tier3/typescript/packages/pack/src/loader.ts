// k1s0 tier3 pack config loader（設計方針 27: pack config loader）
// 業種別 pack（manufacturing / retail / logistics 等）の設定を動的ロードする
// pack config は JSON ファイルとして管理し、dynamic import で遅延ロードする
// pack ID は tenant claim から取得する（直接引数で tenant_id を渡すことを禁止する）
// 4 level resolver: Level1（業界横断デフォルト）→ Level2（業界共通）→ Level3（業界固有）→ Level4（テナント固有）

// PackResolutionLevel: 4 level resolver の各レベルを識別する型
export type PackResolutionLevel =
  // Level 1: 業界横断デフォルト（全 pack 共通の基底設定）
  | "cross_industry_default"
  // Level 2: 業界共通設定（pack 固有のデフォルト設定）
  | "industry_common"
  // Level 3: 業界固有設定（pack 内の variant 設定）
  | "industry_specific"
  // Level 4: テナント固有 override（tenant claim に基づく上書き設定）
  | "tenant_override";

// PackResolvedLayer: 4 level resolver の解決済みレイヤーを表す型
// 各レベルの設定を保持して最終設定を合成するために使用する
export interface PackResolvedLayer {
  // 解決レベルを識別する
  readonly level: PackResolutionLevel;
  // このレベルで適用される部分設定（override がない場合は空オブジェクト）
  readonly partialConfig: Partial<Omit<PackConfig, 'packId'>>;
}

// TenantPackOverride: テナント固有の pack override 設定（tenant_id は BFF cookie から注入する）
// tenant_id を直接フィールドに持たない（tier3/CLAUDE.md: 公開 type に tenant_id フィールド禁止）
export interface TenantPackOverride {
  // テナント固有で上書きする feature flags（存在するキーのみ上書きする）
  readonly featureFlags?: Partial<Record<string, boolean>>;
  // テナント固有で上書きする有効画面 ID リスト（指定した場合は完全置換する）
  readonly enabledScreenIds?: readonly string[];
  // テナント固有の i18n 名前空間（指定した場合はデフォルトに suffix を追加する）
  readonly i18nNamespaceSuffix?: string;
}

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

// resolveFourLevelPackConfig: 4 level resolver で pack 設定を解決する
// packId: ロードする pack の識別子（tenant claim から取得した値）
// tenantOverride: Level4 テナント固有 override（tenant_id は BFF cookie から注入。引数に含めない）
// 4 level resolver: Level1 → Level2 → Level3 → Level4 の順に適用する（後のレベルが優先される）
export function resolveFourLevelPackConfig(
  // ロードする pack の識別子を受け取る（tenant claim から取得した値）
  packId: PackId,
  // Level4: テナント固有 override を受け取る（省略時は Level4 適用なし）
  tenantOverride?: TenantPackOverride
): PackConfig {
  // Level1: 業界横断デフォルト設定を取得する（全 pack 共通の基底設定）
  const level1: PackResolvedLayer = {
    // 業界横断デフォルトレベルを識別する
    level: "cross_industry_default",
    // 業界横断デフォルト: feature flags をすべて false に設定する（明示的に有効化が必要）
    partialConfig: {
      // デフォルト feature flags（全て false で初期化する）
      featureFlags: {},
    },
  };

  // Level2: 業界共通設定を取得する（pack 固有のデフォルト設定）
  const baseConfig = PACK_REGISTRY[packId];
  // Level2: pack の基本設定を業界共通設定として使用する
  const level2: PackResolvedLayer = {
    // 業界共通設定レベルを識別する
    level: "industry_common",
    // pack の基本設定を部分設定として使用する
    partialConfig: {
      // 業界共通の feature flags を設定する
      featureFlags: baseConfig.featureFlags,
      // 業界共通の有効画面 ID リストを設定する
      enabledScreenIds: baseConfig.enabledScreenIds,
      // 業界共通の i18n 名前空間を設定する
      i18nNamespace: baseConfig.i18nNamespace,
    },
  };

  // Level3: 業界固有設定を取得する（pack 内の variant 設定）
  // 現 v1 では Level3 は Level2 と同一（将来の variant 設定拡張のための予約）
  const level3: PackResolvedLayer = {
    // 業界固有設定レベルを識別する
    level: "industry_specific",
    // Level3 は現時点では空の override（将来の業界 variant 対応のプレースホルダー）
    partialConfig: {},
  };

  // Level4: テナント固有 override を適用する（tenant_id は BFF cookie から注入）
  const level4: PackResolvedLayer = {
    // テナント固有 override レベルを識別する
    level: "tenant_override",
    // テナント固有 override が存在する場合のみ適用する
    partialConfig: tenantOverride !== undefined ? {
      // テナント固有 feature flags を上書きする（存在するキーのみ上書き）
      featureFlags: tenantOverride.featureFlags !== undefined
        ? { ...baseConfig.featureFlags, ...tenantOverride.featureFlags }
        : undefined,
      // テナント固有 enabledScreenIds を完全置換する（指定した場合のみ）
      enabledScreenIds: tenantOverride.enabledScreenIds,
      // テナント固有 i18n 名前空間 suffix を追加する（指定した場合のみ）
      i18nNamespace: tenantOverride.i18nNamespaceSuffix !== undefined
        ? `${baseConfig.i18nNamespace}.${tenantOverride.i18nNamespaceSuffix}`
        : undefined,
    } : {},
  };

  // 4 level を順に合成する（後のレベルが優先: Level4 > Level3 > Level2 > Level1）
  const resolvedLayers = [level1, level2, level3, level4];
  // 各レベルの部分設定を順に合成する
  let mergedFeatureFlags: Record<string, boolean> = {};
  // mergedEnabledScreenIds: 最後に指定されたレベルの値を使用する
  let mergedEnabledScreenIds: readonly string[] = baseConfig.enabledScreenIds;
  // mergedI18nNamespace: 最後に指定されたレベルの値を使用する
  let mergedI18nNamespace: string = baseConfig.i18nNamespace;

  // 各レベルの設定を順に適用する（後のレベルが優先される）
  for (const resolvedLayer of resolvedLayers) {
    // feature flags をマージする（後のレベルのキーが優先）
    if (resolvedLayer.partialConfig.featureFlags !== undefined) {
      // 現在のレベルの feature flags を上書きマージする
      mergedFeatureFlags = { ...mergedFeatureFlags, ...resolvedLayer.partialConfig.featureFlags };
    }
    // enabledScreenIds を上書きする（後のレベルが優先）
    if (resolvedLayer.partialConfig.enabledScreenIds !== undefined) {
      // 現在のレベルの enabledScreenIds で完全置換する
      mergedEnabledScreenIds = resolvedLayer.partialConfig.enabledScreenIds;
    }
    // i18nNamespace を上書きする（後のレベルが優先）
    if (resolvedLayer.partialConfig.i18nNamespace !== undefined) {
      // 現在のレベルの i18nNamespace で置換する
      mergedI18nNamespace = resolvedLayer.partialConfig.i18nNamespace;
    }
  }

  // 4 level 解決済み pack 設定を返す
  return {
    // pack 識別子はベース設定から取得する（変更しない）
    packId: baseConfig.packId,
    // pack 表示名はベース設定から取得する（変更しない）
    displayNameKey: baseConfig.displayNameKey,
    // pack バージョンはベース設定から取得する（変更しない）
    version: baseConfig.version,
    // 4 level で解決した enabledScreenIds を設定する
    enabledScreenIds: mergedEnabledScreenIds,
    // 4 level で解決した feature flags を設定する
    featureFlags: mergedFeatureFlags,
    // 4 level で解決した i18n 名前空間を設定する
    i18nNamespace: mergedI18nNamespace,
  };
}

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

// getFeatureFlag: feature flag の値を取得する（未設定の場合は false を返す）
// 4 level resolver を通じて取得する（tenant_id は BFF cookie から注入済みの TenantPackOverride で渡す）
export function getFeatureFlag(
  // ロードする pack の識別子を受け取る
  packId: PackId,
  // 取得する feature flag のキーを受け取る
  flagKey: string,
  // Level4 テナント固有 override を受け取る（省略時は Level1-3 のみ適用する）
  tenantOverride?: TenantPackOverride
): boolean {
  // 4 level resolver で pack 設定を解決する
  const config = resolveFourLevelPackConfig(packId, tenantOverride);
  // feature flag の値を返す（未設定の場合は false）
  return config.featureFlags[flagKey] ?? false;
}
