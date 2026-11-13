/**
 * Plain-language guide for the spatial canvas. Opened from the header `?`
 * control. It explains the nodes, the policies, root control, remote/daemon
 * truth, and the first-run path without adding another nav destination.
 */
import { useEffect } from 'react';

const NODES = [
  { name: 'HYPER', tone: 'bg-honey-300', text: 'The planner. It creates the plan, opens two bounded control turns, reviews results, and writes the final answer.' },
  { name: 'SUPER', tone: 'bg-honey-200', text: 'A branch lead. It splits a large goal across workers and combines their results.' },
  { name: 'WORKER', tone: 'bg-panel-alt', text: 'An executor. It edits files, runs approved commands, runs tests, and reports back.' },
  { name: 'SUB', tone: 'bg-panel', text: 'A temporary helper under one worker. Read-only, no spawning, no shell, no writes.' },
  { name: 'TOOL', tone: 'bg-panel-alt', text: 'A tool-call record: what the agent tried, its outcome, and any refusal code.' },
  { name: 'BEE ROUTE', tone: 'bg-honey-100', text: 'A delegation edge. Click a node to inspect it; drag a worker onto another agent to reparent it while a run is active.' },
];

const POLICIES = [
  ['classic', 'Ask before writes and shell. Good default.'],
  ['plan', 'Read and plan only. No workspace changes.'],
  ['standard', 'Workspace writes are allowed; risky actions still ask.'],
  ['trusted', 'Full local automation. Only use on a workspace you trust.'],
];

const STEPS = [
  'Click New session name, pick a policy, and press Create — or use the Default session.',
  'Type a prompt in the composer and press Enter. The Hive AI Assistant sends it to the local server.',
  'Watch the graph appear: Hyper plans, then Super, Worker, and Sub nodes execute.',
  'Approvals appear as a modal. Read the preview, then approve or deny.',
  'Open Agents for todos, jobs, memory, and the roster; Data for runs; History to replay.',
];

export function GuideOverlay({ open, onClose }: { open: boolean; onClose: () => void }) {
  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-ink-900/35 p-4" role="presentation" onMouseDown={onClose}>
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="hivemind-guide-title"
        className="surface-panel max-h-[min(760px,90vh)] w-[min(760px,96vw)] overflow-y-auto rounded-[18px] p-5"
        style={{ boxShadow: 'var(--shadow-e3)' }}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="flex items-start justify-between gap-4">
          <div>
            <div className="type-mono-meta text-[10px] tracking-[0.16em] text-meta">QUICK GUIDE</div>
            <h2 id="hivemind-guide-title" className="font-display text-[26px] leading-tight text-ink-900">
              What am I looking at?
            </h2>
            <p className="mt-1 max-w-[58ch] text-[13px] leading-5 text-ink-500">
              HiveMind is a local multi-agent workspace. A HyperAgent plans; SuperAgents split the
              work; Workers execute; Subagents help read-only. Every side effect is approved or
              refused by name, and every run is recorded in the local database.
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close guide"
            className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-hairline text-ink-700 hover:bg-panel-alt"
          >
            ×
          </button>
        </div>

        <h3 className="mt-5 font-display text-[17px] text-ink-900">The nodes</h3>
        <div className="mt-2 grid gap-2 sm:grid-cols-2">
          {NODES.map((node) => (
            <div key={node.name} className="rounded-[10px] border border-hairline bg-panel-alt/60 p-2.5">
              <div className="flex items-center gap-2">
                <span className={`inline-block h-3 w-3 rounded-[3px] border border-hairline ${node.tone}`} />
                <span className="type-mono-meta text-[10px] tracking-[0.12em] text-ink-800">{node.name}</span>
              </div>
              <p className="mt-1 text-[11.5px] leading-4 text-ink-500">{node.text}</p>
            </div>
          ))}
        </div>

        <h3 className="mt-5 font-display text-[17px] text-ink-900">Permissions</h3>
        <table className="mt-2 w-full border-collapse text-left text-[11.5px]">
          <tbody>
            {POLICIES.map(([name, text]) => (
              <tr key={name} className="border-t border-hairline">
                <th className="w-[92px] py-1.5 pr-2 font-mono text-[10px] uppercase tracking-[0.1em] text-ink-800">{name}</th>
                <td className="py-1.5 text-ink-500">{text}</td>
              </tr>
            ))}
          </tbody>
        </table>

        <h3 className="mt-5 font-display text-[17px] text-ink-900">What the agents can actually see</h3>
        <p className="mt-1 text-[11.5px] leading-4 text-ink-500">
          The agents are text-and-files only. They cannot see your screen, windows, browser, or
          desktop and have no vision or computer-use tool. An empty workspace honestly returns empty
          file lists. They can read logs/diagnostics once a workspace is populated; screenshots must
          be supplied as readable data.
        </p>

        <h3 className="mt-5 font-display text-[17px] text-ink-900">Root control, remotes, daemons</h3>
        <p className="mt-1 text-[11.5px] leading-4 text-ink-500">
          Root-control buttons only fill a prompt. The server decides whether the HyperAgent may
          actually spawn a remote or daemon. Remotes are in-process principals today, not remote
          machines; daemons are enrolled records, not background Windows services. Zero remotes and
          zero daemons is normal.
        </p>

        <h3 className="mt-5 font-display text-[17px] text-ink-900">First run</h3>
        <ol className="mt-1 list-decimal space-y-1 pl-5 text-[11.5px] leading-4 text-ink-500">
          {STEPS.map((step) => <li key={step}>{step}</li>)}
        </ol>

        <div className="mt-5 flex flex-wrap items-center justify-between gap-2 border-t border-hairline pt-3">
          <span className="type-mono-meta text-[10px] text-meta">ESC CLOSES · CTRL+O RECENTRES THE GRAPH</span>
          <button
            type="button"
            onClick={onClose}
            className="type-label h-9 rounded-[var(--radius-sm)] border border-honey-500 bg-honey-300 px-4 font-semibold text-ink-900 hover:bg-honey-400"
          >
            Got it
          </button>
        </div>
      </section>
    </div>
  );
}
