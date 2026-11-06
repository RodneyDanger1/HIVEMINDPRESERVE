/** DEV-ONLY state-pack gallery (15 states). Never rendered in production builds. */
import { AGENT_STATES } from '../state/agentState';
import { StateGlyph } from '../flow/nodes';
import { visualFor } from '../state/agentState';
import { STRINGS } from '../state/strings';

export function DevGallery() {
  if (!import.meta.env.DEV) {
    return null;
  }
  return (
    <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3" aria-label={STRINGS.gallery.title}>
      <div className="type-label text-ink-500">{STRINGS.gallery.title}</div>
      <p className="type-mono-meta mt-1 text-meta">{STRINGS.gallery.note}</p>
      <ul className="mt-2 grid grid-cols-2 gap-1 sm:grid-cols-3">
        {AGENT_STATES.map((state) => (
          <li key={state} className="rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-2 py-1">
            <StateGlyph state={state} />
            <div className="type-mono-meta mt-1 text-meta">{visualFor(state).shape}</div>
          </li>
        ))}
      </ul>
    </section>
  );
}
