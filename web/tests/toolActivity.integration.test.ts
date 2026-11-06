import { describe, expect, it } from 'vitest';
import { createUiStore, type ToolActivityRecord, type UiState } from '../src/state/store';
import {
  SCHEMA_VERSION,
  type DecodedEvent,
  type Envelope,
  type Refusal,
  type ServerCommandName,
  type ToolOutcome,
} from '../src/api/contract';
import type { ConnectionStatus, SessionInfo, UiApi } from '../src/api/client';
import {
  TOOL_ACTIVITY_OUTCOMES,
  ToolActivityPanel,
  formatActivityArgsSha256,
  isToolActivityOutcome,
  sortToolActivity,
  toolActivityOutcomeLabel,
  toolActivitySummary,
  truncateActivityValue,
  type ToolActivityEntry,
} from '../src/panels/ToolActivityPanel';

/**
 * W12 / Wave 2-3 — tool-activity selector + prop integration.
 *
 * Static-source suite (no build/test execution this wave): pins the store field, the
 * store→panel structural contract, the reducer's no-fabrication rules, and the actual
 * mount point (AgentsPanel renders ToolActivityPanel; App is not its owner).
 */
const SOURCE_FILES = import.meta.glob('../src/**/*.{ts,tsx}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function sourceOf(suffix: string): string {
  const key = Object.keys(SOURCE_FILES).find((candidate) => candidate.endsWith(suffix));
  return key ? SOURCE_FILES[key] : '';
}

class FakeUiApi implements UiApi {
  readonly kind = 'ws' as const;
  readonly sends: Array<{
    readonly name: ServerCommandName;
    readonly payload: unknown;
    readonly resolve: () => void;
    readonly reject: (error: Error) => void;
  }> = [];
  private readonly events = new Set<(event: DecodedEvent, envelope: Envelope<unknown>) => void>();
  private readonly statuses = new Set<(status: ConnectionStatus, detail?: Refusal) => void>();
  private readonly clientErrors = new Set<(refusal: Refusal) => void>();

  async open(): Promise<SessionInfo> {
    return { token: 'fake', schema: SCHEMA_VERSION, last_seq: 0, ws: '/ws' };
  }
  close(): void {}
  send<N extends ServerCommandName>(name: N, payload: unknown, _options?: { readonly correlationId?: string }): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      this.sends.push({ name, payload, resolve, reject });
    });
  }
  subscribe(listener: (event: DecodedEvent, envelope: Envelope<unknown>) => void): () => void {
    this.events.add(listener);
    return () => {
      this.events.delete(listener);
    };
  }
  subscribeStatus(listener: (status: ConnectionStatus, detail?: Refusal) => void): () => void {
    this.statuses.add(listener);
    return () => {
      this.statuses.delete(listener);
    };
  }
  subscribeClientError(listener: (refusal: Refusal) => void): () => void {
    this.clientErrors.add(listener);
    return () => {
      this.clientErrors.delete(listener);
    };
  }
  emit(event: DecodedEvent): void {
    const envelope: Envelope<unknown> = {
      schema: SCHEMA_VERSION,
      type: 'EVENT',
      name: event.name,
      id: 'e',
      correlation_id: 'e',
      run_id: null,
      task_id: null,
      seq: 1,
      ts: new Date().toISOString(),
      payload: {},
    };
    for (const listener of this.events) listener(event, envelope);
  }
}

async function connectedStore(): Promise<{ api: FakeUiApi; store: ReturnType<typeof createUiStore> }> {
  const api = new FakeUiApi();
  const store = createUiStore(api);
  await store.getState().init();
  return { api, store };
}

describe('W12 tool-activity selector/prop integration', () => {
  it('exposes the frozen toolActivity store field with an empty default', async () => {
    const { store } = await connectedStore();
    const selector: (state: UiState) => readonly ToolActivityRecord[] = (state) => state.toolActivity;
    expect(Array.isArray(selector(store.getState()))).toBe(true);
    expect(selector(store.getState())).toEqual([]);
    expect(store.getState().toolActivity).toEqual([]);
    const storeSource = sourceOf('/state/store.ts');
    expect(storeSource).toMatch(/readonly toolActivity: readonly ToolActivityRecord\[\]/);
    expect(storeSource).toMatch(/case 'TOOL_CALL'/);
    expect(storeSource).toMatch(/case 'TOOL_RESULT'/);
  });

  it('store records are structurally valid panel entries (selector → prop)', async () => {
    const { api, store } = await connectedStore();
    api.emit({
      name: 'TOOL_CALL',
      payload: {
        run_id: 'run-1',
        task_id: 'task-1',
        tool_call_id: 'tc-1',
        tool: 'file_read',
        args_sha256: 'a'.repeat(64),
        turn: 1,
      },
    });
    const record = store.getState().toolActivity[0];
    expect(record).toBeDefined();
    expect(record?.toolCallId).toBe('tc-1');
    expect(record?.tool).toBe('file_read');
    expect(record?.turn).toBe(1);
    expect(record?.outcome).toBe('pending');

    // Compile-time + runtime structural proof that the store field feeds the panel prop.
    const entries: readonly ToolActivityEntry[] = store.getState().toolActivity;
    expect(sortToolActivity(entries)[0]?.toolCallId).toBe('tc-1');
    expect(toolActivitySummary(entries).counts.pending).toBe(1);
    expect(toolActivityOutcomeLabel(entries[0]?.outcome ?? 'pending')).toBe('Pending');
    expect(formatActivityArgsSha256(record?.argsSha256)).toBe(`${'a'.repeat(12)}…`);
    expect(typeof ToolActivityPanel).toBe('function');
  });

  it('settles one row in place; malformed, duplicate and unmatched results never fabricate state', async () => {
    const { api, store } = await connectedStore();
    const call: DecodedEvent = {
      name: 'TOOL_CALL',
      payload: {
        run_id: 'run-1',
        task_id: 'task-1',
        tool_call_id: 'tc-settle',
        tool: 'file_list',
        args_sha256: 'c'.repeat(64),
        turn: 2,
      },
    };
    api.emit(call);
    api.emit(call); // duplicate identity must not duplicate the row
    expect(store.getState().toolActivity).toHaveLength(1);

    api.emit({
      name: 'TOOL_RESULT',
      payload: { run_id: 'run-1', task_id: 'task-1', tool_call_id: 'tc-settle', outcome: 'ok', code: null },
    });
    expect(store.getState().toolActivity[0]?.outcome).toBe('ok');

    // Late/duplicate terminal result cannot rewrite a terminal outcome.
    api.emit({
      name: 'TOOL_RESULT',
      payload: { run_id: 'run-1', task_id: 'task-1', tool_call_id: 'tc-settle', outcome: 'denied' },
    });
    expect(store.getState().toolActivity[0]?.outcome).toBe('ok');

    // Unknown outcome token is ignored (no fabricated state).
    const bogus: DecodedEvent = {
      name: 'TOOL_RESULT',
      payload: {
        run_id: 'run-1',
        task_id: 'task-1',
        tool_call_id: 'tc-settle',
        outcome: 'bogus' as unknown as ToolOutcome,
      },
    };
    api.emit(bogus);
    expect(store.getState().toolActivity[0]?.outcome).toBe('ok');

    // Result without its call is unmatched and must not create an activity row.
    api.emit({
      name: 'TOOL_RESULT',
      payload: { run_id: 'run-1', task_id: 'task-1', tool_call_id: 'tc-unknown', outcome: 'ok' },
    });
    expect(store.getState().toolActivity).toHaveLength(1);
  });

  it('empty and malformed activity render honestly (empty state, unknown label, zero counts)', async () => {
    const empty = toolActivitySummary([]);
    expect(empty.total).toBe(0);
    for (const outcome of TOOL_ACTIVITY_OUTCOMES) {
      expect(empty.counts[outcome]).toBe(0);
    }
    expect(toolActivityOutcomeLabel('bogus')).toBe('Unknown');
    expect(isToolActivityOutcome('bogus')).toBe(false);
    expect(truncateActivityValue('')).toBe('—');
    expect(truncateActivityValue(null)).toBe('—');
    expect(formatActivityArgsSha256(undefined)).toBe('—');

    const malformed = [
      { toolCallId: '', tool: '', argsSha256: '', turn: -1, outcome: 'bogus' as never },
    ] satisfies ToolActivityEntry[];
    const summary = toolActivitySummary(malformed);
    expect(summary.total).toBe(1);
    expect(summary.counts.ok + summary.counts.pending).toBe(0);

    const panelSource = sourceOf('/panels/ToolActivityPanel.tsx');
    expect(panelSource).toMatch(/EmptyState/);
    expect(panelSource).toMatch(/data-outcome=/);
    expect(panelSource).toContain('STRINGS.toolActivity.sectionAria');
    expect(panelSource).toContain('STRINGS.toolActivity.empty');
  });

  it('AgentsPanel is the actual mount point for ToolActivityPanel with the store selector prop', () => {
    const agentsSource = sourceOf('/panels/AgentsPanel.tsx');
    expect(agentsSource, 'AgentsPanel must import ToolActivityPanel').toContain("from './ToolActivityPanel'");
    expect(agentsSource, 'AgentsPanel must select state.toolActivity').toMatch(/state\.toolActivity/);
    expect(agentsSource, 'AgentsPanel must mount ToolActivityPanel').toContain('<ToolActivityPanel');
    expect(agentsSource, 'entries must come from the store selector').toMatch(/entries=\{toolActivity\}/);
    // App is intentionally NOT asserted as a mount point (it does not own this panel).
    const appSource = sourceOf('/App.tsx');
    expect(appSource.includes('<ToolActivityPanel'), 'App must not double-mount the panel').toBe(false);
  });
});
