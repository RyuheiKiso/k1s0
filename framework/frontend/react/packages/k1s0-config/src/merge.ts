/**
 * オブジェクトが単純なオブジェクト（配列でない）かどうかを判定
 */
function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * 2つのオブジェクトを深くマージする
 * - 後のオブジェクトの値が優先される
 * - ネストされたオブジェクトは再帰的にマージされる
 * - 配列は置換される（マージではない）
 */
export function deepMerge<T extends Record<string, unknown>>(
  target: T,
  source: Partial<T>
): T {
  const result = { ...target };

  for (const key of Object.keys(source) as (keyof T)[]) {
    const sourceValue = source[key];
    const targetValue = target[key];

    if (isPlainObject(sourceValue) && isPlainObject(targetValue)) {
      result[key] = deepMerge(
        targetValue as Record<string, unknown>,
        sourceValue as Record<string, unknown>
      ) as T[keyof T];
    } else if (sourceValue !== undefined) {
      result[key] = sourceValue as T[keyof T];
    }
  }

  return result;
}

/**
 * 複数のオブジェクトを順番にマージする
 */
export function mergeConfigs<T extends Record<string, unknown>>(
  ...configs: (T | Partial<T>)[]
): T {
  if (configs.length === 0) {
    return {} as T;
  }

  return configs.reduce((acc, config) => {
    return deepMerge(acc as T, config as Partial<T>);
  }, configs[0]) as T;
}

/**
 * 環境別設定をマージする
 */
export interface EnvironmentConfigs<T> {
  /** ベース設定（必須） */
  default: T;
  /** 開発環境用オーバーライド */
  dev?: Partial<T>;
  /** ステージング環境用オーバーライド */
  stg?: Partial<T>;
  /** 本番環境用オーバーライド */
  prod?: Partial<T>;
}

/**
 * 環境に応じた設定をマージして返す
 */
export function mergeEnvironmentConfig<T extends Record<string, unknown>>(
  configs: EnvironmentConfigs<T>,
  env: "dev" | "stg" | "prod"
): T {
  const baseConfig = configs.default;
  const envConfig = configs[env];

  if (!envConfig) {
    return baseConfig;
  }

  return deepMerge(baseConfig, envConfig);
}

/**
 * 設定のサブセットを抽出する
 */
export function extractConfigSection<T, K extends keyof T>(
  config: T,
  key: K
): T[K] | undefined {
  return config[key];
}

/**
 * 設定に特定のキーが存在するか確認する
 */
export function hasConfigKey<T>(
  config: T,
  key: string
): boolean {
  if (!isPlainObject(config)) {
    return false;
  }
  return key in config;
}

/**
 * ネストされたキーで設定値を取得する
 * 例: getNestedValue(config, "api.baseUrl")
 */
export function getNestedValue<T = unknown>(
  config: Record<string, unknown>,
  path: string
): T | undefined {
  const keys = path.split(".");
  let current: unknown = config;

  for (const key of keys) {
    if (!isPlainObject(current)) {
      return undefined;
    }
    current = current[key];
  }

  return current as T;
}

/**
 * ネストされたキーで設定値を設定する
 * 例: setNestedValue(config, "api.baseUrl", "https://api.example.com")
 */
export function setNestedValue<T extends Record<string, unknown>>(
  config: T,
  path: string,
  value: unknown
): T {
  const keys = path.split(".");
  const result = { ...config };
  let current: Record<string, unknown> = result;

  for (let i = 0; i < keys.length - 1; i++) {
    const key = keys[i];
    if (!isPlainObject(current[key])) {
      current[key] = {};
    } else {
      current[key] = { ...current[key] as Record<string, unknown> };
    }
    current = current[key] as Record<string, unknown>;
  }

  current[keys[keys.length - 1]] = value;
  return result;
}
