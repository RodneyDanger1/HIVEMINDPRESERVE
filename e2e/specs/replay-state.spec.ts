import { expect, test } from '@playwright/test';
import {
  eventFrame,
  installMockWs,
  startDistHarness,
  type DistHarness,
} from '../helpers/replay-harness';

/**
 * W4 browser proof: HISTORY_SNAPSHOT replay is rendered and READ-ONLY.
 *
 * Deterministic by construction: the current `web/dist` build is served from a loopback
 * static server and the WebSocket is mocked at the transport boundary via
 * `page.routeWebSocket`; the test pushes the exact event sequence. No live server, no
 * provider call, no key. Assertions:
 *   1. live graph renders and clicking a live node selects it;
 *   2. HISTORY_SNAPSHOT projects valid nodes + parent→child edge, skips malformed nodes,
 *      and re-roots the orphan parent;
 *   3. replay nodes carry no `draggable`/`selectable` classes, dragging cannot enter a
 *      `dragging` state, and clicking a replay node does NOT mutate live selection.
 */
test.describe('S10 history replay — read-only browser proof', () => {
  let harness: DistHarness;

  test.beforeAll(async () => {
    harness = await startDistHarness();
  });

  test.afterAll(async () => {
    await harness.close();
  });

  test('HISTORY_SNAPSHOT renders replay nodes inertly (no drag/select/selection mutation)', async ({
    page,
  }) => {
    const ws = await installMockWs(page);
    await page.goto(harness.url);

    await expect(page.locator('[aria-label="HiveMind spatial canvas"]')).toBeVisible({
      timeout: 15_000,
    });
    // The client has completed /api/session + WS open + hello.
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });
    // The pinned chat overlay sits above the canvas centre; hide it so node clicks are
    // unambiguous (this is a test arrangement, not a product assertion).
    await page.getByRole('button', { name: /Hide the chat core/ }).click();
    await expect(page.getByRole('button', { name: /Show chat/ })).toBeVisible();

    // 1) Live graph: 2 nodes; live nodes ARE draggable/selectable (discriminator).
    const route = ws.route();
    route.send(eventFrame('RUN_CREATED', { run_id: 'run-live', status: 'running' }, 1));
    route.send(
      eventFrame(
        'NODE_SPAWN',
        {
          task_id: 'live-a',
          state: 'running',
          kind: 'superagent',
          label: 'Backend Architecture',
          parent_task_id: null,
        },
        2,
      ),
    );
    route.send(
      eventFrame(
        'NODE_SPAWN',
        {
          task_id: 'live-b',
          state: 'running',
          kind: 'worker',
          label: 'tailwind-builder',
          parent_task_id: 'live-a',
        },
        3,
      ),
    );

    await expect(page.locator('.react-flow__node')).toHaveCount(2, { timeout: 10_000 });
    await expect(page.locator('.react-flow__node.draggable')).toHaveCount(2);
    await expect(page.locator('.react-flow__node.selectable')).toHaveCount(2);

    const liveNode = page.locator('.react-flow__node').filter({ hasText: 'Backend Architecture' });
    await expect(liveNode).toHaveCount(1);
    await liveNode.click();
    const liveRegion = page.locator('#hive-canvas [aria-live="polite"]');
    await expect(liveRegion).toContainText('Backend Architecture: running');

    // 2) HISTORY_SNAPSHOT: 3 valid nodes (1 orphan parent) + 2 malformed entries skipped.
    route.send(
      eventFrame(
        'HISTORY_SNAPSHOT',
        {
          run_id: 'run-live',
          revision: 1,
          graph: {
            nodes: [
              { task_id: 'snap-1', state: 'running', rev: 1 },
              { task_id: 'snap-2', parent_task_id: 'snap-1', state: 'completed', rev: 2 },
              { task_id: 'snap-orphan', parent_task_id: 'missing-parent', state: 'healthy', rev: 3 },
              { state: 'running', rev: 4 }, // malformed: no task_id
              { task_id: 'snap-bogus', state: 'not-a-real-state', rev: 5 }, // unknown state
            ],
            edges: [],
          },
        },
        4,
      ),
    );

    await expect(page.getByText(/Replay \(read-only\)/)).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.react-flow__node')).toHaveCount(3);
    await expect(page.locator('.react-flow__edge')).toHaveCount(1);
    // Fit the replay graph so nodes are inside the viewport (replay does not auto-fit by
    // design: `fitView={!history.replayGraph}`).
    await page.locator('[aria-label="Fit swarm"]').click();
    await page.waitForTimeout(450);

    // 3) Replay nodes must be non-draggable / non-selectable (React Flow classes derive
    //    directly from node.draggable / node.selectable).
    await expect(page.locator('.react-flow__node.draggable')).toHaveCount(0);
    await expect(page.locator('.react-flow__node.selectable')).toHaveCount(0);

    const replayNode = page.locator('.react-flow__node').filter({ hasText: 'task snap-1' });
    await expect(replayNode).toHaveCount(1);

    // Clicking replay must not mutate live selection (live selection stays on live-a).
    // `force` is used because W4-F1 (reported) makes same-kind replay nodes overlap after
    // the shared-dagre-label defect; any replay-node click must still be inert.
    await replayNode.click({ force: true });
    await expect(liveRegion).toContainText('Backend Architecture: running');
    await expect(liveRegion).not.toContainText('task snap');
    await expect(page.locator('.react-flow__node.selected')).toHaveCount(0);

    // Drag attempt: a non-draggable node must never enter React Flow's `dragging` state.
    const box = await replayNode.boundingBox();
    expect(box).not.toBeNull();
    if (box !== null) {
      const centerX = box.x + box.width / 2;
      const centerY = box.y + box.height / 2;
      await page.mouse.move(centerX, centerY);
      await page.mouse.down();
      await page.mouse.move(centerX + 120, centerY + 60, { steps: 8 });
      await page.mouse.up();
    }
    await expect(page.locator('.react-flow__node.dragging')).toHaveCount(0);
    await expect(liveRegion).toContainText('Backend Architecture: running');
    await expect(page.locator('.react-flow__node')).toHaveCount(3);
    await expect(page.getByText(/Replay \(read-only\)/)).toBeVisible();
  });
});
