import { test, expect } from '@playwright/test';
import { randomUUID } from 'node:crypto';
import {
  findServerBinary,
  openWsProbe,
  paidGate,
  redactSecrets,
  serverSkipReason,
  startLiveServer,
  type LiveServer,
  type ProbeEnvelope,
} from '../helpers/live-server';
import { captureFrames, shot, writeEvidence, type FrameRecord } from '../helpers/evidence';

/**
 * RUN-3+ live tool-turn proof (real Rust server, real provider, PAID-GATED).
 *
 * Lanes and gates (checked BEFORE any server start; a missing gate is a named
 * `test.skip`, never a fake pass and never a spend):
 * - `paidGate('tool-turn')` requires the independent `LIVE_TOOL_TURN=1` plus
 *   `DEEPSEEK_API_KEY` and a server binary (`LIVE_ONE_PROMPT` is NOT sufficient).
 * - The approval tests additionally require `HIVEROUTER_SIDE_EFFECTS=1`: the
 *   suspension/side-effect engine wiring is a separate capability from the
 *   read-only tool turn. When it is absent they skip with the named
 *   `SIDE_EFFECTS_NOT_ENABLED` reason instead of failing or pretending success.
 *
 * What each test can prove (and only that):
 * - T1 (no gate): the tool-turn lane is fail-closed with a named reason; no server
 *   is started and no prompt is submitted.
 * - T2 (paid): one live read-only `file_list` tool turn renders exactly one
 *   `li[data-outcome="ok"]` row inside `[data-testid="agents-tool-activity"]`, and
 *   the WS order is TOOL_CALL -> TOOL_RESULT ok -> REPLY_DONE -> RUN_STATUS completed.
 * - T3/T4 (paid + side-effects flag): the approval dialog (`/file_write/`) sends
 *   exactly one `APPROVAL_RESOLVE` (approve/deny, exact approval id) through the
 *   app socket, then one terminal `TOOL_RESULT`; no success is claimed before the
 *   click. The server's temp workspace is not exposed by the helper, so the file
 *   *content* is NOT asserted here (see the run receipt's assumptions).
 *
 * No mock, no synthetic event: every assertion reads real server frames / real UI.
 */

const APPROVAL_TOOL = 'file_write';
const APPROVE_LABEL = `Approve tool call for ${APPROVAL_TOOL}`;
const DENY_LABEL = `Deny tool call for ${APPROVAL_TOOL}`;
const SIDE_EFFECTS_ENV = 'HIVEROUTER_SIDE_EFFECTS';

function serverSkip(): string | null {
  return serverSkipReason();
}

function stringField(value: unknown): string {
  return typeof value === 'string' ? value : '';
}

function runIdOf(frame: ProbeEnvelope): string {
  return stringField(frame.payload?.run_id) || stringField(frame.run_id);
}

function seqOf(frame: ProbeEnvelope): number {
  return typeof frame.seq === 'number' ? frame.seq : -1;
}

function numbered(frames: ProbeEnvelope[]): ProbeEnvelope[] {
  return frames.filter((frame) => typeof frame.seq === 'number').sort((a, b) => seqOf(a) - seqOf(b));
}

/** Parsed COMMAND frames sent by the app page socket (the UI's own network). */
function sentCommands(frames: FrameRecord[]): Array<Record<string, unknown>> {
  const commands: Array<Record<string, unknown>> = [];
  for (const frame of frames) {
    if (frame.dir !== 'sent') continue;
    try {
      const parsed = JSON.parse(frame.payload) as Record<string, unknown>;
      // A COMMAND envelope always has a frozen command name; events do not travel
      // in the sent direction, so the name filter is sufficient and does not
      // depend on optional identity fields being present in a partial capture.
      if (typeof parsed.name === 'string') commands.push(parsed);
    } catch {
      // Non-JSON or partial frames are not commands; evidence keeps the raw bytes.
    }
  }
  return commands;
}

/** APPROVAL_RESOLVE commands observed on the app socket (spec-local, live lane). */
function approvalResolveCommands(frames: FrameRecord[]): Array<Record<string, unknown>> {
  return sentCommands(frames).filter((command) => command.name === 'APPROVAL_RESOLVE');
}

/** Named skip for the paid approval lane when the side-effect flag is absent. */
function sideEffectsSkipReason(): string | null {
  if (process.env[SIDE_EFFECTS_ENV] === '1') return null;
  return (
    `SIDE_EFFECTS_NOT_ENABLED: ${SIDE_EFFECTS_ENV} is not 1; live approve/deny execution is ` +
    'not wired/enabled in this build. The read-only tool-turn proof stays available; the ' +
    'approval lane must not fake a pass. Run with the side-effect wiring committed and set ' +
    `${SIDE_EFFECTS_ENV}=1 explicitly.`
  );
}

test.describe('live tool turn (real server, paid gate)', () => {
  test.setTimeout(420_000);

  let live: LiveServer | null = null;

  test.afterEach(async () => {
    if (live !== null) {
      const stopped = await live.stop();
      live = null;
      expect(redactSecrets(stopped.logs.join('\n'))).not.toMatch(/sk-[A-Za-z0-9_-]{8,}/);
    }
  });

  test('gate negative: tool-turn paid lane refuses without LIVE_TOOL_TURN=1 (no server, no spend)', async () => {
    // This must hold even when the operator's shell carries other LIVE flags: the
    // tool-turn lane is independent, so a missing LIVE_TOOL_TURN is always a skip.
    const binary = findServerBinary();
    const previous = process.env.LIVE_TOOL_TURN;
    delete process.env.LIVE_TOOL_TURN;
    try {
      const gate = paidGate('tool-turn');
      expect(gate.enabled, 'tool-turn gate must fail closed without LIVE_TOOL_TURN=1').toBe(false);
      expect(gate.reason.length, 'a named skip reason is mandatory').toBeGreaterThan(0);
      if (binary !== null) {
        expect(gate.reason).toContain('LIVE_TOOL_TURN');
      }
    } finally {
      if (previous === undefined) delete process.env.LIVE_TOOL_TURN;
      else process.env.LIVE_TOOL_TURN = previous;
    }
    // Deliberately no startLiveServer()/page/submit: the gate check itself is the proof.
  });

  test('live read-only tool turn: TOOL_CALL -> TOOL_RESULT ok -> RUN_STATUS completed (paid)', async ({
    page,
  }, testInfo) => {
    const missing = serverSkip();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    const gate = paidGate('tool-turn');
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;

    const pageFrames = captureFrames(page);
    live = await startLiveServer({ withProviderKey: true });
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      text:
        'Use the file_list tool exactly once on the workspace root (no other tool), then reply ' +
        'exactly HIVE-TOOL-OK and nothing else.',
      project_id: randomUUID(),
    });

    const created = await probe.waitFor((frame) => frame.name === 'RUN_CREATED', 60_000, 'RUN_CREATED');
    const runId = stringField(created.payload?.run_id);
    expect(runId.length, 'RUN_CREATED carries a run id').toBeGreaterThan(0);

    const toolCall = await probe.waitFor(
      (frame) => frame.name === 'TOOL_CALL' && runIdOf(frame) === runId,
      180_000,
      'TOOL_CALL for the run',
    );
    expect(stringField(toolCall.payload?.tool), `TOOL_CALL tool=${JSON.stringify(toolCall)}`).toBe(
      'file_list',
    );
    const toolResult = await probe.waitFor(
      (frame) =>
        frame.name === 'TOOL_RESULT' && runIdOf(frame) === runId && frame.payload?.outcome === 'ok',
      180_000,
      'TOOL_RESULT ok for the run',
    );
    const done = await probe.waitFor(
      (frame) => frame.name === 'REPLY_DONE' && runIdOf(frame) === runId,
      180_000,
      'REPLY_DONE for the run',
    );
    const completed = await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' && runIdOf(frame) === runId && frame.payload?.status === 'completed',
      60_000,
      'RUN_STATUS completed for the run',
    );

    const frames = await probe.frames();
    const callSeq = seqOf(toolCall);
    const resultSeq = seqOf(toolResult);
    const doneSeq = seqOf(done);
    const completedSeq = seqOf(completed);
    expect(callSeq, 'TOOL_CALL has a server seq').toBeGreaterThan(0);
    expect(resultSeq, 'TOOL_RESULT has a server seq').toBeGreaterThan(callSeq);
    expect(doneSeq, 'REPLY_DONE has a server seq').toBeGreaterThan(resultSeq);
    expect(completedSeq, 'RUN_STATUS completed is last').toBeGreaterThan(doneSeq);

    const results = frames.filter((frame) => frame.name === 'TOOL_RESULT' && runIdOf(frame) === runId);
    expect(results.filter((frame) => frame.payload?.outcome === 'ok')).toHaveLength(1);
    expect(results.filter((frame) => frame.payload?.outcome !== 'ok')).toEqual([]);
    expect(
      frames.filter(
        (frame) => frame.name === 'RUN_STATUS' && runIdOf(frame) === runId && frame.payload?.status === 'failed',
      ),
      'no failed terminal for a successful tool turn',
    ).toEqual([]);

    // Real UI: the Agents view hosts the tool-activity region and settles one ok row.
    const nav = page.locator('nav[aria-label="Primary navigation"]');
    await nav.getByRole('button', { name: 'Agents', exact: true }).click();
    const host = page.getByTestId('agents-tool-activity');
    await expect(host, 'AgentsPanel must host the tool-activity region').toBeVisible({ timeout: 15_000 });
    const panel = host.locator('section[aria-label="Tool activity"]');
    await expect(panel).toBeVisible();
    const row = panel.locator('li[data-outcome="ok"]').filter({ hasText: 'file_list' });
    await expect(row, 'exactly one ok file_list row').toHaveCount(1);
    await expect(page.locator('#composer')).toBeEnabled();

    const screenshot = await shot(page, 'live-tool-turn', testInfo);
    writeEvidence(
      'live-tool-turn.json',
      JSON.parse(
        redactSecrets(
          JSON.stringify({
            captured_at: new Date().toISOString(),
            release_binary: process.env.E2E_SERVER_BIN ?? null,
            binary: live.binary,
            run_id: runId,
            gate: { mode: 'tool-turn', flags: ['LIVE_TOOL_TURN', 'DEEPSEEK_API_KEY'] },
            prompt_count: 1,
            tool_call: { tool: stringField(toolCall.payload?.tool), seq: callSeq },
            tool_result: { outcome: stringField(toolResult.payload?.outcome), seq: resultSeq },
            terminal: { name: completed.name, status: stringField(completed.payload?.status), seq: completedSeq },
            probe_frames: frames,
            app_socket_frames: pageFrames.frames,
            screenshot,
          }),
        ),
      ) as unknown,
    );
    await probe.close();
  });

  test('live approval approve: one APPROVAL_RESOLVE(approve), then one ok TOOL_RESULT (paid, side effects required)', async ({
    page,
  }, testInfo) => {
    const missing = serverSkip();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    const gate = paidGate('tool-turn');
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;
    const sideSkip = sideEffectsSkipReason();
    test.skip(sideSkip !== null, sideSkip ?? '');
    if (sideSkip !== null) return;

    const pageFrames = captureFrames(page);
    live = await startLiveServer({ withProviderKey: true });
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      text:
        'Use the file_write tool exactly once to create approval-proof.txt with the exact content ' +
        'HIVE-APPROVAL-OK, then reply exactly HIVE-APPROVAL-OK and nothing else.',
      project_id: randomUUID(),
    });

    const created = await probe.waitFor((frame) => frame.name === 'RUN_CREATED', 60_000, 'RUN_CREATED');
    const runId = stringField(created.payload?.run_id);
    expect(runId.length).toBeGreaterThan(0);
    const request = await probe.waitFor(
      (frame) => frame.name === 'APPROVAL_REQUEST' && runIdOf(frame) === runId,
      180_000,
      'APPROVAL_REQUEST for the run',
    );
    const approvalId = stringField(request.payload?.approval_id);
    expect(approvalId.length, 'APPROVAL_REQUEST carries the approval id').toBeGreaterThan(0);

    // No optimistic success: nothing may execute before the operator gesture.
    const before = await probe.frames();
    expect(
      before.filter((frame) => frame.name === 'TOOL_RESULT' && frame.payload?.outcome === 'ok'),
      'no ok result before approval',
    ).toEqual([]);

    const dialog = page.getByRole('dialog', { name: /file_write/ });
    await expect(dialog, 'live side-effect call must show the approval dialog').toBeVisible({
      timeout: 30_000,
    });
    await dialog.getByRole('button', { name: APPROVE_LABEL }).click();

    await expect
      .poll(async () => approvalResolveCommands(pageFrames.frames).length, { timeout: 20_000 })
      .toBeGreaterThanOrEqual(1);
    const commands = approvalResolveCommands(pageFrames.frames);
    expect(commands, 'exactly one APPROVAL_RESOLVE on the app socket').toHaveLength(1);
    expect(commands[0]?.approval_id).toBe(approvalId);
    expect(commands[0]?.decision).toBe('approve');

    await probe.waitFor(
      (frame) => frame.name === 'APPROVAL_RESOLVED' && runIdOf(frame) === runId,
      30_000,
      'APPROVAL_RESOLVED for the run',
    );
    const okResult = await probe.waitFor(
      (frame) =>
        frame.name === 'TOOL_RESULT' && runIdOf(frame) === runId && frame.payload?.outcome === 'ok',
      180_000,
      'TOOL_RESULT ok after approval',
    );
    await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' && runIdOf(frame) === runId && frame.payload?.status === 'completed',
      60_000,
      'RUN_STATUS completed after approval',
    );

    const after = await probe.frames();
    expect(
      after.filter((frame) => frame.name === 'TOOL_RESULT' && frame.payload?.outcome === 'ok'),
      'exactly one ok execution after approval',
    ).toHaveLength(1);
    expect(
      after.filter(
        (frame) => frame.name === 'TOOL_RESULT' && frame.payload?.outcome === 'denied',
      ),
      'approved call is not denied',
    ).toEqual([]);

    await page
      .locator('nav[aria-label="Primary navigation"]')
      .getByRole('button', { name: 'Agents', exact: true })
      .click();
    const panel = page.getByTestId('agents-tool-activity').locator('section[aria-label="Tool activity"]');
    await expect(panel.locator('li[data-outcome="ok"]').filter({ hasText: APPROVAL_TOOL })).toHaveCount(1);

    const screenshot = await shot(page, 'live-tool-approval-approve', testInfo);
    writeEvidence(
      'live-tool-approval-approve.json',
      JSON.parse(
        redactSecrets(
          JSON.stringify({
            captured_at: new Date().toISOString(),
            release_binary: process.env.E2E_SERVER_BIN ?? null,
            run_id: runId,
            approval_id: approvalId,
            gate: { mode: 'tool-turn', flags: ['LIVE_TOOL_TURN', 'DEEPSEEK_API_KEY', SIDE_EFFECTS_ENV] },
            resolve_command: commands[0] ?? null,
            ok_result: { seq: seqOf(okResult), outcome: stringField(okResult.payload?.outcome) },
            probe_frames: after,
            app_socket_frames: pageFrames.frames,
            screenshot,
          }),
        ),
      ) as unknown,
    );
    await probe.close();
  });

  test('live approval deny: one APPROVAL_RESOLVE(deny), denied TOOL_RESULT, no ok execution (paid, side effects required)', async ({
    page,
  }, testInfo) => {
    const missing = serverSkip();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    const gate = paidGate('tool-turn');
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;
    const sideSkip = sideEffectsSkipReason();
    test.skip(sideSkip !== null, sideSkip ?? '');
    if (sideSkip !== null) return;

    const pageFrames = captureFrames(page);
    live = await startLiveServer({ withProviderKey: true });
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      text:
        'Use the file_write tool exactly once to create approval-proof.txt with content ' +
        'HIVE-APPROVAL-OK, then reply exactly HIVE-APPROVAL-OK and nothing else.',
      project_id: randomUUID(),
    });
    const created = await probe.waitFor((frame) => frame.name === 'RUN_CREATED', 60_000, 'RUN_CREATED');
    const runId = stringField(created.payload?.run_id);
    const request = await probe.waitFor(
      (frame) => frame.name === 'APPROVAL_REQUEST' && runIdOf(frame) === runId,
      180_000,
      'APPROVAL_REQUEST for the run',
    );
    const approvalId = stringField(request.payload?.approval_id);
    const dialog = page.getByRole('dialog', { name: /file_write/ });
    await expect(dialog).toBeVisible({ timeout: 30_000 });
    await dialog.getByRole('button', { name: DENY_LABEL }).click();

    await expect
      .poll(async () => approvalResolveCommands(pageFrames.frames).length, { timeout: 20_000 })
      .toBeGreaterThanOrEqual(1);
    const commands = approvalResolveCommands(pageFrames.frames);
    expect(commands).toHaveLength(1);
    expect(commands[0]?.approval_id).toBe(approvalId);
    expect(commands[0]?.decision).toBe('deny');

    const denied = await probe.waitFor(
      (frame) =>
        frame.name === 'TOOL_RESULT' && runIdOf(frame) === runId && frame.payload?.outcome === 'denied',
      180_000,
      'TOOL_RESULT denied after the deny gesture',
    );
    await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' && runIdOf(frame) === runId && frame.payload?.status === 'completed',
      60_000,
      'RUN_STATUS completed (deny continues the loop)',
    );
    const frames = await probe.frames();
    expect(
      frames.filter((frame) => frame.name === 'TOOL_RESULT' && frame.payload?.outcome === 'ok'),
      'denied approval must never produce an ok execution',
    ).toEqual([]);
    expect(
      frames.filter((frame) => frame.name === 'TOOL_RESULT' && frame.payload?.outcome === 'denied'),
      'exactly one denied tool result',
    ).toHaveLength(1);

    await page
      .locator('nav[aria-label="Primary navigation"]')
      .getByRole('button', { name: 'Agents', exact: true })
      .click();
    const panel = page.getByTestId('agents-tool-activity').locator('section[aria-label="Tool activity"]');
    await expect(panel.locator('li[data-outcome="denied"]').filter({ hasText: APPROVAL_TOOL })).toHaveCount(1);

    const screenshot = await shot(page, 'live-tool-approval-deny', testInfo);
    writeEvidence(
      'live-tool-approval-deny.json',
      JSON.parse(
        redactSecrets(
          JSON.stringify({
            captured_at: new Date().toISOString(),
            release_binary: process.env.E2E_SERVER_BIN ?? null,
            run_id: runId,
            approval_id: approvalId,
            gate: { mode: 'tool-turn', flags: ['LIVE_TOOL_TURN', 'DEEPSEEK_API_KEY', SIDE_EFFECTS_ENV] },
            resolve_command: commands[0] ?? null,
            denied_result: { seq: seqOf(denied), outcome: stringField(denied.payload?.outcome) },
            probe_frames: frames,
            app_socket_frames: pageFrames.frames,
            screenshot,
          }),
        ),
      ) as unknown,
    );
    await probe.close();
  });
});
