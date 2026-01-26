// Schema exports
export {
  apiConfigSchema,
  authConfigSchema,
  loggingConfigSchema,
  telemetryConfigSchema,
  featureFlagsSchema,
  appConfigSchema,
  validateConfig,
  validatePartialConfig,
  type ApiConfig,
  type AuthConfig,
  type LoggingConfig,
  type TelemetryConfig,
  type FeatureFlags,
  type AppConfig,
} from "./schema.js";

// Loader exports
export {
  parseConfig,
  loadConfigFromUrl,
  loadConfigsFromUrls,
  resolveConfigPaths,
  ConfigLoader,
  type ConfigFormat,
  type LoadOptions,
} from "./loader.js";

// Merge exports
export {
  deepMerge,
  mergeConfigs,
  mergeEnvironmentConfig,
  extractConfigSection,
  hasConfigKey,
  getNestedValue,
  setNestedValue,
  type EnvironmentConfigs,
} from "./merge.js";
