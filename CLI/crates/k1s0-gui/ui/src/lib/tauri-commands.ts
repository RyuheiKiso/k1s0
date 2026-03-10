import { Channel, invoke } from '@tauri-apps/api/core';

export type Kind = 'Server' | 'Client' | 'Library' | 'Database';
export type Tier = 'System' | 'Business' | 'Service';
export type Language = 'Go' | 'Rust' | 'TypeScript' | 'Dart';
export type Framework = 'React' | 'Flutter';
export type ApiStyle = 'Rest' | 'Grpc' | 'GraphQL';
export type Rdbms = 'PostgreSQL' | 'MySQL' | 'SQLite';
export type BuildMode = 'Development' | 'Production';
export type TestKind = 'Unit' | 'Integration' | 'All';
export type Environment = 'Dev' | 'Staging' | 'Prod';
export type GenerateTarget = 'typescript' | 'dart';

export interface DbInfo {
  name: string;
  rdbms: Rdbms;
}

export type LangFw =
  | { Language: Language }
  | { Framework: Framework }
  | { Database: DbInfo };

export interface DetailConfig {
  name: string | null;
  api_styles: ApiStyle[];
  db: DbInfo | null;
  kafka: boolean;
  redis: boolean;
  bff_language: Language | null;
}

export interface GenerateConfig {
  kind: Kind;
  tier: Tier;
  placement: string | null;
  lang_fw: LangFw;
  detail: DetailConfig;
}

export interface InitConfig {
  project_name: string;
  git_init: boolean;
  sparse_checkout: boolean;
  tiers: Tier[];
}

export interface BuildConfig {
  targets: string[];
  mode: BuildMode;
}

export interface TestConfig {
  kind: TestKind;
  targets: string[];
}

export interface DeployConfig {
  environment: Environment;
  targets: string[];
}

export interface CliConfig {
  docker_registry: string;
  go_module_base: string;
  [key: string]: unknown;
}

export type ProgressEvent =
  | { kind: 'StepStarted'; step: number; total: number; message: string }
  | { kind: 'StepCompleted'; step: number; total: number; message: string }
  | { kind: 'Log'; message: string }
  | { kind: 'Warning'; message: string }
  | { kind: 'Error'; message: string }
  | { kind: 'Finished'; success: boolean; message: string };

export interface DeviceAuthorizationChallenge {
  issuer: string;
  client_id: string;
  scope: string;
  token_endpoint: string;
  device_code: string;
  user_code: string;
  verification_uri: string;
  verification_uri_complete: string;
  interval: number;
  expires_in: number;
}

export interface AuthTokens {
  access_token: string;
  refresh_token: string | null;
  id_token: string | null;
  token_type: string;
  expires_in: number;
  scope: string | null;
}

export type DeviceAuthorizationPollResult =
  | { status: 'Pending'; interval: number; message: string }
  | { status: 'Success'; tokens: AuthTokens }
  | { status: 'Error'; message: string };

function createProgressChannel(onEvent: (event: ProgressEvent) => void): Channel<ProgressEvent> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onEvent;
  return channel;
}

export async function getConfig(configPath: string): Promise<CliConfig> {
  return invoke<CliConfig>('get_config', { configPath });
}

export async function executeInit(config: InitConfig): Promise<void> {
  return invoke<void>('execute_init', { config });
}

export async function executeGenerate(config: GenerateConfig): Promise<void> {
  return invoke<void>('execute_generate', { config });
}

export async function executeGenerateAt(
  config: GenerateConfig,
  baseDir: string,
): Promise<void> {
  return invoke<void>('execute_generate_at', { config, baseDir });
}

export async function executeBuild(config: BuildConfig): Promise<void> {
  return invoke<void>('execute_build', { config });
}

export async function executeTest(config: TestConfig): Promise<void> {
  return invoke<void>('execute_test', { config });
}

export async function executeTestAt(config: TestConfig, baseDir: string): Promise<void> {
  return invoke<void>('execute_test_at', { config, baseDir });
}

export async function executeDeploy(config: DeployConfig): Promise<void> {
  return invoke<void>('execute_deploy', { config });
}

export async function executeDeployRollback(target: string): Promise<string> {
  return invoke<string>('execute_deploy_rollback', { target });
}

export async function scanBuildableTargets(baseDir: string): Promise<string[]> {
  return invoke<string[]>('scan_buildable_targets', { baseDir });
}

export async function scanDeployableTargets(baseDir: string): Promise<string[]> {
  return invoke<string[]>('scan_deployable_targets', { baseDir });
}

export async function scanTestableTargets(baseDir: string): Promise<string[]> {
  return invoke<string[]>('scan_testable_targets', { baseDir });
}

export async function scanPlacements(tier: Tier, baseDir: string): Promise<string[]> {
  return invoke<string[]>('scan_placements', { tier, baseDir });
}

export async function validateName(name: string): Promise<void> {
  return invoke<void>('validate_name', { name });
}

export async function executeValidateConfigSchema(path: string): Promise<number> {
  return invoke<number>('execute_validate_config_schema', { path });
}

export async function executeValidateNavigation(path: string): Promise<number> {
  return invoke<number>('execute_validate_navigation', { path });
}

export async function executeGenerateConfigTypes(
  schemaPath: string,
  target: GenerateTarget,
): Promise<string> {
  return invoke<string>('execute_generate_config_types', { schemaPath, target });
}

export async function executeGenerateNavigationTypes(
  navPath: string,
  target: GenerateTarget,
): Promise<string> {
  return invoke<string>('execute_generate_navigation_types', { navPath, target });
}

export async function executeTestWithProgress(
  config: TestConfig,
  onEvent: (event: ProgressEvent) => void,
): Promise<void> {
  return invoke<void>('execute_test_with_progress', {
    config,
    onEvent: createProgressChannel(onEvent),
  });
}

export async function executeTestWithProgressAt(
  config: TestConfig,
  baseDir: string,
  onEvent: (event: ProgressEvent) => void,
): Promise<void> {
  return invoke<void>('execute_test_with_progress_at', {
    config,
    baseDir,
    onEvent: createProgressChannel(onEvent),
  });
}

export async function executeBuildWithProgress(
  config: BuildConfig,
  onEvent: (event: ProgressEvent) => void,
): Promise<void> {
  return invoke<void>('execute_build_with_progress', {
    config,
    onEvent: createProgressChannel(onEvent),
  });
}

export async function executeDeployWithProgress(
  config: DeployConfig,
  onEvent: (event: ProgressEvent) => void,
): Promise<void> {
  return invoke<void>('execute_deploy_with_progress', {
    config,
    onEvent: createProgressChannel(onEvent),
  });
}

export async function detectWorkspaceRoot(): Promise<string | null> {
  return invoke<string | null>('detect_workspace_root');
}

export async function resolveWorkspaceRoot(path: string): Promise<string> {
  return invoke<string>('resolve_workspace_root', { path });
}

export async function startDeviceAuthorization(): Promise<DeviceAuthorizationChallenge> {
  return invoke<DeviceAuthorizationChallenge>('start_device_authorization');
}

export async function pollDeviceAuthorization(
  challenge: DeviceAuthorizationChallenge,
): Promise<DeviceAuthorizationPollResult> {
  return invoke<DeviceAuthorizationPollResult>('poll_device_authorization', { challenge });
}
