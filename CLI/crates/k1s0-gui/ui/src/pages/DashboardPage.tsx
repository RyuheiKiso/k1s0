import { Link } from '@tanstack/react-router';

const quickActions = [
  {
    id: 'init',
    to: '/init',
    eyebrow: 'Bootstrap',
    title: 'Start a new workspace',
    description: 'Run project initialization before generating modules or validating config.',
  },
  {
    id: 'generate',
    to: '/generate',
    eyebrow: 'Scaffold',
    title: 'Generate implementation assets',
    description: 'Create services, clients, libraries, or database layers from the GUI flow.',
  },
  {
    id: 'deps',
    to: '/deps',
    eyebrow: 'Architecture',
    title: 'Inspect dependency boundaries',
    description: 'Run the dependency map flow and export Mermaid output from the workspace.',
  },
  {
    id: 'dev',
    to: '/dev',
    eyebrow: 'Operations',
    title: 'Control local development',
    description: 'Start dependencies, inspect state, and collect container logs without leaving the GUI.',
  },
  {
    id: 'migrate',
    to: '/migrate',
    eyebrow: 'Database',
    title: 'Manage database migrations',
    description: 'Create, apply, roll back, and repair migration state for detected services.',
  },
  {
    id: 'template-migrate',
    to: '/template-migrate',
    eyebrow: 'Scaffold',
    title: 'Review template upgrades',
    description:
      'Preview template drift, resolve merge conflicts, and roll back generated modules safely.',
  },
  {
    id: 'config-types',
    to: '/config-types',
    eyebrow: 'Types',
    title: 'Refresh config contracts',
    description: 'Inspect and regenerate shared configuration types for downstream packages.',
  },
  {
    id: 'navigation-types',
    to: '/navigation-types',
    eyebrow: 'Types',
    title: 'Review navigation contracts',
    description: 'Keep navigation definitions aligned with the generated application structure.',
  },
  {
    id: 'event-codegen',
    to: '/event-codegen',
    eyebrow: 'Events',
    title: 'Generate event assets',
    description: 'Preview and generate proto, producer, consumer, and outbox files from events.yaml.',
  },
  {
    id: 'validate',
    to: '/validate',
    eyebrow: 'Quality',
    title: 'Validate the workspace',
    description: 'Run structural validation before build, test, and deployment steps.',
  },
  {
    id: 'build',
    to: '/build',
    eyebrow: 'Delivery',
    title: 'Build release artifacts',
    description: 'Compile the current workspace and prepare distributable outputs.',
  },
  {
    id: 'test',
    to: '/test',
    eyebrow: 'Quality',
    title: 'Run test suites',
    description: 'Execute the available test pipeline once generation and validation are complete.',
  },
  {
    id: 'deploy',
    to: '/deploy',
    eyebrow: 'Delivery',
    title: 'Open deployment workflow',
    description: 'Continue into the deployment flow after the workspace is green.',
  },
];

export default function DashboardPage() {
  return (
    <div className="space-y-6" data-testid="dashboard-page">
      <section className="glass overflow-hidden p-8">
        <div className="mb-4 inline-flex rounded-full border border-cyan-200/20 bg-cyan-200/10 px-3 py-1 text-xs uppercase tracking-[0.3em] text-cyan-100/80">
          Workspace Command Center
        </div>
        <div className="grid gap-6 lg:grid-cols-[1.4fr_0.8fr]">
          <div className="space-y-4">
            <h1 className="max-w-3xl text-4xl font-semibold leading-tight text-white">
              Move from initialization to delivery without hunting through individual tools.
            </h1>
            <p className="max-w-2xl text-base leading-7 text-slate-200/80">
              This dashboard surfaces the core GUI workflows in the order operators actually use
              them: bootstrap, generate, validate, then ship.
            </p>
          </div>
          <div className="grid gap-3 sm:grid-cols-3 lg:grid-cols-1">
            <Metric label="Primary route" value="/" />
            <Metric label="Init route" value="/init" />
            <Metric label="Quick actions" value={String(quickActions.length)} />
          </div>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {quickActions.map((action) => (
          <Link
            key={action.id}
            to={action.to}
            className="glass-subtle group flex min-h-52 flex-col justify-between border border-white/10 p-5 no-underline transition-all duration-200 hover:-translate-y-1 hover:border-cyan-200/30 hover:bg-white/10"
            data-testid={`dashboard-link-${action.id}`}
          >
            <div className="space-y-3">
              <p className="text-xs uppercase tracking-[0.28em] text-cyan-100/60">
                {action.eyebrow}
              </p>
              <h2 className="text-xl font-semibold text-white">{action.title}</h2>
              <p className="text-sm leading-6 text-slate-200/75">{action.description}</p>
            </div>
            <div className="pt-6 text-sm font-medium text-cyan-100/90">Open flow</div>
          </Link>
        ))}
      </section>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="glass-subtle border border-white/10 p-4">
      <p className="text-xs uppercase tracking-[0.24em] text-slate-200/55">{label}</p>
      <p className="mt-3 text-2xl font-semibold text-white">{value}</p>
    </div>
  );
}
