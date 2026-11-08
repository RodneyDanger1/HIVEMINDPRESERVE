#!/usr/bin/env node
/**
 * HiveMind agent-native CLI.
 *
 * One command surface for humans, AI agents, and orchestration systems:
 * - stdout: JSON envelope when piped / --json / --format json
 * - stderr: human diagnostics
 * - exit codes: stable classes (see EXIT)
 * - no provider key is ever read, printed, or stored by this CLI
 *
 * The Rust server remains the sole authority. This file only validates input,
 * starts/stops the local server, and maps HTTP/WS operations to bounded output.
 */
import fs from 'node:fs';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { spawn, spawnSync } from 'node:child_process';
import { createHash, randomBytes, randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const CLI_VERSION = '1.0.0';
const CLI_SCHEMA = 'hivemind.cli.v1';
const SERVER_SCHEMA = 'hiverouter.v1';
const DEFAULT_HOST = '127.0.0.1';
const DEFAULT_PORT = 8099;
const MAX_PROMPT_CHARS = 8_000;
const MAX_OUTPUT_ITEMS = 1_000;

const EXIT = {
  OK: 0,
  USAGE: 2,
  REFUSAL: 3,
  INTERNAL: 4,
};
// Semantic aliases keep call sites readable while the process contract stays
// identical to the shipped Rust CLI (0 ok / 2 usage / 3 refusal / 4 internal).
const EXIT_NOT_RUNNING = EXIT.REFUSAL;
const EXIT_AUTH = EXIT.REFUSAL;
const EXIT_TIMEOUT = EXIT.REFUSAL;
const EXIT_UNSAFE = EXIT.USAGE;

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '..');
const LOCAL_APP_DATA = process.env.LOCALAPPDATA || path.join(os.homedir(), 'AppData', 'Local');
const DEFAULT_DATA_DIR = process.env.HIVEROUTER_DATA_DIR || path.join(LOCAL_APP_DATA, 'HiveMind');
const HUMAN = Boolean(process.stdout.isTTY) && process.env.NO_COLOR === undefined;

// ---------------------------------------------------------------------------
// Command registry: the single source of truth for discovery, help and schema.
// ---------------------------------------------------------------------------

const ARG = (name, type, required, description, extra = {}) => ({
  name,
  type,
  required,
  description,
  ...extra,
});

const COMMANDS = {
  start: {
    summary: 'Start the local HiveMind server headlessly and wait for health.',
    safety: 'mutating',
    transport: 'process',
    args: [
      ARG('port', 'number', false, 'Loopback port (default 8099).'),
      ARG('bind', 'string', false, 'Bind address; must be loopback.'),
      ARG('data-dir', 'path', false, 'HiveMind data directory.'),
      ARG('db', 'path', false, 'SQLite database file (default <data-dir>/hiverouter.db).'),
      ARG('workspace-root', 'path', false, 'Workspace root for file tools.'),
      ARG('exe', 'path', false, 'Server executable path.'),
      ARG('timeout-ms', 'number', false, 'Health wait limit (default 60000).'),
      ARG('no-wait', 'boolean', false, 'Return after spawn without waiting for health.'),
      ARG('open', 'boolean', false, 'Open the UI after health becomes ready.'),
    ],
  },
  stop: {
    summary: 'Stop the CLI-owned local server. Requires --yes.',
    safety: 'destructive',
    transport: 'process',
    args: [
      ARG('pid', 'number', false, 'PID to stop (must match the CLI state by default).'),
      ARG('force', 'boolean', false, 'Force kill if graceful stop fails.'),
      ARG('timeout-ms', 'number', false, 'Shutdown wait limit (default 10000).'),
    ],
  },
  restart: {
    summary: 'Stop then start the local server headlessly.',
    safety: 'destructive',
    transport: 'process',
    args: [
      ARG('port', 'number', false, 'Loopback port.'),
      ARG('timeout-ms', 'number', false, 'Health wait limit.'),
    ],
  },
  status: {
    summary: 'Show CLI state, server health and version.',
    safety: 'read',
    transport: 'process/http',
    args: [],
  },
  doctor: {
    summary: 'Check executable, state, loopback health, key presence and UI assets.',
    safety: 'read',
    transport: 'process/http',
    args: [],
  },
  open: {
    summary: 'Open the HiveMind UI in Edge/Chrome app mode (human action).',
    safety: 'mutating',
    transport: 'process',
    args: [
      ARG('browser', 'path', false, 'Browser executable path.'),
      ARG('no-app', 'boolean', false, 'Open a normal tab instead of app mode.'),
    ],
  },
  logs: {
    summary: 'Tail the server stdout/stderr logs from the data directory.',
    safety: 'read',
    transport: 'process',
    args: [
      ARG('lines', 'number', false, 'Lines per log (default 40, max 500).'),
      ARG('stream', 'string', false, 'stdout, stderr or both (default both).'),
    ],
  },

  health: {
    summary: 'Read GET /health (status, checks, schema, transport counters).',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  diagnose: {
    summary: 'Read GET /diagnose (census/refusal/provider truth).',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  runtime: {
    summary: 'Read effective capacity, provider config, and active-run status.',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  runs: {
    summary: 'List committed runs, newest first.',
    safety: 'read',
    transport: 'http',
    args: [ARG('limit', 'number', false, 'Max runs (default 20, max 100).')],
  },
  revisions: {
    summary: 'List committed revisions for one run.',
    safety: 'read',
    transport: 'http',
    args: [ARG('run', 'uuid', true, 'Run id.')],
  },
  state: {
    summary: 'Read one committed graph revision (latest if --revision omitted).',
    safety: 'read',
    transport: 'http',
    args: [
      ARG('run', 'uuid', true, 'Run id.'),
      ARG('revision', 'number', false, 'Committed revision number.'),
    ],
  },
  messages: {
    summary: 'Read the persisted chat transcript window for one run.',
    safety: 'read',
    transport: 'http',
    args: [
      ARG('run', 'uuid', true, 'Run id.'),
      ARG('limit', 'number', false, 'Max messages (default 50, max 1000).'),
    ],
  },
  'agent-messages': {
    summary: 'Read durable A2A messages for one run, optionally one recipient task.',
    safety: 'read',
    transport: 'http',
    args: [
      ARG('run', 'uuid', true, 'Run id.'),
      ARG('task', 'uuid', false, 'Optional recipient task id.'),
    ],
  },
  roster: {
    summary: 'Read the authoritative agent roster for one run.',
    safety: 'read',
    transport: 'http',
    args: [ARG('run', 'uuid', true, 'Run id.')],
  },
  approvals: {
    summary: 'Read pending approvals (optionally for one run); never consumes a nonce.',
    safety: 'read',
    transport: 'http',
    args: [ARG('run', 'uuid', false, 'Optional run id.')],
  },
  templates: {
    summary: 'Read the template catalog (ids, names, inputs, licence/source).',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  workspace: {
    summary: 'Read the effective data dir, workspace root and database path.',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  'active-run': {
    summary: 'Read the live engine slot and latest non-terminal persisted run.',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  profiles: {
    summary: 'List agent profiles (built-in and imported).',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  session: {
    summary: 'Read the session schema/last_seq without exposing the token.',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  'profile-import': {
    summary: 'Import an agent profile from a JSON file.',
    safety: 'mutating',
    transport: 'http-mutation',
    args: [ARG('file', 'path', true, 'JSON profile file.')],
  },
  'profile-delete': {
    summary: 'Delete an imported profile (restores a built-in if it exists). Requires --yes.',
    safety: 'destructive',
    transport: 'http-mutation',
    args: [ARG('id', 'string', true, 'Profile id.')],
  },
  'memory-clear': {
    summary: 'Clear durable memory for one agent key. Requires --yes.',
    safety: 'destructive',
    transport: 'http-mutation',
    args: [ARG('agent', 'string', true, 'Agent/persona key.')],
  },
  memory: {
    summary: 'List durable role memory, or one run’s branch-scoped memory rows.',
    safety: 'read',
    transport: 'http',
    args: [ARG('run', 'uuid', false, 'Optional run id for branch-scoped memory.')],
  },
  projects: {
    summary: 'List projects known to the local server.',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  models: {
    summary: 'List provider model metadata; keyless returns E_KEY_MISSING.',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  balance: {
    summary: 'Read provider balance; keyless returns E_KEY_MISSING.',
    safety: 'read',
    transport: 'http',
    args: [],
  },
  usage: {
    summary: 'Read measured usage for one run (nulls stay null).',
    safety: 'read',
    transport: 'http',
    args: [ARG('run', 'uuid', true, 'Run id.')],
  },

  submit: {
    summary: 'Submit a prompt (keyless refuses E_KEY_MISSING before a run is created).',
    safety: 'mutating',
    transport: 'ws',
    args: [
      ARG('prompt', 'string', false, 'Prompt text (1..8000 chars).'),
      ARG('prompt-file', 'path', false, 'Read prompt text from a UTF-8 file.'),
      ARG('project', 'string', false, 'Project id (default workspace).'),
      ARG('wait', 'boolean', false, 'Watch until the run is terminal.'),
      ARG('timeout-ms', 'number', false, 'Wait limit (default 120000).'),
    ],
  },
  run: {
    summary: 'Submit a prompt and follow the run to a terminal state.',
    safety: 'mutating',
    transport: 'ws',
    args: [
      ARG('prompt', 'string', false, 'Prompt text (1..8000 chars).'),
      ARG('prompt-file', 'path', false, 'Read prompt text from a UTF-8 file.'),
      ARG('project', 'string', false, 'Project id (default workspace).'),
      ARG('timeout-ms', 'number', false, 'Wait limit (default 180000).'),
    ],
  },
  cancel: {
    summary: 'Cancel a run. Requires --yes.',
    safety: 'destructive',
    transport: 'ws',
    args: [ARG('run', 'uuid', true, 'Run id to cancel.')],
  },
  kill: {
    summary: 'Kill one worker task. Requires --yes.',
    safety: 'destructive',
    transport: 'ws',
    args: [ARG('task', 'uuid', true, 'Task id to kill.')],
  },
  rewire: {
    summary: 'Move one task under a new parent (optimistic CAS).',
    safety: 'mutating',
    transport: 'ws',
    args: [
      ARG('task', 'uuid', true, 'Task id to move.'),
      ARG('parent', 'uuid', true, 'New parent task id.'),
      ARG('expected-rev', 'number', true, 'Committed graph node revision observed by the caller.'),
    ],
  },
  'spawn-subhive': {
    summary: 'Spawn a sub-hive under a parent task.',
    safety: 'mutating',
    transport: 'ws',
    args: [
      ARG('parent', 'uuid', true, 'Parent task id.'),
      ARG('role', 'string', true, 'Agent role/tier name.'),
    ],
  },
  approve: {
    summary: 'Resolve one approval as approve. Requires --yes and the single-use nonce.',
    safety: 'destructive',
    transport: 'ws',
    args: [
      ARG('approval', 'uuid', true, 'Approval id.'),
      ARG('nonce', 'string', true, 'Single-use approval nonce.'),
      ARG('reason', 'string', false, 'Optional reason.'),
    ],
  },
  deny: {
    summary: 'Resolve one approval as deny. Requires --yes and the single-use nonce.',
    safety: 'destructive',
    transport: 'ws',
    args: [
      ARG('approval', 'uuid', true, 'Approval id.'),
      ARG('nonce', 'string', true, 'Single-use approval nonce.'),
      ARG('reason', 'string', false, 'Optional reason.'),
    ],
  },
  template: {
    summary: 'Insert a template as planned tasks.',
    safety: 'mutating',
    transport: 'ws',
    args: [
      ARG('template', 'string', true, 'Template manifest id.'),
      ARG('project', 'string', true, 'Project id.'),
    ],
  },
  'select-revision': {
    summary: 'Request a committed revision snapshot for read-only replay.',
    safety: 'read',
    transport: 'ws',
    args: [
      ARG('run', 'uuid', true, 'Run id.'),
      ARG('revision', 'number', true, 'Committed revision number.'),
    ],
  },
  settings: {
    summary: 'Request effective settings (model/thinking/key presence boolean).',
    safety: 'read',
    transport: 'ws',
    args: [],
  },
  'file-tree': {
    summary: 'List a workspace-relative directory through the jailed file tool.',
    safety: 'read',
    transport: 'ws',
    args: [ARG('path', 'string', false, 'Workspace-relative directory (default root).')],
  },
  'file-read': {
    summary: 'Read one workspace-relative file through the jailed file tool.',
    safety: 'read',
    transport: 'ws',
    args: [ARG('path', 'string', true, 'Workspace-relative file path.')],
  },
  'chat-pin': {
    summary: 'Send the UI chat pin state (client-only surface; explicit ACK).',
    safety: 'mutating',
    transport: 'ws',
    args: [ARG('pinned', 'boolean', true, 'Pinned state.')],
  },
  viewport: {
    summary: 'Send a canvas viewport update (client-only surface; explicit ACK).',
    safety: 'mutating',
    transport: 'ws',
    args: [
      ARG('x', 'number', true, 'Canvas x translation.'),
      ARG('y', 'number', true, 'Canvas y translation.'),
      ARG('zoom', 'number', true, 'Zoom in 0.1..3.0.'),
    ],
  },
  recenter: {
    summary: 'Ask the UI to recenter the canvas (client-only surface; explicit ACK).',
    safety: 'mutating',
    transport: 'ws',
    args: [],
  },
  watch: {
    summary: 'Collect or follow server events (bounded by default).',
    safety: 'read',
    transport: 'ws',
    args: [
      ARG('run', 'uuid', false, 'Only events for this run.'),
      ARG('max-events', 'number', false, 'Max events to collect (default 100, max 1000).'),
      ARG('timeout-ms', 'number', false, 'Total wait limit (default 30000).'),
      ARG('follow', 'boolean', false, 'Emit each event as a JSON line until timeout/limit.'),
    ],
  },

  commands: {
    summary: 'List every command with safety class and transport.',
    safety: 'read',
    transport: 'local',
    args: [],
  },
  schema: {
    summary: 'Return the typed command/argument schema (or one command).',
    safety: 'read',
    transport: 'local',
    args: [ARG('command', 'string', false, 'Optional command name.')],
  },
  version: { summary: 'Print CLI and contract versions.', safety: 'read', transport: 'local', args: [] },
};

// ---------------------------------------------------------------------------
// Parsing and validation
// ---------------------------------------------------------------------------

function parseArgs(argv) {
  const options = { json: !HUMAN, format: null, help: false, yes: false, color: HUMAN };
  const positional = [];
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '-h') {
      options.help = true;
      continue;
    }
    if (token === '-V') {
      options.version = true;
      continue;
    }
    if (!token.startsWith('--')) {
      positional.push(token);
      continue;
    }
    if (token === '--') {
      positional.push(...argv.slice(index + 1));
      break;
    }
    const [rawName, rawValue] = token.slice(2).split('=', 2);
    const name = rawName.trim();
    if (name === 'json') {
      options.json = true;
      continue;
    }
    if (name === 'human') {
      options.json = false;
      continue;
    }
    if (name === 'format') {
      const value = rawValue ?? argv[++index];
      if (value !== 'json' && value !== 'table' && value !== 'jsonl' && value !== 'human') {
        throw cliError('E_CLI_USAGE', '--format must be json, table, jsonl or human.');
      }
      options.format = value;
      options.json = value === 'json' || value === 'jsonl';
      continue;
    }
    if (name === 'help' || name === 'h') {
      options.help = true;
      continue;
    }
    if (name === 'version' || name === 'V') {
      options.version = true;
      continue;
    }
    if (name === 'yes' || name === 'y') {
      options.yes = true;
      continue;
    }
    if (name === 'no-color') {
      options.color = false;
      continue;
    }
    if (name === 'url' || name === 'base-url') {
      options.url = rawValue ?? argv[++index];
      continue;
    }
    if (name === 'data-dir') {
      options.dataDir = rawValue ?? argv[++index];
      continue;
    }
    if (name === 'workspace-root') {
      options.workspaceRoot = rawValue ?? argv[++index];
      continue;
    }
    if (name === 'exe') {
      options.exe = rawValue ?? argv[++index];
      continue;
    }
    if (name === 'dry-run') {
      options.dryRun = true;
      continue;
    }
    if (name === 'timeout-ms') {
      options.timeoutMs = readNumber(rawValue ?? argv[++index], '--timeout-ms', 1, 3_600_000);
      continue;
    }
    // Command-specific flag: --flag value or --flag=value; bare = true.
    const value = rawValue ?? (argv[index + 1] !== undefined && !argv[index + 1].startsWith('--') ? argv[++index] : true);
    options[name] = value;
  }
  return { options, positional };
}

function cliError(code, message, howToFix = 'Run `hivemind-cli schema <command>` for valid arguments.', exitCode = EXIT.USAGE, extra = {}) {
  const error = new Error(message);
  error.code = code;
  error.howToFix = howToFix;
  error.exitCode = exitCode;
  error.retryable = false;
  error.extra = extra;
  return error;
}

function readNumber(value, flag, min, max) {
  const number = Number(value);
  if (!Number.isFinite(number) || !Number.isInteger(number) || number < min || number > max) {
    throw cliError('E_CLI_USAGE', `${flag} must be an integer in [${min}, ${max}].`);
  }
  return number;
}

function assertUuid(value, flag) {
  if (typeof value !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(value)) {
    throw cliError('E_CLI_USAGE', `${flag} must be a UUID.`);
  }
  return value.toLowerCase();
}

function assertRequired(options, name, flag = `--${name}`) {
  if (options[name] === undefined || options[name] === null || options[name] === '' || options[name] === true) {
    throw cliError('E_CLI_USAGE', `Missing required ${flag}.`);
  }
  return options[name];
}

function assertSafePath(value, flag) {
  if (typeof value !== 'string' || value.trim() === '') {
    throw cliError('E_CLI_USAGE', `${flag} must be a non-empty path.`);
  }
  if (value.includes('..')) {
    throw cliError('E_CLI_UNSAFE', `${flag} must not contain '..'.`, 'Use an absolute path inside the workspace.', EXIT_UNSAFE);
  }
  return path.resolve(value);
}

function assertLoopbackUrl(raw) {
  const candidate = raw ?? 'http://127.0.0.1:8099';
  let parsed;
  try {
    parsed = new URL(candidate);
  } catch {
    throw cliError('E_CLI_USAGE', `Invalid URL: ${candidate}`);
  }
  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    throw cliError('E_CLI_UNSAFE', 'Only http(s) loopback URLs are allowed.', 'Set --url http://127.0.0.1:8099.', EXIT_UNSAFE);
  }
  const hostname = parsed.hostname.toLowerCase();
  const loopback = hostname === '127.0.0.1' || hostname === 'localhost' || hostname === '[::1]' || hostname === '::1';
  if (!loopback) {
    throw cliError('E_CLI_UNSAFE', 'Only loopback URLs are allowed.', 'Set --url http://127.0.0.1:<port>.', EXIT_UNSAFE);
  }
  return parsed;
}

// ---------------------------------------------------------------------------
// State and lifecycle
// ---------------------------------------------------------------------------

function dataDirectory(options) {
  return path.resolve(options.dataDir || process.env.HIVEROUTER_DATA_DIR || DEFAULT_DATA_DIR);
}

function statePath(options) {
  return path.join(dataDirectory(options), 'agent-cli-state.json');
}

function readState(options = {}) {
  try {
    const raw = fs.readFileSync(statePath(options), 'utf8');
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

function writeState(state, options = {}) {
  const file = statePath(options);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  const temp = `${file}.${process.pid}.tmp`;
  fs.writeFileSync(temp, `${JSON.stringify(state, null, 2)}\n`, { mode: 0o600 });
  fs.renameSync(temp, file);
}

function clearState(options = {}) {
  try {
    fs.unlinkSync(statePath(options));
  } catch {
    // already absent
  }
}

function resolveBaseUrl(options) {
  const fromOptions = options.url || process.env.HIVEROUTER_URL || process.env.HIVEMIND_URL;
  if (fromOptions) {
    return assertLoopbackUrl(fromOptions);
  }
  const state = readState(options);
  if (state?.url) {
    return assertLoopbackUrl(state.url);
  }
  return assertLoopbackUrl(`http://${DEFAULT_HOST}:${DEFAULT_PORT}`);
}

function resolveExe(options) {
  if (options.exe) {
    const resolved = path.resolve(options.exe);
    if (!fs.existsSync(resolved)) {
      throw cliError('E_SERVER_NOT_FOUND', `Server executable not found: ${resolved}`, 'Build with tools/build-desktop.ps1 or pass --exe.');
    }
    return resolved;
  }
  const candidates = [
    path.join(REPO_ROOT, 'dist', 'HiveMind', 'hiverouter.exe'),
    path.join(REPO_ROOT, 'target', 'release', 'hiverouter.exe'),
    path.join(REPO_ROOT, 'target', 'debug', 'hiverouter.exe'),
    path.join(REPO_ROOT, 'backend', 'target', 'release', 'hiverouter.exe'),
    path.join(REPO_ROOT, 'backend', 'target', 'debug', 'hiverouter.exe'),
  ];
  const found = candidates.find((candidate) => fs.existsSync(candidate));
  if (!found) {
    throw cliError('E_SERVER_NOT_FOUND', 'No hiverouter server executable found.', 'Run tools/build-desktop.ps1, or pass --exe.', EXIT_NOT_RUNNING);
  }
  return found;
}

function resolveWorkspaceRoot(options) {
  const resolved = path.resolve(options.workspaceRoot || process.env.HIVEROUTER_WORKSPACE_ROOT || REPO_ROOT);
  let stat;
  try {
    stat = fs.statSync(resolved);
  } catch {
    throw cliError('E_WORKSPACE_INVALID', `Workspace root does not exist: ${resolved}`, 'Pass an existing --workspace-root.', EXIT.USAGE);
  }
  if (!stat.isDirectory()) {
    throw cliError('E_WORKSPACE_INVALID', `Workspace root is not a directory: ${resolved}`, 'Pass an existing --workspace-root.', EXIT.USAGE);
  }
  return resolved;
}

function portOf(url) {
  return Number(url.port || (url.protocol === 'https:' ? 443 : 80));
}

function isPortOpen(host, port, timeoutMs = 600) {
  return new Promise((resolve) => {
    const socket = net.connect({ host, port });
    const done = (open) => {
      socket.destroy();
      resolve(open);
    };
    socket.setTimeout(timeoutMs);
    socket.once('connect', () => done(true));
    socket.once('timeout', () => done(false));
    socket.once('error', () => done(false));
  });
}

function processAlive(pid) {
  if (!Number.isInteger(pid) || pid <= 0) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error.code === 'EPERM';
  }
}

async function waitForHealth(baseUrl, timeoutMs, pid = null, stderrPath = null) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;
  while (Date.now() < deadline) {
    if (Number.isInteger(pid) && !processAlive(pid)) {
      let tail = '';
      if (stderrPath) {
        try {
          tail = fs.readFileSync(stderrPath, 'utf8').split(/\r?\n/).slice(-12).join('\n');
        } catch {
          tail = '';
        }
      }
      throw cliError(
        'E_SERVER_EXITED',
        `HiveMind exited before becoming healthy${tail ? `:\n${tail}` : '.'}`,
        'Inspect the stderr log and fix the boot refusal; no server is running.',
        EXIT_NOT_RUNNING,
      );
    }
    try {
      const response = await fetchWithTimeout(new URL('/health', baseUrl), { timeoutMs: 2_000 });
      if (response.ok) {
        const body = await response.json();
        if (body && typeof body.status === 'string' && body.database && typeof body.schema === 'string') {
          return body;
        }
        lastError = '/health answered but not as HiveMind';
      } else {
        lastError = `HTTP ${response.status}`;
      }
    } catch (error) {
      lastError = error.message;
    }
    await sleep(300);
  }
  throw cliError(
    'E_SERVER_NOT_READY',
    `HiveMind did not become healthy at ${baseUrl.href} within ${timeoutMs}ms${lastError ? ` (${lastError})` : ''}.`,
    'Check the server stderr log in the data directory; then run start again.',
    EXIT_NOT_RUNNING,
  );
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function startServer(options) {
  const baseUrl = options.url ? assertLoopbackUrl(options.url) : assertLoopbackUrl(`http://${options.bind || `${DEFAULT_HOST}:${options.port || DEFAULT_PORT}`}`);
  const host = baseUrl.hostname;
  const port = portOf(baseUrl);
  if (!Number.isInteger(port) || port < 1 || port > 65_535) {
    throw cliError('E_CLI_USAGE', `Invalid port: ${baseUrl.port || '(empty)'}`);
  }
  const existingState = readState(options);
  if (
    existingState?.pid &&
    processAlive(Number(existingState.pid)) &&
    existingState.url !== baseUrl.href.replace(/\/$/, '')
  ) {
    throw cliError(
      'E_INSTANCE_CONFLICT',
      `This data directory already has a live server at ${existingState.url}.`,
      'Stop it first, use its URL, or use a different --data-dir.',
      EXIT_NOT_RUNNING,
    );
  }
  if (existingState?.pid && !processAlive(Number(existingState.pid))) {
    clearState(options);
  }
  let exe = options['dry-run'] ? resolveExe(options) : null;
  const workspaceRoot = resolveWorkspaceRoot(options);
  const dataDir = dataDirectory(options);
  const dbPath = options.db ? path.resolve(options.db) : path.resolve(dataDir, 'hiverouter.db');
  const args = ['--bind', `${host}:${port}`, '--data-dir', dataDir, '--workspace-root', workspaceRoot, '--db', dbPath];
  if (options['dry-run']) {
    // A dry run has no side effects: no health probe, no state write, no mkdir.
    return {
      dry_run: true,
      request: { executable: exe, args, cwd: workspaceRoot },
      already_running_unknown: true,
    };
  }
  try {
    const health = await fetchWithTimeout(new URL('/health', baseUrl), { timeoutMs: 2_000 });
    if (health.ok) {
      const body = await health.json();
      if (!body || typeof body.status !== 'string' || !body.database || typeof body.schema !== 'string') {
        throw cliError('E_PORT_IN_USE', `${host}:${port} answered /health but not as HiveMind.`, 'Use --port with a free loopback port.', EXIT_NOT_RUNNING);
      }
      const expectedDb = dbPath;
      const actualDb = body.database?.path ? path.resolve(String(body.database.path)) : null;
      if (actualDb !== null && actualDb !== expectedDb) {
        throw cliError(
          'E_INSTANCE_FOREIGN',
          `${host}:${port} is a different HiveMind instance (db ${actualDb}).`,
          'Use the matching --data-dir/--url, or a different --port.',
          EXIT_NOT_RUNNING,
        );
      }
      const sameOwned =
        existingState?.url === baseUrl.href.replace(/\/$/, '') &&
        processAlive(Number(existingState?.pid)) &&
        existingState?.external !== true &&
        (!existingState?.boot_id || existingState.boot_id === body.boot_id);
      writeState(
        {
          state_schema: 'hivemind.cli.state.v1',
          status: 'ready',
          url: baseUrl.href.replace(/\/$/, ''),
          pid: sameOwned ? existingState.pid : null,
          external: !sameOwned,
          started_at: existingState?.started_at ?? new Date().toISOString(),
          exe: existingState?.exe ?? exe,
          workspace_root: existingState?.workspace_root ?? workspaceRoot,
          data_dir: dataDir,
          db_path: expectedDb,
          boot_id: typeof body.boot_id === 'string' ? body.boot_id : null,
          server_pid: Number.isInteger(body.pid) ? body.pid : null,
        },
        options,
      );
      return { already_running: true, managed: sameOwned, health: body, url: baseUrl.href.replace(/\/$/, '') };
    }
  } catch (error) {
    if (error.code === 'E_PORT_IN_USE' || error.code === 'E_INSTANCE_FOREIGN') throw error;
    // not running
  }
  if (await isPortOpen(host, port)) {
    throw cliError('E_PORT_IN_USE', `${host}:${port} is listening but is not a healthy HiveMind server.`, 'Use --port with a free loopback port.', EXIT_NOT_RUNNING);
  }
  exe ??= resolveExe(options);
  fs.mkdirSync(dataDir, { recursive: true });
  const stdoutPath = path.join(dataDir, 'server.stdout.log');
  const stderrPath = path.join(dataDir, 'server.stderr.log');
  const stdoutFd = fs.openSync(stdoutPath, 'a');
  const stderrFd = fs.openSync(stderrPath, 'a');
  let child;
  try {
    child = spawn(exe, args, {
      cwd: workspaceRoot,
      detached: true,
      windowsHide: true,
      stdio: ['ignore', stdoutFd, stderrFd],
      env: process.env,
    });
  } catch (error) {
    throw cliError('E_SERVER_START_FAILED', `Could not spawn ${exe}: ${error.message}`, 'Check the executable path and permissions.', EXIT_NOT_RUNNING);
  } finally {
    fs.closeSync(stdoutFd);
    fs.closeSync(stderrFd);
  }
  if (!child.pid) {
    throw cliError('E_SERVER_START_FAILED', 'The server process could not be spawned.', 'Inspect the stderr log.', EXIT_NOT_RUNNING);
  }
  child.unref();
  writeState(
    {
      state_schema: 'hivemind.cli.state.v1',
      status: 'starting',
      url: baseUrl.href.replace(/\/$/, ''),
      pid: child.pid,
      external: false,
      started_at: new Date().toISOString(),
      exe,
      workspace_root: workspaceRoot,
      data_dir: dataDir,
      db_path: dbPath,
      stdout_log: stdoutPath,      stderr_log: stderrPath,
    },
    options,
  );
  if (options['no-wait']) {
    return { started: true, pid: child.pid, url: baseUrl.href.replace(/\/$/, ''), wait: false };
  }
  const health = await waitForHealth(baseUrl, options.timeoutMs || 60_000, child.pid, stderrPath);
  const currentState = readState(options);
  if (currentState) {
    const keyCheck = (health.checks || []).find((check) => check.name === 'deepseek_api_key');
    writeState(
      {
        ...currentState,
        status: 'ready',
        provider_key_present: keyCheck ? keyCheck.ok === true : null,
        boot_id: typeof health.boot_id === 'string' ? health.boot_id : null,
        server_pid: Number.isInteger(health.pid) ? health.pid : child.pid,
      },
      options,
    );
  }
  return { started: true, pid: child.pid, url: baseUrl.href.replace(/\/$/, ''), health };
}

async function stopServer(options) {
  const state = readState(options);
  if (state === null) {
    if (options.pid) {
      throw cliError('E_CLI_UNSAFE', 'No CLI-owned state file exists; refusing to kill an arbitrary pid.', 'Use the process manager.', EXIT_UNSAFE);
    }
    const probeUrl = resolveBaseUrl(options);
    if (await isPortOpen(probeUrl.hostname, portOf(probeUrl))) {
      throw cliError('E_NOT_OWNED', 'A server is listening but was not started by this CLI.', 'Use its own manager or start it through this CLI.', EXIT_NOT_RUNNING);
    }
    return { stopped: false, already_stopped: true, reason: 'no CLI state and no reachable server' };
  }
  const pid = options.pid ? Number(options.pid) : state?.pid;
  if (!Number.isInteger(pid) || pid <= 0) {
    if (state.external === true) {
      throw cliError(
        'E_NOT_OWNED',
        'The recorded server is external and has no CLI-owned pid.',
        'Use the process manager that started it, or start through this CLI.',
        EXIT_NOT_RUNNING,
      );
    }
    return { stopped: false, reason: 'no CLI-owned server pid is recorded' };
  }
  if (state.external === true) {
    if (!options.force) {
      throw cliError(
        'E_NOT_OWNED',
        'The recorded server was not started by this CLI (external).',
        'Stop it with its own process manager, or pass --force --yes after verifying it.',
        EXIT_NOT_RUNNING,
      );
    }
    if (!options.yes) {
      throw cliError('E_CONFIRMATION_REQUIRED', '--force on an external server requires --yes.', 'Verify the pid and retry with --yes.', EXIT_UNSAFE);
    }
  }
  if (options.pid && state?.pid && Number(options.pid) !== Number(state.pid)) {
    throw cliError('E_CLI_UNSAFE', '--pid does not match the CLI-owned server state.', 'Refusing to kill an unrelated process.', EXIT_UNSAFE);
  }
  if (!processAlive(pid)) {
    clearState(options);
    return { stopped: false, reason: `pid ${pid} is not running`, stale: true };
  }
  if (state.db_path || state.boot_id) {
    try {
      const health = await httpJson(resolveBaseUrl(options), '/health');
      if (state.db_path) {
        const actual = health?.database?.path ? path.resolve(String(health.database.path)) : null;
        if (actual !== null && path.resolve(String(state.db_path)) !== actual) {
          throw cliError(
            'E_NOT_OWNED',
            'The live server database does not match the CLI-owned state.',
            'Refusing to kill a foreign instance; use its own manager.',
            EXIT_NOT_RUNNING,
          );
        }
      }
      if (state.boot_id && health?.boot_id && health.boot_id !== state.boot_id) {
        throw cliError(
          'E_NOT_OWNED',
          'The live server boot identity does not match the CLI-owned state.',
          'The recorded process was replaced; inspect state before killing anything.',
          EXIT_NOT_RUNNING,
        );
      }
    } catch (error) {
      if (error.code === 'E_NOT_OWNED') throw error;
      // Server may already be stopping; process identity is checked by pid next.
    }
  }
  if (options['dry-run']) {
    return { dry_run: true, request: { kill_pid: pid, force: Boolean(options.force) } };
  }
  const timeoutMs = options.timeoutMs || 10_000;
  let escalated = false;
  const sendTermination = (force) => {
    if (process.platform === 'win32') {
      spawnSync('taskkill', ['/PID', String(pid), '/T', ...(force ? ['/F'] : [])], { windowsHide: true });
    } else {
      try {
        process.kill(pid, force ? 'SIGKILL' : 'SIGTERM');
      } catch {
        // already gone
      }
    }
  };
  sendTermination(Boolean(options.force));
  let deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline && processAlive(pid)) {
    await sleep(250);
  }
  if (processAlive(pid) && !options.force) {
    // Windows console processes do not honour taskkill without /F; escalate once
    // and report the truth instead of forcing every caller to run stop twice.
    escalated = true;
    sendTermination(true);
    deadline = Date.now() + Math.min(5_000, timeoutMs);
    while (Date.now() < deadline && processAlive(pid)) {
      await sleep(250);
    }
  }
  if (processAlive(pid)) {
    throw cliError('E_STOP_FAILED', `pid ${pid} is still running after forced stop.`, 'Inspect the process manually.', EXIT_NOT_RUNNING);
  }
  clearState(options);
  return options.force
    ? { stopped: true, pid, graceful: false, escalated: false }
    : { stopped: true, pid, graceful: !escalated, escalated };
}

// ---------------------------------------------------------------------------
// HTTP transport
// ---------------------------------------------------------------------------

async function fetchWithTimeout(url, { timeoutMs = 15_000, ...init } = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetch(url, { ...init, redirect: 'error', signal: controller.signal });
  } catch (error) {
    if (error.name === 'AbortError') {
      throw cliError('E_TIMEOUT', `Request timed out after ${timeoutMs}ms.`, 'Retry or increase --timeout-ms.', EXIT_TIMEOUT);
    }
    throw cliError('E_SERVER_UNREACHABLE', `Cannot reach ${url.origin}: ${error.message}`, 'Run `hivemind-cli start` or pass --url.', EXIT_NOT_RUNNING);
  } finally {
    clearTimeout(timer);
  }
}

async function httpJson(baseUrl, route, { method = 'GET', body, timeoutMs } = {}) {
  const url = new URL(route, baseUrl);
  const headers = { accept: 'application/json', origin: baseUrl.origin };
  if (body !== undefined) headers['content-type'] = 'application/json';
  const response = await fetchWithTimeout(url, {
    method,
    timeoutMs,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await response.text();
  let parsed = null;
  try {
    parsed = text.length === 0 ? null : JSON.parse(text);
  } catch {
    parsed = { raw: text };
  }
  if (!response.ok) {
    throw refusalFromBody(parsed, response.status);
  }
  return parsed;
}

function refusalFromBody(body, status) {
  const code = body?.code || (status === 401 ? 'E_TOKEN_INVALID' : status === 403 ? 'E_ORIGIN_REJECTED' : status === 404 ? 'E_NOT_FOUND' : 'E_HTTP_ERROR');
  const error = cliError(
    code,
    body?.message || `HTTP ${status}`,
    body?.how_to_fix || 'Inspect the server response with --json.',
    status === 401 || status === 403 ? EXIT_AUTH : status === 404 ? EXIT.REFUSAL : EXIT.REFUSAL,
    { status },
  );
  error.retryable = Boolean(body?.retryable);
  return error;
}

// ---------------------------------------------------------------------------
// WebSocket transport
// ---------------------------------------------------------------------------

/**
 * Minimal RFC 6455 client for the HiveMind loopback socket.
 *
 * Node's built-in WebSocket cannot set the Origin header, and the server
 * correctly rejects a missing Origin (DNS-rebinding defense). This client
 * performs a normal upgrade with an explicit loopback Origin and only handles
 * text/close/ping frames used by the frozen event protocol.
 */
class MiniWebSocket {
  constructor() {
    this.listeners = new Map();
    this.buffer = Buffer.alloc(0);
    this.pendingMessages = [];
    this.open = false;
    this.closed = false;
    this.fragments = [];
    this.fragmentOpcode = null;
    this.socket = null;
  }

  static connect(url, timeoutMs = 10_000) {
    return new Promise((resolve, reject) => {
      const socket = new MiniWebSocket();
      socket._connect(url, resolve, reject, timeoutMs);
    });
  }

  addEventListener(type, callback, options = {}) {
    const entry = { callback, once: Boolean(options.once) };
    const list = this.listeners.get(type) || [];
    list.push(entry);
    this.listeners.set(type, list);
    if (type === 'message' && this.pendingMessages.length > 0) {
      const queued = this.pendingMessages.splice(0);
      queueMicrotask(() => queued.forEach((data) => this._emit('message', { data })));
    }
    if (type === 'open' && this.open) {
      queueMicrotask(() => this._emit('open', {}));
    }
  }

  _emit(type, event) {
    const list = this.listeners.get(type) || [];
    for (const entry of [...list]) {
      if (entry.once) {
        this.listeners.set(type, (this.listeners.get(type) || []).filter((item) => item !== entry));
      }
      try {
        entry.callback(event);
      } catch {
        // A listener throwing must not corrupt the frame stream.
      }
    }
  }

  _connect(rawUrl, resolve, reject, timeoutMs) {
    let parsed;
    try {
      parsed = new URL(rawUrl);
    } catch {
      reject(cliError('E_CLI_USAGE', `Invalid WebSocket URL: ${rawUrl}`));
      return;
    }
    if (parsed.protocol !== 'ws:' && parsed.protocol !== 'wss:') {
      reject(cliError('E_CLI_UNSAFE', 'Only ws/wss loopback URLs are allowed.', 'Use the local server URL.', EXIT_UNSAFE));
      return;
    }
    const host = parsed.hostname;
    const port = Number(parsed.port || (parsed.protocol === 'wss:' ? 443 : 80));
    const origin = `${parsed.protocol === 'wss:' ? 'https:' : 'http:'}//${parsed.host}`;
    const key = randomBytes(16).toString('base64');
    const handshake =
      `GET ${parsed.pathname}${parsed.search} HTTP/1.1\r\n` +
      `Host: ${parsed.host}\r\n` +
      'Upgrade: websocket\r\n' +
      'Connection: Upgrade\r\n' +
      `Sec-WebSocket-Key: ${key}\r\n` +
      'Sec-WebSocket-Version: 13\r\n' +
      `Origin: ${origin}\r\n` +
      '\r\n';
    let settled = false;
    let handshakeBuffer = Buffer.alloc(0);
    const timer = setTimeout(() => {
      if (!settled) {
        settled = true;
        reject(cliError('E_WS_TIMEOUT', 'WebSocket handshake timed out.', 'Retry or check the server.', EXIT_TIMEOUT));
      }
      this.socket?.destroy();
    }, timeoutMs);
    const socket = net.connect({ host, port });
    this.socket = socket;
    socket.once('error', (error) => {
      if (!settled) {
        settled = true;
        clearTimeout(timer);
        reject(cliError('E_WS_DISCONNECTED', `WebSocket connect failed: ${error.message}`, 'Check the server is running.', EXIT_NOT_RUNNING));
      } else if (!this.closed) {
        this.closed = true;
        this._emit('error', { message: error.message });
        this._emit('close', { code: 1006 });
      }
    });
    socket.once('close', () => {
      if (!settled) {
        settled = true;
        clearTimeout(timer);
        reject(cliError('E_WS_DISCONNECTED', 'WebSocket closed during handshake.', 'Check the token and server.', EXIT_NOT_RUNNING));
      } else if (!this.closed) {
        this.closed = true;
        this._emit('close', { code: 1006 });
      }
    });
    socket.on('data', (chunk) => {
      if (!settled) {
        handshakeBuffer = Buffer.concat([handshakeBuffer, chunk]);
        const end = handshakeBuffer.indexOf('\r\n\r\n');
        if (end < 0) return;
        const header = handshakeBuffer.subarray(0, end).toString('latin1');
        const rest = handshakeBuffer.subarray(end + 4);
        if (!/^HTTP\/1\.1 101\b/.test(header)) {
          settled = true;
          clearTimeout(timer);
          const status = header.split('\r\n')[0];
          reject(cliError('E_WS_DISCONNECTED', `WebSocket upgrade rejected: ${status}`, 'Check the token and Origin allowlist.', EXIT_AUTH));
          socket.destroy();
          return;
        }
        const expectedAccept = createHash('sha1')
          .update(`${key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`)
          .digest('base64');
        const acceptLine = header
          .split('\r\n')
          .find((line) => line.toLowerCase().startsWith('sec-websocket-accept:'));
        if (!acceptLine || acceptLine.split(':').slice(1).join(':').trim() !== expectedAccept) {
          settled = true;
          clearTimeout(timer);
          reject(cliError('E_WS_DISCONNECTED', 'WebSocket handshake accept key did not match.', 'Check the server implementation.', EXIT.REFUSAL));
          socket.destroy();
          return;
        }
        settled = true;
        clearTimeout(timer);
        this.open = true;
        if (rest.length > 0) this.buffer = Buffer.concat([this.buffer, rest]);
        this._parseFrames();
        this._emit('open', {});
        resolve(this);
        return;
      }
      this.buffer = Buffer.concat([this.buffer, chunk]);
      this._parseFrames();
    });
    socket.once('connect', () => {
      socket.write(handshake, 'latin1');
    });
  }

  _parseFrames() {
    while (this.buffer.length >= 2) {
      const first = this.buffer[0];
      const second = this.buffer[1];
      const fin = (first & 0x80) !== 0;
      const opcode = first & 0x0f;
      const masked = (second & 0x80) !== 0;
      let length = second & 0x7f;
      let offset = 2;
      if (length === 126) {
        if (this.buffer.length < 4) return;
        length = this.buffer.readUInt16BE(2);
        offset = 4;
      } else if (length === 127) {
        if (this.buffer.length < 10) return;
        const big = this.buffer.readBigUInt64BE(2);
        if (big > 16_777_216n) {
          this.close();
          return;
        }
        length = Number(big);
        offset = 10;
      }
      if (masked) offset += 4;
      if (this.buffer.length < offset + length) return;
      const payload = this.buffer.subarray(offset, offset + length);
      this.buffer = this.buffer.subarray(offset + length);
      if (opcode === 0x8) {
        this.close();
        return;
      }
      if (opcode === 0x9) {
        this._sendFrame(0xa, payload);
        continue;
      }
      if (opcode === 0xa) continue;
      if (opcode === 0x1 || opcode === 0x0) {
        if (opcode === 0x1) {
          this.fragments = [payload];
          this.fragmentOpcode = 1;
        } else {
          this.fragments.push(payload);
        }
        if (!fin) continue;
        const text = Buffer.concat(this.fragments).toString('utf8');
        this.fragments = [];
        this.fragmentOpcode = null;
        if (this.listeners.has('message')) this._emit('message', { data: text });
        else this.pendingMessages.push(text);
      }
      // Binary frames are outside the frozen protocol and ignored.
    }
  }

  _sendFrame(opcode, payload) {
    if (!this.open || !this.socket || this.closed) {
      throw cliError('E_WS_DISCONNECTED', 'WebSocket is not open.', 'Retry after the server is reachable.', EXIT_NOT_RUNNING);
    }
    const mask = randomBytes(4);
    const body = Buffer.from(payload);
    let header;
    if (body.length < 126) {
      header = Buffer.from([0x80 | opcode, 0x80 | body.length]);
    } else if (body.length < 65_536) {
      header = Buffer.alloc(4);
      header[0] = 0x80 | opcode;
      header[1] = 0x80 | 126;
      header.writeUInt16BE(body.length, 2);
    } else {
      header = Buffer.alloc(10);
      header[0] = 0x80 | opcode;
      header[1] = 0x80 | 127;
      header.writeBigUInt64BE(BigInt(body.length), 2);
    }
    const masked = Buffer.alloc(body.length);
    for (let index = 0; index < body.length; index += 1) {
      masked[index] = body[index] ^ mask[index % 4];
    }
    this.socket.write(Buffer.concat([header, mask, masked]));
  }

  send(data) {
    this._sendFrame(0x1, Buffer.from(String(data), 'utf8'));
  }

  close() {
    if (this.closed) return;
    this.closed = true;
    try {
      if (this.open) this._sendFrame(0x8, Buffer.alloc(0));
    } catch {
      // socket may already be gone
    }
    this.socket?.destroy();
    this._emit('close', { code: 1000 });
  }
}

async function wsSession(baseUrl) {
  const session = await httpJson(baseUrl, '/api/session');
  if (!session || typeof session.token !== 'string' || typeof session.ws !== 'string') {
    throw cliError('E_SCHEMA_INVALID', 'GET /api/session did not return token/ws/last_seq.', 'Check the server version.', EXIT.REFUSAL);
  }
  const wsUrl = new URL(session.ws, baseUrl);
  wsUrl.protocol = wsUrl.protocol === 'https:' ? 'wss:' : 'ws:';
  wsUrl.searchParams.set('token', session.token);
  wsUrl.searchParams.set('last_seq', String(session.last_seq ?? 0));
  const socket = await MiniWebSocket.connect(wsUrl.href, 10_000);
  const queue = [];
  const waiters = [];
  let closed = null;
  socket.addEventListener('message', (event) => {
    let value;
    try {
      value = JSON.parse(typeof event.data === 'string' ? event.data : Buffer.from(event.data).toString('utf8'));
    } catch {
      return;
    }
    queue.push(value);
    for (const waiter of [...waiters]) {
      if (waiter.predicate(value)) {
        waiters.splice(waiters.indexOf(waiter), 1);
        clearTimeout(waiter.timer);
        waiter.resolve(value);
      }
    }
  });
  socket.addEventListener('close', (event) => {
    closed = event.code;
    for (const waiter of waiters.splice(0)) {
      clearTimeout(waiter.timer);
      waiter.reject(cliError('E_WS_DISCONNECTED', `WebSocket closed (code ${event.code}).`, 'Restart the server or retry.', EXIT_NOT_RUNNING));
    }
  });
  socket.addEventListener('error', () => {
    // close handler carries the actionable failure
  });
  socket.send(JSON.stringify({ type: 'hello', last_seq: session.last_seq ?? 0 }));
  const next = async (predicate, timeoutMs = 15_000) => {
    const foundIndex = queue.findIndex(predicate);
    if (foundIndex >= 0) return queue.splice(foundIndex, 1)[0];
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        const index = waiters.findIndex((waiter) => waiter.resolve === resolve);
        if (index >= 0) waiters.splice(index, 1);
        reject(cliError('E_EVENT_TIMEOUT', 'No matching server event arrived.', 'Increase --timeout-ms or check the server log.', EXIT_TIMEOUT));
      }, timeoutMs);
      waiters.push({ predicate, resolve, reject, timer });
    });
  };
  const close = () => {
    try {
      socket.close();
    } catch {
      // already closed
    }
  };
  return { session, socket, next, close, closed: () => closed };
}

function commandEnvelope(name, payload) {
  const id = randomUUID();
  return {
    schema: SERVER_SCHEMA,
    type: 'COMMAND',
    name,
    id,
    correlation_id: id,
    run_id: null,
    task_id: null,
    seq: null,
    ts: new Date().toISOString(),
    payload,
  };
}

async function wsCommand(baseUrl, name, payload, { timeoutMs = 15_000, awaitEvent = null } = {}) {
  const session = await wsSession(baseUrl);
  try {
    const envelope = commandEnvelope(name, payload);
    session.socket.send(JSON.stringify(envelope));
    const response = await session.next((frame) => {
      if (frame?.type !== 'EVENT') return false;
      if (frame.name === 'ACK' && frame.payload?.correlation_id === envelope.correlation_id) return true;
      if (frame.name === 'ERROR' && frame.correlation_id === envelope.correlation_id) return true;
      if (frame.name === 'ERROR' && frame.payload?.correlation_id === envelope.correlation_id) return true;
      return false;
    }, timeoutMs);
    if (response.name === 'ERROR') {
      throw refusalFromBody(response.payload, 200);
    }
    let event = null;
    if (awaitEvent) {
      event = await session.next((frame) => frame?.type === 'EVENT' && frame.name === awaitEvent, timeoutMs);
    }
    return { ack: response, event, envelope };
  } finally {
    session.close();
  }
}

async function wsWatch(baseUrl, { runId = null, maxEvents = 100, timeoutMs = 30_000, follow = false, onEvent = null } = {}) {
  const session = await wsSession(baseUrl);
  const events = [];
  const deadline = Date.now() + timeoutMs;
  try {
    while (events.length < maxEvents && Date.now() < deadline) {
      const remaining = Math.max(0, deadline - Date.now());
      let frame;
      try {
        frame = await session.next((candidate) => {
          if (candidate?.type !== 'EVENT') return false;
          if (runId && candidate.run_id && candidate.run_id !== runId) return false;
          return candidate.name !== 'PONG';
        }, remaining);
      } catch (error) {
        if (error.code === 'E_EVENT_TIMEOUT') break;
        throw error;
      }
      events.push(frame);
      if (onEvent) onEvent(frame);
      if (frame.name === 'RUN_STATUS' && ['completed', 'failed', 'cancelled', 'interrupted'].includes(frame.payload?.status)) {
        break;
      }
      if (!follow && frame.name === 'SERVER_HELLO') {
        // keep waiting for the requested window
      }
    }
    return { events, truncated: events.length >= maxEvents, timed_out: Date.now() >= deadline };
  } finally {
    session.close();
  }
}

// ---------------------------------------------------------------------------
// Read commands
// ---------------------------------------------------------------------------

function limitOption(options, fallback, max = 100) {
  if (options.limit === undefined) return fallback;
  return readNumber(options.limit, '--limit', 1, max);
}

function runIdOption(options) {
  return assertUuid(assertRequired(options, 'run'), '--run');
}

async function latestRevision(baseUrl, runId) {
  const revisions = await httpJson(baseUrl, `/api/history/runs/${runId}/revisions`);
  if (typeof revisions.latest_revision !== 'number') {
    throw cliError('E_NO_REVISION', 'Run has no committed revision.', 'Wait for a commit or pick another run.', EXIT.REFUSAL);
  }
  return revisions.latest_revision;
}

async function executeRead(command, options, baseUrl) {
  switch (command) {
    case 'health':
      return httpJson(baseUrl, '/health');
    case 'diagnose':
      return httpJson(baseUrl, '/diagnose');
    case 'runtime':
      return httpJson(baseUrl, '/api/runtime');
    case 'runs': {
      const body = await httpJson(baseUrl, '/api/history/runs');
      const limit = limitOption(options, 20);
      return { ...body, runs: (body.runs || []).slice(0, limit), truncated: (body.runs || []).length > limit };
    }
    case 'revisions':
      return httpJson(baseUrl, `/api/history/runs/${runIdOption(options)}/revisions`);
    case 'state': {
      const runId = runIdOption(options);
      const revision = options.revision === undefined ? await latestRevision(baseUrl, runId) : readNumber(options.revision, '--revision', 0, Number.MAX_SAFE_INTEGER);
      return httpJson(baseUrl, `/api/history/runs/${runId}/state?revision=${revision}`);
    }
    case 'messages': {
      const runId = runIdOption(options);
      const body = await httpJson(baseUrl, `/api/history/runs/${runId}/messages`);
      const limit = limitOption(options, 50, MAX_OUTPUT_ITEMS);
      return { ...body, messages: body.messages.slice(-limit), truncated: body.truncated || body.messages.length > limit };
    }
    case 'agent-messages': {
      const run = runIdOption(options);
      const task = options.task ? assertUuid(options.task, '--task') : null;
      return httpJson(
        baseUrl,
        task
          ? `/api/agents/runs/${run}/messages?task_id=${encodeURIComponent(task)}`
          : `/api/agents/runs/${run}/messages`,
      );
    }
    case 'roster':
      return httpJson(baseUrl, `/api/agents/runs/${runIdOption(options)}/roster`);
    case 'approvals': {
      const run = options.run ? assertUuid(options.run, '--run') : null;
      return httpJson(baseUrl, run ? `/api/approvals?run_id=${encodeURIComponent(run)}` : '/api/approvals');
    }
    case 'templates':
      return httpJson(baseUrl, '/api/templates');
    case 'workspace':
      return httpJson(baseUrl, '/api/workspace');
    case 'active-run':
      return httpJson(baseUrl, '/api/runs/active');
    case 'profiles':
      return httpJson(baseUrl, '/api/agents/profiles');
    case 'session': {
      const session = await httpJson(baseUrl, '/api/session');
      return {
        schema: session.schema,
        last_seq: session.last_seq,
        ws: session.ws,
        token: session.token ? '[redacted]' : null,
      };
    }
    case 'memory':
      return options.run
        ? httpJson(baseUrl, `/api/agents/runs/${runIdOption(options)}/memory`)
        : httpJson(baseUrl, '/api/agents/memory');
    case 'projects':
      return httpJson(baseUrl, '/api/projects');
    case 'models':
      return httpJson(baseUrl, '/api/provider/models');
    case 'balance':
      return httpJson(baseUrl, '/api/provider/balance');
    case 'usage':
      return httpJson(baseUrl, `/api/usage/runs/${runIdOption(options)}`);
    default:
      throw cliError('E_CLI_USAGE', `Unknown read command: ${command}`);
  }
}

// ---------------------------------------------------------------------------
// Mutating commands
// ---------------------------------------------------------------------------

async function resolvePrompt(options) {
  let text = options.prompt;
  if (options['prompt-file']) {
    const file = assertSafePath(options['prompt-file'], '--prompt-file');
    try {
      text = fs.readFileSync(file, 'utf8');
    } catch (error) {
      throw cliError('E_FILE_READ', `Cannot read prompt file: ${error.message}`, 'Check the path and permissions.', EXIT.USAGE);
    }
  }
  if (typeof text !== 'string' || text.trim().length === 0) {
    throw cliError('E_PROMPT_EMPTY', 'A non-empty --prompt or --prompt-file is required.');
  }
  if (text.length > MAX_PROMPT_CHARS) {
    throw cliError('E_PROMPT_TOO_LONG', `Prompt exceeds ${MAX_PROMPT_CHARS} characters.`, 'Shorten the prompt or split the work.');
  }
  return text;
}

function projectOption(options) {
  return options.project || options['project-id'] || 'workspace';
}

async function executeMutation(command, options, baseUrl) {
  switch (command) {
    case 'submit':
    case 'run': {
      const text = await resolvePrompt(options);
      const result = await wsCommand(baseUrl, 'SUBMIT_PROMPT', { text, project_id: projectOption(options) });
      const output = { accepted: true, ack: result.ack.payload };
      if (options.wait || command === 'run') {
        const timeoutMs = options.timeoutMs || (command === 'run' ? 180_000 : 120_000);
        const watched = await wsWatch(baseUrl, { timeoutMs, maxEvents: 1000, follow: false });
        output.events = watched.events;
        output.truncated = watched.truncated;
        output.timed_out = watched.timed_out;
      }
      return output;
    }
    case 'cancel':
      requireYes(options, 'cancel');
      return { cancelled: true, ack: (await wsCommand(baseUrl, 'CANCEL_RUN', { run_id: runIdOption(options) })).ack.payload };
    case 'kill':
      requireYes(options, 'kill');
      return { killed: true, ack: (await wsCommand(baseUrl, 'KILL_WORKER', { task_id: assertUuid(assertRequired(options, 'task'), '--task') })).ack.payload };
    case 'rewire':
      return {
        rewired: true,
        ack: (
          await wsCommand(baseUrl, 'REWIRE_TASK', {
            task_id: assertUuid(assertRequired(options, 'task'), '--task'),
            new_parent_id: assertUuid(assertRequired(options, 'parent'), '--parent'),
            expected_rev: readNumber(assertRequired(options, 'expected-rev'), '--expected-rev', 0, Number.MAX_SAFE_INTEGER),
          })
        ).ack.payload,
      };
    case 'spawn-subhive':
      return {
        spawned: true,
        ack: (
          await wsCommand(baseUrl, 'SPAWN_SUB_HIVE', {
            parent_task_id: assertUuid(assertRequired(options, 'parent'), '--parent'),
            role: String(assertRequired(options, 'role')),
          })
        ).ack.payload,
      };
    case 'approve':
    case 'deny': {
      requireYes(options, command);
      const payload = {
        approval_id: assertUuid(assertRequired(options, 'approval'), '--approval'),
        nonce: String(assertRequired(options, 'nonce')),
        decision: command,
        ...(options.reason ? { reason: String(options.reason) } : {}),
      };
      return { resolved: true, decision: command, ack: (await wsCommand(baseUrl, 'APPROVAL_RESOLVE', payload)).ack.payload };
    }
    case 'template':
      return {
        inserted: true,
        ack: (
          await wsCommand(baseUrl, 'TEMPLATE_INSERT', {
            template_id: String(assertRequired(options, 'template')),
            project_id: String(assertRequired(options, 'project')),
          })
        ).ack.payload,
      };
    case 'select-revision': {
      const result = await wsCommand(
        baseUrl,
        'HISTORY_REVISION_SET',
        {
          run_id: runIdOption(options),
          revision: readNumber(assertRequired(options, 'revision'), '--revision', 0, Number.MAX_SAFE_INTEGER),
        },
        { awaitEvent: 'HISTORY_SNAPSHOT' },
      );
      return { ack: result.ack.payload, snapshot: result.event?.payload ?? null };
    }
    case 'settings': {
      const result = await wsCommand(baseUrl, 'SETTINGS_STATUS', {}, { awaitEvent: 'SETTINGS_STATUS_RESULT' });
      return { ack: result.ack.payload, settings: result.event?.payload ?? null };
    }
    case 'file-tree': {
      const result = await wsCommand(
        baseUrl,
        'FILE_TREE_LIST',
        options.path ? { path: String(options.path) } : {},
        { awaitEvent: 'FILE_TREE_RESULT' },
      );
      return { ack: result.ack.payload, result: result.event?.payload ?? null };
    }
    case 'file-read': {
      const result = await wsCommand(
        baseUrl,
        'FILE_READ',
        { path: String(assertRequired(options, 'path')) },
        { awaitEvent: 'FILE_CONTENT' },
      );
      return { ack: result.ack.payload, result: result.event?.payload ?? null };
    }
    case 'chat-pin':
      return {
        pinned: true,
        ack: (
          await wsCommand(baseUrl, 'CHAT_PIN', {
            pinned: Boolean(options.pinned === true || options.pinned === 'true'),
          })
        ).ack.payload,
      };
    case 'viewport':
      return {
        sent: true,
        ack: (
          await wsCommand(baseUrl, 'VIEWPORT_SET', {
            x: readNumber(assertRequired(options, 'x'), '--x', -1_000_000, 1_000_000),
            y: readNumber(assertRequired(options, 'y'), '--y', -1_000_000, 1_000_000),
            zoom: readNumber(assertRequired(options, 'zoom'), '--zoom', 0.1, 3),
          })
        ).ack.payload,
      };
    case 'recenter':
      return { sent: true, ack: (await wsCommand(baseUrl, 'VIEWPORT_RECENTER', {})).ack.payload };
    default:
      throw cliError('E_CLI_USAGE', `Unknown mutation command: ${command}`);
  }
}

async function executeHttpMutation(command, options, baseUrl) {
  switch (command) {
    case 'profile-import': {
      const file = assertSafePath(assertRequired(options, 'file'), '--file');
      let profile;
      try {
        profile = JSON.parse(fs.readFileSync(file, 'utf8'));
      } catch (error) {
        throw cliError('E_FILE_READ', `Cannot parse profile JSON: ${error.message}`, 'Provide a valid profile JSON file.', EXIT.USAGE);
      }
      return httpJson(baseUrl, '/api/agents/profiles/import', {
        method: 'POST',
        body: { profile },
      });
    }
    case 'profile-delete': {
      requireYes(options, 'profile-delete');
      const id = String(assertRequired(options, 'id'));
      if (id.length > 128) throw cliError('E_CLI_USAGE', '--id is too long (max 128).');
      return httpJson(baseUrl, `/api/agents/profiles/${encodeURIComponent(id)}`, { method: 'DELETE' });
    }
    case 'memory-clear': {
      requireYes(options, 'memory-clear');
      const key = String(assertRequired(options, 'agent'));
      if (key.length > 128) throw cliError('E_CLI_USAGE', '--agent is too long (max 128).');
      return httpJson(baseUrl, `/api/agents/memory/${encodeURIComponent(key)}`, { method: 'DELETE' });
    }
    default:
      throw cliError('E_CLI_USAGE', `Unknown HTTP mutation command: ${command}`);
  }
}

function requireYes(options, operation) {
  if (!options.yes) {
    throw cliError(
      'E_CONFIRMATION_REQUIRED',
      `${operation} is destructive; pass --yes to confirm.`,
      'Re-run the same command with --yes after verifying the run/task id.',
      EXIT_UNSAFE,
    );
  }
}

// ---------------------------------------------------------------------------
// Lifecycle commands and local output
// ---------------------------------------------------------------------------

async function executeLocal(command, options, baseUrl) {
  switch (command) {
    case 'commands':
      return {
        commands: Object.entries(COMMANDS).map(([name, definition]) => ({
          name,
          summary: definition.summary,
          safety: definition.safety,
          transport: definition.transport,
        })),
      };
    case 'schema': {
      const selected = options.command || options._positional?.[0];
      if (selected) {
        const definition = COMMANDS[selected];
        if (!definition) throw cliError('E_CLI_USAGE', `Unknown command: ${selected}`);
        return { command: selected, ...definition };
      }
      return { schema: CLI_SCHEMA, version: CLI_VERSION, commands: COMMANDS };
    }
    case 'version':
      return { cli_version: CLI_VERSION, cli_schema: CLI_SCHEMA, server_contract: SERVER_SCHEMA };
    case 'status': {
      const state = readState(options);
      let health = null;
      let error = null;
      try {
        health = await httpJson(baseUrl, '/health');
      } catch (failure) {
        error = { code: failure.code, message: failure.message };
      }
      return {
        state,
        url: baseUrl.href.replace(/\/$/, ''),
        running: health !== null,
        health,
        error,
      };
    }
    case 'doctor': {
      const report = { cli_version: CLI_VERSION, node: process.version, checks: [] };
      report.checks.push({ name: 'cli', ok: true, detail: `v${CLI_VERSION}` });
      try {
        const exe = resolveExe(options);
        report.checks.push({ name: 'executable', ok: true, detail: exe });
      } catch (error) {
        report.checks.push({ name: 'executable', ok: false, code: error.code, detail: error.message });
      }
      const state = readState(options);
      report.state = state;
      report.checks.push({ name: 'state', ok: state !== null, detail: state ? state.url : 'no CLI state file' });
      try {
        const health = await httpJson(baseUrl, '/health');
        report.health = health;
        const key = (health.checks || []).find((check) => check.name === 'deepseek_api_key');
        report.checks.push({
          name: 'server',
          ok: true,
          detail: `${health.status} · schema ${health.database?.schema_version ?? 'unknown'}`,
        });
        report.checks.push({
          name: 'provider_key',
          ok: key?.ok === true,
          detail: key ? (key.ok ? 'present' : 'absent') : 'not configured',
          code: key?.code ?? null,
        });
        report.checks.push({ name: 'loopback', ok: true, detail: baseUrl.host });
      } catch (error) {
        report.checks.push({ name: 'server', ok: false, code: error.code, detail: error.message });
      }
      report.ready = report.checks.every((check) => check.name === 'provider_key' ? true : check.ok);
      return report;
    }
    case 'logs': {
      const state = readState(options);
      const dir = dataDirectory(options);
      const lines = options.lines === undefined ? 40 : readNumber(options.lines, '--lines', 1, 500);
      const stdoutPath = state?.stdout_log || path.join(dir, 'server.stdout.log');
      const stderrPath = state?.stderr_log || path.join(dir, 'server.stderr.log');
      const readTail = (file) => {
        try {
          return fs.readFileSync(file, 'utf8').split(/\r?\n/).slice(-lines).join('\n');
        } catch {
          return null;
        }
      };
      const stream = options.stream || 'both';
      return {
        stdout: stream === 'stderr' ? null : readTail(stdoutPath),
        stderr: stream === 'stdout' ? null : readTail(stderrPath),
        stdout_path: stdoutPath,
        stderr_path: stderrPath,
      };
    }
    case 'open': {
      return openUi(baseUrl, options);
    }
    default:
      throw cliError('E_CLI_USAGE', `Unknown local command: ${command}`);
  }
}

function openUi(baseUrl, options) {
  const url = baseUrl.href.replace(/\/$/, '');
  const browser = options.browser || findBrowser();
  if (!browser) {
    throw cliError('E_BROWSER_NOT_FOUND', 'No Edge or Chrome executable found.', 'Pass --browser <path>.', EXIT.REFUSAL);
  }
  const args = options['no-app'] ? [url] : [`--app=${url}`, '--window-size=1440,900'];
  if (options['dry-run']) return { dry_run: true, request: { executable: browser, args } };
  const child = spawn(browser, args, { detached: true, stdio: 'ignore', windowsHide: true });
  child.unref();
  return { opened: true, url, browser };
}

function findBrowser() {
  const candidates =
    process.platform === 'win32'
      ? [
          'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
          'C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe',
          'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
          'C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe',
        ]
      : ['/usr/bin/microsoft-edge', '/usr/bin/google-chrome', '/usr/bin/chromium'];
  return candidates.find((candidate) => fs.existsSync(candidate)) || null;
}

// ---------------------------------------------------------------------------
// Output and error contract
// ---------------------------------------------------------------------------

function emitSuccess(command, data, options, startedAt) {
  const envelope = {
    schema: CLI_SCHEMA,
    ok: command === 'doctor' ? data.ready !== false : true,
    command,
    data,
    meta: {
      cli_schema: CLI_SCHEMA,
      cli_version: CLI_VERSION,
      server_schema: SERVER_SCHEMA,
      generated_at: new Date().toISOString(),
      duration_ms: Date.now() - startedAt,
      ...(options.url ? { url: options.url } : {}),
    },
  };
  if (options.json) {
    process.stdout.write(`${JSON.stringify(envelope, null, options.format === 'jsonl' ? 0 : 2)}\n`);
  } else {
    printHuman(command, data);
  }
}

function emitFailure(command, error, options, startedAt) {
  const record = {
    schema: CLI_SCHEMA,
    ok: false,
    command,
    error: {
      code: error.code || 'E_CLI_ERROR',
      message: error.message,
      how_to_fix: error.howToFix || 'Run `hivemind-cli doctor --json`.',
      retryable: Boolean(error.retryable),
      ...(error.extra?.status ? { http_status: error.extra.status } : {}),
    },
    meta: {
      cli_schema: CLI_SCHEMA,
      cli_version: CLI_VERSION,
      server_schema: SERVER_SCHEMA,
      generated_at: new Date().toISOString(),
      duration_ms: Date.now() - startedAt,
    },
  };
  if (options.json) {
    process.stdout.write(`${JSON.stringify(record, null, 2)}\n`);
  }
  process.stderr.write(`${record.error.code}: ${record.error.message}\n`);
  process.stderr.write(`fix: ${record.error.how_to_fix}\n`);
}

function printHuman(command, data) {
  const write = (line) => process.stderr.write(`${line}\n`);
  if (command === 'commands') {
    for (const row of data.commands) write(`${row.name.padEnd(16)} ${row.safety.padEnd(11)} ${row.summary}`);
    return;
  }
  if (command === 'status') {
    write(`running: ${data.running ? 'yes' : 'no'}`);
    write(`url: ${data.url}`);
    if (data.state?.pid) write(`pid: ${data.state.pid}`);
    if (data.health) write(`health: ${data.health.status}`);
    if (data.error) write(`error: ${data.error.code} — ${data.error.message}`);
    return;
  }
  if (command === 'runs' && Array.isArray(data.runs)) {
    for (const run of data.runs) write(`${run.run_id}  ${String(run.status).padEnd(10)}  ${run.prompt ?? ''}`);
    return;
  }
  if (command === 'schema' && data.command) {
    write(`${data.command} — ${data.summary}`);
    for (const arg of data.args || []) write(`  --${arg.name}${arg.required ? ' (required)' : ''}: ${arg.description}`);
    return;
  }
  if (command === 'schema' && data.commands) {
    write('Commands:');
    for (const [name, definition] of Object.entries(data.commands)) write(`  ${name.padEnd(16)} ${definition.summary}`);
    return;
  }
  if (command === 'doctor') {
    for (const check of data.checks || []) write(`${check.ok ? 'ok' : 'FAIL'}  ${check.name}: ${check.detail}`);
    write(`ready: ${data.ready ? 'yes' : 'no'}`);
    return;
  }
  if (command === 'logs') {
    if (data.stdout) write(`--- stdout (${data.stdout_path}) ---\n${data.stdout}`);
    if (data.stderr) write(`--- stderr (${data.stderr_path}) ---\n${data.stderr}`);
    return;
  }
  write(JSON.stringify(data, null, 2));
}

function humanHelp(command) {
  if (command) {
    const definition = COMMANDS[command];
    if (!definition) return `Unknown command: ${command}\n`;
    const lines = [`${command} — ${definition.summary}`, `safety: ${definition.safety} · transport: ${definition.transport}`];
    for (const arg of definition.args) lines.push(`  --${arg.name}${arg.required ? ' (required)' : ''}: ${arg.description}`);
    return `${lines.join('\n')}\n`;
  }
  const lines = [
    'HiveMind agent-native CLI',
    '',
    'Usage: hivemind-cli <command> [options]',
    '',
    'Lifecycle: start, stop, restart, status, doctor, open, logs',
    'Read: health, diagnose, runs, revisions, state, messages, agent-messages, roster, profiles, memory, projects, models, balance, usage',
    'Mutate: submit, run, cancel, kill, rewire, spawn-subhive, approve, deny, template',
    'Protocol: select-revision, settings, file-tree, file-read, chat-pin, watch',
    'Discovery: commands, schema, version',
    '',
    'Global: --json --format json|table|jsonl --url --data-dir --timeout-ms --dry-run --yes --no-color',
    'Agents: stdout is JSON whenever stdout is not a TTY; errors carry code/message/how_to_fix and a stable exit code.',
  ];
  return `${lines.join('\n')}\n`;
}

function exitCodeFor(error) {
  if (Number.isInteger(error.exitCode)) return error.exitCode;
  return EXIT.REFUSAL;
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

async function main() {
  const startedAt = Date.now();
  let parsed;
  let options = { json: !HUMAN };
  try {
    parsed = parseArgs(process.argv.slice(2));
    options = parsed.options;
    const argv = [...parsed.positional];
    const command = argv.shift();
    if (options.version || command === '-V' || command === '--version') {
      process.stdout.write(`${JSON.stringify({ cli_version: CLI_VERSION, cli_schema: CLI_SCHEMA, server_contract: SERVER_SCHEMA })}\n`);
      return EXIT.OK;
    }
    if (!command || command === 'help' || options.help) {
      if (command && command !== 'help' && options.help) {
        process.stderr.write(humanHelp(command));
        return EXIT.OK;
      }
      process.stderr.write(humanHelp());
      return EXIT.OK;
    }
    if (!COMMANDS[command]) {
      throw cliError('E_CLI_USAGE', `Unknown command: ${command}`, 'Run `hivemind-cli commands`.');
    }
    options._positional = argv;
    const definition = COMMANDS[command];
    if (definition.safety === 'destructive' && options.dryRun !== true) {
      // --yes is checked by the operation itself so dry-run can preview safely.
    }
    const startLike = command === 'start' || command === 'restart';
    const requestedBind = startLike && (options.bind || options.port)
      ? options.bind || `${DEFAULT_HOST}:${options.port}`
      : null;
    const baseUrl = requestedBind ? assertLoopbackUrl(`http://${requestedBind}`) : resolveBaseUrl(options);
    options.url = baseUrl.href.replace(/\/$/, '');
    if (definition.transport === 'process') {
      let data;
      if (command === 'start') data = await startServer(options);
      else if (command === 'stop') data = await stopServer(options);
      else if (command === 'restart') {
        const stopped = await stopServer({ ...options, yes: true, force: options.force });
        const started = await startServer(options);
        data = { stopped, started };
      } else {
        data = await executeLocal(command, options, baseUrl);
      }
      if ((command === 'start' || command === 'restart') && options.open && options['dry-run'] !== true) {
        try {
          data = { ...data, ui: openUi(baseUrl, options) };
        } catch (error) {
          data = {
            ...data,
            ui_error: { code: error.code || 'E_BROWSER_NOT_FOUND', message: error.message },
          };
        }
      }
      emitSuccess(command, data, options, startedAt);
      if (command === 'doctor' && data.ready === false) {
        return EXIT.REFUSAL;
      }
      return EXIT.OK;
    }
    if (definition.transport === 'local') {
      const data = await executeLocal(command, options, baseUrl);
      emitSuccess(command, data, options, startedAt);
      if (command === 'doctor' && data.ready === false) {
        return EXIT.REFUSAL;
      }
      return EXIT.OK;
    }
    if (definition.transport === 'http') {
      if (options.dryRun) {
        emitSuccess(command, { dry_run: true, request: { method: 'GET', path: command } }, options, startedAt);
        return EXIT.OK;
      }
      const data = await executeRead(command, options, baseUrl);
      emitSuccess(command, data, options, startedAt);
      return EXIT.OK;
    }
    if (definition.transport === 'http-mutation') {
      if (options.dryRun) {
        emitSuccess(command, { dry_run: true, request: { method: command === 'profile-import' ? 'POST' : 'DELETE', command } }, options, startedAt);
        return EXIT.OK;
      }
      const data = await executeHttpMutation(command, options, baseUrl);
      emitSuccess(command, data, options, startedAt);
      return EXIT.OK;
    }
    if (definition.transport === 'ws') {
      if (options.dryRun) {
        emitSuccess(command, { dry_run: true, request: { transport: 'ws', command, options: { ...options, _positional: undefined } } }, options, startedAt);
        return EXIT.OK;
      }
      if (command === 'watch') {
        const maxEvents = options['max-events'] === undefined ? 100 : readNumber(options['max-events'], '--max-events', 1, 1000);
        const timeoutMs = options.timeoutMs || 30_000;
        const follow = Boolean(options.follow);
        if (follow) {
          await wsWatch(baseUrl, {
            runId: options.run ? assertUuid(options.run, '--run') : null,
            maxEvents,
            timeoutMs,
            follow: true,
            onEvent: (frame) => process.stdout.write(`${JSON.stringify(frame)}\n`),
          });
          return EXIT.OK;
        }
        const data = await wsWatch(baseUrl, {
          runId: options.run ? assertUuid(options.run, '--run') : null,
          maxEvents,
          timeoutMs,
          follow: false,
        });
        emitSuccess(command, data, options, startedAt);
        return EXIT.OK;
      }
      const data = await executeMutation(command, options, baseUrl);
      emitSuccess(command, data, options, startedAt);
      return EXIT.OK;
    }
    throw cliError('E_CLI_USAGE', `Command has no transport: ${command}`);
  } catch (error) {
    emitFailure(parsed?.positional?.[0] || 'unknown', error, options, startedAt);
    return exitCodeFor(error);
  }
}

process.exitCode = await main();
