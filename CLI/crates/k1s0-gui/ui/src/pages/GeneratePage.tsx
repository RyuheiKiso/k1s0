/**
 * 生成ページのオーケストレーター
 * 各ステップコンポーネントとカスタムフックを統合し、ウィザード全体のフローを制御する
 */

import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import { STEP_LABELS } from '../lib/generate-wizard';
import { useGenerateForm } from '../lib/useGenerateForm';
import { useWorkspace } from '../lib/workspace';
import StepConfirm from './generate/StepConfirm';
import StepDetail from './generate/StepDetail';
import StepKind from './generate/StepKind';
import StepLangFw from './generate/StepLangFw';
import StepPlacement from './generate/StepPlacement';
import StepTier from './generate/StepTier';

/** 生成ページ本体: ワークスペース・認証状態に基づいてウィザードを表示する */
export default function GeneratePage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  /** カスタムフックから全状態・ハンドラーを取得する */
  const form = useGenerateForm(activeWorkspaceRoot, workspaceUnavailable);

  /** 次へボタン用のラッパー（void化） */
  const handleNext = () => {
    void form.goNext();
  };

  return (
    <div className="glass max-w-5xl p-6 p3-animate-in" data-testid="generate-page">
      {/* ページヘッダー */}
      <p className="p3-eyebrow-reveal text-xs uppercase tracking-[0.24em] text-cyan-100/55">生成</p>
      <h1 className="p3-heading-glitch mt-2 text-3xl font-semibold text-white">ワークスペースアセットの生成</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        GUIはプロセスの作業ディレクトリではなく、選択したワークスペースルートから生成します。
      </p>

      {/* ワークスペース未設定の警告 */}
      {workspaceUnavailable && (
        <p className="p3-warning-flicker mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          ファイルを生成する前に有効なワークスペースルートを設定してください。
        </p>
      )}
      {/* 認証保護の通知 */}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      {/* ステッパー: 現在のステップをハイライト表示 */}
      <div className="mt-6 flex flex-wrap gap-2" data-testid="stepper">
        {STEP_LABELS.map((label, index) => (
          <div
            key={label}
            className={`p3-stagger-in px-3 py-1 text-sm ${
              index === form.step
                ? 'bg-cyan-500/85 text-white'
                : index < form.step
                  ? 'bg-cyan-500/20 text-cyan-100'
                  : 'bg-[rgba(0,200,255,0.06)] text-slate-200/45'
            }`}
            style={{ '--p3-stagger': index } as React.CSSProperties}
          >
            {label}
          </div>
        ))}
      </div>

      {/* Step 0: 種別選択 */}
      {form.step === 0 && (
        <StepKind
          kind={form.kind}
          onKindChange={form.handleKindChange}
          onNext={handleNext}
        />
      )}

      {/* Step 1: ティア選択 */}
      {form.step === 1 && (
        <StepTier
          kind={form.kind}
          tier={form.tier}
          onTierChange={form.handleTierChange}
          onNext={handleNext}
          onBack={form.goBack}
          availabilityErrorMessage={form.availabilityErrorMessage}
        />
      )}

      {/* Step 2: 配置選択（Systemティア以外） */}
      {form.step === 2 && form.showPlacementStep && (
        <StepPlacement
          placement={form.placement}
          onPlacementChange={form.setPlacement}
          existingPlacements={form.existingPlacements}
          isNewPlacement={form.isNewPlacement}
          onIsNewPlacementChange={form.setIsNewPlacement}
          placementError={form.placementError}
          onValidatePlacement={form.validatePlacementValue}
          onNext={handleNext}
          onBack={form.goBack}
          availabilityErrorMessage={form.availabilityErrorMessage}
          onSetFormValue={(value) => form.setValue('placement', value)}
          onClearPlacementError={() => form.clearErrors('placement')}
        />
      )}

      {/* Step 3: 言語/フレームワーク選択 */}
      {form.step === 3 && (
        <StepLangFw
          kind={form.kind}
          language={form.language}
          onLanguageChange={form.setLanguage}
          framework={form.framework}
          onFrameworkChange={form.setFramework}
          databaseName={form.databaseName}
          onDatabaseNameChange={form.setDatabaseName}
          databaseEngine={form.databaseEngine}
          onDatabaseEngineChange={form.setDatabaseEngine}
          nameError={form.nameError}
          onValidateDatabaseName={form.validateNameField}
          onNext={handleNext}
          onBack={form.goBack}
          availabilityErrorMessage={form.availabilityErrorMessage}
          onSetDatabaseNameFormValue={(value) => form.setValue('databaseName', value)}
          onClearDatabaseNameError={() => form.clearErrors('databaseName')}
        />
      )}

      {/* Step 4: 詳細オプション（Database以外、Client+Service以外） */}
      {form.step === 4 && form.showDetailStep && (
        <StepDetail
          kind={form.kind}
          tier={form.tier}
          placement={form.placement}
          detail={form.detail}
          onDetailChange={form.setDetail}
          onToggleApiStyle={form.toggleApiStyle}
          nameError={form.nameError}
          detailError={form.detailError}
          onValidateDetailName={form.validateDetailName}
          serverDatabaseMode={form.serverDatabaseMode}
          onServerDatabaseModeChange={form.setServerDatabaseMode}
          availableDatabases={form.availableDatabases}
          selectedDatabasePath={form.selectedDatabasePath}
          onSelectedDatabasePathChange={form.setSelectedDatabasePath}
          newDatabaseName={form.newDatabaseName}
          onNewDatabaseNameChange={form.setNewDatabaseName}
          newDatabaseEngine={form.newDatabaseEngine}
          onNewDatabaseEngineChange={form.setNewDatabaseEngine}
          serverDatabaseError={form.serverDatabaseError}
          onValidateServerDatabaseName={form.validateDetailName}
          generateBff={form.generateBff}
          onGenerateBffChange={form.setGenerateBff}
          showBffControls={form.showBffControls}
          onNext={handleNext}
          onBack={form.goBack}
          onSetFormValue={(field, value) => form.setValue(field as 'detailName', value as string)}
          onClearErrors={(name) => {
            if (Array.isArray(name)) {
              form.clearErrors(name as ('detailName')[]);
            } else if (name) {
              form.clearErrors(name as 'detailName');
            } else {
              form.clearErrors();
            }
          }}
        />
      )}

      {/* Step 5: 確認・実行 */}
      {form.step === 5 && (
        <StepConfirm
          kind={form.kind}
          tier={form.tier}
          showPlacementStep={form.showPlacementStep}
          placement={form.placement}
          currentRuntime={form.currentRuntime}
          resolvedDetailName={form.getResolvedDetailName()}
          detail={form.detail}
          resolvedServerDatabase={form.resolvedServerDatabase}
          databaseEngine={form.databaseEngine}
          databaseName={form.databaseName}
          serverDatabaseMode={form.serverDatabaseMode}
          selectedExistingDatabase={form.selectedExistingDatabase}
          showBffControls={form.showBffControls}
          generateBff={form.generateBff}
          selectedBffLanguage={form.selectedBffLanguage}
          status={form.status}
          errorMessage={form.errorMessage}
          availabilityErrorMessage={form.availabilityErrorMessage}
          workspaceUnavailable={workspaceUnavailable}
          actionsLocked={actionsLocked}
          onGenerate={() => {
            void form.handleGenerate();
          }}
          onBack={form.goBack}
        />
      )}
    </div>
  );
}
