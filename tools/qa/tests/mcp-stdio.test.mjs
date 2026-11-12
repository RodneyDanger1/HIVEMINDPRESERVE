import test from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const serverPath = path.resolve(here, '..', '..', 'hivemind-mcp.mjs');

function rpc(child, message) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('MCP timeout')), 5_000);
    child.stdout.once('data', (chunk) => {
      clearTimeout(timer);
      try {
        resolve(JSON.parse(String(chunk).trim().split('\n')[0]));
      } catch (error) {
        reject(error);
      }
    });
    child.stdin.write(`${JSON.stringify(message)}\n`);
  });
}

test('MCP initializes and lists only read-only HiveMind tools', async () => {
  const child = spawn(process.execPath, [serverPath], { stdio: ['pipe', 'pipe', 'pipe'] });
  try {
    const init = await rpc(child, {
      jsonrpc: '2.0',
      id: 1,
      method: 'initialize',
      params: { protocolVersion: '2024-11-05', capabilities: {}, clientInfo: { name: 'test', version: '1' } },
    });
    assert.equal(init.result.serverInfo.name, 'hivemind');
    assert.equal(init.result.capabilities.tools !== undefined, true);

    const list = await rpc(child, { jsonrpc: '2.0', id: 2, method: 'tools/list' });
    assert.deepEqual(list.result.tools.map((tool) => tool.name).sort(), [
      'hivemind_jobs',
      'hivemind_runs',
      'hivemind_status',
    ]);
    for (const tool of list.result.tools) {
      assert.equal(tool.inputSchema.type, 'object');
      assert.ok(tool.description.length > 0);
    }
  } finally {
    child.stdin.end();
    child.kill();
  }
});
