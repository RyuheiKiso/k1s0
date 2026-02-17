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
    <div className="max-w-lg" data-testid="init-page">
      <h1 className="text-2xl font-bold mb-6">プロジェクト初期化</h1>
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
        <div>
          <Label.Root htmlFor="projectName" className="block text-sm font-medium mb-1">プロジェクト名</Label.Root>
          <input
            {...register('projectName')}
            id="projectName"
            className="w-full border rounded px-3 py-2"
            placeholder="my-project"
            data-testid="input-project-name"
          />
          {errors.projectName && (
            <p className="text-red-500 text-sm mt-1" data-testid="error-project-name">{errors.projectName.message}</p>
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
          <Label.Root htmlFor="gitInit" className="text-sm">Git リポジトリを初期化する</Label.Root>
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
          <Label.Root htmlFor="sparseCheckout" className="text-sm">sparse-checkout を有効にする</Label.Root>
        </div>

        {sparseCheckout && (
          <div data-testid="tier-selection">
            <Label.Root className="block text-sm font-medium mb-1">Tier 選択</Label.Root>
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
                      <Label.Root htmlFor={`tier-${tier}`} className="text-sm">{tier.toLowerCase()}</Label.Root>
                    </div>
                  ))}
                </>
              )}
            />
            {errors.tiers && (
              <p className="text-red-500 text-sm mt-1">{errors.tiers.message}</p>
            )}
          </div>
        )}

        <button
          type="submit"
          disabled={status === 'loading'}
          className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50"
          data-testid="btn-submit"
        >
          {status === 'loading' ? '初期化中...' : '初期化'}
        </button>

        {status === 'success' && (
          <p className="text-green-600" data-testid="success-message">プロジェクトの初期化が完了しました。</p>
        )}
        {status === 'error' && (
          <p className="text-red-500" data-testid="error-message">{errorMessage}</p>
        )}
      </form>
    </div>
  );
}
