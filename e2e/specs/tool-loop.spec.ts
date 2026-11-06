import { test, expect, type Locator, type Page } from '@playwright/test';
import {
  installMockWs,
  sendMalformedEvent,
  sendMalformedFrame,
  sendToolCall,
  sendToolResult,
  startDistHarness,
  toolCallFrame,
  type DistHarness,
  type MockWsHarness,
} from '../helpers/replay-harness';
import type { ToolCallPayload, ToolResultPayload } from '../../web/src/api/contract';

/**
 * WAVE 3 / SEAT 7 â€” e2e adoption & truth for the W12 tool-activity surface.
 *
 * Transport double only (`startDistHarness` on `web/dist` + `installMockWs`); no live
 * server, no provider call, no key, no build/run by this seat.
 *
 * ADOPTED VALIDATED HELPERS (`e2e/helpers/replay-harness.ts`):
 *   - `sendToolCall` / `sendToolResult` (validated `TOOL_CALL`/`TOOL_RESULT` builders:
 *     non-empty ids, 64-hex args_sha256, integer turn >= 1, frozen outcome vocabulary).
 *   - `sendMalformedEvent` for deliberate decoder-refusal cases only (a spec can no
 *     longer hand-roll an invalid tool payload by accident).
 *   - `toolCallFrame` used once to prove the builder itself rejects bad payloads.
 *
 * ACTUAL UI (read 2026-11-06 17:1x): `AgentsPanel` hosts the panel inside
 *   `div[data-testid="agents-tool-activity"]` (role=region) â†’
 *   `section[aria-label="Tool activity"]` â†’ `ul[aria-label="Tool calls"]` â†’
 *   `li[data-outcome]` rows.
 *
 * PHASE 1 (logic-gatekeeper â€” written before this rewrite):
 * {
 *   "phase1": {
 *     "architecture": "Mock WS -> app WsApiClient -> contract decoder -> store.toolActivity reducer -> ToolActivityPanel hosted by AgentsPanel (data-testid='agents-tool-activity'). Validated harness builders construct only schema-valid frames; malformed frames go through a separate explicit helper. This spec owns neither product nor helper code.",
 *     "dataFlow": "Playwright WebSocket mock -> EVENT frames -> decoder (required identity fields, closed outcome enum) -> store upsert by tool_call_id -> AgentsPanel/ToolActivityPanel rows. Duplicate and out-of-order frames must converge to one row with the terminal outcome preserved; malformed frames stop at the decoder (E_SCHEMA_INVALID toast) with zero mutation.",
 *     "edgeCases": [
 *       "TOOL_CALL renders one li[data-outcome=pending]; TOOL_RESULT settles the same row",
 *       "all five frozen outcomes render their actual labels and data-outcome values",
 *       "duplicate TOOL_CALL/TOOL_RESULT are idempotent (store first-wins semantics)",
 *       "out-of-order TOOL_RESULT is buffered (not rendered) and replayed onto the late TOOL_CALL: zero rows before the call, exactly one terminal row after",
 *       "unknown outcome token and missing args_sha256 are refused via sendMalformedEvent with no row mutation",
 *       "the validated builder rejects an invalid payload before it can reach the socket",
 *       "invalid raw bytes are refused without losing the socket"
 *     ],
 *     "rootCause": "Wave-1 returned early on a TOOL_RESULT without a matching row, losing fan-out-ordered terminal outcomes. Wave-3 now buffers bounded/deduped results and replays them onto the late TOOL_CALL without fabricating a row; this spec pins the observable contract: zero rows before the call, one terminal row after, terminal-once and dedupe preserved."
 *   }
 * }
 */

const OUTCOME_LABEL: Readonly<Record<string, string>> = {
  pending: 'Pending',
  ok: 'OK',
  refused: 'Refused',
  denied: 'Denied',
  timeout: 'Timed out',
  cancelled: 'Cancelled',
};

const TOOL_OUTCOME_LABEL: Readonly<Record<string, RegExp>> = {
  ok: /OK/,
  refused: /Refused/,
  denied: /Denied/,
  timeout: /Timed out/,
  cancelled: /Cancelled/,
};

const BASE_HASH = 'a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90';

let harness: DistHarness;

test.beforeAll(async () => {
  harness = await startDistHarness();
});

test.afterAll(async () => {
  await harness.close();
});

async function openConnected(page: Page): Promise<MockWsHarness> {
  const ws = await installMockWs(page);
  await page.goto(harness.url);
  await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });
  return ws;
}

/** Agents view now hosts the panel: assert the real host region, then scope to the panel. */
async function openToolActivity(page: Page): Promise<{ ws: MockWsHarness; panel: Locator }> {
  const ws = await openConnected(page);
  const nav = page.locator('nav[aria-label="Primary navigation"]');
  await nav.getByRole('button', { name: 'Agents', exact: true }).click();
  const host = page.getByTestId('agents-tool-activity');
  await expect(host, 'AgentsPanel must host the tool-activity region (data-testid)').toBeVisible({
    timeout: 10_000,
  });
  const panel = host.locator('section[aria-label="Tool activity"]');
  await expect(panel).toBeVisible();
  return { ws, panel };
}

function activityList(panel: Locator): Locator {
  return panel.locator('ul[aria-label="Tool calls"]');
}

function rows(panel: Locator, tool: string): Locator {
  return panel.locator('li[data-outcome]').filter({ hasText: tool });
}

function toolCall(overrides: Partial<ToolCallPayload> = {}): ToolCallPayload {
  return {
    run_id: 'run-e2e-tool',
    task_id: 'task-e2e-tool',
    tool_call_id: 'call-e2e-1',
    tool: 'file_read',
    args_sha256: BASE_HASH,
    turn: 1,
    ...overrides,
  };
}

function toolResult(overrides: Partial<ToolResultPayload> = {}): ToolResultPayload {
  return {
    run_id: 'run-e2e-tool',
    task_id: 'task-e2e-tool',
    tool_call_id: 'call-e2e-1',
    outcome: 'ok',
    ...overrides,
  };
}

test.describe('W12 tool activity â€” validated builders + AgentsPanel host (Wave 3)', () => {
  test('TOOL_CALL renders exactly one pending row; TOOL_RESULT settles it in place', async ({ page }) => {
    const { ws, panel } = await openToolActivity(page);
    sendToolCall(ws, toolCall({ tool_call_id: 'call-pending' }), 1);

    await expect(activityList(panel)).toBeVisible();
    await expect(panel.locator('li[data-outcome="pending"]')).toHaveCount(1);
    const row = rows(panel, 'file_read');
    await expect(row).toContainText('file_read');
    await expect(row).toContainText(OUTCOME_LABEL.pending);
    await expect(row).toContainText(`args_sha256 ${BASE_HASH.slice(0, 12)}`);
    await expect(row).toContainText('turn 1');

    sendToolResult(ws, toolResult({ tool_call_id: 'call-pending', outcome: 'ok' }), 2);
    await expect(panel.locator('li[data-outcome="ok"]')).toHaveCount(1);
    await expect(panel.locator('li[data-outcome="pending"]')).toHaveCount(0);
    await expect(rows(panel, 'file_read')).toContainText(TOOL_OUTCOME_LABEL.ok);
  });

  test('all five outcomes render their actual label via data-outcome (no fabricated success)', async ({ page }) => {
    const { ws, panel } = await openToolActivity(page);
    const scenarios = [
      { outcome: 'ok', tool: 'file_read', code: null },
      { outcome: 'refused', tool: 'file_list', code: 'E_FILE_TOO_LARGE' },
      { outcome: 'denied', tool: 'shell_exec', code: null },
      { outcome: 'timeout', tool: 'spawn_agent', code: null },
      { outcome: 'cancelled', tool: 'report_result', code: null },
    ] as const;

    let seq = 1;
    for (const [index, scenario] of scenarios.entries()) {
      const toolCallId = `call-outcome-${index}`;
      sendToolCall(
        ws,
        toolCall({ tool_call_id: toolCallId, tool: scenario.tool, args_sha256: BASE_HASH, turn: 1 }),
        seq++,
      );
      sendToolResult(
        ws,
        toolResult({
          tool_call_id: toolCallId,
          tool: scenario.tool,
          outcome: scenario.outcome,
          ...(scenario.code === null ? {} : { code: scenario.code }),
        }),
        seq++,
      );
    }

    for (const scenario of scenarios) {
      const row = rows(panel, scenario.tool);
      await expect(row, `${scenario.tool} one row`).toHaveCount(1);
      await expect(row, `${scenario.tool} data-outcome`).toHaveAttribute('data-outcome', scenario.outcome);
      await expect(row).toContainText(TOOL_OUTCOME_LABEL[scenario.outcome] ?? new RegExp(scenario.outcome, 'i'));
      if (scenario.code) {
        await expect(row, 'named refusal code rendered').toContainText(`code ${scenario.code}`);
      }
    }
  });

  test('duplicate TOOL_CALL / TOOL_RESULT upsert idempotently (one row, one terminal outcome)', async ({ page }) => {
    const { ws, panel } = await openToolActivity(page);
    const call = toolCall({ tool_call_id: 'call-dupe', turn: 2 });
    const result = toolResult({ tool_call_id: 'call-dupe', outcome: 'ok' });

    sendToolCall(ws, call, 1);
    sendToolCall(ws, call, 2); // duplicate identity (store: first wins)
    await expect(panel.locator('li[data-outcome="pending"]')).toHaveCount(1);
    await expect(rows(panel, 'file_read')).toHaveCount(1);

    sendToolResult(ws, result, 3);
    sendToolResult(ws, result, 4); // duplicate terminal (store: first wins)
    await expect(panel.locator('li[data-outcome="ok"]')).toHaveCount(1);
    await expect(rows(panel, 'file_read')).toHaveCount(1);
  });

  test('out-of-order TOOL_RESULT reconciles onto the late TOOL_CALL (one row, terminal preserved)', async ({ page }) => {
    const { ws, panel } = await openToolActivity(page);
    // A result alone has no tool/args identity, so it must not fabricate a row.
    // The terminal outcome is buffered and replayed when the matching call arrives.
    sendToolResult(ws, toolResult({ tool_call_id: 'call-ooo', outcome: 'denied' }), 1);
    await expect(
      rows(panel, 'file_list'),
      'a result alone must not fabricate a row',
    ).toHaveCount(0);

    sendToolCall(
      ws,
      toolCall({ tool_call_id: 'call-ooo', tool: 'file_list', args_sha256: BASE_HASH, turn: 1 }),
      2,
    );
    await expect(rows(panel, 'file_list')).toHaveCount(1);
    await expect(rows(panel, 'file_list')).toHaveAttribute('data-outcome', 'denied');
    await expect(rows(panel, 'file_list')).toContainText(`args_sha256 ${BASE_HASH.slice(0, 12)}`);
  });

  test('unknown outcome token is refused by the decoder (E_SCHEMA_INVALID, no row mutation)', async ({ page }) => {
    const { ws, panel } = await openToolActivity(page);
    // Deliberate malformed payload: bypasses the validated builder on purpose.
    sendMalformedEvent(
      ws,
      'TOOL_RESULT',
      {
        run_id: 'run-e2e-tool',
        task_id: 'task-e2e-tool',
        tool_call_id: 'call-bad-outcome',
        outcome: 'invented',
      },
      1,
    );
    await expect(rows(panel, 'mystery_tool')).toHaveCount(0);
    await expect(page.getByRole('alert').filter({ hasText: 'E_SCHEMA_INVALID' }).first()).toBeVisible({
      timeout: 10_000,
    });
    // Socket stays usable: a later valid event from the validated builder still renders.
    sendToolCall(ws, toolCall({ tool_call_id: 'call-after-bad', tool: 'file_read' }), 2);
    await expect(rows(panel, 'file_read')).toHaveCount(1);
  });

  test('TOOL_CALL missing args_sha256 is refused (no fabricated row)', async ({ page }) => {
    const { ws, panel } = await openToolActivity(page);
    sendMalformedEvent(ws, 'TOOL_CALL', { ...toolCall(), args_sha256: undefined }, 1);
    await expect(page.getByRole('alert').filter({ hasText: 'E_SCHEMA_INVALID' }).first()).toBeVisible({
      timeout: 10_000,
    });
    await expect(rows(panel, 'file_read')).toHaveCount(0);
  });

  test('validated builder rejects an invalid payload before it can hit the socket', () => {
    // Adoption guard: malformed frames must be sent only through sendMalformedEvent.
    expect(() => toolCallFrame({ ...toolCall(), args_sha256: 'not-a-sha' }, 1)).toThrow(/args_sha256/);
    expect(() => toolCallFrame({ ...toolCall(), turn: 0 }, 1)).toThrow(/turn/);
    const frame = JSON.parse(toolCallFrame(toolCall(), 1)) as { type: string; name: string };
    expect(frame.type).toBe('EVENT');
    expect(frame.name).toBe('TOOL_CALL');
  });

  test('invalid raw bytes are refused by the client without losing the socket', async ({ page }) => {
    const { ws, panel } = await openToolActivity(page);
    sendMalformedFrame(ws, '{"schema":"hiverouter.v1","type":"EVENT","name":"TOOL_RESULT"'); // truncated JSON
    await expect(page.getByRole('alert').filter({ hasText: 'E_SCHEMA_INVALID' }).first()).toBeVisible({
      timeout: 10_000,
    });
    sendToolCall(ws, toolCall({ tool_call_id: 'call-after-raw', tool: 'file_read' }), 1);
    await expect(rows(panel, 'file_read')).toHaveCount(1);
  });
});

