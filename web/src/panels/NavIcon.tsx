/** Minimal line icons for the 72px rail; no icon dependency. */
import type { NavSection } from '../state/store';

export function NavIcon({ id, active }: { readonly id: NavSection; readonly active: boolean }) {
  const stroke = active ? 'var(--color-honey-300)' : 'var(--color-ink-700)';
  const common = { fill: 'none', stroke, strokeWidth: 1.6, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const };
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" aria-hidden="true">
      {id === 'workspace' ? (
        <path {...common} d="M12 2.8 20 7.2v9.6L12 21.2 4 16.8V7.2Z M12 7.4l4 2.3v4.6l-4 2.3-4-2.3V9.7Z" />
      ) : null}
      {id === 'agents' ? (
        <path {...common} d="M6 16h12l-1.2-2.2V10a4.8 4.8 0 0 0-9.6 0v3.8Z M10 19a2 2 0 0 0 4 0" />
      ) : null}
      {id === 'data' ? (
        <path {...common} d="m12 3 8 4.5-8 4.5-8-4.5Z M4 12l8 4.5 8-4.5 M4 16.5 12 21l8-4.5" />
      ) : null}
      {id === 'models' ? (
        <path {...common} d="m12 3 8 4.5v9L12 21l-8-4.5v-9Z M4 7.5l8 4.5 8-4.5 M12 12v9" />
      ) : null}
      {id === 'routing' ? (
        <path {...common} d="M6 4v5a3 3 0 0 0 3 3h6a3 3 0 0 1 3 3v5 M6 20v-3 M18 4v3 M4 2l2 2-2 2 M16 18l2 2 2-2" />
      ) : null}
      {id === 'remote' ? (
        <path {...common} d="M12 4v16 M5 17a7 7 0 0 1 14 0 M3 20h18 M8 7a5 5 0 0 1 8 0" />
      ) : null}
      {id === 'settings' ? (
        <path {...common} d="M12 8.2A3.8 3.8 0 1 0 12 15.8 3.8 3.8 0 0 0 12 8.2Z M12 2.8v2.4 M12 18.8v2.4 M4.5 4.5l1.7 1.7 M17.8 17.8l1.7 1.7 M2.8 12h2.4 M18.8 12h2.4 M4.5 19.5l1.7-1.7 M17.8 6.2l1.7-1.7" />
      ) : null}
      {id === 'help' ? (
        <>
          <path {...common} d="M12 2.6 20.1 7.3v9.4L12 21.4 3.9 16.7V7.3Z" />
          <path {...common} d="M9.6 9.2a2.5 2.5 0 1 1 3.5 2.3c-.7.3-1.1.8-1.1 1.6" />
          <path {...common} d="M12 16.6h.01" />
        </>
      ) : null}
    </svg>
  );
}
