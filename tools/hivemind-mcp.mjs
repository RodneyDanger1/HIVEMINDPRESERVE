#!/usr/bin/env node
/**
 * HiveMind MCP stdio server (CAP-8).
 *
 * Read-only tools over the local HiveMind HTTP API:
 *   - hivemind_status  GET /health
 *   - hivemind_runs    GET /api/history/runs
 *   - hivemind_jobs    GET /api/jobs
 *
 * Protocol: newline-delimited JSON-RPC 2.0 on stdin/stdout. stderr is for
 * diagnostics only. The server never reads, prints, or stores the provider key.
 * Mutation is deliberately absent: approval-gated actions belong to the CLI/UI
 * where the human decision is recorded, not to an MCP client.
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';

const CLI_VERSION = '1.0.0';
const DATA_DIR = path.resolve(
  process.env.HIVEROUTER_DATA_DIR ||
    path.join(process.env.LOCALAPPDATA || path.join(os.homedir(), 'AppData', 'Local'), 'HiveMind'),
);
function cliStateUrl() {
  try {
    const state = JSON.parse(fs.readFileSync(path.join(DATA_DIR, 'agent-cli-state.json'), 'utf8'));
    return typeof state.url === 'string' ? state.url : null;
  } catch {
    return null;
  }
}
function launchToken() {
  try {
    const token = fs.readFileSync(path.join(DATA_DIR, 'launch.token'), 'utf8').trim();
    return token.length > 0 ? token : null;
  } catch {
    return null;
  }
}
const DEFAULT_URL = process.env.HIVEROUTER_URL || cliStateUrl() || 'http://127.0.0.1:8099';
const TOOLS = [
  {
    name: 'hivemind_status',
    description: 'Read the local HiveMind server health and schema version.',
    inputSchema: { type: 'object', properties: {}, additionalProperties: false },
  },
  {
    name: 'hivemind_runs',
    description: 'List committed HiveMind runs, newest first.',
    inputSchema: {
      type: 'object',
      properties: { limit: { type: 'integer', minimum: 1, maximum: 100 } },
      additionalProperties: false,
    },
  },
  {
    name: 'hivemind_jobs',
    description: 'List live and persisted HiveMind background jobs, newest first.',
    inputSchema: {
      type: 'object',
      properties: { limit: { type: 'integer', minimum: 1, maximum: 200 } },
      additionalProperties: false,
    },
  },
];

function send(message) {
  process.stdout.write(`${JSON.stringify(message)}\n`);
}

function response(id, result) {
  send({ jsonrpc: '2.0', id, result });
}

function error(id, code, message) {
  send({ jsonrpc: '2.0', id, error: { code, message } });
}

async function localFetch(pathname) {
  const base = DEFAULT_URL.replace(/\/$/, '');
  const launch = launchToken();
  const sessionHeaders = { Accept: 'application/json' };
  if (launch) sessionHeaders['x-hivemind-launch'] = launch;
  const sessionResponse = await fetch(`${base}/api/session`, {
    headers: sessionHeaders,
  });
  if (!sessionResponse.ok) {
    throw new Error(`session unavailable (HTTP ${sessionResponse.status})`);
  }
  const session = await sessionResponse.json();
  const headers = { Accept: 'application/json' };
  if (session.token) headers.Authorization = `Bearer ${session.token}`;
  if (launch) headers['x-hivemind-launch'] = launch;
  const response = await fetch(`${base}${pathname}`, { headers });
  if (!response.ok) {
    throw new Error(`${pathname} returned HTTP ${response.status}`);
  }
  return response.json();
}

async function callTool(name, args) {
  const limit = Number.isInteger(args?.limit) ? args.limit : null;
  if (name === 'hivemind_status') {
    return await localFetch('/health');
  }
  if (name === 'hivemind_runs') {
    const body = await localFetch('/api/history/runs');
    return limit === null ? body : { ...body, runs: (body.runs || []).slice(0, limit) };
  }
  if (name === 'hivemind_jobs') {
    const body = await localFetch('/api/jobs');
    return limit === null ? body : { ...body, jobs: (body.jobs || []).slice(0, limit) };
  }
  throw new Error(`unknown tool: ${name}`);
}

async function handle(request) {
  const id = request?.id;
  const method = request?.method;
  if (request?.jsonrpc !== '2.0' || typeof method !== 'string') {
    if (id !== undefined) error(id, -32600, 'invalid JSON-RPC request');
    return;
  }
  if (method === 'initialize') {
    response(id, {
      protocolVersion: '2024-11-05',
      capabilities: { tools: {} },
      serverInfo: { name: 'hivemind', version: CLI_VERSION },
    });
    return;
  }
  if (method === 'notifications/initialized') return;
  if (method === 'ping') {
    response(id, {});
    return;
  }
  if (method === 'tools/list') {
    response(id, { tools: TOOLS });
    return;
  }
  if (method === 'tools/call') {
    const name = request.params?.name;
    const args = request.params?.arguments ?? {};
    try {
      const result = await callTool(name, args);
      response(id, {
        content: [{ type: 'text', text: JSON.stringify(result) }],
        isError: false,
      });
    } catch (failure) {
      response(id, {
        content: [{ type: 'text', text: JSON.stringify({ error: String(failure.message || failure) }) }],
        isError: true,
      });
    }
    return;
  }
  if (id !== undefined) error(id, -32601, `method not found: ${method}`);
}

let buffer = '';
let pending = 0;
let ending = false;
function finishIfDrained() {
  if (ending && pending === 0) process.exit(0);
}
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => {
  buffer += chunk;
  for (;;) {
    const newline = buffer.indexOf('\n');
    if (newline < 0) break;
    const line = buffer.slice(0, newline).trim();
    buffer = buffer.slice(newline + 1);
    if (!line) continue;
    let request;
    try {
      request = JSON.parse(line);
    } catch {
      error(null, -32700, 'parse error');
      continue;
    }
    pending += 1;
    void handle(request).finally(() => {
      pending -= 1;
      finishIfDrained();
    });
  }
});
// A client may close stdin immediately after sending tools/call. Do not exit
// before the async HTTP response is written; drain pending requests first.
process.stdin.on('end', () => {
  ending = true;
  finishIfDrained();
});
