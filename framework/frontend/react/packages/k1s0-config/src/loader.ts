import YAML from "yaml";

/**
 * 設定ファイルの形式
 */
export type ConfigFormat = "json" | "yaml";

/**
 * 設定ロードオプション
 */
export interface LoadOptions {
  /** ファイル形式（自動検出する場合は省略） */
  format?: ConfigFormat;
  /** 環境変数の展開を有効にするか */
  expandEnv?: boolean;
}

/**
 * 環境変数プレースホルダを展開する
 * 形式: ${ENV_VAR} または ${ENV_VAR:default_value}
 */
function expandEnvironmentVariables(value: string): string {
  return value.replace(/\$\{([^}:]+)(?::([^}]*))?\}/g, (_, envVar, defaultValue) => {
    let envValue: string | undefined;
    if (typeof window !== "undefined") {
      // ブラウザ環境: __ENV__ オブジェクトから取得
      envValue = (window as unknown as Record<string, Record<string, string>>).__ENV__?.[envVar];
    } else if (typeof globalThis !== "undefined" && "process" in globalThis) {
      // Node.js 環境: process.env から取得
      envValue = (globalThis as unknown as { process: { env: Record<string, string | undefined> } }).process.env[envVar];
    }
    return envValue ?? defaultValue ?? "";
  });
}

/**
 * オブジェクト内の全文字列値の環境変数を展開する
 */
function expandEnvInObject(obj: unknown): unknown {
  if (typeof obj === "string") {
    return expandEnvironmentVariables(obj);
  }
  if (Array.isArray(obj)) {
    return obj.map(expandEnvInObject);
  }
  if (obj !== null && typeof obj === "object") {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = expandEnvInObject(value);
    }
    return result;
  }
  return obj;
}

/**
 * ファイルパスから形式を推測する
 */
function detectFormat(path: string): ConfigFormat {
  const ext = path.split(".").pop()?.toLowerCase();
  if (ext === "yaml" || ext === "yml") {
    return "yaml";
  }
  return "json";
}

/**
 * 文字列から設定をパースする
 */
export function parseConfig<T = unknown>(
  content: string,
  options: LoadOptions = {}
): T {
  const format = options.format ?? "json";

  let parsed: unknown;
  if (format === "yaml") {
    parsed = YAML.parse(content);
  } else {
    parsed = JSON.parse(content);
  }

  if (options.expandEnv !== false) {
    parsed = expandEnvInObject(parsed);
  }

  return parsed as T;
}

/**
 * URLから設定を読み込む
 */
export async function loadConfigFromUrl<T = unknown>(
  url: string,
  options: LoadOptions = {}
): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to load config from ${url}: ${response.status} ${response.statusText}`);
  }

  const content = await response.text();
  const format = options.format ?? detectFormat(url);

  return parseConfig<T>(content, { ...options, format });
}

/**
 * 複数のURLから設定を読み込む
 */
export async function loadConfigsFromUrls<T = unknown>(
  urls: string[],
  options: LoadOptions = {}
): Promise<T[]> {
  return Promise.all(urls.map((url) => loadConfigFromUrl<T>(url, options)));
}

/**
 * 設定ファイルパスを環境に基づいて解決する
 */
export function resolveConfigPaths(
  basePath: string,
  env: string,
  extension: ConfigFormat = "yaml"
): string[] {
  const ext = extension === "yaml" ? "yaml" : "json";
  return [
    `${basePath}/default.${ext}`,
    `${basePath}/${env}.${ext}`,
  ];
}

/**
 * ブラウザ環境で利用可能な設定ローダー
 */
export class ConfigLoader<T = unknown> {
  private basePath: string;
  private format: ConfigFormat;
  private cache: Map<string, T> = new Map();

  constructor(basePath: string, format: ConfigFormat = "yaml") {
    this.basePath = basePath;
    this.format = format;
  }

  /**
   * 指定された環境の設定を読み込む
   */
  async load(env: string, useCache = true): Promise<T> {
    const cacheKey = env;

    if (useCache && this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey)!;
    }

    const paths = resolveConfigPaths(this.basePath, env, this.format);
    const config = await loadConfigFromUrl<T>(paths[0], { format: this.format });

    if (useCache) {
      this.cache.set(cacheKey, config);
    }

    return config;
  }

  /**
   * キャッシュをクリアする
   */
  clearCache(): void {
    this.cache.clear();
  }
}
