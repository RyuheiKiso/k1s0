// OnboardingTutorial.tsx — オンボーディングチュートリアル実装
// 設計方針 20_UX 原則: 初回ユーザーへのガイダンス

// React の useState/useCallback と名前空間を含む React をインポートする（JSX 型解決用）
import React, { useState, useCallback } from 'react';

// チュートリアルステップの型を定義する
interface TutorialStep {
  // ステップタイトル
  title: string;
  // ステップの説明
  description: string;
  // 対象要素のセレクタ（オプション）
  targetSelector?: string;
}

// チュートリアルステップ一覧を定義する
const TUTORIAL_STEPS: TutorialStep[] = [
  // ステップ 1: ダッシュボードの説明
  { title: 'ダッシュボード', description: '工場の稼働状況をリアルタイムで確認できます' },
  // ステップ 2: 発注管理の説明
  { title: '発注管理', description: '購買依頼から発注承認まで一元管理します' },
  // ステップ 3: 検査記録の説明
  { title: '検査記録', description: 'オフライン環境でも検査結果を記録できます' },
];

// OnboardingTutorial コンポーネントを定義する（React.JSX.Element で namespace 参照を解決する）
export function OnboardingTutorial({ onComplete }: { onComplete: () => void }): React.JSX.Element | null {
  // 現在のステップ番号を管理する state
  const [currentStep, setCurrentStep] = useState(0);
  // チュートリアルの表示状態を管理する state
  const [isVisible, setIsVisible] = useState(true);

  // 次のステップに進む処理
  const handleNext = useCallback(() => {
    // 最後のステップでない場合は次のステップに移動する
    if (currentStep < TUTORIAL_STEPS.length - 1) {
      // 次のステップに移動する
      setCurrentStep((prev) => prev + 1);
    } else {
      // チュートリアルを非表示にする
      setIsVisible(false);
      // 完了コールバックを呼ぶ
      onComplete();
    }
  }, [currentStep, onComplete]);

  // 表示されていない場合は null を返す
  if (!isVisible) return null;

  // 現在のステップ情報を取得する（noUncheckedIndexedAccess により undefined の可能性があるため early return する）
  const step = TUTORIAL_STEPS[currentStep];
  if (!step) return null;

  return (
    <div role="dialog" aria-label="チュートリアル" className="onboarding-tutorial">
      {/* ステップタイトルを表示する */}
      <h2>{step.title}</h2>
      {/* ステップの説明を表示する */}
      <p>{step.description}</p>
      {/* 次へ / 完了ボタンを表示する */}
      <button onClick={handleNext}>
        {currentStep < TUTORIAL_STEPS.length - 1 ? '次へ' : '完了'}
      </button>
      {/* ステップ進捗を表示する */}
      <span>{currentStep + 1} / {TUTORIAL_STEPS.length}</span>
    </div>
  );
}
