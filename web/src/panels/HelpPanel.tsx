/** In-app tutorials and plain-English explanations. No dead controls. */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';

const TUTORIALS = [
  {
    title: 'First run',
    steps: [
      'In the bottom dock, open SESSION & PERMISSIONS and choose a policy.',
      'Type a small task in the composer; press Enter.',
      'Watch the graph: HYPER plans, SUPER splits, WORKER executes.',
      'Approve or deny any side effect in the inline card.',
    ],
  },
  {
    title: 'The dock',
    steps: [
      'Drag the top grip up or down to resize the chat.',
      'CONTROLS opens spawn / re-route / cancel / kill.',
      'STATE & SETTINGS shows provider, model, key, reasoning, and version.',
      'HIDE removes the dock; the header Chat button brings it back.',
    ],
  },
  {
    title: 'Safety',
    steps: [
      'classic asks before side effects; plan refuses writes.',
      'standard allows workspace writes; trusted removes prompts.',
      'Never use trusted on a workspace you do not trust.',
      'Every refusal has a name; the run ledger keeps it.',
    ],
  },
];

const FEATURES = [
  ['Workspace', 'The live agent graph and the resizable chat dock.'],
  ['Agents', 'Roster, jobs, todos, memory, lessons, and proposals.'],
  ['Data', 'Runs, revisions, evidence, and snapshots.'],
  ['Models', 'Model catalogue, usage, cache counters, and balance.'],
  ['Routing', 'Which model or strategy handles a task.'],
  ['Remote', 'Remote principals and daemons today are local records, not network machines.'],
  ['Settings', 'Provider profile, key status, diagnostics, and FigCraft gallery.'],
  ['FigCraft', 'Design boards and the design gallery; the live UI is coded from tokens.'],
  ['Screen access', 'Agents cannot see your desktop, browser, or screen. They can read workspace files and report diagnostics; a screenshot must be supplied as readable text/data.'],
  ['Workspace', 'The agent sees only the configured workspace root. If it is empty, file tools honestly return empty results.'],
];

export function HelpPanel() {
  const setNav = useUiStore((state) => state.setNav);
  return (
    <div className="h-full overflow-y-auto p-5">
      <div className="mx-auto flex max-w-[980px] items-center gap-4">
        <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-[18px] border border-honey-500 bg-honey-100">
          <svg width="36" height="36" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 2.6 20.1 7.3v9.4L12 21.4 3.9 16.7V7.3Z" fill="none" stroke="var(--color-ink-800)" strokeWidth="1.4" />
            <path d="M9.8 9.3a2.4 2.4 0 1 1 3.4 2.2c-.7.3-1.1.8-1.1 1.5" fill="none" stroke="var(--color-ink-800)" strokeWidth="1.6" strokeLinecap="round" />
            <circle cx="12" cy="16.3" r="1" fill="var(--color-ink-800)" />
          </svg>
        </div>
        <div>
          <div className="type-mono-meta text-[10px] tracking-[0.16em] text-meta">HELP & TUTORIALS</div>
          <h1 className="font-display text-[30px] leading-tight text-ink-900">What do you want to do?</h1>
          <p className="mt-1 max-w-[64ch] text-[13px] leading-5 text-ink-500">
            HiveMind is a local multi-agent workspace. The HyperAgent plans; SuperAgents split the
            work; Workers execute; Subagents help read-only. Every risky action is approved or refused
            by name, and every run is recorded locally.
          </p>
        </div>
      </div>

      <div className="mx-auto mt-5 grid max-w-[980px] gap-3 lg:grid-cols-3">
        {TUTORIALS.map((tutorial) => (
          <section key={tutorial.title} className="rounded-[14px] border border-hairline bg-panel p-4 transition duration-200 hover:-translate-y-0.5 hover:shadow-md">
            <h2 className="font-display text-[19px] text-ink-900">{tutorial.title}</h2>
            <ol className="mt-2 list-decimal space-y-1.5 pl-5 text-[12px] leading-[18px] text-ink-600">
              {tutorial.steps.map((step) => <li key={step}>{step}</li>)}
            </ol>
          </section>
        ))}
      </div>

      <div className="mx-auto mt-3 grid max-w-[980px] gap-2 sm:grid-cols-2">
        {FEATURES.map(([name, text]) => (
          <div key={name} className="rounded-[12px] border border-hairline bg-panel-alt/60 p-3 transition duration-200 hover:-translate-y-0.5 hover:border-honey-400 hover:bg-honey-100/40">
            <div className="type-mono-meta text-[10px] tracking-[0.12em] text-ink-800">{name}</div>
            <p className="mt-1 text-[12px] leading-[17px] text-ink-500">{text}</p>
          </div>
        ))}
      </div>

      <div className="mx-auto mt-4 flex max-w-[980px] flex-wrap gap-2">
        <button type="button" className="type-label h-9 rounded-[var(--radius-sm)] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400" onClick={() => setNav('workspace')}>
          Go to Workspace
        </button>
        <button type="button" className="type-label h-9 rounded-[var(--radius-sm)] border border-hairline bg-panel px-3 text-ink-800 hover:bg-panel-alt" onClick={() => setNav('settings')}>
          Provider & key settings
        </button>
        <button type="button" className="type-label h-9 rounded-[var(--radius-sm)] border border-hairline bg-panel px-3 text-ink-800 hover:bg-panel-alt" onClick={() => setNav('agents')}>
          Agent roster & jobs
        </button>
      </div>

      <p className="mx-auto mt-4 max-w-[980px] type-mono-meta text-[10px] text-meta">
        {STRINGS.brand.wordmark} · press ? in the header for the quick canvas guide.
      </p>
    </div>
  );
}
