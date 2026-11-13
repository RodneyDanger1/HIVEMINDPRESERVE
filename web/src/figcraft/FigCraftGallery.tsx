/**
 * Accessible FigCraft representation gallery (Lead task-11).
 *
 * Renders the FigCraft boards copied from `docs/ui/figcraft/**` into
 * `web/public/figcraft/`. Static, read-only representation: selecting a board
 * never mutates application state and no network request is made beyond the
 * SVG `<img>` load from the Vite public asset path.
 *
 * Accessibility contract:
 * - tablist/tab/tabpanel roles with roving tabindex + ArrowLeft/Right/Home/End;
 * - every board is a toggle button (`aria-pressed`) with a descriptive image
 *   alt text; the preview is a labelled region with a polite live status;
 * - a board whose SVG cannot load degrades to an explicit "preview
 *   unavailable" note (never a broken image or a fabricated preview).
 *
 * Audit honesty: each board carries its generated audit status. This gallery
 * never presents a self-verified or proposed board as watchdog-audited.
 */
import { useId, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';

import { FIGCRAFT_BOARDS, FIGCRAFT_COUNTS, FIGCRAFT_GROUPS } from './catalog';
import type { FigCraftBoard, FigCraftGroup, FigCraftOwner } from './types';

const SECONDARY_SURFACE_LIMIT = 8;

/** Human audit label; never inflate self-verified/proposed into watchdog-accepted. */
function auditLabel(board: FigCraftBoard): string {
  const { status, verdict } = board.audit;
  if (status === 'watchdog-audited') {
    return `watchdog-audited (${verdict ?? 'PASS'})`;
  }
  if (status === 'self-verified') {
    return 'self-verified - not watchdog-audited';
  }
  if (status === 'proposed') {
    return 'proposed - not watchdog-audited';
  }
  return 'unverified - no accepted audit evidence';
}

function auditShortLabel(board: FigCraftBoard): string {
  const { status, verdict } = board.audit;
  if (status === 'watchdog-audited') {
    return `watchdog ${verdict ?? 'PASS'}`;
  }
  if (status === 'self-verified') {
    return 'self-verified';
  }
  if (status === 'proposed') {
    return 'PROPOSED';
  }
  return 'UNVERIFIED';
}

function auditTone(board: FigCraftBoard): string {
  if (board.audit.status === 'watchdog-audited') {
    return 'text-verify';
  }
  if (board.audit.status === 'proposed') {
    return 'text-amber';
  }
  if (board.audit.status === 'unverified') {
    return 'text-signal';
  }
  return 'text-ink-500';
}

/**
 * Roving tablist target. `null` when no group exists, so callers cannot do a
 * `% 0` (NaN index) on an empty catalog.
 */
export function nextFigCraftGroupIndex(
  index: number,
  count: number,
  delta: number,
): number | null {
  if (count <= 0 || !Number.isInteger(index) || index < 0 || index >= count) {
    return null;
  }
  return (index + delta + count) % count;
}

function ownerLabel(owner: FigCraftOwner): string {
  return FIGCRAFT_GROUPS.find((group) => group.id === owner)?.label ?? owner;
}

function publicUrl(board: FigCraftBoard): string {
  return `${import.meta.env.BASE_URL}figcraft/${board.svgFile}`;
}

function boardAltText(board: FigCraftBoard): string {
  const surfaces =
    board.surfaces.length === 0
      ? 'a design board with no mapped surface'
      : `${board.surfaces.length} mapped surface${board.surfaces.length === 1 ? '' : 's'}`;
  return `${board.title} — ${ownerLabel(board.owner)} board, ${board.width} by ${board.height} pixels, ${surfaces}, ${auditLabel(board)}`;
}

function boardMeta(board: FigCraftBoard): string {
  return `${ownerLabel(board.owner)} · ${board.width}×${board.height} · ${board.surfaces.length} surface${
    board.surfaces.length === 1 ? '' : 's'
  }`;
}

function BoardCard(props: {
  readonly board: FigCraftBoard;
  readonly selected: boolean;
  readonly unavailable: boolean;
  readonly onSelect: (board: FigCraftBoard) => void;
  readonly onUnavailable: (boardId: string) => void;
}) {
  const { board, selected, unavailable, onSelect, onUnavailable } = props;
  return (
    <li className="min-w-0">
      <figure className="m-0 flex h-full flex-col">
        <button
          type="button"
          aria-pressed={selected}
          data-figcraft-board={board.id}
          data-figcraft-owner={board.owner}
          onClick={() => onSelect(board)}
          className={`flex h-full flex-col overflow-hidden rounded-[var(--radius-card)] border text-left transition-colors ${
            selected
              ? 'border-honey-500 bg-honey-100'
              : 'border-hairline bg-panel hover:border-honey-400 hover:bg-panel-alt'
          }`}
        >
          {unavailable ? (
            <span
              role="img"
              aria-label={`Preview unavailable for ${board.title}`}
              className="type-mono-meta flex h-[112px] w-full items-center justify-center border-b border-hairline bg-recessed px-2 text-center text-ink-500"
            >
              Preview unavailable
            </span>
          ) : (
            <img
              src={publicUrl(board)}
              alt={boardAltText(board)}
              width={board.width}
              height={board.height}
              loading="lazy"
              decoding="async"
              onError={() => onUnavailable(board.id)}
              className="h-[112px] w-full border-b border-hairline bg-panel-alt object-cover object-top"
            />
          )}
          <span className="type-label flex min-w-0 flex-1 items-center px-2 py-1.5 text-ink-900">
            {board.title}
          </span>
        </button>
        <figcaption className="type-mono-meta mt-1 px-1 text-ink-500">
          <span className="block truncate">{boardMeta(board)}</span>
          <span className={`block truncate ${auditTone(board)}`}>{auditShortLabel(board)}</span>
        </figcaption>
      </figure>
    </li>
  );
}

function BoardGrid(props: {
  readonly boards: readonly FigCraftBoard[];
  readonly group: FigCraftGroup;
  readonly selectedId: string;
  readonly unavailable: ReadonlySet<string>;
  readonly onSelect: (board: FigCraftBoard) => void;
  readonly onUnavailable: (boardId: string) => void;
}) {
  const { boards, group, selectedId, unavailable, onSelect, onUnavailable } = props;
  if (boards.length === 0) {
    return (
      <p role="status" className="type-mono-code rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-3 py-2 text-ink-500">
        {group.label} has no boards in the catalog. Nothing is rendered rather than fabricated.
      </p>
    );
  }
  return (
    <ul aria-label={`${group.label} boards`} className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(148px,1fr))] gap-2 p-0">
      {boards.map((board) => (
        <BoardCard
          key={board.id}
          board={board}
          selected={board.id === selectedId}
          unavailable={unavailable.has(board.id)}
          onSelect={onSelect}
          onUnavailable={onUnavailable}
        />
      ))}
    </ul>
  );
}

function BoardPreview(props: {
  readonly board: FigCraftBoard | null;
  readonly unavailable: boolean;
  readonly onUnavailable: (boardId: string) => void;
}) {
  const { board, unavailable, onUnavailable } = props;
  if (board === null) {
    return (
      <p className="type-mono-code m-0 text-ink-500">
        Select a board to preview its FigCraft SVG, dimensions, provenance and covered surfaces.
      </p>
    );
  }
  const surfaces = board.surfaces.slice(0, SECONDARY_SURFACE_LIMIT);
  const hidden = board.surfaces.length - surfaces.length;
  return (
    <div className="flex min-h-0 flex-col gap-3">
      {unavailable ? (
        <p role="note" className="type-mono-code m-0 rounded-[var(--radius-sm)] border border-amber bg-panel-alt px-3 py-2 text-ink-800">
          Preview unavailable — the FigCraft SVG could not be loaded from the public asset path. Source
          evidence: {board.sourcePath}
        </p>
      ) : (
        <img
          src={publicUrl(board)}
          alt={boardAltText(board)}
          width={board.width}
          height={board.height}
          decoding="async"
          onError={() => onUnavailable(board.id)}
          className="max-h-[46vh] w-full rounded-[var(--radius-md)] border border-hairline bg-panel-alt object-contain object-top"
        />
      )}
      <dl className="m-0 grid grid-cols-[max-content_1fr] gap-x-3 gap-y-1">
        <dt className="type-mono-meta text-ink-500">dimensions</dt>
        <dd className="type-mono-code m-0 text-ink-900">
          {board.width}×{board.height} (manifest board size; SVG export is vector)
        </dd>
        <dt className="type-mono-meta text-ink-500">set</dt>
        <dd className="type-mono-code m-0 text-ink-900">{ownerLabel(board.owner)}</dd>
        <dt className="type-mono-meta text-ink-500">surfaces</dt>
        <dd className="type-mono-code m-0 text-ink-900">{board.surfaces.length}</dd>
        <dt className="type-mono-meta text-ink-500">source</dt>
        <dd className="type-mono-code m-0 break-all text-ink-800">{board.sourcePath}</dd>
        <dt className="type-mono-meta text-ink-500">sha256</dt>
        <dd className="type-mono-code m-0 break-all text-ink-800">{board.sha256}</dd>
        <dt className="type-mono-meta text-ink-500">manifest</dt>
        <dd className="type-mono-code m-0 break-all text-ink-800">{board.manifest}</dd>
        <dt className="type-mono-meta text-ink-500">audit</dt>
        <dd className={`type-mono-code m-0 ${auditTone(board)}`}>{auditLabel(board)}</dd>
        <dt className="type-mono-meta text-ink-500">audit basis</dt>
        <dd className="type-mono-code m-0 break-words text-ink-800">{board.audit.basis}</dd>
        <dt className="type-mono-meta text-ink-500">audit source</dt>
        <dd className="type-mono-code m-0 break-all text-ink-800">{board.audit.ledger ?? 'none'}</dd>
      </dl>
      {board.audit.status === 'proposed' ? (
        <p role="note" className="type-mono-code m-0 rounded-[var(--radius-sm)] border border-amber bg-panel-alt px-3 py-2 text-ink-800">
          Proposed board: it is real FigCraft evidence, but it is not watchdog-audited and does not
          change the accepted design truth. Do not treat it as an approved surface.
        </p>
      ) : board.audit.status === 'self-verified' ? (
        <p role="note" className="type-mono-code m-0 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-3 py-2 text-ink-800">
          Self-verified board: the owning set's own verify report passed, but this board is not in the
          watchdog ledger. Only the A/B sets are watchdog-audited.
        </p>
      ) : null}
      {surfaces.length > 0 ? (
        <div className="min-h-0">
          <div className="type-label text-ink-500">Surfaces covered by this board</div>
          <ul aria-label="Surfaces covered by this board" className="m-0 mt-1 list-none space-y-0.5 p-0">
            {surfaces.map((surface) => (
              <li key={surface.surfaceId} className="type-mono-code truncate text-ink-800">
                <span className="font-mono text-ink-500">{surface.surfaceId}</span> {surface.name}
              </li>
            ))}
            {hidden > 0 ? (
              <li className="type-mono-meta text-ink-500">and {hidden} more surface{hidden === 1 ? '' : 's'} in the source manifest</li>
            ) : null}
          </ul>
        </div>
      ) : (
        <p className="type-mono-meta m-0 text-ink-500">
          This board is a design board; the source manifest maps no product surface to it.
        </p>
      )}
    </div>
  );
}

/** The gallery body used by `FigCraftPanel` (and reusable by a Lead-owned view). */
export function FigCraftGallery() {
  const baseId = useId();
  const [activeOwner, setActiveOwner] = useState<FigCraftOwner | null>(null);
  const [selectedId, setSelectedId] = useState<string>(() => FIGCRAFT_BOARDS[0]?.id ?? '');
  const [unavailable, setUnavailable] = useState<ReadonlySet<string>>(() => new Set<string>());
  const tabRefs = useRef<Array<HTMLButtonElement | null>>([]);

  const selectedBoard = FIGCRAFT_BOARDS.find((board) => board.id === selectedId) ?? null;

  const markUnavailable = (boardId: string): void => {
    setUnavailable((previous) => {
      if (previous.has(boardId)) {
        return previous;
      }
      const next = new Set(previous);
      next.add(boardId);
      return next;
    });
  };

  const activateGroup = (index: number): void => {
    const group = FIGCRAFT_GROUPS[index];
    if (group === undefined) {
      return;
    }
    setActiveOwner(group.id);
    const first = FIGCRAFT_BOARDS.find((board) => board.owner === group.id);
    setSelectedId(first?.id ?? '');
    tabRefs.current[index]?.focus();
  };

  const onTabKeyDown = (event: KeyboardEvent<HTMLButtonElement>, index: number): void => {
    const count = FIGCRAFT_GROUPS.length;
    if (count === 0) {
      return;
    }
    if (event.key === 'ArrowRight') {
      event.preventDefault();
      const next = nextFigCraftGroupIndex(index, count, 1);
      if (next !== null) {
        activateGroup(next);
      }
    } else if (event.key === 'ArrowLeft') {
      event.preventDefault();
      const next = nextFigCraftGroupIndex(index, count, -1);
      if (next !== null) {
        activateGroup(next);
      }
    } else if (event.key === 'Home') {
      event.preventDefault();
      activateGroup(0);
    } else if (event.key === 'End') {
      event.preventDefault();
      activateGroup(count - 1);
    }
  };

  const firstGroup = FIGCRAFT_GROUPS[0];
  if (firstGroup === undefined) {
    return (
      <p role="status" className="type-mono-code m-0 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-3 py-2 text-ink-500">
        No accepted FigCraft asset sets exist in the catalog. The representation surface is
        unavailable rather than fabricated.
      </p>
    );
  }
  const activeOwnerId: FigCraftOwner = activeOwner ?? firstGroup.id;

  if (FIGCRAFT_BOARDS.length === 0) {
    return (
      <p role="status" className="type-mono-code m-0 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-3 py-2 text-ink-500">
        No FigCraft boards exist in the catalog. The representation surface is unavailable rather
        than fabricated.
      </p>
    );
  }

  return (
    <div className="flex min-h-0 flex-col gap-3">
      <div role="tablist" aria-label="FigCraft asset sets" className="flex flex-wrap gap-1">
        {FIGCRAFT_GROUPS.map((group, index) => {
          const active = group.id === activeOwnerId;
          return (
            <button
              key={group.id}
              ref={(node) => {
                tabRefs.current[index] = node;
              }}
              type="button"
              role="tab"
              id={`${baseId}-tab-${group.id}`}
              aria-selected={active}
              aria-controls={`${baseId}-panel-${group.id}`}
              tabIndex={active ? 0 : -1}
              title={group.description}
              data-figcraft-set={group.id}
              onClick={() => activateGroup(index)}
              onKeyDown={(event) => onTabKeyDown(event, index)}
              className={`type-label rounded-[var(--radius-pill)] border px-3 py-1.5 transition-colors ${
                active
                  ? 'border-honey-500 bg-honey-300 text-ink-900'
                  : 'border-hairline bg-panel text-ink-500 hover:border-honey-400 hover:text-ink-900'
              }`}
            >
              {group.label}
            </button>
          );
        })}
      </div>
      <p role="note" className="type-mono-meta m-0 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-3 py-2 text-ink-500">
        Audit provenance is shown per board: {FIGCRAFT_COUNTS.watchdogAuditedBoards} watchdog-audited,{' '}
        {FIGCRAFT_COUNTS.selfVerifiedBoards} self-verified (not watchdog-audited),{' '}
        {FIGCRAFT_COUNTS.proposedBoards} proposed (not watchdog-audited). The watchdog verdict{' '}
        {FIGCRAFT_COUNTS.watchdog} covers the A/B sets only; it is not a verdict on the HiveMind set.
      </p>
      <div className="grid min-h-0 gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)]">
        <div className="min-w-0">
          {FIGCRAFT_GROUPS.map((group) => {
            const boards = FIGCRAFT_BOARDS.filter((board) => board.owner === group.id);
            return (
              <div
                key={group.id}
                role="tabpanel"
                id={`${baseId}-panel-${group.id}`}
                aria-labelledby={`${baseId}-tab-${group.id}`}
                hidden={group.id !== activeOwnerId}
                tabIndex={0}
              >
                <p className="type-mono-code m-0 mb-2 text-ink-500">{group.description}</p>
                <BoardGrid
                  boards={boards}
                  group={group}
                  selectedId={selectedId}
                  unavailable={unavailable}
                  onSelect={(board) => setSelectedId(board.id)}
                  onUnavailable={markUnavailable}
                />
              </div>
            );
          })}
        </div>
        <section
          aria-label="Selected FigCraft board"
          data-figcraft-preview
          className="surface-panel min-w-0 overflow-auto p-3"
        >
          <p role="status" aria-live="polite" className="sr-only">
            {selectedBoard === null ? 'No board selected' : `Selected ${selectedBoard.title}`}
          </p>
          <h3 className="type-heading-h3 m-0 text-ink-900">
            {selectedBoard === null ? 'Preview' : selectedBoard.title}
          </h3>
          <BoardPreview
            board={selectedBoard}
            unavailable={selectedBoard !== null && unavailable.has(selectedBoard.id)}
            onUnavailable={markUnavailable}
          />
        </section>
      </div>
    </div>
  );
}
