import { Controller, useForm, useWatch } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useEffect, useRef, useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import { getCurrentDirectory, executeInitAt, type Tier } from '../lib/tauri-commands';
import { useWorkspace } from '../lib/workspace';

const initSchema = z.object({
  projectName: z
    .string()
    .min(1, 'プロジェクト名は必須です。')
    .regex(
      /^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/,
      '小文字、数字、ハイフンのみ使用できます。',
    ),
  baseDir: z.string().trim().min(1, '親ディレクトリは必須です。'),
  gitInit: z.boolean(),
  sparseCheckout: z.boolean(),
  tiers: z.array(z.enum(['System', 'Business', 'Service'])).min(1, '1つ以上のティアを選択してください。'),
});

type InitFormData = z.infer<typeof initSchema>;

function joinDestination(baseDir: string, projectName: string) {
  if (!baseDir.trim() || !projectName.trim()) {
    return '';
  }

  const trimmedBase = baseDir.trim().replace(/[\\/]+$/, '');
  return `${trimmedBase}/${projectName.trim()}`;
}

export default function InitPage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [createdWorkspaceRoot, setCreatedWorkspaceRoot] = useState('');
  const baseDirEditedRef = useRef(false);
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const {
    register,
    control,
    handleSubmit,
    getValues,
    setValue,
    formState: { errors },
  } = useForm<InitFormData>({
    resolver: zodResolver(initSchema),
    defaultValues: {
      projectName: '',
      baseDir: '',
      gitInit: true,
      sparseCheckout: false,
      tiers: ['System', 'Business', 'Service'],
    },
  });

  const sparseCheckout = useWatch({ control, name: 'sparseCheckout' });
  const projectName = useWatch({ control, name: 'projectName' });
  const baseDir = useWatch({ control, name: 'baseDir' });
  const destinationPreview = joinDestination(baseDir, projectName);
  const baseDirField = register('baseDir');

  useEffect(() => {
    if (getValues('baseDir').trim()) {
      return;
    }

    if (workspace.draftPath.trim()) {
      setValue('baseDir', workspace.draftPath.trim(), { shouldDirty: false });
      return;
    }

    void getCurrentDirectory()
      .then((currentDirectory) => {
        if (!baseDirEditedRef.current && !getValues('baseDir').trim()) {
          setValue('baseDir', currentDirectory, { shouldDirty: false });
        }
      })
      .catch(() => undefined);
  }, [getValues, setValue, workspace.draftPath]);

  async function onSubmit(data: InitFormData) {
    setStatus('loading');
    setErrorMessage('');
    setCreatedWorkspaceRoot('');

    try {
      const workspaceRoot = await executeInitAt(
        {
          project_name: data.projectName,
          git_init: data.gitInit,
          sparse_checkout: data.sparseCheckout,
          tiers: data.tiers as Tier[],
        },
        data.baseDir,
      );

      const adopted = await workspace.adoptWorkspace(workspaceRoot);
      if (!adopted) {
        throw new Error(
          'ワークスペースの初期化は成功しましたが、GUIが新しいワークスペースルートを適用できませんでした。',
        );
      }

      setCreatedWorkspaceRoot(workspaceRoot);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-3xl p-6 p3-animate-in" data-testid="init-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55 p3-eyebrow-reveal">初期設定</p>
      <h1 className="mt-2 text-3xl font-semibold text-white p3-heading-glitch">k1s0ワークスペースの初期化</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        親ディレクトリを明示的に指定することで、生成されるワークスペースがデスクトップアプリのプロセス作業ディレクトリに依存しなくなります。
      </p>
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <form onSubmit={handleSubmit(onSubmit)} className="mt-6 space-y-5">
        <div>
          <label htmlFor="baseDir" className="block text-sm font-medium text-slate-200/82">
            親ディレクトリ
          </label>
          <input
            {...baseDirField}
            id="baseDir"
            placeholder="C:/work/github"
            className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
            data-testid="input-base-dir"
            onFocus={() => {
              baseDirEditedRef.current = true;
            }}
            onChange={(event) => {
              baseDirEditedRef.current = true;
              baseDirField.onChange(event);
            }}
          />
          {errors.baseDir && (
            <p className="mt-2 text-sm text-rose-300" data-testid="error-base-dir">
              {errors.baseDir.message}
            </p>
          )}
        </div>

        <div>
          <label htmlFor="projectName" className="block text-sm font-medium text-slate-200/82">
            プロジェクト名
          </label>
          <input
            {...register('projectName')}
            id="projectName"
            placeholder="my-project"
            className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
            data-testid="input-project-name"
          />
          {errors.projectName && (
            <p className="mt-2 text-sm text-rose-300" data-testid="error-project-name">
              {errors.projectName.message}
            </p>
          )}
        </div>

        <div
          className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-4"
          data-testid="destination-preview"
        >
          <p className="text-xs uppercase tracking-[0.18em] text-slate-200/55 p3-badge-pulse">生成先</p>
          <p className="mt-2 break-all text-sm text-slate-100">
            {destinationPreview || '親ディレクトリとプロジェクト名を入力してください。'}
          </p>
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
              Gitリポジトリを初期化する
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
              スパースチェックアウトを有効にする
            </label>
          )}
        />

        {sparseCheckout && (
          <div
            className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-4 p3-expand-in"
            data-testid="tier-selection"
          >
            <p className="text-sm font-medium text-slate-200/82">含めるティア</p>
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
          className="bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
          data-testid="btn-submit"
        >
          {status === 'loading' ? '初期化中...' : '初期化'}
        </button>

        {status === 'success' && (
          <p className="text-sm text-cyan-300" data-testid="success-message">
            ワークスペースの初期化が完了しました。アクティブなワークスペースルート: {createdWorkspaceRoot}
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
