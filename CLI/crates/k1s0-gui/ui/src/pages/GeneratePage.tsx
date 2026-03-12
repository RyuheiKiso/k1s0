import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { useEffect, useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import {
  executeGenerateAt,
  scanDatabases,
  scanPlacements,
  validateName,
  type ApiStyle,
  type DetailConfig,
  type Framework,
  type Kind,
  type LangFw,
  type Language,
  type Rdbms,
  type ScaffoldDatabaseInfo,
  type Tier,
} from '../lib/tauri-commands';
import { useWorkspace } from '../lib/workspace';

const STEP_LABELS = ['Kind', 'Tier', 'Placement', 'Language', 'Detail', 'Confirm'] as const;
type ServerDatabaseMode = 'none' | 'existing' | 'new';
const API_STYLE_VALUES = ['Rest', 'Grpc', 'GraphQL'] as const;
const BFF_LANGUAGE_VALUES = ['Go', 'Rust'] as const;

const generateSchema = z.object({
  placement: z.string().trim().min(1, 'Placement is required.'),
  detailName: z.string().trim().min(1, 'Name is required.'),
  databaseName: z.string().trim().min(1, 'Name is required.'),
  newDatabaseName: z.string().trim().min(1, 'Database name is required.'),
  apiStyles: z.array(z.enum(API_STYLE_VALUES)),
  selectedDatabasePath: z.string(),
  generateBff: z.boolean(),
  bffLanguage: z.enum(BFF_LANGUAGE_VALUES).nullable(),
});

type GenerateFormData = z.infer<typeof generateSchema>;

function getAvailableTiers(kind: Kind): Tier[] {
  switch (kind) {
    case 'Server':
      return ['System', 'Business', 'Service'];
    case 'Client':
      return ['Business', 'Service'];
    case 'Library':
      return ['System', 'Business'];
    case 'Database':
      return ['System', 'Business', 'Service'];
  }
}

function getLanguageOptions(kind: Kind): Language[] {
  switch (kind) {
    case 'Server':
      return ['Go', 'Rust'];
    case 'Library':
      return ['Go', 'Rust', 'TypeScript', 'Dart'];
    default:
      return ['Go', 'Rust', 'TypeScript', 'Dart'];
  }
}

function shouldSkipPlacement(tier: Tier): boolean {
  return tier === 'System';
}

function shouldSkipDetail(kind: Kind, tier: Tier): boolean {
  return kind === 'Database' || (kind === 'Client' && tier === 'Service');
}

function getNextStep(currentStep: number, kind: Kind, tier: Tier): number {
  let nextStep = currentStep + 1;

  if (nextStep === 2 && shouldSkipPlacement(tier)) {
    nextStep += 1;
  }

  if (nextStep === 4 && shouldSkipDetail(kind, tier)) {
    nextStep += 1;
  }

  return Math.min(nextStep, 5);
}

function getPreviousStep(currentStep: number, kind: Kind, tier: Tier): number {
  let previousStep = currentStep - 1;

  if (previousStep === 4 && shouldSkipDetail(kind, tier)) {
    previousStep -= 1;
  }

  if (previousStep === 2 && shouldSkipPlacement(tier)) {
    previousStep -= 1;
  }

  return Math.max(previousStep, 0);
}

function getDefaultDetailName(kind: Kind): string {
  switch (kind) {
    case 'Client':
      return 'app';
    case 'Library':
      return 'shared';
    case 'Database':
      return 'main';
    case 'Server':
      return 'service';
  }
}

export default function GeneratePage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [step, setStep] = useState(0);
  const [kind, setKind] = useState<Kind>('Server');
  const [tier, setTier] = useState<Tier>('System');
  const [placement, setPlacement] = useState('');
  const [existingPlacements, setExistingPlacements] = useState<string[]>([]);
  const [isNewPlacement, setIsNewPlacement] = useState(true);
  const [language, setLanguage] = useState<Language>('Go');
  const [framework, setFramework] = useState<Framework>('React');
  const [databaseName, setDatabaseName] = useState('main');
  const [databaseEngine, setDatabaseEngine] = useState<Rdbms>('PostgreSQL');
  const [serverDatabaseMode, setServerDatabaseMode] = useState<ServerDatabaseMode>('none');
  const [availableDatabases, setAvailableDatabases] = useState<ScaffoldDatabaseInfo[]>([]);
  const [selectedDatabasePath, setSelectedDatabasePath] = useState('');
  const [newDatabaseName, setNewDatabaseName] = useState('service-db');
  const [newDatabaseEngine, setNewDatabaseEngine] = useState<Rdbms>('PostgreSQL');
  const [generateBff, setGenerateBff] = useState(false);
  const [detail, setDetail] = useState<DetailConfig>({
    name: 'service',
    api_styles: ['Rest'],
    db: null,
    kafka: false,
    redis: false,
    bff_language: null,
  });
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
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

  const placementError = errors.placement?.message ?? '';
  const nameError = errors.databaseName?.message ?? errors.detailName?.message ?? '';
  const detailError = errors.apiStyles?.message ?? errors.bffLanguage?.message ?? '';
  const serverDatabaseError =
    errors.newDatabaseName?.message ?? errors.selectedDatabasePath?.message ?? '';

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

  function buildLangFw(): LangFw {
    if (kind === 'Client') {
      return { Framework: framework };
    }

    if (kind === 'Database') {
      return { Database: { name: databaseName, rdbms: databaseEngine } };
    }

    return { Language: language };
  }

  function getResolvedDetailName(): string {
    if (tier === 'Service' && placement) {
      return placement;
    }

    if (detail.name && detail.name.length > 0) {
      return detail.name;
    }

    return getDefaultDetailName(kind);
  }

  function getSelectedExistingDatabase() {
    return availableDatabases.find((database) => database.path === selectedDatabasePath) ?? null;
  }

  function resolveServerDatabase() {
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

  async function validatePlacementValue(value: string): Promise<boolean> {
    return validateNameField(
      'placement',
      value,
      existingPlacements.includes(value.trim()) ? 'This placement already exists.' : undefined,
    );
  }

  async function validateDetailName(value: string): Promise<boolean> {
    return validateNameField('detailName', value);
  }

  async function validateServerDatabaseName(value: string): Promise<boolean> {
    return validateNameField('newDatabaseName', value);
  }

  async function goNext() {
    clearErrors();

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
    }

    if (step === 4) {
      if (tier !== 'Service') {
        const ok = await validateDetailName(getResolvedDetailName());
        if (!ok) {
          return;
        }
      }

      if (kind === 'Server' && detail.api_styles.length === 0) {
        setError('apiStyles', { type: 'manual', message: 'Select at least one API style.' });
        return;
      }

      if (showBffControls && generateBff && !detail.bff_language) {
        setError('bffLanguage', { type: 'manual', message: 'Select a BFF language.' });
        return;
      }

      if (kind === 'Server' && serverDatabaseMode === 'existing' && !getSelectedExistingDatabase()) {
        setError('selectedDatabasePath', {
          type: 'manual',
          message: 'Select an existing database.',
        });
        return;
      }

      if (kind === 'Server' && serverDatabaseMode === 'new') {
        const ok = await validateServerDatabaseName(newDatabaseName);
        if (!ok) {
          return;
        }
      }
    }

    setStep(getNextStep(step, kind, tier));
  }

  function goBack() {
    setStep(getPreviousStep(step, kind, tier));
  }

  function toggleApiStyle(style: ApiStyle) {
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

  function handleKindChange(nextKind: Kind) {
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

  function handleTierChange(nextTier: Tier) {
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

  async function handleGenerate() {
    setStatus('loading');
    setErrorMessage('');

    try {
      await executeGenerateAt(
        {
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
        },
        activeWorkspaceRoot,
      );
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  const showPlacementStep = !shouldSkipPlacement(tier);
  const showDetailStep = !shouldSkipDetail(kind, tier);
  const showBffControls =
    kind === 'Server' && tier === 'Service' && detail.api_styles.includes('GraphQL');
  const selectedBffLanguage = showBffControls && generateBff ? detail.bff_language : null;
  const currentRuntime = buildLangFw();
  const selectedExistingDatabase = getSelectedExistingDatabase();
  const resolvedServerDatabase = resolveServerDatabase();

  return (
    <div className="glass max-w-5xl p-6" data-testid="generate-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Scaffold</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Generate workspace assets</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        The GUI generates from the selected workspace root instead of the process working
        directory.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
          Configure a valid workspace root before generating files.
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 flex flex-wrap gap-2" data-testid="stepper">
        {STEP_LABELS.map((label, index) => (
          <div
            key={label}
            className={`rounded-full px-3 py-1 text-sm ${
              index === step
                ? 'bg-emerald-500/85 text-white'
                : index < step
                  ? 'bg-emerald-500/20 text-emerald-100'
                  : 'bg-white/8 text-slate-200/45'
            }`}
          >
            {label}
          </div>
        ))}
      </div>

      {step === 0 && (
        <section
          className="mt-6 rounded-2xl border border-white/10 bg-white/5 p-5"
          data-testid="step-kind"
        >
          <h2 className="text-lg font-semibold text-white">Choose a module kind</h2>
          <div className="mt-4 grid gap-3 sm:grid-cols-2">
            {(['Server', 'Client', 'Library', 'Database'] as Kind[]).map((value) => (
              <label
                key={value}
                className="flex items-center gap-3 rounded-xl border border-white/8 bg-slate-950/20 px-4 py-3 text-sm text-slate-100"
              >
                <input
                  type="radio"
                  checked={kind === value}
                  onChange={() => handleKindChange(value)}
                  name="generate-kind"
                />
                {value}
              </label>
            ))}
          </div>
          <button
            type="button"
            onClick={() => {
              void goNext();
            }}
            className="mt-5 rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500"
            data-testid="btn-next"
          >
            Next
          </button>
        </section>
      )}

      {step === 1 && (
        <section
          className="mt-6 rounded-2xl border border-white/10 bg-white/5 p-5"
          data-testid="step-tier"
        >
          <h2 className="text-lg font-semibold text-white">Choose a tier</h2>
          <div className="mt-4 space-y-2">
            {getAvailableTiers(kind).map((value) => (
              <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={tier === value}
                  onChange={() => handleTierChange(value)}
                  name="generate-tier"
                />
                {value.toLowerCase()}
              </label>
            ))}
          </div>
          <div className="mt-5 flex gap-3">
            <button
              type="button"
              onClick={goBack}
              className="rounded-xl border border-white/15 bg-white/6 px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10"
              data-testid="btn-back"
            >
              Back
            </button>
            <button
              type="button"
              onClick={() => {
                void goNext();
              }}
              className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500"
              data-testid="btn-next"
            >
              Next
            </button>
          </div>
        </section>
      )}

      {step === 2 && showPlacementStep && (
        <section
          className="mt-6 rounded-2xl border border-white/10 bg-white/5 p-5"
          data-testid="step-placement"
        >
          <h2 className="text-lg font-semibold text-white">Choose a placement</h2>

          {existingPlacements.length > 0 && (
            <div className="mt-4">
              <label className="block text-sm font-medium text-slate-200/82">
                Existing placement
              </label>
              <select
                className="mt-2 w-full rounded-xl border border-white/15 bg-slate-950/35 px-3 py-2 text-white"
                value={isNewPlacement ? '__new__' : placement}
                onChange={(event) => {
                  if (event.target.value === '__new__') {
                    setIsNewPlacement(true);
                    setPlacement('');
                    setValue('placement', '');
                  } else {
                    setIsNewPlacement(false);
                    setPlacement(event.target.value);
                    setValue('placement', event.target.value);
                    clearErrors('placement');
                  }
                }}
                data-testid="select-placement"
              >
                <option value="__new__">Create new placement</option>
                {existingPlacements.map((value) => (
                  <option key={value} value={value}>
                    {value}
                  </option>
                ))}
              </select>
            </div>
          )}

          {(isNewPlacement || existingPlacements.length === 0) && (
            <div className="mt-4">
              <label className="block text-sm font-medium text-slate-200/82">Placement name</label>
              <input
                value={placement}
                onChange={(event) => {
                  setPlacement(event.target.value);
                  setValue('placement', event.target.value);
                  clearErrors('placement');
                }}
                onBlur={() => {
                  void validatePlacementValue(placement);
                }}
                placeholder="placement-name"
                className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
                data-testid={
                  existingPlacements.length > 0 ? 'input-new-placement' : 'input-placement'
                }
              />
              {placementError && (
                <p className="mt-2 text-sm text-rose-300" data-testid="error-placement">
                  {placementError}
                </p>
              )}
            </div>
          )}

          <div className="mt-5 flex gap-3">
            <button
              type="button"
              onClick={goBack}
              className="rounded-xl border border-white/15 bg-white/6 px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10"
              data-testid="btn-back"
            >
              Back
            </button>
            <button
              type="button"
              onClick={() => {
                void goNext();
              }}
              className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500"
              data-testid="btn-next"
            >
              Next
            </button>
          </div>
        </section>
      )}

      {step === 3 && (
        <section
          className="mt-6 rounded-2xl border border-white/10 bg-white/5 p-5"
          data-testid="step-langfw"
        >
          <h2 className="text-lg font-semibold text-white">Language or framework</h2>

          {kind === 'Client' && (
            <div className="mt-4 space-y-2">
              {(['React', 'Flutter'] as Framework[]).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="radio"
                    checked={framework === value}
                    onChange={() => setFramework(value)}
                    name="client-framework"
                  />
                  {value}
                </label>
              ))}
            </div>
          )}

          {kind === 'Database' && (
            <div className="mt-4 space-y-5">
              <div>
                <label className="block text-sm font-medium text-slate-200/82">Database name</label>
                <input
                  value={databaseName}
                  onChange={(event) => {
                    setDatabaseName(event.target.value);
                    setValue('databaseName', event.target.value);
                    clearErrors('databaseName');
                  }}
                  onBlur={() => {
                    void validateNameField('databaseName', databaseName);
                  }}
                  placeholder="main"
                  className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
                  data-testid="input-db-name"
                />
                {nameError && (
                  <p className="mt-2 text-sm text-rose-300" data-testid="error-name">
                    {nameError}
                  </p>
                )}
              </div>
              <div className="space-y-2">
                {(['PostgreSQL', 'MySQL', 'SQLite'] as Rdbms[]).map((value) => (
                  <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                    <input
                      type="radio"
                      checked={databaseEngine === value}
                      onChange={() => setDatabaseEngine(value)}
                      name="database-engine"
                    />
                    {value}
                  </label>
                ))}
              </div>
            </div>
          )}

          {kind !== 'Client' && kind !== 'Database' && (
            <div className="mt-4 space-y-2">
              {getLanguageOptions(kind).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="radio"
                    checked={language === value}
                    onChange={() => setLanguage(value)}
                    name="module-language"
                  />
                  {value}
                </label>
              ))}
            </div>
          )}

          <div className="mt-5 flex gap-3">
            <button
              type="button"
              onClick={goBack}
              className="rounded-xl border border-white/15 bg-white/6 px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10"
              data-testid="btn-back"
            >
              Back
            </button>
            <button
              type="button"
              onClick={() => {
                void goNext();
              }}
              className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500"
              data-testid="btn-next"
            >
              Next
            </button>
          </div>
        </section>
      )}

      {step === 4 && showDetailStep && (
        <section
          className="mt-6 rounded-2xl border border-white/10 bg-white/5 p-5"
          data-testid="step-detail"
        >
          <h2 className="text-lg font-semibold text-white">Detail options</h2>

          {tier !== 'Service' && (
            <div className="mt-4">
              <label className="block text-sm font-medium text-slate-200/82">Module name</label>
              <input
                value={detail.name ?? ''}
                onChange={(event) => {
                  setDetail((current) => ({
                    ...current,
                    name: event.target.value,
                  }));
                  setValue('detailName', event.target.value);
                  clearErrors('detailName');
                }}
                onBlur={() => {
                  void validateDetailName(detail.name ?? '');
                }}
                placeholder={getDefaultDetailName(kind)}
                className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
                data-testid="input-name"
              />
              {nameError && (
                <p className="mt-2 text-sm text-rose-300" data-testid="error-name">
                  {nameError}
                </p>
              )}
            </div>
          )}

          {tier === 'Service' && (
            <div className="mt-4 rounded-2xl border border-white/10 bg-slate-950/20 p-4 text-sm text-slate-200/82">
              Service name is derived from the placement: <strong>{placement || 'not set'}</strong>
            </div>
          )}

          {kind === 'Server' && (
            <>
              <div className="mt-5">
                <p className="text-sm font-medium text-slate-200/82">API styles</p>
                <div className="mt-3 space-y-2">
                  {(['Rest', 'Grpc', 'GraphQL'] as ApiStyle[]).map((value) => (
                    <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                      <input
                        type="checkbox"
                        checked={detail.api_styles.includes(value)}
                        onChange={() => toggleApiStyle(value)}
                      />
                      {value}
                    </label>
                  ))}
                </div>
              </div>

              <div className="mt-5">
                <p className="text-sm font-medium text-slate-200/82">Database</p>
                <div className="mt-3 space-y-3">
                  {(['none', 'existing', 'new'] as ServerDatabaseMode[]).map((value) => (
                    <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                        <input
                          type="radio"
                          checked={serverDatabaseMode === value}
                          onChange={() => {
                            setServerDatabaseMode(value);
                            clearErrors(['selectedDatabasePath', 'newDatabaseName']);
                          }}
                          name="server-database-mode"
                        />
                      {value === 'none'
                        ? 'No database'
                        : value === 'existing'
                          ? 'Use existing database'
                          : 'Create new database'}
                    </label>
                  ))}
                </div>

                {serverDatabaseMode === 'existing' && (
                  <div className="mt-4">
                    {availableDatabases.length === 0 ? (
                      <p className="text-sm text-slate-200/55">
                        No existing databases were found for this tier.
                      </p>
                    ) : (
                      <>
                        <label className="block text-sm font-medium text-slate-200/82">
                          Existing database
                        </label>
                        <select
                          value={selectedDatabasePath}
                          onChange={(event) => {
                            setSelectedDatabasePath(event.target.value);
                            setValue('selectedDatabasePath', event.target.value);
                            clearErrors('selectedDatabasePath');
                          }}
                          className="mt-2 w-full rounded-xl border border-white/15 bg-slate-950/35 px-3 py-2 text-white"
                          data-testid="select-server-db"
                        >
                          {availableDatabases.map((database) => (
                            <option key={database.path} value={database.path}>
                              {database.name} ({database.rdbms})
                            </option>
                          ))}
                        </select>
                      </>
                    )}
                  </div>
                )}

                {serverDatabaseMode === 'new' && (
                  <div className="mt-4 space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-slate-200/82">
                        Database name
                      </label>
                      <input
                        value={newDatabaseName}
                        onChange={(event) => {
                          setNewDatabaseName(event.target.value);
                          setValue('newDatabaseName', event.target.value);
                          clearErrors('newDatabaseName');
                        }}
                        onBlur={() => {
                          void validateServerDatabaseName(newDatabaseName);
                        }}
                        placeholder="service-db"
                        className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
                        data-testid="input-server-db-name"
                      />
                    </div>
                    <div className="space-y-2">
                      {(['PostgreSQL', 'MySQL', 'SQLite'] as Rdbms[]).map((value) => (
                        <label
                          key={value}
                          className="flex items-center gap-3 text-sm text-slate-200/82"
                        >
                          <input
                            type="radio"
                            checked={newDatabaseEngine === value}
                            onChange={() => setNewDatabaseEngine(value)}
                            name="server-database-engine"
                          />
                          {value}
                        </label>
                      ))}
                    </div>
                  </div>
                )}

                {serverDatabaseError && (
                  <p className="mt-3 text-sm text-rose-300">{serverDatabaseError}</p>
                )}
              </div>

              <div className="mt-5 space-y-2">
                <label className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="checkbox"
                    checked={detail.kafka}
                    onChange={(event) =>
                      setDetail((current) => ({
                        ...current,
                        kafka: event.target.checked,
                      }))
                    }
                  />
                  Enable Kafka integration
                </label>
                <label className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="checkbox"
                    checked={detail.redis}
                    onChange={(event) =>
                      setDetail((current) => ({
                        ...current,
                        redis: event.target.checked,
                      }))
                    }
                  />
                  Enable Redis integration
                </label>
              </div>

              {showBffControls && (
                <div className="mt-5">
                  <p className="text-sm font-medium text-slate-200/82">
                    Generate GraphQL BFF
                  </p>
                  <div className="mt-3 space-y-2">
                    {[
                      { label: 'Yes', enabled: true },
                      { label: 'No', enabled: false },
                    ].map(({ label, enabled }) => (
                      <label key={label} className="flex items-center gap-3 text-sm text-slate-200/82">
                        <input
                          type="radio"
                          checked={generateBff === enabled}
                          onChange={() => {
                            setGenerateBff(enabled);
                            setValue('generateBff', enabled);
                            if (!enabled) {
                              setDetail((current) => ({
                                ...current,
                                bff_language: null,
                              }));
                              setValue('bffLanguage', null);
                              clearErrors('bffLanguage');
                            }
                          }}
                          name="generate-bff"
                        />
                        {label}
                      </label>
                    ))}
                  </div>

                  {generateBff && (
                    <div className="mt-4">
                      <p className="text-sm font-medium text-slate-200/82">BFF language</p>
                      <div className="mt-3 space-y-2">
                        {BFF_LANGUAGE_VALUES.map((value) => (
                          <label
                            key={value}
                            className="flex items-center gap-3 text-sm text-slate-200/82"
                          >
                            <input
                              type="radio"
                              checked={detail.bff_language === value}
                              onChange={() => {
                                setDetail((current) => ({
                                  ...current,
                                  bff_language: value,
                                }));
                                setValue('bffLanguage', value);
                                clearErrors('bffLanguage');
                              }}
                              name="bff-language"
                            />
                            {value}
                          </label>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </>
          )}

          {detailError && (
            <p className="mt-4 text-sm text-rose-300" data-testid="detail-error">
              {detailError}
            </p>
          )}

          <div className="mt-5 flex gap-3">
            <button
              type="button"
              onClick={goBack}
              className="rounded-xl border border-white/15 bg-white/6 px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10"
              data-testid="btn-back"
            >
              Back
            </button>
            <button
              type="button"
              onClick={() => {
                void goNext();
              }}
              className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500"
              data-testid="btn-next"
            >
              Next
            </button>
          </div>
        </section>
      )}

      {step === 5 && (
        <section
          className="mt-6 rounded-2xl border border-white/10 bg-white/5 p-5"
          data-testid="step-confirm"
        >
          <h2 className="text-lg font-semibold text-white">Confirm generation</h2>
          <div className="mt-4 space-y-3 text-sm text-slate-200/82">
            <p>Kind: {kind}</p>
            <p>Tier: {tier}</p>
            <p>Placement: {showPlacementStep ? placement || 'not set' : 'not required'}</p>
            <p>
              Runtime:{' '}
              {'Framework' in currentRuntime
                ? currentRuntime.Framework
                : 'Database' in currentRuntime
                  ? `${currentRuntime.Database.rdbms} (${currentRuntime.Database.name})`
                  : currentRuntime.Language}
            </p>
            <p>Name: {getResolvedDetailName()}</p>
            {kind === 'Server' && (
              <>
                <p>API styles: {detail.api_styles.length > 0 ? detail.api_styles.join(', ') : 'none'}</p>
                <p>
                  Database:{' '}
                  {resolvedServerDatabase
                    ? `${resolvedServerDatabase.name} (${resolvedServerDatabase.rdbms})`
                    : 'none'}
                </p>
                <p>Kafka: {detail.kafka ? 'enabled' : 'disabled'}</p>
                <p>Redis: {detail.redis ? 'enabled' : 'disabled'}</p>
                <p>Generate BFF: {showBffControls ? (generateBff ? 'yes' : 'no') : 'not available'}</p>
                <p>BFF language: {selectedBffLanguage ?? 'not required'}</p>
              </>
            )}
            {kind === 'Database' && (
              <p>
                RDBMS: {databaseEngine} ({databaseName})
              </p>
            )}
            {serverDatabaseMode === 'existing' && selectedExistingDatabase && (
              <p className="text-slate-300/60">
                Existing DB path: {selectedExistingDatabase.path}
              </p>
            )}
          </div>

          <div className="mt-5 flex gap-3">
            <button
              type="button"
              onClick={goBack}
              className="rounded-xl border border-white/15 bg-white/6 px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10"
              data-testid="btn-back"
            >
              Back
            </button>
            <button
              type="button"
              onClick={() => {
                void handleGenerate();
              }}
              disabled={status === 'loading' || workspaceUnavailable || actionsLocked}
              className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
              data-testid="btn-generate"
            >
              {status === 'loading' ? 'Generating...' : 'Generate'}
            </button>
          </div>

          {status === 'success' && (
            <p className="mt-4 text-sm text-emerald-300" data-testid="success-message">
              Generation completed successfully.
            </p>
          )}
          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>
      )}
    </div>
  );
}
