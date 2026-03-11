import { Controller, useForm, useWatch } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import { executeInit, type Tier } from '../lib/tauri-commands';

const initSchema = z.object({
  projectName: z
    .string()
    .min(1, 'Project name is required.')
    .regex(
      /^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/,
      'Use lowercase letters, numbers, and hyphens only.',
    ),
  gitInit: z.boolean(),
  sparseCheckout: z.boolean(),
  tiers: z.array(z.enum(['System', 'Business', 'Service'])).min(1, 'Select at least one tier.'),
});

type InitFormData = z.infer<typeof initSchema>;

export default function InitPage() {
  const auth = useAuth();
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const {
    register,
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<InitFormData>({
    resolver: zodResolver(initSchema),
    defaultValues: {
      projectName: '',
      gitInit: true,
      sparseCheckout: false,
      tiers: ['System', 'Business', 'Service'],
    },
  });

  const sparseCheckout = useWatch({ control, name: 'sparseCheckout' });

  async function onSubmit(data: InitFormData) {
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
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-2xl p-6" data-testid="init-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Bootstrap</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Initialize a k1s0 workspace</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Set the project name and optional sparse checkout tiers before scaffolding modules.
      </p>
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <form onSubmit={handleSubmit(onSubmit)} className="mt-6 space-y-5">
        <div>
          <label htmlFor="projectName" className="block text-sm font-medium text-slate-200/82">
            Project name
          </label>
          <input
            {...register('projectName')}
            id="projectName"
            placeholder="my-project"
            className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
            data-testid="input-project-name"
          />
          {errors.projectName && (
            <p className="mt-2 text-sm text-rose-300" data-testid="error-project-name">
              {errors.projectName.message}
            </p>
          )}
        </div>

        <Controller
          name="gitInit"
          control={control}
          render={({ field }) => (
            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="checkbox"
                checked={field.value}
                onChange={(event) => field.onChange(event.target.checked)}
                data-testid="checkbox-git-init"
              />
              Initialize a Git repository
            </label>
          )}
        />

        <Controller
          name="sparseCheckout"
          control={control}
          render={({ field }) => (
            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="checkbox"
                checked={field.value}
                onChange={(event) => field.onChange(event.target.checked)}
                data-testid="checkbox-sparse"
              />
              Enable sparse checkout
            </label>
          )}
        />

        {sparseCheckout && (
          <div
            className="rounded-2xl border border-white/10 bg-white/5 p-4"
            data-testid="tier-selection"
          >
            <p className="text-sm font-medium text-slate-200/82">Included tiers</p>
            <div className="mt-3 space-y-2">
              {(['System', 'Business', 'Service'] as const).map((tier) => (
                <Controller
                  key={tier}
                  name="tiers"
                  control={control}
                  render={({ field }) => (
                    <label className="flex items-center gap-3 text-sm text-slate-200/82">
                      <input
                        type="checkbox"
                        checked={field.value.includes(tier)}
                        onChange={(event) => {
                          if (event.target.checked) {
                            field.onChange([...new Set([...field.value, tier])]);
                          } else {
                            field.onChange(field.value.filter((value) => value !== tier));
                          }
                        }}
                      />
                      {tier.toLowerCase()}
                    </label>
                  )}
                />
              ))}
            </div>
            {errors.tiers && <p className="mt-2 text-sm text-rose-300">{errors.tiers.message}</p>}
          </div>
        )}

        <button
          type="submit"
          disabled={status === 'loading' || actionsLocked}
          className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
          data-testid="btn-submit"
        >
          {status === 'loading' ? 'Initializing...' : 'Initialize'}
        </button>

        {status === 'success' && (
          <p className="text-sm text-emerald-300" data-testid="success-message">
            Workspace initialization completed.
          </p>
        )}
        {status === 'error' && (
          <p className="text-sm text-rose-300" data-testid="error-message">
            {errorMessage}
          </p>
        )}
      </form>
    </div>
  );
}
