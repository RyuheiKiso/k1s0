import { Link } from '@tanstack/react-router';

const quickActions = [
  {
    id: 'init',
    to: '/init',
    eyebrow: '初期設定',
    title: '新しいワークスペースを作成',
    description: 'モジュールの生成や設定の検証の前にプロジェクトの初期化を実行します。',
  },
  {
    id: 'generate',
    to: '/generate',
    eyebrow: 'スキャフォールド',
    title: '実装アセットを生成',
    description: 'GUIフローからサービス、クライアント、ライブラリ、データベースレイヤーを作成します。',
  },
  {
    id: 'deps',
    to: '/deps',
    eyebrow: 'アーキテクチャ',
    title: '依存関係の境界を検査',
    description: '依存関係マップフローを実行し、ワークスペースからMermaid出力をエクスポートします。',
  },
  {
    id: 'dev',
    to: '/dev',
    eyebrow: 'オペレーション',
    title: 'ローカル開発の制御',
    description: 'GUIを離れることなく依存関係の起動、状態の確認、コンテナログの収集を行います。',
  },
  {
    id: 'migrate',
    to: '/migrate',
    eyebrow: 'データベース',
    title: 'データベースマイグレーション管理',
    description: '検出されたサービスのマイグレーションの作成、適用、ロールバック、修復を行います。',
  },
  {
    id: 'template-migrate',
    to: '/template-migrate',
    eyebrow: 'スキャフォールド',
    title: 'テンプレートのアップグレードを確認',
    description:
      'テンプレートのドリフトをプレビューし、マージコンフリクトを解決して、生成済みモジュールを安全にロールバックします。',
  },
  {
    id: 'config-types',
    to: '/config-types',
    eyebrow: '型定義',
    title: '設定コントラクトを更新',
    description: '下流パッケージ向けの共有設定型を検査・再生成します。',
  },
  {
    id: 'navigation-types',
    to: '/navigation-types',
    eyebrow: '型定義',
    title: 'ナビゲーションコントラクトを確認',
    description: 'ナビゲーション定義を生成されたアプリケーション構造と整合させます。',
  },
  {
    id: 'event-codegen',
    to: '/event-codegen',
    eyebrow: 'イベント',
    title: 'イベントアセットを生成',
    description: 'events.yamlからproto、プロデューサー、コンシューマー、アウトボックスファイルをプレビュー・生成します。',
  },
  {
    id: 'validate',
    to: '/validate',
    eyebrow: '品質',
    title: 'ワークスペースを検証',
    description: 'ビルド、テスト、デプロイの前に構造検証を実行します。',
  },
  {
    id: 'build',
    to: '/build',
    eyebrow: 'デリバリー',
    title: 'リリースアーティファクトをビルド',
    description: '現在のワークスペースをコンパイルし、配布可能な出力を準備します。',
  },
  {
    id: 'test',
    to: '/test',
    eyebrow: '品質',
    title: 'テストスイートを実行',
    description: '生成と検証が完了したら利用可能なテストパイプラインを実行します。',
  },
  {
    id: 'deploy',
    to: '/deploy',
    eyebrow: 'デリバリー',
    title: 'デプロイワークフローを開始',
    description: 'ワークスペースがグリーンになったらデプロイフローに進みます。',
  },
];

export default function DashboardPage() {
  return (
    <div className="p3-animate-in space-y-6" data-testid="dashboard-page">
      <section className="glass overflow-hidden p-8">
        <div className="mb-4 inline-flex border border-cyan-200/20 bg-cyan-200/10 px-3 py-1 text-xs uppercase tracking-[0.3em] text-cyan-100/80 p3-eyebrow-reveal">
          ワークスペースコマンドセンター
        </div>
        <div className="grid gap-6 lg:grid-cols-[1.4fr_0.8fr]">
          <div className="space-y-4">
            <h1 className="max-w-3xl text-4xl font-semibold leading-tight text-white p3-heading-glitch">
              個別のツールを探し回ることなく初期化からデリバリーまで一貫して操作できます。
            </h1>
            <p className="max-w-2xl text-base leading-7 text-slate-200/80">
              このダッシュボードはオペレーターが実際に使用する順序でGUIワークフローを表示します：初期化、生成、検証、そしてデリバリー。
            </p>
          </div>
          <div className="grid gap-3 sm:grid-cols-3 lg:grid-cols-1">
            <Metric label="プライマリルート" value="/" />
            <Metric label="初期化ルート" value="/init" />
            <Metric label="クイックアクション" value={String(quickActions.length)} />
          </div>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {quickActions.map((action, index) => (
          <Link
            key={action.id}
            to={action.to}
            className="glass-subtle group flex min-h-52 flex-col justify-between border border-[rgba(0,200,255,0.12)] p-5 no-underline transition-all duration-200 hover:-translate-y-1 hover:border-cyan-200/30 hover:bg-[rgba(0,200,255,0.08)] p3-card-hover p3-stagger-in"
            data-testid={`dashboard-link-${action.id}`}
            style={{ '--p3-stagger': index } as React.CSSProperties}
          >
            <div className="space-y-3">
              <p className="text-xs uppercase tracking-[0.28em] text-cyan-100/60">
                {action.eyebrow}
              </p>
              <h2 className="text-xl font-semibold text-white">{action.title}</h2>
              <p className="text-sm leading-6 text-slate-200/75">{action.description}</p>
            </div>
            <div className="pt-6 text-sm font-medium text-cyan-100/90">フローを開く</div>
          </Link>
        ))}
      </section>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="glass-subtle border border-[rgba(0,200,255,0.12)] p-4">
      <p className="text-xs uppercase tracking-[0.24em] text-slate-200/55 p3-badge-pulse">{label}</p>
      <p className="mt-3 text-2xl font-semibold text-white p3-metric-flash">{value}</p>
    </div>
  );
}
