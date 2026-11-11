/**
 * CAP-4 workbench — command palette.
 *
 * HoneyGlass is used only here, because the palette is a transient overlay. The
 * action list is caller-owned; running an action invokes its callback and closes
 * the palette. The component never invents commands, results or success state.
 *
 * Keyboard: ArrowUp/Down/Home/End move, Enter runs, Escape closes, Tab is
 * trapped inside the dialog. Disabled commands stay visible with their reason.
 */
import { useEffect, useId, useMemo, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';

export type PaletteStatus = 'ready' | 'loading' | 'error';

export interface PaletteAction {
  readonly id: string;
  readonly label: string;
  readonly description?: string | null;
  readonly group?: string | null;
  readonly keywords?: readonly string[];
  readonly shortcut?: string | null;
  readonly disabled?: boolean;
  readonly disabledReason?: string | null;
  readonly requiresApproval?: boolean;
  readonly onRun: () => void;
}

export interface CommandPaletteProps {
  readonly open: boolean;
  readonly actions: readonly PaletteAction[];
  readonly onClose: () => void;
  readonly status?: PaletteStatus;
  readonly errorMessage?: string | null;
  readonly onRetry?: () => void;
  readonly placeholder?: string;
  readonly emptyMessage?: string | null;
  readonly className?: string;
}

export const PALETTE_COPY = {
  title: 'Command palette',
  search: 'Search commands',
  placeholder: 'Search commands…',
  empty: 'No matching commands.',
  loading: 'Loading commands…',
  error: 'Commands unavailable',
  retry: 'Retry',
  close: 'Close',
  disabled: 'Unavailable',
  approval: 'approval',
  run: 'Run',
  results: 'results',
  navigateHint: '↑↓ navigate · Enter run · Esc close',
  escapeHint: 'Esc close',
} as const;

function normalize(value: string): string {
  return value.toLowerCase().replace(/\s+/g, ' ').trim();
}

function subsequence(haystack: string, needle: string): boolean {
  let index = 0;
  for (const character of needle) {
    index = haystack.indexOf(character, index);
    if (index < 0) {
      return false;
    }
    index += 1;
  }
  return true;
}

function scoreTerm(term: string, action: PaletteAction): number | null {
  const label = normalize(action.label);
  const description = normalize(action.description ?? '');
  const group = normalize(action.group ?? '');
  const keywords = normalize((action.keywords ?? []).join(' '));
  if (label === term) {
    return 1_000;
  }
  if (label.startsWith(term)) {
    return 800 - Math.min(100, label.length - term.length);
  }
  if (label.split(' ').some((word) => word.startsWith(term))) {
    return 650;
  }
  if (label.includes(term)) {
    return 500 - Math.min(100, label.indexOf(term));
  }
  if (keywords.split(' ').some((word) => word === term || word.startsWith(term))) {
    return 400;
  }
  if (group === term || group.startsWith(term)) {
    return 350;
  }
  if (description.includes(term)) {
    return 300 - Math.min(100, description.indexOf(term));
  }
  if (subsequence(label, term)) {
    return 150;
  }
  if (subsequence(description, term)) {
    return 100;
  }
  return null;
}

/**
 * Multi-term, deterministic ranking. An empty query preserves caller order so
 * command groups keep their authored sequence. Ties break by label then id.
 */
export function filterPaletteActions(
  actions: readonly PaletteAction[],
  query: string,
): readonly PaletteAction[] {
  const terms = normalize(query).split(' ').filter((term) => term.length > 0);
  if (terms.length === 0) {
    return [...actions];
  }
  const scored: { action: PaletteAction; score: number }[] = [];
  for (const action of actions) {
    const score = scorePaletteAction(action, query);
    if (score !== null) {
      scored.push({ action, score });
    }
  }
  scored.sort((left, right) => {
    if (left.score !== right.score) {
      return right.score - left.score;
    }
    const labelOrder = left.action.label.localeCompare(right.action.label);
    return labelOrder !== 0 ? labelOrder : left.action.id.localeCompare(right.action.id);
  });
  return scored.map((entry) => entry.action);
}

export function scorePaletteAction(action: PaletteAction, query: string): number | null {
  const normalizedQuery = normalize(query);
  const terms = normalizedQuery.split(' ').filter((term) => term.length > 0);
  if (terms.length === 0) {
    return 0;
  }
  // A full-phrase match outranks the sum of its individual term scores.
  const label = normalize(action.label);
  if (label === normalizedQuery) {
    return 1_000;
  }
  if (label.startsWith(normalizedQuery)) {
    return 800 - Math.min(100, label.length - normalizedQuery.length);
  }
  let total = 0;
  for (const term of terms) {
    const score = scoreTerm(term, action);
    if (score === null) {
      return null;
    }
    total += score;
  }
  return total;
}

export type PaletteNavigation =
  | { readonly kind: 'move'; readonly index: number }
  | { readonly kind: 'run' }
  | { readonly kind: 'close' };

export function paletteNavigationTarget(
  index: number,
  count: number,
  key: string,
): PaletteNavigation | null {
  if (key === 'Escape') {
    return { kind: 'close' };
  }
  if (key === 'Enter') {
    return { kind: 'run' };
  }
  if (count <= 0) {
    return null;
  }
  const current = Math.min(Math.max(index, 0), count - 1);
  switch (key) {
    case 'ArrowDown':
      return { kind: 'move', index: (current + 1) % count };
    case 'ArrowUp':
      return { kind: 'move', index: (current - 1 + count) % count };
    case 'Home':
      return { kind: 'move', index: 0 };
    case 'End':
      return { kind: 'move', index: count - 1 };
    default:
      return null;
  }
}

export function CommandPalette({
  open,
  actions,
  onClose,
  status = 'ready',
  errorMessage = null,
  onRetry,
  placeholder = PALETTE_COPY.placeholder,
  emptyMessage = null,
  className = '',
}: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const dialogRef = useRef<HTMLElement | null>(null);
  const optionRefs = useRef<Map<string, HTMLButtonElement>>(new Map());
  const previousFocus = useRef<Element | null>(null);
  const baseId = useId().replace(/[^a-zA-Z0-9_-]/g, '');
  const listId = `${baseId}-palette-list`;
  const filtered = useMemo(() => filterPaletteActions(actions, query), [actions, query]);
  const safeIndex = filtered.length === 0 ? 0 : Math.min(Math.max(activeIndex, 0), filtered.length - 1);
  const activeAction = filtered[safeIndex];

  useEffect(() => {
    if (!open) {
      return undefined;
    }
    if (typeof document !== 'undefined') {
      previousFocus.current = document.activeElement;
      const input = inputRef.current;
      input?.focus();
      input?.select();
    }
    return () => {
      const previous = previousFocus.current;
      if (previous instanceof HTMLElement) {
        previous.focus();
      }
    };
  }, [open]);

  useEffect(() => {
    if (!open || typeof document === 'undefined') {
      return;
    }
    const element = activeAction === undefined ? undefined : optionRefs.current.get(activeAction.id);
    element?.scrollIntoView({ block: 'nearest' });
  }, [open, activeAction]);

  if (!open) {
    return null;
  }

  const runAction = (action: PaletteAction | undefined): void => {
    if (action === undefined || action.disabled === true) {
      return;
    }
    action.onRun();
    onClose();
  };

  const onSearchKeyDown = (event: KeyboardEvent<HTMLInputElement>): void => {
    const target = paletteNavigationTarget(safeIndex, filtered.length, event.key);
    if (target === null) {
      return;
    }
    event.preventDefault();
    if (target.kind === 'close') {
      onClose();
      return;
    }
    if (target.kind === 'run') {
      runAction(activeAction);
      return;
    }
    setActiveIndex(target.index);
  };

  const onDialogKeyDown = (event: KeyboardEvent<HTMLElement>): void => {
    if (event.key !== 'Tab') {
      return;
    }
    const focusable = Array.from(
      event.currentTarget.querySelectorAll<HTMLElement>(
        'button:not([disabled]):not([tabindex="-1"]), input:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ),
    );
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (first === undefined || last === undefined) {
      event.preventDefault();
      return;
    }
    const ownerDocument = event.currentTarget.ownerDocument;
    if (event.shiftKey && ownerDocument.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && ownerDocument.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-ink-900/25 p-3 pt-[8vh]"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <section
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-label={PALETTE_COPY.title}
        className={`w-[min(680px,calc(100vw-24px))] overflow-hidden rounded-[var(--radius-lg)] border border-honey-200 bg-panel shadow-[var(--shadow-e3)] supports-[backdrop-filter]:bg-panel/85 supports-[backdrop-filter]:backdrop-blur-xl ${className}`}
        onKeyDown={onDialogKeyDown}
      >
        <div className="flex items-center gap-2 border-b border-hairline bg-panel-alt/80 px-3 py-2">
          <svg aria-hidden="true" width="14" height="14" viewBox="0 0 16 16" className="shrink-0 text-honey-700">
            <circle cx="6.8" cy="6.8" r="4.4" fill="none" stroke="currentColor" strokeWidth="1.4" />
            <path d="m10.2 10.2 3.2 3.2" fill="none" stroke="currentColor" strokeWidth="1.4" />
          </svg>
          <input
            ref={inputRef}
            type="text"
            role="combobox"
            aria-expanded={status === 'ready' && filtered.length > 0}
            aria-controls={status === 'ready' && filtered.length > 0 ? listId : undefined}
            aria-autocomplete="list"
            aria-activedescendant={
              activeAction === undefined ? undefined : `${baseId}-option-${safeIndex}`
            }
            aria-label={PALETTE_COPY.search}
            className="type-body-l min-w-0 flex-1 bg-transparent text-ink-900 outline-none placeholder:text-ink-500"
            placeholder={placeholder}
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setActiveIndex(0);
            }}
            onKeyDown={onSearchKeyDown}
          />
          <button
            type="button"
            className="type-mono-meta shrink-0 rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-700 hover:bg-honey-100"
            aria-label={PALETTE_COPY.close}
            onClick={() => onClose()}
          >
            Esc
          </button>
        </div>

        {status === 'loading' ? (
          <p role="status" className="type-mono-meta px-3 py-4 text-meta">
            {PALETTE_COPY.loading}
          </p>
        ) : null}

        {status === 'error' ? (
          <div role="alert" className="m-3 rounded-[var(--radius-md)] border border-signal bg-error-surface p-3">
            <p className="type-label text-signal">{PALETTE_COPY.error}</p>
            <p className="type-body-l mt-1 text-ink-900">
              {errorMessage ?? 'The command list could not be loaded.'}
            </p>
            {onRetry === undefined ? null : (
              <button
                type="button"
                className="type-label mt-2 rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal"
                onClick={() => onRetry()}
              >
                {PALETTE_COPY.retry}
              </button>
            )}
          </div>
        ) : null}

        {status === 'ready' ? (
          filtered.length === 0 ? (
            <p role="status" className="type-mono-meta px-3 py-4 text-meta">
              {emptyMessage ?? PALETTE_COPY.empty}
            </p>
          ) : (
            <ul
              id={listId}
              role="listbox"
              aria-label={PALETTE_COPY.title}
              className="m-0 max-h-[min(56vh,460px)] list-none overflow-y-auto p-1"
            >
              {filtered.map((action, index) => {
                const selected = index === safeIndex;
                const disabled = action.disabled === true;
                return (
                  <li key={action.id} role="none" className="m-0 p-0">
                    <button
                      ref={(element) => {
                        if (element === null) {
                          optionRefs.current.delete(action.id);
                        } else {
                          optionRefs.current.set(action.id, element);
                        }
                      }}
                      id={`${baseId}-option-${index}`}
                      type="button"
                      role="option"
                      aria-selected={selected}
                      aria-disabled={disabled}
                      tabIndex={-1}
                      data-action-id={action.id}
                      onMouseEnter={() => setActiveIndex(index)}
                      onClick={() => runAction(action)}
                      className={`w-full rounded-[var(--radius-sm)] border px-2 py-1.5 text-left ${
                        selected
                          ? 'border-honey-400 bg-honey-100'
                          : 'border-transparent hover:bg-panel-alt'
                      } ${disabled ? 'cursor-not-allowed opacity-60' : ''}`}
                    >
                      <div className="flex items-baseline gap-2">
                        <span className="type-body-l min-w-0 flex-1 truncate text-ink-900">
                          {action.label}
                        </span>
                        {action.requiresApproval === true ? (
                          <span className="type-mono-meta shrink-0 rounded-[var(--radius-pill)] border border-amber bg-panel-alt px-1.5 py-0.5 text-ink-800">
                            {PALETTE_COPY.approval}
                          </span>
                        ) : null}
                        {action.shortcut === null || action.shortcut === undefined ? null : (
                          <kbd className="type-mono-meta shrink-0 rounded-[var(--radius-xs)] border border-hairline bg-panel px-1.5 py-0.5 text-ink-700">
                            {action.shortcut}
                          </kbd>
                        )}
                      </div>
                      {action.description === null || action.description === undefined ? null : (
                        <p className="type-mono-meta mt-0.5 truncate text-meta">{action.description}</p>
                      )}
                      {action.group === null || action.group === undefined ? null : (
                        <span className="type-mono-meta mt-0.5 block text-meta">{action.group}</span>
                      )}
                      {disabled ? (
                        <p className="type-mono-meta mt-0.5 text-signal">
                          {action.disabledReason ?? PALETTE_COPY.disabled}
                        </p>
                      ) : null}
                    </button>
                  </li>
                );
              })}
            </ul>
          )
        ) : null}

        <footer className="flex items-center justify-between gap-2 border-t border-hairline bg-panel-alt/80 px-3 py-1.5">
          <span className="type-mono-meta hidden text-meta sm:inline">{PALETTE_COPY.navigateHint}</span>
          <span className="type-mono-meta text-meta sm:hidden">{PALETTE_COPY.escapeHint}</span>
          <span className="type-mono-meta text-meta">
            {filtered.length} {PALETTE_COPY.results}
          </span>
        </footer>
      </section>
    </div>
  );
}
