/**
 * Structured, display-only renderer for `APPROVAL_REQUEST.args_preview`.
 *
 * The wire preview is bounded (512 UTF-16 code units, mirroring
 * `MAX_ARGS_PREVIEW_CHARS`), redacted by the server, and may end mid-token when
 * the server cut it. It is never authoritative: `args_sha256` remains the
 * binding. This component therefore:
 * - parses defensively and never throws on a cut or malformed preview;
 * - distinguishes a server-cut preview from balanced-but-invalid JSON;
 * - renders `apply_patch` / `file_write` arguments as reviewable path +
 *   monospace text blocks, and other tools as key/value rows;
 * - leaves a missing preview visibly unknown instead of reconstructing it.
 *
 * It makes no claim that a patch was applied, that hunks match, or that line
 * numbers were verified; those are executor outcomes, not preview properties.
 */
import { Fragment, type ReactNode } from 'react';

import { MAX_ARGS_PREVIEW_CHARS } from '../api/contract';

/** Local display copy (the shared STRINGS table is outside this change's scope). */
export const APPROVAL_PREVIEW_STRINGS = {
  unavailable: 'preview unavailable',
  truncated: 'truncated preview',
  truncatedDetail:
    'The bounded args_preview ends before the JSON is complete; only the raw preview below is available.',
  malformed: 'preview is not valid JSON',
  malformedDetail:
    'The bounded args_preview could not be parsed; the raw preview below is shown unmodified.',
  rawLabel: 'Raw preview',
  pathLabel: 'File path',
  pathsLabel: 'File paths',
  pathUnavailable: 'file path unavailable in preview',
  patchLabel: 'Patch text',
  patchUnavailable: 'patch text unavailable in preview',
  /** Never claims application; the executor result is the only such evidence. */
  patchNote:
    'Preview only — nothing has been applied by this dialog. Hunk application and line numbers are not verified here.',
  contentLabel: 'Content',
  contentUnavailable: 'content unavailable in preview',
  contentNote:
    'Content is bounded by the args_preview cap and may be shortened.',
  displayTruncated: 'Truncated at the display cap.',
  otherFields: 'Other fields',
  empty: 'preview contains no fields',
  valueUnavailable: '[unavailable]',
} as const;

export type ArgsPreviewStatus = 'missing' | 'parsed' | 'truncated' | 'malformed';

export interface ArgsPreviewParse {
  readonly status: ArgsPreviewStatus;
  /** The exact preview bytes received; empty only when the preview is absent. */
  readonly raw: string;
  /** Present only when the preview parsed as JSON. */
  readonly value?: unknown;
}

/** Generic value rows are bounded independently so one field cannot dominate. */
const GENERIC_VALUE_CHARS = 240;
/** A preview object may carry at most this many generic rows before a summary row. */
const GENERIC_OBJECT_ROWS = 20;
/** An array may show at most this many indexed rows before a summary row. */
const GENERIC_ARRAY_ROWS = 12;
/** Path lists stay reviewable even if a tolerant caller supplies many. */
const MAX_PATHS = 8;

const PATH_KEYS = [
  'path',
  'file_path',
  'filePath',
  'target_path',
  'targetPath',
  'file',
  'target',
] as const;
const PATH_LIST_KEYS = ['paths', 'files'] as const;
const PATCH_KEYS = ['patch', 'diff', 'unified_diff', 'unifiedDiff'] as const;
const PATH_FIELD_KEYS: ReadonlySet<string> = new Set<string>([
  ...PATH_KEYS,
  ...PATH_LIST_KEYS,
]);
const APPLY_PATCH_FIELD_KEYS: ReadonlySet<string> = new Set<string>([
  ...PATH_FIELD_KEYS,
  ...PATCH_KEYS,
]);
const FILE_WRITE_FIELD_KEYS: ReadonlySet<string> = new Set<string>([
  ...PATH_FIELD_KEYS,
  'content',
]);

/**
 * Scans JSON text for structural incompleteness (an unclosed string or
 * object/array) without trusting engine-specific parse messages. A balanced
 * but invalid document returns false and is reported as malformed.
 */
function jsonIsIncomplete(text: string): boolean {
  const stack: string[] = [];
  let inString = false;
  let escaped = false;
  for (const char of text) {
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (char === '\\') {
        escaped = true;
      } else if (char === '"') {
        inString = false;
      }
      continue;
    }
    if (char === '"') {
      inString = true;
    } else if (char === '{') {
      stack.push('}');
    } else if (char === '[') {
      stack.push(']');
    } else if (char === '}' || char === ']') {
      if (stack.pop() !== char) {
        return false;
      }
    }
  }
  return inString || stack.length > 0;
}

/**
 * Defensive preview parse. Missing/blank -> `missing`; parseable -> `parsed`;
 * structurally cut (unclosed string/collection, server ellipsis, or the
 * server's `[truncated]` marker) -> `truncated`; otherwise `malformed`.
 * Never throws.
 */
export function parseArgsPreview(raw: string | null | undefined): ArgsPreviewParse {
  if (typeof raw !== 'string' || raw.trim().length === 0) {
    return { status: 'missing', raw: typeof raw === 'string' ? raw : '' };
  }
  try {
    return { status: 'parsed', raw, value: JSON.parse(raw) as unknown };
  } catch {
    const cutMarker = /…\s*$/.test(raw) || raw.includes('[truncated]');
    return {
      status: jsonIsIncomplete(raw) || cutMarker ? 'truncated' : 'malformed',
      raw,
    };
  }
}

/** Bounds text for display without splitting a surrogate pair. */
function boundedText(
  text: string,
  limit = MAX_ARGS_PREVIEW_CHARS,
): { readonly text: string; readonly truncated: boolean } {
  if (text.length <= limit) {
    return { text, truncated: false };
  }
  let end = Math.max(0, limit - 1);
  if (
    end > 0 &&
    end < text.length &&
    isHighSurrogate(text.charCodeAt(end - 1)) &&
    isLowSurrogate(text.charCodeAt(end))
  ) {
    end -= 1;
  }
  return { text: `${text.slice(0, end)}…`, truncated: true };
}

function isHighSurrogate(code: number): boolean {
  return code >= 0xd800 && code <= 0xdbff;
}

function isLowSurrogate(code: number): boolean {
  return code >= 0xdc00 && code <= 0xdfff;
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function nonEmptyString(value: unknown): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value : null;
}

function valueToText(value: unknown): string {
  if (typeof value === 'string') {
    return value;
  }
  if (value === null) {
    return 'null';
  }
  if (typeof value === 'number' || typeof value === 'boolean' || typeof value === 'bigint') {
    return String(value);
  }
  try {
    return JSON.stringify(value) ?? String(value);
  } catch {
    return APPROVAL_PREVIEW_STRINGS.valueUnavailable;
  }
}

function pathValues(args: Record<string, unknown>): string[] {
  const paths: string[] = [];
  for (const key of PATH_KEYS) {
    const value = nonEmptyString(args[key]);
    if (value !== null) {
      paths.push(value);
    }
  }
  for (const key of PATH_LIST_KEYS) {
    const value = args[key];
    if (Array.isArray(value)) {
      for (const item of value) {
        const itemPath = nonEmptyString(item);
        if (itemPath !== null) {
          paths.push(itemPath);
        }
      }
    } else {
      const itemPath = nonEmptyString(value);
      if (itemPath !== null) {
        paths.push(itemPath);
      }
    }
  }
  return [...new Set(paths)];
}

function patchText(args: Record<string, unknown>): string | null {
  for (const key of PATCH_KEYS) {
    const value = nonEmptyString(args[key]);
    if (value !== null) {
      return value;
    }
  }
  return null;
}

/**
 * Paths named by patch headers. Derived only from text already present in the
 * preview; nothing is guessed when a header is absent. Bare `+++`/`---` lines
 * are trusted only as a paired header block, so an added diff line that starts
 * with `++` cannot be mistaken for a file path.
 */
export function pathsInPatch(patch: string): string[] {
  const found: string[] = [];
  const add = (candidate: string): void => {
    const cleaned = candidate.trim();
    if (cleaned.length > 0 && cleaned !== '/dev/null') {
      found.push(cleaned);
    }
  };
  const explicitHeaders = [
    /^\*\*\* (?:Update|Add|Delete) File: (.+?)[ \t]*$/gm,
    /^diff --git a\/(.+?) b\/(.+?)[ \t]*$/gm,
  ];
  for (const pattern of explicitHeaders) {
    let match: RegExpExecArray | null;
    while ((match = pattern.exec(patch)) !== null) {
      for (const group of match.slice(1)) {
        if (group !== undefined) {
          add(group);
        }
      }
    }
  }
  // Unified-diff file headers are a pair; use the post-image path, falling
  // back to the pre-image only for a deletion (`+++ /dev/null`).
  const pairedHeaders = /^--- (?:a\/)?(.+?)[ \t]*\r?\n\+\+\+ (?:b\/)?(.+?)[ \t]*$/gm;
  let pair: RegExpExecArray | null;
  while ((pair = pairedHeaders.exec(patch)) !== null) {
    const before = pair[1];
    const after = pair[2];
    if (after !== undefined && after.trim() !== '/dev/null') {
      add(after);
    } else if (before !== undefined) {
      add(before);
    }
  }
  return [...new Set(found)];
}

function applyPatchPaths(args: Record<string, unknown>): string[] {
  const paths = pathValues(args);
  const patch = patchText(args);
  if (patch !== null) {
    paths.push(...pathsInPatch(patch));
  }
  return [...new Set(paths)].slice(0, MAX_PATHS);
}

function extraFields(
  args: Record<string, unknown>,
  excluded: ReadonlySet<string>,
): Array<[string, unknown]> {
  return Object.keys(args)
    .filter((key) => !excluded.has(key))
    .slice(0, GENERIC_OBJECT_ROWS)
    .map((key): [string, unknown] => [key, args[key]]);
}

function genericEntries(value: unknown): Array<[string, unknown]> {
  if (Array.isArray(value)) {
    const shown = Math.min(value.length, GENERIC_ARRAY_ROWS);
    const entries: Array<[string, unknown]> = [];
    for (let index = 0; index < shown; index += 1) {
      entries.push([`[${index}]`, value[index]]);
    }
    if (value.length > shown) {
      entries.push(['…', `${value.length - shown} more items`]);
    }
    return entries;
  }
  const record = asRecord(value);
  if (record !== null) {
    const keys = Object.keys(record);
    const shown = keys.slice(0, GENERIC_OBJECT_ROWS);
    const entries = shown.map((key): [string, unknown] => [key, record[key]]);
    if (keys.length > shown.length) {
      entries.push(['…', `${keys.length - shown.length} more fields`]);
    }
    return entries;
  }
  return [['value', value]];
}

const PRE_CLASS =
  'type-mono-code mt-1 max-h-56 overflow-auto whitespace-pre rounded-[var(--radius-sm)] border border-hairline bg-panel-alt p-2 text-ink-900';

function PathSection({
  paths,
  label,
}: {
  readonly paths: readonly string[];
  readonly label: string;
}): ReactNode {
  if (paths.length === 0) {
    return (
      <p data-testid="approval-preview-path-unavailable" className="type-mono-meta text-signal">
        {APPROVAL_PREVIEW_STRINGS.pathUnavailable}
      </p>
    );
  }
  return (
    <div data-testid="approval-preview-paths">
      <p className="type-mono-meta text-meta">{label}</p>
      <ul className="mt-1 list-none space-y-0.5 p-0">
        {paths.map((path) => (
          <li key={path} className="type-mono-code break-all text-ink-900">
            {path}
          </li>
        ))}
      </ul>
    </div>
  );
}

function OtherFields({
  args,
  excluded,
}: {
  readonly args: Record<string, unknown>;
  readonly excluded: ReadonlySet<string>;
}): ReactNode {
  const fields = extraFields(args, excluded);
  if (fields.length === 0) {
    return null;
  }
  return (
    <div data-testid="approval-preview-other-fields">
      <p className="type-mono-meta text-meta">{APPROVAL_PREVIEW_STRINGS.otherFields}</p>
      <dl className="mt-1 grid grid-cols-[minmax(72px,140px)_1fr] gap-x-2 gap-y-1">
        {fields.map(([key, value]) => {
          const text = boundedText(valueToText(value), GENERIC_VALUE_CHARS);
          return (
            <Fragment key={key}>
              <dt className="type-mono-meta break-all text-meta">{key}</dt>
              <dd className="type-mono-code min-w-0 whitespace-pre-wrap break-words text-ink-900">
                {text.text}
              </dd>
            </Fragment>
          );
        })}
      </dl>
    </div>
  );
}

function ApplyPatchPreview({ args }: { readonly args: Record<string, unknown> }): ReactNode {
  const paths = applyPatchPaths(args);
  const patch = patchText(args);
  const patchPreview = patch === null ? null : boundedText(patch);
  return (
    <div
      data-testid="approval-preview-apply-patch"
      data-preview-status="parsed"
      className="min-w-0 space-y-2"
    >
      <PathSection
        paths={paths}
        label={paths.length > 1 ? APPROVAL_PREVIEW_STRINGS.pathsLabel : APPROVAL_PREVIEW_STRINGS.pathLabel}
      />
      {patchPreview === null ? (
        <p data-testid="approval-preview-patch-unavailable" className="type-mono-meta text-signal">
          {APPROVAL_PREVIEW_STRINGS.patchUnavailable}
        </p>
      ) : (
        <div>
          <p className="type-mono-meta text-meta">{APPROVAL_PREVIEW_STRINGS.patchLabel}</p>
          <pre
            tabIndex={0}
            aria-label={APPROVAL_PREVIEW_STRINGS.patchLabel}
            data-testid="approval-patch-text"
            className={PRE_CLASS}
          >
            {patchPreview.text}
          </pre>
          {patchPreview.truncated ? (
            <p className="type-mono-meta mt-1 text-meta">
              {APPROVAL_PREVIEW_STRINGS.displayTruncated}
            </p>
          ) : null}
        </div>
      )}
      <p className="type-mono-meta text-meta">{APPROVAL_PREVIEW_STRINGS.patchNote}</p>
      <OtherFields args={args} excluded={APPLY_PATCH_FIELD_KEYS} />
    </div>
  );
}

function FileWritePreview({ args }: { readonly args: Record<string, unknown> }): ReactNode {
  const paths = pathValues(args);
  const contentValue = args['content'];
  const content = typeof contentValue === 'string' ? contentValue : null;
  const contentPreview = content === null ? null : boundedText(content);
  return (
    <div
      data-testid="approval-preview-file-write"
      data-preview-status="parsed"
      className="min-w-0 space-y-2"
    >
      <PathSection
        paths={paths}
        label={paths.length > 1 ? APPROVAL_PREVIEW_STRINGS.pathsLabel : APPROVAL_PREVIEW_STRINGS.pathLabel}
      />
      {contentPreview === null ? (
        <p data-testid="approval-preview-content-unavailable" className="type-mono-meta text-signal">
          {APPROVAL_PREVIEW_STRINGS.contentUnavailable}
        </p>
      ) : (
        <div>
          <p className="type-mono-meta text-meta">{APPROVAL_PREVIEW_STRINGS.contentLabel}</p>
          <pre
            tabIndex={0}
            aria-label={APPROVAL_PREVIEW_STRINGS.contentLabel}
            data-testid="approval-file-write-content"
            className={PRE_CLASS}
          >
            {contentPreview.text}
          </pre>
          {contentPreview.truncated ? (
            <p className="type-mono-meta mt-1 text-meta">
              {APPROVAL_PREVIEW_STRINGS.displayTruncated}
            </p>
          ) : null}
        </div>
      )}
      <p className="type-mono-meta text-meta">{APPROVAL_PREVIEW_STRINGS.contentNote}</p>
      <OtherFields args={args} excluded={FILE_WRITE_FIELD_KEYS} />
    </div>
  );
}

function GenericPreview({ value }: { readonly value: unknown }): ReactNode {
  const entries = genericEntries(value);
  if (entries.length === 0) {
    return (
      <p
        data-testid="approval-preview-empty"
        data-preview-status="parsed"
        className="type-mono-meta text-meta"
      >
        {APPROVAL_PREVIEW_STRINGS.empty}
      </p>
    );
  }
  return (
    <dl
      data-testid="approval-preview-fields"
      data-preview-status="parsed"
      className="grid grid-cols-[minmax(72px,140px)_1fr] gap-x-2 gap-y-1"
    >
      {entries.map(([key, entry], index) => {
        const text = boundedText(valueToText(entry), GENERIC_VALUE_CHARS);
        return (
          <Fragment key={`${key}-${index}`}>
            <dt className="type-mono-meta break-all text-meta">{key}</dt>
            <dd className="type-mono-code min-w-0 whitespace-pre-wrap break-words text-ink-900">
              {text.text}
            </dd>
          </Fragment>
        );
      })}
    </dl>
  );
}

function UnparsedPreview({ parse }: { readonly parse: ArgsPreviewParse }): ReactNode {
  const truncated = parse.status === 'truncated';
  const raw = boundedText(parse.raw);
  return (
    <div
      data-testid="approval-preview-unparsed"
      data-preview-status={parse.status}
      className="min-w-0 space-y-1"
    >
      <p role="status" data-testid="approval-preview-status" className="type-mono-meta text-signal">
        {truncated ? APPROVAL_PREVIEW_STRINGS.truncated : APPROVAL_PREVIEW_STRINGS.malformed}
      </p>
      <p className="type-mono-meta text-meta">
        {truncated
          ? APPROVAL_PREVIEW_STRINGS.truncatedDetail
          : APPROVAL_PREVIEW_STRINGS.malformedDetail}
      </p>
      <p className="type-mono-meta text-meta">{APPROVAL_PREVIEW_STRINGS.rawLabel}</p>
      <pre
        tabIndex={0}
        aria-label={APPROVAL_PREVIEW_STRINGS.rawLabel}
        data-testid="approval-preview-raw"
        className={PRE_CLASS}
      >
        {raw.text}
      </pre>
      {raw.truncated ? (
        <p className="type-mono-meta text-meta">{APPROVAL_PREVIEW_STRINGS.displayTruncated}</p>
      ) : null}
    </div>
  );
}

export interface ApprovalPreviewProps {
  readonly tool: string;
  readonly argsPreview: string | null | undefined;
}

/**
 * Renders the bounded args preview for one approval. Pure and static-render
 * safe; the caller keeps approve/deny and policy gating outside this component.
 */
export function ApprovalPreview({ tool, argsPreview }: ApprovalPreviewProps): ReactNode {
  const parsed = parseArgsPreview(argsPreview);
  if (parsed.status === 'missing') {
    return (
      <p
        data-testid="approval-preview-unavailable"
        data-preview-status="missing"
        className="type-mono-meta text-meta"
      >
        {APPROVAL_PREVIEW_STRINGS.unavailable}
      </p>
    );
  }
  if (parsed.status !== 'parsed') {
    return <UnparsedPreview parse={parsed} />;
  }
  const record = asRecord(parsed.value);
  if (tool === 'apply_patch' && record !== null) {
    return <ApplyPatchPreview args={record} />;
  }
  if (tool === 'file_write' && record !== null) {
    return <FileWritePreview args={record} />;
  }
  return <GenericPreview value={parsed.value} />;
}
