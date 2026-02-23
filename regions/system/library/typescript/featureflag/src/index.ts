export class FeatureFlagError extends Error {
  constructor(
    message: string,
    public readonly code: string,
  ) {
    super(message);
    this.name = 'FeatureFlagError';
  }
}

export interface FlagVariant {
  name: string;
  value: string;
  weight: number;
}

export interface FeatureFlag {
  id: string;
  flagKey: string;
  description: string;
  enabled: boolean;
  variants: FlagVariant[];
}

export interface EvaluationContext {
  userId?: string;
  tenantId?: string;
  attributes?: Record<string, string>;
}

export interface EvaluationResult {
  flagKey: string;
  enabled: boolean;
  variant?: string;
  reason: string;
}

export interface FeatureFlagClient {
  evaluate(flagKey: string, context: EvaluationContext): Promise<EvaluationResult>;
  getFlag(flagKey: string): Promise<FeatureFlag>;
  isEnabled(flagKey: string, context: EvaluationContext): Promise<boolean>;
}

export class InMemoryFeatureFlagClient implements FeatureFlagClient {
  private flags = new Map<string, FeatureFlag>();

  setFlag(flag: FeatureFlag): void {
    this.flags.set(flag.flagKey, flag);
  }

  async evaluate(flagKey: string, _context: EvaluationContext): Promise<EvaluationResult> {
    const flag = this.flags.get(flagKey);
    if (!flag) {
      throw new FeatureFlagError(`フラグが見つかりません: ${flagKey}`, 'FLAG_NOT_FOUND');
    }
    return {
      flagKey,
      enabled: flag.enabled,
      variant: flag.variants.length > 0 ? flag.variants[0].name : undefined,
      reason: flag.enabled ? 'FLAG_ENABLED' : 'FLAG_DISABLED',
    };
  }

  async getFlag(flagKey: string): Promise<FeatureFlag> {
    const flag = this.flags.get(flagKey);
    if (!flag) {
      throw new FeatureFlagError(`フラグが見つかりません: ${flagKey}`, 'FLAG_NOT_FOUND');
    }
    return flag;
  }

  async isEnabled(flagKey: string, context: EvaluationContext): Promise<boolean> {
    const result = await this.evaluate(flagKey, context);
    return result.enabled;
  }
}
