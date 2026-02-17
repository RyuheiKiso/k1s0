import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { executeInit, type Tier } from '../lib/tauri-commands';
import { useState } from 'react';
import * as Checkbox from '@radix-ui/react-checkbox';
import * as Label from '@radix-ui/react-label';

const initSchema = z.object({
  projectName: z.string().min(1, 'プロジェクト名を入力してください').regex(/^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/, '英小文字・ハイフン・数字のみ許可。先頭末尾のハイフンは禁止。'),
  gitInit: z.boolean(),
  sparseCheckout: z.boolean(),
  tiers: z.array(z.enum(['System', 'Business', 'Service'])).min(1, '少なくとも1つのTierを選択してください'),
});

type InitFormData = z.infer<typeof initSchema>;

export default function InitPage() {
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');

  const { register, handleSubmit, formState: { errors }, watch, control } = useForm<InitFormData>({
    resolver: zodResolver(initSchema),
    defaultValues: {
      projectName: '',
      gitInit: true,
      sparseCheckout: false,
      tiers: ['System', 'Business', 'Service'],
    },
  });

  const sparseCheckout = watch('sparseCheckout');

  const onSubmit = async (data: InitFormData) => {
    setStatus('loading');
    setErrorMessage('');
    try {
      await executeInit({
        project_name: data.projectName,
        git_init: data.gitInit,
        sparse_checkout: data.sparseCheckout,
        tiers: data.tiers as Tier[],
      });
      setStatus('success');
    } catch (e) {
      setStatus('error');
      setErrorMessage(String(e));
    }
  };

  return (
    <div className="glass p-6 max-w-lg" data-testid="init-page">
      <h1 className="text-2xl font-bold mb-6 text-white">プロジェクト初期化</h1>
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
        <div>
          <Label.Root htmlFor="projectName" className="block text-sm font-medium mb-1 text-white/70">プロジェクト名</Label.Root>
          <input
            {...register('projectName')}
            id="projectName"
            className="w-full border rounded px-3 py-2"
            placeholder="my-project"
            data-testid="input-project-name"
          />
          {errors.projectName && (
            <p className="text-rose-400 text-sm mt-1" data-testid="error-project-name">{errors.projectName.message}</p>
          )}
        </div>

        <div className="flex items-center gap-2">
          <Controller
            name="gitInit"
            control={control}
            render={({ field: { value, onChange } }) => (
              <Checkbox.Root
                id="gitInit"
                checked={value}
                onCheckedChange={(checked) => onChange(checked === true)}
                data-testid="checkbox-git-init"
                className="w-4 h-4 border rounded flex items-center justify-center"
              >
                <Checkbox.Indicator>
                  <span>✓</span>
                </Checkbox.Indicator>
              </Checkbox.Root>
            )}
          />
          <Label.Root htmlFor="gitInit" className="text-sm text-white/90">Git リポジトリを初期化する</Label.Root>
        </div>

        <div className="flex items-center gap-2">
          <Controller
            name="sparseCheckout"
            control={control}
            render={({ field: { value, onChange } }) => (
              <Checkbox.Root
                id="sparseCheckout"
                checked={value}
                onCheckedChange={(checked) => onChange(checked === true)}
                data-testid="checkbox-sparse"
                className="w-4 h-4 border rounded flex items-center justify-center"
              >
                <Checkbox.Indicator>
                  <span>✓</span>
                </Checkbox.Indicator>
              </Checkbox.Root>
            )}
          />
          <Label.Root htmlFor="sparseCheckout" className="text-sm text-white/90">sparse-checkout を有効にする</Label.Root>
        </div>

        {sparseCheckout && (
          <div data-testid="tier-selection">
            <Label.Root className="block text-sm font-medium mb-1 text-white/70">Tier 選択</Label.Root>
            <Controller
              name="tiers"
              control={control}
              render={({ field: { value, onChange } }) => (
                <>
                  {(['System', 'Business', 'Service'] as const).map((tier) => (
                    <div key={tier} className="flex items-center gap-2">
                      <Checkbox.Root
                        id={`tier-${tier}`}
                        checked={value.includes(tier)}
                        onCheckedChange={(checked) => {
                          if (checked) {
                            onChange([...value, tier]);
                          } else {
                            onChange(value.filter((t) => t !== tier));
                          }
                        }}
                        className="w-4 h-4 border rounded flex items-center justify-center"
                      >
                        <Checkbox.Indicator>
                          <span>✓</span>
                        </Checkbox.Indicator>
                      </Checkbox.Root>
                      <Label.Root htmlFor={`tier-${tier}`} className="text-sm text-white/90">{tier.toLowerCase()}</Label.Root>
                    </div>
                  ))}
                </>
              )}
            />
            {errors.tiers && (
              <p className="text-rose-400 text-sm mt-1">{errors.tiers.message}</p>
            )}
          </div>
        )}

        <button
          type="submit"
          disabled={status === 'loading'}
          className="bg-indigo-500/80 hover:bg-indigo-500 text-white px-5 py-2.5 rounded-xl transition-all duration-200 shadow-lg shadow-indigo-500/20 hover:shadow-indigo-500/30 disabled:opacity-40"
          data-testid="btn-submit"
        >
          {status === 'loading' ? '初期化中...' : '初期化'}
        </button>

        {status === 'success' && (
          <p className="text-emerald-400 mt-3" data-testid="success-message">プロジェクトの初期化が完了しました。</p>
        )}
        {status === 'error' && (
          <p className="text-rose-400 mt-3" data-testid="error-message">{errorMessage}</p>
        )}
      </form>
    </div>
  );
}
