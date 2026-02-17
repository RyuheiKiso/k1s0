import { invoke, Channel } from '@tauri-apps/api/core';

// Types matching k1s0-core Rust types

export type Kind = 'Server' | 'Client' | 'Library' | 'Database';
export type Tier = 'System' | 'Business' | 'Service';
export type Language = 'Go' | 'Rust' | 'TypeScript' | 'Dart';
export type Framework = 'React' | 'Flutter';
export type ApiStyle = 'Rest' | 'Grpc' | 'GraphQL';
export type Rdbms = 'PostgreSQL' | 'MySQL' | 'SQLite';
export type BuildMode = 'Development' | 'Production';
export type TestKind = 'Unit' | 'Integration' | 'E2e' | 'All';
export type Environment = 'Dev' | 'Staging' | 'Prod';

export interface DbInfo {
  name: string;
  rdbms: Rdbms;
}

export type LangFw =
  | { Language: Language }
  | { Framework: Framework }
  | { Database: { name: string; rdbms: Rdbms } };

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

// Tauri command wrappers

export async function getConfig(configPath: string): Promise<CliConfig> {
  return invoke<CliConfig>('get_config', { configPath });
}

export async function executeInit(config: InitConfig): Promise<void> {
  return invoke<void>('execute_init', { config });
}

export async function executeGenerate(config: GenerateConfig): Promise<void> {
  return invoke<void>('execute_generate', { config });
}

export async function executeBuild(config: BuildConfig): Promise<void> {
  return invoke<void>('execute_build', { config });
}

export async function executeTest(config: TestConfig): Promise<void> {
  return invoke<void>('execute_test', { config });
}

export async function executeDeploy(config: DeployConfig): Promise<void> {
  return invoke<void>('execute_deploy', { config });
}

export async function scanBuildableTargets(baseDir: string): Promise<string[]> {
  return invoke<string[]>('scan_buildable_targets', { baseDir });
}

export async function scanDeployableTargets(baseDir: string): Promise<string[]> {
  return invoke<string[]>('scan_deployable_targets', { baseDir });
}

export async function scanPlacements(tier: Tier, baseDir: string): Promise<string[]> {
  return invoke<string[]>('scan_placements', { tier, baseDir });
}

export async function validateName(name: string): Promise<void> {
  return invoke<void>('validate_name', { name });
}

// Progress event types matching k1s0-core ProgressEvent (tagged enum)

export type ProgressEvent =
  | { kind: 'StepStarted'; step: number; total: number; message: string }
  | { kind: 'StepCompleted'; step: number; total: number; message: string }
  | { kind: 'Log'; message: string }
  | { kind: 'Warning'; message: string }
  | { kind: 'Error'; message: string }
  | { kind: 'Finished'; success: boolean; message: string };

export async function executeTestWithProgress(
  config: TestConfig,
  onEvent: (event: ProgressEvent) => void,
): Promise<void> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onEvent;
  return invoke<void>('execute_test_with_progress', { config, onEvent: channel });
}

export async function executeBuildWithProgress(
  config: BuildConfig,
  onEvent: (event: ProgressEvent) => void,
): Promise<void> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onEvent;
  return invoke<void>('execute_build_with_progress', { config, onEvent: channel });
}

export async function executeDeployWithProgress(
  config: DeployConfig,
  onEvent: (event: ProgressEvent) => void,
): Promise<void> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onEvent;
  return invoke<void>('execute_deploy_with_progress', { config, onEvent: channel });
}
