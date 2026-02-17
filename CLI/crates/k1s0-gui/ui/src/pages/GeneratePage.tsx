import { useState, useEffect } from 'react';
import {
  executeGenerate,
  scanPlacements,
  validateName,
  type Kind,
  type Tier,
  type LangFw,
  type DetailConfig,
  type ApiStyle,
  type Rdbms,
  type Language,
} from '../lib/tauri-commands';
import * as RadioGroup from '@radix-ui/react-radio-group';
import * as Checkbox from '@radix-ui/react-checkbox';
import * as Select from '@radix-ui/react-select';

const STEPS = ['種別', 'Tier', '配置先', '言語/FW', '詳細設定', '確認'] as const;

// CLIフロー準拠: 種別に応じて選択可能なTierを制限
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

// Step skipping logic per CLIフロー
function shouldSkipDetail(kind: Kind, tier: Tier): boolean {
  // Database: detail入力はstep4で完了済み
  if (kind === 'Database') return true;
  // Client + Service: アプリ名はステップ3のサービス名を使用するためスキップ
  if (kind === 'Client' && tier === 'Service') return true;
  return false;
}

function getNextStep(currentStep: number, kind: Kind, tier: Tier): number {
  if (currentStep === 1 && tier === 'System') return 3; // skip placement
  if (currentStep === 3 && shouldSkipDetail(kind, tier)) return 5; // skip detail
  return currentStep + 1;
}

function getPrevStep(currentStep: number, kind: Kind, tier: Tier): number {
  if (currentStep === 3 && tier === 'System') return 1; // skip placement
  if (currentStep === 5 && shouldSkipDetail(kind, tier)) return 3; // skip detail
  return currentStep - 1;
}

// Language options per kind (CLIフロー準拠)
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

export default function GeneratePage() {
  const [step, setStep] = useState(0);
  const [kind, setKind] = useState<Kind>('Server');
  const [tier, setTier] = useState<Tier>('System');
  const [placement, setPlacement] = useState('');
  const [langFw, setLangFw] = useState<LangFw>({ Language: 'Go' });
  const [detail, setDetail] = useState<DetailConfig>({
    name: null,
    api_styles: [],
    db: null,
    kafka: false,
    redis: false,
    bff_language: null,
  });
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');

  // Placement state
  const [existingPlacements, setExistingPlacements] = useState<string[]>([]);
  const [isNewPlacement, setIsNewPlacement] = useState(true);

  // Database kind step 4 state
  const [dbName, setDbName] = useState('');

  // BFF state
  const [bffEnabled, setBffEnabled] = useState(false);

  // Validation state
  const [nameError, setNameError] = useState('');
  const [placementError, setPlacementError] = useState('');

  // Load existing placements when entering step 2 (placement) or when tier changes
  useEffect(() => {
    if (step === 2 && tier !== 'System') {
      scanPlacements(tier, '.').then(setExistingPlacements).catch(() => setExistingPlacements([]));
    }
  }, [step, tier]);

  // Auto-set name for Service tier when entering step 4 (detail)
  useEffect(() => {
    if (step === 4 && tier === 'Service') {
      setDetail((prev) => ({ ...prev, name: placement }));
    }
  }, [step, tier, placement]);

  const handleValidateName = async (name: string, setError: (msg: string) => void) => {
    if (!name) {
      setError('');
      return;
    }
    try {
      await validateName(name);
      setError('');
    } catch (e) {
      setError(String(e));
    }
  };

  const handleValidatePlacement = async (name: string) => {
    if (!name) {
      setPlacementError('');
      return;
    }
    if (existingPlacements.includes(name)) {
      setPlacementError('同名の配置先が既に存在します。');
      return;
    }
    try {
      await validateName(name);
      setPlacementError('');
    } catch (e) {
      setPlacementError(String(e));
    }
  };

  const handleGenerate = async () => {
    setStatus('loading');
    try {
      await executeGenerate({
        kind,
        tier,
        placement: placement || null,
        lang_fw: langFw,
        detail,
      });
      setStatus('success');
    } catch (e) {
      setStatus('error');
      setErrorMessage(String(e));
    }
  };

  const toggleApiStyle = (style: ApiStyle) => {
    setDetail((prev) => {
      const newStyles = prev.api_styles.includes(style)
        ? prev.api_styles.filter((s) => s !== style)
        : [...prev.api_styles, style];
      // Clear BFF when GraphQL is unchecked
      const shouldClearBff = style === 'GraphQL' && prev.api_styles.includes('GraphQL');
      return {
        ...prev,
        api_styles: newStyles,
        ...(shouldClearBff ? { bff_language: null } : {}),
      };
    });
    if (style === 'GraphQL' && detail.api_styles.includes('GraphQL')) {
      setBffEnabled(false);
    }
  };

  // 種別変更時、選択不可のTierをリセット
  const handleKindChange = (newKind: Kind) => {
    setKind(newKind);
    const available = getAvailableTiers(newKind);
    if (!available.includes(tier)) {
      setTier(available[0]);
    }
  };

  const goNext = () => setStep(getNextStep(step, kind, tier));
  const goPrev = () => setStep(getPrevStep(step, kind, tier));

  // Format LangFw for display
  const formatLangFw = (): { label: string; value: string } => {
    if ('Language' in langFw) return { label: '言語', value: langFw.Language };
    if ('Framework' in langFw) return { label: 'フレームワーク', value: langFw.Framework };
    if ('Database' in langFw) return { label: 'RDBMS', value: langFw.Database.rdbms };
    return { label: '言語/FW', value: '' };
  };

  // Show BFF option: only when kind=Server, tier=Service, and api_styles includes GraphQL
  const showBffOption = kind === 'Server' && tier === 'Service' && detail.api_styles.includes('GraphQL');

  return (
    <div className="max-w-lg" data-testid="generate-page">
      <h1 className="text-2xl font-bold mb-6">ひな形生成</h1>

      {/* Stepper */}
      <div className="flex gap-2 mb-6" data-testid="stepper">
        {STEPS.map((label, i) => (
          <div
            key={label}
            className={`px-3 py-1 rounded text-sm ${
              i === step ? 'bg-blue-600 text-white' : i < step ? 'bg-blue-100 text-blue-800' : 'bg-gray-200'
            }`}
          >
            {label}
          </div>
        ))}
      </div>

      {/* Step 0: Kind */}
      {step === 0 && (
        <div data-testid="step-kind">
          <h2 className="font-semibold mb-2">何を生成しますか？</h2>
          <RadioGroup.Root value={kind} onValueChange={(v) => handleKindChange(v as Kind)}>
            {(['Server', 'Client', 'Library', 'Database'] as Kind[]).map((k) => (
              <div key={k} className="flex items-center gap-2 mb-1">
                <RadioGroup.Item value={k} id={`kind-${k}`} className="w-4 h-4 border rounded-full flex items-center justify-center">
                  <RadioGroup.Indicator className="w-2 h-2 bg-blue-600 rounded-full" />
                </RadioGroup.Item>
                <label htmlFor={`kind-${k}`}>{k}</label>
              </div>
            ))}
          </RadioGroup.Root>
          <button onClick={() => setStep(1)} className="mt-4 bg-blue-600 text-white px-4 py-2 rounded" data-testid="btn-next">次へ</button>
        </div>
      )}

      {/* Step 1: Tier */}
      {step === 1 && (
        <div data-testid="step-tier">
          <h2 className="font-semibold mb-2">Tier を選択してください</h2>
          <RadioGroup.Root value={tier} onValueChange={(v) => setTier(v as Tier)}>
            {getAvailableTiers(kind).map((t) => (
              <div key={t} className="flex items-center gap-2 mb-1">
                <RadioGroup.Item value={t} id={`tier-${t}`} className="w-4 h-4 border rounded-full flex items-center justify-center">
                  <RadioGroup.Indicator className="w-2 h-2 bg-blue-600 rounded-full" />
                </RadioGroup.Item>
                <label htmlFor={`tier-${t}`}>{t.toLowerCase()}</label>
              </div>
            ))}
          </RadioGroup.Root>
          <div className="flex gap-2 mt-4">
            <button onClick={() => setStep(0)} className="bg-gray-300 px-4 py-2 rounded" data-testid="btn-back">戻る</button>
            <button onClick={goNext} className="bg-blue-600 text-white px-4 py-2 rounded" data-testid="btn-next">次へ</button>
          </div>
        </div>
      )}

      {/* Step 2: Placement */}
      {step === 2 && (
        <div data-testid="step-placement">
          <h2 className="font-semibold mb-2">
            {tier === 'Business' ? '領域名を入力または選択してください' : 'サービス名を入力または選択してください'}
          </h2>

          {existingPlacements.length > 0 && (
            <div className="mb-3">
              <Select.Root
                value={isNewPlacement ? '__new__' : placement}
                onValueChange={(v) => {
                  if (v === '__new__') {
                    setIsNewPlacement(true);
                    setPlacement('');
                  } else {
                    setIsNewPlacement(false);
                    setPlacement(v);
                  }
                }}
              >
                <Select.Trigger className="w-full border rounded px-3 py-2 flex items-center justify-between" data-testid="select-placement">
                  <Select.Value placeholder="選択してください" />
                  <Select.Icon className="ml-2">▼</Select.Icon>
                </Select.Trigger>
                <Select.Portal>
                  <Select.Content className="bg-white border rounded shadow-lg">
                    <Select.Viewport className="p-1">
                      <Select.Item value="__new__" className="px-3 py-2 cursor-pointer hover:bg-gray-100 rounded">
                        <Select.ItemText>(新規作成)</Select.ItemText>
                      </Select.Item>
                      {existingPlacements.map((p) => (
                        <Select.Item key={p} value={p} className="px-3 py-2 cursor-pointer hover:bg-gray-100 rounded">
                          <Select.ItemText>{p}</Select.ItemText>
                        </Select.Item>
                      ))}
                    </Select.Viewport>
                  </Select.Content>
                </Select.Portal>
              </Select.Root>
            </div>
          )}

          {(isNewPlacement || existingPlacements.length === 0) && (
            <>
              <input
                value={placement}
                onChange={(e) => setPlacement(e.target.value)}
                onBlur={() => handleValidatePlacement(placement)}
                className="w-full border rounded px-3 py-2"
                placeholder={tier === 'Business' ? '領域名を入力' : 'サービス名を入力'}
                data-testid={existingPlacements.length > 0 ? 'input-new-placement' : 'input-placement'}
              />
              {placementError && (
                <p className="text-red-500 text-sm mt-1" data-testid="error-placement">{placementError}</p>
              )}
            </>
          )}

          <div className="flex gap-2 mt-4">
            <button onClick={() => setStep(1)} className="bg-gray-300 px-4 py-2 rounded" data-testid="btn-back">戻る</button>
            <button
              onClick={goNext}
              disabled={placementError !== '' || (isNewPlacement && !placement)}
              className="bg-blue-600 text-white px-4 py-2 rounded disabled:opacity-50"
              data-testid="btn-next"
            >次へ</button>
          </div>
        </div>
      )}

      {/* Step 3: Language/Framework */}
      {step === 3 && (
        <div data-testid="step-langfw">
          <h2 className="font-semibold mb-2">言語 / フレームワーク</h2>
          {kind === 'Client' ? (
            <RadioGroup.Root onValueChange={(v) => setLangFw({ Framework: v as 'React' | 'Flutter' })}>
              {(['React', 'Flutter'] as const).map((fw) => (
                <div key={fw} className="flex items-center gap-2 mb-1">
                  <RadioGroup.Item value={fw} className="w-4 h-4 border rounded-full flex items-center justify-center">
                    <RadioGroup.Indicator className="w-2 h-2 bg-blue-600 rounded-full" />
                  </RadioGroup.Item>
                  <label>{fw}</label>
                </div>
              ))}
            </RadioGroup.Root>
          ) : kind === 'Database' ? (
            <>
              <div className="mb-3">
                <label className="block text-sm font-medium mb-1">データベース名</label>
                <input
                  value={dbName}
                  onChange={(e) => {
                    setDbName(e.target.value);
                    // Update langFw with current rdbms selection
                    if ('Database' in langFw) {
                      setLangFw({ Database: { name: e.target.value, rdbms: langFw.Database.rdbms } });
                    }
                  }}
                  onBlur={() => handleValidateName(dbName, setNameError)}
                  className="w-full border rounded px-3 py-2"
                  placeholder="データベース名を入力"
                  data-testid="input-db-name"
                />
                {nameError && (
                  <p className="text-red-500 text-sm mt-1" data-testid="error-name">{nameError}</p>
                )}
              </div>
              <label className="block text-sm font-medium mb-1">RDBMS</label>
              <RadioGroup.Root onValueChange={(v) => setLangFw({ Database: { name: dbName, rdbms: v as Rdbms } })}>
                {(['PostgreSQL', 'MySQL', 'SQLite'] as Rdbms[]).map((db) => (
                  <div key={db} className="flex items-center gap-2 mb-1">
                    <RadioGroup.Item value={db} className="w-4 h-4 border rounded-full flex items-center justify-center">
                      <RadioGroup.Indicator className="w-2 h-2 bg-blue-600 rounded-full" />
                    </RadioGroup.Item>
                    <label>{db}</label>
                  </div>
                ))}
              </RadioGroup.Root>
            </>
          ) : (
            <RadioGroup.Root onValueChange={(v) => setLangFw({ Language: v as Language })}>
              {getLanguageOptions(kind).map((lang) => (
                <div key={lang} className="flex items-center gap-2 mb-1">
                  <RadioGroup.Item value={lang} className="w-4 h-4 border rounded-full flex items-center justify-center">
                    <RadioGroup.Indicator className="w-2 h-2 bg-blue-600 rounded-full" />
                  </RadioGroup.Item>
                  <label>{lang}</label>
                </div>
              ))}
            </RadioGroup.Root>
          )}
          <div className="flex gap-2 mt-4">
            <button onClick={goPrev} className="bg-gray-300 px-4 py-2 rounded" data-testid="btn-back">戻る</button>
            <button onClick={goNext} className="bg-blue-600 text-white px-4 py-2 rounded" data-testid="btn-next">次へ</button>
          </div>
        </div>
      )}

      {/* Step 4: Detail */}
      {step === 4 && (
        <div data-testid="step-detail">
          <h2 className="font-semibold mb-2">詳細設定</h2>

          {/* Server detail */}
          {kind === 'Server' && (
            <>
              {/* Name input: skip if Service tier (auto-set from placement) */}
              {tier !== 'Service' && (
                <div className="mb-3">
                  <label className="block text-sm font-medium mb-1">サービス名</label>
                  <input
                    value={detail.name ?? ''}
                    onChange={(e) => setDetail({ ...detail, name: e.target.value || null })}
                    onBlur={() => handleValidateName(detail.name ?? '', setNameError)}
                    className="w-full border rounded px-3 py-2"
                    placeholder="サービス名"
                    data-testid="input-name"
                  />
                  {nameError && (
                    <p className="text-red-500 text-sm mt-1" data-testid="error-name">{nameError}</p>
                  )}
                </div>
              )}

              <div className="mb-3">
                <label className="block text-sm font-medium mb-1">API 方式</label>
                {(['Rest', 'Grpc', 'GraphQL'] as ApiStyle[]).map((style) => (
                  <div key={style} className="flex items-center gap-2 mb-1">
                    <Checkbox.Root
                      checked={detail.api_styles.includes(style)}
                      onCheckedChange={() => toggleApiStyle(style)}
                      data-testid={`checkbox-api-${style.toLowerCase()}`}
                      className="w-4 h-4 border rounded flex items-center justify-center"
                    >
                      <Checkbox.Indicator>
                        <span>✓</span>
                      </Checkbox.Indicator>
                    </Checkbox.Root>
                    <span className="text-sm">{style}</span>
                  </div>
                ))}
              </div>

              <div className="mb-3">
                <label className="block text-sm font-medium mb-1">データベース</label>
                <Select.Root
                  value={detail.db?.rdbms ?? 'none'}
                  onValueChange={(v) => {
                    if (v !== 'none') {
                      setDetail({
                        ...detail,
                        db: { name: detail.name ?? 'db', rdbms: v as Rdbms },
                      });
                    } else {
                      setDetail({ ...detail, db: null });
                    }
                  }}
                >
                  <Select.Trigger className="w-full border rounded px-3 py-2 flex items-center justify-between" data-testid="select-db">
                    <Select.Value placeholder="なし" />
                    <Select.Icon className="ml-2">▼</Select.Icon>
                  </Select.Trigger>
                  <Select.Portal>
                    <Select.Content className="bg-white border rounded shadow-lg">
                      <Select.Viewport className="p-1">
                        <Select.Item value="none" className="px-3 py-2 cursor-pointer hover:bg-gray-100 rounded">
                          <Select.ItemText>なし</Select.ItemText>
                        </Select.Item>
                        <Select.Item value="PostgreSQL" className="px-3 py-2 cursor-pointer hover:bg-gray-100 rounded">
                          <Select.ItemText>PostgreSQL</Select.ItemText>
                        </Select.Item>
                        <Select.Item value="MySQL" className="px-3 py-2 cursor-pointer hover:bg-gray-100 rounded">
                          <Select.ItemText>MySQL</Select.ItemText>
                        </Select.Item>
                        <Select.Item value="SQLite" className="px-3 py-2 cursor-pointer hover:bg-gray-100 rounded">
                          <Select.ItemText>SQLite</Select.ItemText>
                        </Select.Item>
                      </Select.Viewport>
                    </Select.Content>
                  </Select.Portal>
                </Select.Root>
              </div>

              <div className="mb-3 space-y-1">
                <div className="flex items-center gap-2">
                  <Checkbox.Root
                    checked={detail.kafka}
                    onCheckedChange={(checked) => setDetail({ ...detail, kafka: checked === true })}
                    data-testid="checkbox-kafka"
                    className="w-4 h-4 border rounded flex items-center justify-center"
                  >
                    <Checkbox.Indicator>
                      <span>✓</span>
                    </Checkbox.Indicator>
                  </Checkbox.Root>
                  <span className="text-sm">Kafka</span>
                </div>
                <div className="flex items-center gap-2">
                  <Checkbox.Root
                    checked={detail.redis}
                    onCheckedChange={(checked) => setDetail({ ...detail, redis: checked === true })}
                    data-testid="checkbox-redis"
                    className="w-4 h-4 border rounded flex items-center justify-center"
                  >
                    <Checkbox.Indicator>
                      <span>✓</span>
                    </Checkbox.Indicator>
                  </Checkbox.Root>
                  <span className="text-sm">Redis</span>
                </div>
              </div>

              {/* BFF option: Service tier + GraphQL selected */}
              {showBffOption && (
                <div className="mb-3">
                  <div className="flex items-center gap-2 mb-2">
                    <Checkbox.Root
                      checked={bffEnabled}
                      onCheckedChange={(checked) => {
                        const enabled = checked === true;
                        setBffEnabled(enabled);
                        if (!enabled) {
                          setDetail({ ...detail, bff_language: null });
                        }
                      }}
                      data-testid="checkbox-bff"
                      className="w-4 h-4 border rounded flex items-center justify-center"
                    >
                      <Checkbox.Indicator>
                        <span>✓</span>
                      </Checkbox.Indicator>
                    </Checkbox.Root>
                    <span className="text-sm">GraphQL BFF を生成する</span>
                  </div>
                  {bffEnabled && (
                    <div className="ml-6">
                      <label className="block text-sm font-medium mb-1">BFF 言語</label>
                      <RadioGroup.Root
                        value={detail.bff_language ?? undefined}
                        onValueChange={(v) => setDetail({ ...detail, bff_language: v as Language })}
                        data-testid="radio-bff-lang"
                      >
                        {(['Go', 'Rust'] as Language[]).map((lang) => (
                          <div key={lang} className="flex items-center gap-2 mb-1">
                            <RadioGroup.Item value={lang} className="w-4 h-4 border rounded-full flex items-center justify-center">
                              <RadioGroup.Indicator className="w-2 h-2 bg-blue-600 rounded-full" />
                            </RadioGroup.Item>
                            <label>{lang}</label>
                          </div>
                        ))}
                      </RadioGroup.Root>
                    </div>
                  )}
                </div>
              )}
            </>
          )}

          {/* Client detail */}
          {kind === 'Client' && tier !== 'Service' && (
            <div className="mb-3">
              <label className="block text-sm font-medium mb-1">アプリ名</label>
              <input
                value={detail.name ?? ''}
                onChange={(e) => setDetail({ ...detail, name: e.target.value || null })}
                onBlur={() => handleValidateName(detail.name ?? '', setNameError)}
                className="w-full border rounded px-3 py-2"
                placeholder="アプリ名"
                data-testid="input-name"
              />
              {nameError && (
                <p className="text-red-500 text-sm mt-1" data-testid="error-name">{nameError}</p>
              )}
            </div>
          )}

          {/* Library detail */}
          {kind === 'Library' && (
            <div className="mb-3">
              <label className="block text-sm font-medium mb-1">ライブラリ名</label>
              <input
                value={detail.name ?? ''}
                onChange={(e) => setDetail({ ...detail, name: e.target.value || null })}
                onBlur={() => handleValidateName(detail.name ?? '', setNameError)}
                className="w-full border rounded px-3 py-2"
                placeholder="ライブラリ名"
                data-testid="input-name"
              />
              {nameError && (
                <p className="text-red-500 text-sm mt-1" data-testid="error-name">{nameError}</p>
              )}
            </div>
          )}

          <div className="flex gap-2 mt-4">
            <button onClick={goPrev} className="bg-gray-300 px-4 py-2 rounded" data-testid="btn-back">戻る</button>
            <button onClick={() => setStep(5)} className="bg-blue-600 text-white px-4 py-2 rounded" data-testid="btn-next">次へ</button>
          </div>
        </div>
      )}

      {/* Step 5: Confirm */}
      {step === 5 && (
        <div data-testid="step-confirm">
          <h2 className="font-semibold mb-2">確認</h2>
          <div className="bg-white border rounded p-4 mb-4 space-y-1 text-sm">
            <p><strong>種別:</strong> {kind}</p>
            <p><strong>Tier:</strong> {tier.toLowerCase()}</p>

            {/* Placement display: Business shows 領域, Service shows サービス */}
            {tier === 'Business' && placement && <p><strong>領域:</strong> {placement}</p>}
            {tier === 'Service' && placement && <p><strong>サービス:</strong> {placement}</p>}

            {/* Kind-specific confirm details */}
            {kind === 'Server' && (
              <>
                <p><strong>サービス名:</strong> {detail.name ?? ''}</p>
                <p><strong>{formatLangFw().label}:</strong> {formatLangFw().value}</p>
                {detail.api_styles.length > 0 && (
                  <p><strong>API:</strong> {detail.api_styles.join(', ')}</p>
                )}
                {showBffOption && detail.bff_language && (
                  <p><strong>BFF:</strong> あり ({detail.bff_language})</p>
                )}
                <p><strong>DB:</strong> {detail.db ? `${detail.db.name} (${detail.db.rdbms})` : 'なし'}</p>
                <p><strong>Kafka:</strong> {detail.kafka ? '有効' : '無効'}</p>
                <p><strong>Redis:</strong> {detail.redis ? '有効' : '無効'}</p>
              </>
            )}

            {kind === 'Client' && (
              <>
                <p><strong>{formatLangFw().label}:</strong> {formatLangFw().value}</p>
                <p><strong>アプリ名:</strong> {detail.name ?? ''}</p>
              </>
            )}

            {kind === 'Library' && (
              <>
                <p><strong>{formatLangFw().label}:</strong> {formatLangFw().value}</p>
                <p><strong>ライブラリ名:</strong> {detail.name ?? ''}</p>
              </>
            )}

            {kind === 'Database' && 'Database' in langFw && (
              <>
                <p><strong>データベース名:</strong> {langFw.Database.name}</p>
                <p><strong>RDBMS:</strong> {langFw.Database.rdbms}</p>
              </>
            )}
          </div>
          <div className="flex gap-2">
            <button onClick={goPrev} className="bg-gray-300 px-4 py-2 rounded" data-testid="btn-back">戻る</button>
            <button
              onClick={handleGenerate}
              disabled={status === 'loading'}
              className="bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700 disabled:opacity-50"
              data-testid="btn-generate"
            >
              {status === 'loading' ? '生成中...' : '生成'}
            </button>
          </div>
          {status === 'success' && <p className="text-green-600 mt-2" data-testid="success-message">ひな形の生成が完了しました。</p>}
          {status === 'error' && <p className="text-red-500 mt-2" data-testid="error-message">{errorMessage}</p>}
        </div>
      )}
    </div>
  );
}
