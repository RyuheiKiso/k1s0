/**
 * 生成ウィザードのフォーム状態管理カスタムフック
 * react-hook-form + zodを使ったバリデーション、各種状態管理、生成実行ロジックを提供する
 */

import { zodResolver } from '@hookform/resolvers/zod';
import { useEffect, useState } from 'react';
import { type UseFormSetValue, useForm } from 'react-hook-form';
import { z } from 'zod';

import { BFF_LANGUAGE_VALIDATION_ERROR } from '../constants/messages';
import {
  API_STYLE_VALUES,
  BFF_LANGUAGE_VALUES,
  getAvailableTiers,
  getDefaultDetailName,
  getNextStep,
  getPreviousStep,
  shouldSkipDetail,
  shouldSkipPlacement,
  type ServerDatabaseMode,
} from './generate-wizard';
import { toDisplayPath } from './paths';
import {
  executeGenerateAt,
  scanDatabases,
  scanGenerateConflicts,
  scanPlacements,
  validateName,
  type ApiStyle,
  type DetailConfig,
  type Framework,
  type GenerateConfig,
  type Kind,
  type LangFw,
  type Language,
  type Rdbms,
  type ScaffoldDatabaseInfo,
  type Tier,
} from './tauri-commands';

/** zodスキーマ: フォーム入力のバリデーション定義 */
const generateSchema = z.object({
  placement: z.string().trim().min(1, '配置は必須です。'),
  detailName: z.string().trim().min(1, '名前は必須です。'),
  databaseName: z.string().trim().min(1, '名前は必須です。'),
  newDatabaseName: z.string().trim().min(1, 'データベース名は必須です。'),
  apiStyles: z.array(z.enum(API_STYLE_VALUES)),
  selectedDatabasePath: z.string(),
  generateBff: z.boolean(),
  bffLanguage: z.enum(BFF_LANGUAGE_VALUES).nullable(),
});

/** zodスキーマから推論されたフォームデータ型 */
type GenerateFormData = z.infer<typeof generateSchema>;

/** 利用可否チェックのエラー情報 */
interface AvailabilityError {
  key: string;
  message: string;
}

/** カスタムフックの戻り値の型定義 */
export interface UseGenerateFormReturn {
  // ステップ制御
  step: number;
  goNext: () => Promise<void>;
  goBack: () => void;

  // 種別・ティア
  kind: Kind;
  tier: Tier;
  handleKindChange: (nextKind: Kind) => void;
  handleTierChange: (nextTier: Tier) => void;

  // 配置関連
  placement: string;
  setPlacement: React.Dispatch<React.SetStateAction<string>>;
  existingPlacements: string[];
  isNewPlacement: boolean;
  setIsNewPlacement: React.Dispatch<React.SetStateAction<boolean>>;
  placementError: string;
  validatePlacementValue: (value: string) => Promise<boolean>;

  // 言語・フレームワーク
  language: Language;
  setLanguage: React.Dispatch<React.SetStateAction<Language>>;
  framework: Framework;
  setFramework: React.Dispatch<React.SetStateAction<Framework>>;

  // データベース（種別=Database用）
  databaseName: string;
  setDatabaseName: React.Dispatch<React.SetStateAction<string>>;
  databaseEngine: Rdbms;
  setDatabaseEngine: React.Dispatch<React.SetStateAction<Rdbms>>;
  nameError: string;

  // サーバーデータベース設定
  serverDatabaseMode: ServerDatabaseMode;
  setServerDatabaseMode: React.Dispatch<React.SetStateAction<ServerDatabaseMode>>;
  availableDatabases: ScaffoldDatabaseInfo[];
  selectedDatabasePath: string;
  setSelectedDatabasePath: React.Dispatch<React.SetStateAction<string>>;
  newDatabaseName: string;
  setNewDatabaseName: React.Dispatch<React.SetStateAction<string>>;
  newDatabaseEngine: Rdbms;
  setNewDatabaseEngine: React.Dispatch<React.SetStateAction<Rdbms>>;
  serverDatabaseError: string;

  // 詳細オプション
  detail: DetailConfig;
  setDetail: React.Dispatch<React.SetStateAction<DetailConfig>>;
  toggleApiStyle: (style: ApiStyle) => void;
  detailError: string;
  validateDetailName: (value: string) => Promise<boolean>;
  validateNameField: (
    field: 'placement' | 'detailName' | 'databaseName' | 'newDatabaseName',
    value: string,
    duplicateMessage?: string,
  ) => Promise<boolean>;

  // BFF関連
  generateBff: boolean;
  setGenerateBff: React.Dispatch<React.SetStateAction<boolean>>;
  showBffControls: boolean;
  selectedBffLanguage: Language | null;

  // 生成実行
  status: 'idle' | 'loading' | 'success' | 'error';
  errorMessage: string;
  handleGenerate: () => Promise<void>;

  // 表示制御
  showPlacementStep: boolean;
  showDetailStep: boolean;
  availabilityErrorMessage: string;
  currentGenerateConfig: GenerateConfig;
  currentRuntime: LangFw;
  selectedExistingDatabase: ScaffoldDatabaseInfo | null;
  resolvedServerDatabase: { name: string; rdbms: Rdbms } | null;

  // react-hook-formの機能をステップコンポーネントに委譲するための関数群
  setValue: UseFormSetValue<GenerateFormData>;
  clearErrors: (name?: keyof GenerateFormData | (keyof GenerateFormData)[]) => void;
  getResolvedDetailName: () => string;
}

/**
 * 生成ウィザードの全状態を管理するカスタムフック
 * ワークスペースルート・準備状態・認証ロック状態を外部から受け取る
 */
export function useGenerateForm(
  activeWorkspaceRoot: string,
  workspaceUnavailable: boolean,
): UseGenerateFormReturn {
  // ステップ管理
  const [step, setStep] = useState(0);

  // 種別・ティア
  const [kind, setKind] = useState<Kind>('Server');
  const [tier, setTier] = useState<Tier>('System');

  // 配置関連
  const [placement, setPlacement] = useState('');
  const [existingPlacements, setExistingPlacements] = useState<string[]>([]);
  const [isNewPlacement, setIsNewPlacement] = useState(true);

  // 言語・フレームワーク
  const [language, setLanguage] = useState<Language>('Go');
  const [framework, setFramework] = useState<Framework>('React');

  // データベース（Database種別用）
  const [databaseName, setDatabaseName] = useState('main');
  const [databaseEngine, setDatabaseEngine] = useState<Rdbms>('PostgreSQL');

  // サーバーデータベース設定
  const [serverDatabaseMode, setServerDatabaseMode] = useState<ServerDatabaseMode>('none');
  const [availableDatabases, setAvailableDatabases] = useState<ScaffoldDatabaseInfo[]>([]);
  const [selectedDatabasePath, setSelectedDatabasePath] = useState('');
  const [newDatabaseName, setNewDatabaseName] = useState('service-db');
  const [newDatabaseEngine, setNewDatabaseEngine] = useState<Rdbms>('PostgreSQL');

  // BFF設定
  const [generateBff, setGenerateBff] = useState(false);

  // 詳細設定
  const [detail, setDetail] = useState<DetailConfig>({
    name: 'service',
    api_styles: ['Rest'],
    db: null,
    kafka: false,
    redis: false,
    bff_language: null,
  });

  // 生成ステータス
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [availabilityError, setAvailabilityError] = useState<AvailabilityError | null>(null);

  // react-hook-formの初期化
  const {
    clearErrors,
    formState: { errors },
    setError,
    setValue,
    trigger,
  } = useForm<GenerateFormData>({
    resolver: zodResolver(generateSchema),
    defaultValues: {
      placement: '',
      detailName: 'service',
      databaseName: 'main',
      newDatabaseName: 'service-db',
      apiStyles: ['Rest'],
      selectedDatabasePath: '',
      generateBff: false,
      bffLanguage: null,
    },
  });

  // エラーメッセージの集約
  const placementError = errors.placement?.message ?? '';
  const nameError = errors.databaseName?.message ?? errors.detailName?.message ?? '';
  const detailError = errors.apiStyles?.message ?? errors.bffLanguage?.message ?? '';
  const serverDatabaseError =
    errors.newDatabaseName?.message ?? errors.selectedDatabasePath?.message ?? '';

  /** 配置ステップに遷移した際に既存の配置一覧を取得する副作用 */
  useEffect(() => {
    if (step !== 2 || shouldSkipPlacement(tier) || workspaceUnavailable) {
      return;
    }

    let cancelled = false;

    scanPlacements(tier, activeWorkspaceRoot)
      .then((placements) => {
        if (!cancelled) {
          setExistingPlacements(placements);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setExistingPlacements([]);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [activeWorkspaceRoot, step, tier, workspaceUnavailable]);

  /** Server種別の場合に既存データベース一覧を取得する副作用 */
  useEffect(() => {
    if (kind !== 'Server' || workspaceUnavailable) {
      return;
    }

    let cancelled = false;

    scanDatabases(tier, activeWorkspaceRoot)
      .then((databases) => {
        const safeDatabases = Array.isArray(databases) ? databases : [];
        if (!cancelled) {
          setAvailableDatabases(safeDatabases);
          setSelectedDatabasePath((current) => {
            if (current && safeDatabases.some((database) => database.path === current)) {
              setValue('selectedDatabasePath', current);
              return current;
            }
            const nextPath = safeDatabases[0]?.path ?? '';
            setValue('selectedDatabasePath', nextPath);
            return nextPath;
          });
        }
      })
      .catch(() => {
        if (!cancelled) {
          setAvailableDatabases([]);
          setSelectedDatabasePath('');
          setValue('selectedDatabasePath', '');
        }
      });

    return () => {
      cancelled = true;
    };
  }, [activeWorkspaceRoot, kind, setValue, tier, workspaceUnavailable]);

  /** LangFwのユニオン型を構築する */
  function buildLangFw(): LangFw {
    if (kind === 'Client') {
      return { Framework: framework };
    }

    if (kind === 'Database') {
      return { Database: { name: databaseName, rdbms: databaseEngine } };
    }

    return { Language: language };
  }

  /** 解決済みの詳細名を取得する（Serviceティアの場合は配置名を使用） */
  function getResolvedDetailName(): string {
    if (tier === 'Service' && placement) {
      return placement;
    }

    if (detail.name && detail.name.length > 0) {
      return detail.name;
    }

    return getDefaultDetailName(kind);
  }

  /** 選択中の既存データベース情報を取得する */
  function getSelectedExistingDatabase(): ScaffoldDatabaseInfo | null {
    return availableDatabases.find((database) => database.path === selectedDatabasePath) ?? null;
  }

  /** サーバーのデータベース設定を解決する */
  function resolveServerDatabase(): { name: string; rdbms: Rdbms } | null {
    if (serverDatabaseMode === 'none') {
      return null;
    }

    if (serverDatabaseMode === 'existing') {
      const database = getSelectedExistingDatabase();
      return database ? { name: database.name, rdbms: database.rdbms } : null;
    }

    return {
      name: newDatabaseName.trim(),
      rdbms: newDatabaseEngine,
    };
  }

  /** 生成設定オブジェクトを構築する */
  function buildGenerateConfig(): GenerateConfig {
    return {
      kind,
      tier,
      placement: shouldSkipPlacement(tier) ? null : placement || null,
      lang_fw: buildLangFw(),
      detail: {
        ...detail,
        name: getResolvedDetailName(),
        db:
          kind === 'Database'
            ? { name: databaseName, rdbms: databaseEngine }
            : kind === 'Server'
              ? resolveServerDatabase()
              : null,
        bff_language: showBffControls && generateBff ? detail.bff_language : null,
      },
    };
  }

  /** 名前フィールドの共通バリデーション処理 */
  async function validateNameField(
    field: 'placement' | 'detailName' | 'databaseName' | 'newDatabaseName',
    value: string,
    duplicateMessage?: string,
  ): Promise<boolean> {
    const valid = await trigger(field);
    if (!valid) {
      return false;
    }

    if (duplicateMessage) {
      setError(field, { type: 'manual', message: duplicateMessage });
      return false;
    }

    try {
      await validateName(value.trim());
      clearErrors(field);
      return true;
    } catch (error) {
      setError(field, { type: 'manual', message: String(error) });
      return false;
    }
  }

  /** 配置名のバリデーション（重複チェック含む） */
  async function validatePlacementValue(value: string): Promise<boolean> {
    return validateNameField(
      'placement',
      value,
      existingPlacements.includes(value.trim()) ? 'この配置はすでに存在します。' : undefined,
    );
  }

  /** 詳細名のバリデーション */
  async function validateDetailNameValue(value: string): Promise<boolean> {
    return validateNameField('detailName', value);
  }

  /** サーバーデータベース名のバリデーション */
  async function validateServerDatabaseName(value: string): Promise<boolean> {
    return validateNameField('newDatabaseName', value);
  }

  // 表示制御フラグの算出
  const showPlacementStep = !shouldSkipPlacement(tier);
  const showDetailStep = !shouldSkipDetail(kind, tier);
  const showBffControls =
    kind === 'Server' && tier === 'Service' && detail.api_styles.includes('GraphQL');
  const selectedBffLanguage = showBffControls && generateBff ? detail.bff_language : null;
  const currentGenerateConfig = buildGenerateConfig();
  const currentAvailabilityKey = JSON.stringify({
    workspaceRoot: activeWorkspaceRoot,
    config: currentGenerateConfig,
  });
  const availabilityErrorMessage =
    availabilityError?.key === currentAvailabilityKey ? availabilityError.message : '';
  const currentRuntime = currentGenerateConfig.lang_fw;
  const selectedExistingDatabase = getSelectedExistingDatabase();
  const resolvedServerDb = currentGenerateConfig.detail.db;

  /** 利用可否（競合チェック）のバリデーション */
  async function validateAvailability(): Promise<boolean> {
    if (workspaceUnavailable) {
      setAvailabilityError({
        key: currentAvailabilityKey,
        message: 'ファイルを生成する前に有効なワークスペースルートを設定してください。',
      });
      return false;
    }

    try {
      const conflicts = await scanGenerateConflicts(currentGenerateConfig, activeWorkspaceRoot);
      if (conflicts.length === 0) {
        setAvailabilityError((current) =>
          current?.key === currentAvailabilityKey ? null : current,
        );
        return true;
      }

      const visibleConflicts = conflicts
        .slice(0, 3)
        .map((conflict) => toDisplayPath(activeWorkspaceRoot, conflict));
      const suffix = conflicts.length > 3 ? ` 他${conflicts.length - 3}件。` : '。';
      setAvailabilityError({
        key: currentAvailabilityKey,
        message: `競合する生成済みアセットがすでに存在します: ${visibleConflicts.join(', ')}${suffix}`,
      });
      return false;
    } catch (error) {
      setAvailabilityError({
        key: currentAvailabilityKey,
        message: String(error),
      });
      return false;
    }
  }

  /** 次のステップに進む（バリデーション付き） */
  async function goNext(): Promise<void> {
    clearErrors();
    setAvailabilityError((current) => (current?.key === currentAvailabilityKey ? null : current));

    if (step === 2 && isNewPlacement) {
      const ok = await validatePlacementValue(placement);
      if (!ok) {
        return;
      }
    }

    if (step === 3 && kind === 'Database') {
      const ok = await validateNameField('databaseName', databaseName);
      if (!ok) {
        return;
      }

      const available = await validateAvailability();
      if (!available) {
        return;
      }
    }

    if (step === 4) {
      if (tier !== 'Service') {
        const ok = await validateDetailNameValue(getResolvedDetailName());
        if (!ok) {
          return;
        }
      }

      if (kind === 'Server' && detail.api_styles.length === 0) {
        setError('apiStyles', { type: 'manual', message: '1つ以上のAPIスタイルを選択してください。' });
        return;
      }

      if (showBffControls && generateBff && !detail.bff_language) {
        setError('bffLanguage', { type: 'manual', message: BFF_LANGUAGE_VALIDATION_ERROR });
        return;
      }

      if (kind === 'Server' && serverDatabaseMode === 'existing' && !getSelectedExistingDatabase()) {
        setError('selectedDatabasePath', {
          type: 'manual',
          message: '既存のデータベースを選択してください。',
        });
        return;
      }

      if (kind === 'Server' && serverDatabaseMode === 'new') {
        const ok = await validateServerDatabaseName(newDatabaseName);
        if (!ok) {
          return;
        }
      }

      const available = await validateAvailability();
      if (!available) {
        return;
      }
    }

    if (step === 3 && !showDetailStep && kind !== 'Database') {
      const available = await validateAvailability();
      if (!available) {
        return;
      }
    }

    setStep(getNextStep(step, kind, tier));
  }

  /** 前のステップに戻る */
  function goBack(): void {
    setStep(getPreviousStep(step, kind, tier));
  }

  /** APIスタイルのトグル切り替え */
  function toggleApiStyle(style: ApiStyle): void {
    const nextStyles = detail.api_styles.includes(style)
      ? detail.api_styles.filter((value) => value !== style)
      : [...detail.api_styles, style];

    setDetail((current) => ({
      ...current,
      api_styles: nextStyles,
      bff_language: nextStyles.includes('GraphQL') ? current.bff_language : null,
    }));
    setValue('apiStyles', nextStyles);
    clearErrors('apiStyles');

    if (style === 'GraphQL' && detail.api_styles.includes('GraphQL')) {
      setGenerateBff(false);
      setValue('generateBff', false);
      setValue('bffLanguage', null);
      clearErrors('bffLanguage');
    }
  }

  /** 種別変更ハンドラー（ティア・詳細名のリセット含む） */
  function handleKindChange(nextKind: Kind): void {
    const availableTiers = getAvailableTiers(nextKind);
    const nextTier = availableTiers.includes(tier) ? tier : availableTiers[0];

    setKind(nextKind);
    setTier(nextTier);
    setDetail((current) => ({
      ...current,
      name: getDefaultDetailName(nextKind),
      bff_language:
        nextKind === 'Server' && nextTier === 'Service' && current.api_styles.includes('GraphQL')
          ? current.bff_language
          : null,
    }));
    setValue('detailName', getDefaultDetailName(nextKind));
    clearErrors(['detailName', 'apiStyles', 'selectedDatabasePath', 'newDatabaseName', 'bffLanguage']);

    if (nextKind !== 'Server') {
      setServerDatabaseMode('none');
      setGenerateBff(false);
      setValue('generateBff', false);
      setValue('bffLanguage', null);
    }
  }

  /** ティア変更ハンドラー（BFF設定のリセット含む） */
  function handleTierChange(nextTier: Tier): void {
    setTier(nextTier);

    if (kind === 'Server') {
      setDetail((current) => ({
        ...current,
        bff_language:
          nextTier === 'Service' && current.api_styles.includes('GraphQL')
            ? current.bff_language
            : null,
      }));
    }

    if (nextTier !== 'Service') {
      setGenerateBff(false);
      setValue('generateBff', false);
      setValue('bffLanguage', null);
      clearErrors('bffLanguage');
    }
  }

  /** 生成実行ハンドラー */
  async function handleGenerate(): Promise<void> {
    setStatus('loading');
    setErrorMessage('');
    setAvailabilityError((current) => (current?.key === currentAvailabilityKey ? null : current));

    const available = await validateAvailability();
    if (!available) {
      setStatus('error');
      return;
    }

    try {
      await executeGenerateAt(currentGenerateConfig, activeWorkspaceRoot);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return {
    step,
    goNext,
    goBack,

    kind,
    tier,
    handleKindChange,
    handleTierChange,

    placement,
    setPlacement,
    existingPlacements,
    isNewPlacement,
    setIsNewPlacement,
    placementError,
    validatePlacementValue,

    language,
    setLanguage,
    framework,
    setFramework,

    databaseName,
    setDatabaseName,
    databaseEngine,
    setDatabaseEngine,
    nameError,

    serverDatabaseMode,
    setServerDatabaseMode,
    availableDatabases,
    selectedDatabasePath,
    setSelectedDatabasePath,
    newDatabaseName,
    setNewDatabaseName,
    newDatabaseEngine,
    setNewDatabaseEngine,
    serverDatabaseError,

    detail,
    setDetail,
    toggleApiStyle,
    detailError,
    validateDetailName: validateDetailNameValue,
    validateNameField,

    generateBff,
    setGenerateBff,
    showBffControls,
    selectedBffLanguage,

    status,
    errorMessage,
    handleGenerate,

    showPlacementStep,
    showDetailStep,
    availabilityErrorMessage,
    currentGenerateConfig,
    currentRuntime,
    selectedExistingDatabase,
    resolvedServerDatabase: resolvedServerDb,

    setValue,
    clearErrors,
    getResolvedDetailName,
  };
}
