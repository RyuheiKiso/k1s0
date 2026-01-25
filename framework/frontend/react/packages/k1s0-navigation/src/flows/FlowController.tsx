/**
 * FlowController - フロー（ウィザード等）の遷移制御
 */

import {
  createContext,
  useContext,
  useState,
  useCallback,
  useMemo,
  type ReactNode,
} from 'react';
import { useNavigate } from 'react-router-dom';
import { useNavigationContext } from '../router/NavigationProvider';
import type { FlowConfig, FlowNodeConfig, FlowTransitionConfig } from '../schema/types';

/** フォームデータの型 */
export type FlowFormData = Record<string, unknown>;

/** フローコンテキストの値 */
export interface FlowContextValue {
  /** フロー設定 */
  flow: FlowConfig;
  /** 現在のノード */
  currentNode: FlowNodeConfig;
  /** 現在のノードインデックス */
  currentIndex: number;
  /** ノード総数 */
  totalNodes: number;
  /** フォームデータ */
  formData: FlowFormData;
  /** フォームデータを更新 */
  setFormData: (key: string, value: unknown) => void;
  /** 指定イベントで遷移可能か */
  canTransition: (event: string) => boolean;
  /** 指定イベントで遷移 */
  transition: (event: string) => boolean;
  /** フローを完了（リダイレクト） */
  complete: (redirectTo?: string) => void;
  /** フローをキャンセル */
  cancel: () => void;
}

/** フローコンテキスト */
const FlowContext = createContext<FlowContextValue | null>(null);

/** FlowProvider のプロパティ */
export interface FlowProviderProps {
  /** フローID */
  flowId: string;
  /** 初期フォームデータ */
  initialFormData?: FlowFormData;
  /** 子要素 */
  children: ReactNode;
  /** 完了時のコールバック */
  onComplete?: (formData: FlowFormData) => void;
  /** キャンセル時のコールバック */
  onCancel?: () => void;
}

/**
 * FlowProvider コンポーネント
 *
 * フローの状態管理と遷移制御を提供する。
 */
export function FlowProvider({
  flowId,
  initialFormData = {},
  children,
  onComplete,
  onCancel,
}: FlowProviderProps) {
  const navigate = useNavigate();
  const { config, checkRequires, auth } = useNavigationContext();

  // フロー設定を取得
  const flow = useMemo(() => {
    return config.flows?.find((f) => f.id === flowId);
  }, [config.flows, flowId]);

  if (!flow) {
    throw new Error(`Flow "${flowId}" が見つかりません`);
  }

  // フロー実行権限チェック
  if (!checkRequires(flow.requires)) {
    throw new Error(`Flow "${flowId}" を実行する権限がありません`);
  }

  // 現在のノードID
  const [currentNodeId, setCurrentNodeId] = useState(() => {
    const startNode = flow.nodes.find((n) => n.screen_id === flow.start.screen_id);
    return startNode?.node_id ?? flow.nodes[0]?.node_id;
  });

  // フォームデータ
  const [formData, setFormDataState] = useState<FlowFormData>(initialFormData);

  // 現在のノード
  const currentNode = useMemo(() => {
    return flow.nodes.find((n) => n.node_id === currentNodeId) ?? flow.nodes[0];
  }, [flow.nodes, currentNodeId]);

  // 現在のインデックス
  const currentIndex = useMemo(() => {
    return flow.nodes.findIndex((n) => n.node_id === currentNodeId);
  }, [flow.nodes, currentNodeId]);

  // フォームデータ更新
  const setFormData = useCallback((key: string, value: unknown) => {
    setFormDataState((prev) => ({ ...prev, [key]: value }));
  }, []);

  // 遷移可能か判定
  const canTransition = useCallback(
    (event: string): boolean => {
      const transition = flow.transitions.find(
        (t) => t.from === currentNodeId && t.event === event
      );
      if (!transition) return false;

      // 遷移条件チェック
      if (transition.when) {
        // 必要なフォームキーのチェック
        if (transition.when.required_form_keys) {
          const hasAllKeys = transition.when.required_form_keys.every(
            (key) => formData[key] !== undefined && formData[key] !== ''
          );
          if (!hasAllKeys) return false;
        }

        // フラグチェック
        if (transition.when.flags) {
          const hasAllFlags = transition.when.flags.every((f) =>
            auth.flags.includes(f)
          );
          if (!hasAllFlags) return false;
        }
      }

      return true;
    },
    [flow.transitions, currentNodeId, formData, auth.flags]
  );

  // 遷移実行
  const transition = useCallback(
    (event: string): boolean => {
      const trans = flow.transitions.find(
        (t) => t.from === currentNodeId && t.event === event
      );
      if (!trans || !canTransition(event)) return false;

      setCurrentNodeId(trans.to);
      return true;
    },
    [flow.transitions, currentNodeId, canTransition]
  );

  // フロー完了
  const complete = useCallback(
    (redirectTo?: string) => {
      onComplete?.(formData);
      navigate(redirectTo ?? flow.on_error?.redirect_to ?? '/');
    },
    [navigate, flow.on_error, formData, onComplete]
  );

  // フローキャンセル
  const cancel = useCallback(() => {
    onCancel?.();
    navigate(flow.on_error?.redirect_to ?? '/');
  }, [navigate, flow.on_error, onCancel]);

  const value: FlowContextValue = {
    flow,
    currentNode,
    currentIndex,
    totalNodes: flow.nodes.length,
    formData,
    setFormData,
    canTransition,
    transition,
    complete,
    cancel,
  };

  return <FlowContext.Provider value={value}>{children}</FlowContext.Provider>;
}

/**
 * フローコンテキストを取得するフック
 */
export function useFlowContext(): FlowContextValue {
  const context = useContext(FlowContext);
  if (!context) {
    throw new Error('useFlowContext は FlowProvider 内で使用してください');
  }
  return context;
}

/**
 * フロー遷移用のフック
 */
export function useFlowTransition() {
  const { canTransition, transition, complete, cancel } = useFlowContext();

  return {
    /** 次へ遷移 */
    next: () => transition('next'),
    /** 戻る */
    back: () => transition('back'),
    /** 送信 */
    submit: () => transition('submit'),
    /** 次へ遷移可能か */
    canNext: canTransition('next'),
    /** 戻れるか */
    canBack: canTransition('back'),
    /** 送信可能か */
    canSubmit: canTransition('submit'),
    /** 完了 */
    complete,
    /** キャンセル */
    cancel,
  };
}

/**
 * 現在のフロー画面を表示するコンポーネント
 */
export function FlowScreen() {
  const { currentNode } = useFlowContext();
  const { screens } = useNavigationContext();

  const screen = screens.get(currentNode.screen_id);
  if (!screen) {
    return (
      <div style={{ padding: 20, color: 'red' }}>
        Error: Screen "{currentNode.screen_id}" not found
      </div>
    );
  }

  const ScreenComponent = screen.component;
  return <ScreenComponent />;
}
