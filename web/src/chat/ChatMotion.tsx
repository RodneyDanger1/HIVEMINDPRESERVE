/**
 * Chat motion primitives: the working bee, streaming typing dots, fenced code
 * blocks, and a safe renderer for streamed assistant text.
 *
 * Assistant text is displayed verbatim. Fenced blocks are the only structure
 * the renderer recognizes, code is emitted as text (never HTML), and no
 * content is synthesized. Colour and type come from the existing honey/ink
 * tokens; the animation names live in the shared motion stylesheet.
 */

import { memo } from 'react';

export type MessageBlock =
  | { readonly kind: 'text'; readonly text: string }
  | { readonly kind: 'code'; readonly code: string; readonly language?: string };

const TYPING_DOT_DELAYS = ['0ms', '160ms', '320ms'] as const;

/** The info string of an opening fence line, or null when the line is not one. */
function fenceInfo(line: string): string | null {
  const match = /^[ \t]*```([^`]*)$/.exec(line);
  return match === null ? null : (match[1] ?? '').trim();
}

function isClosingFence(line: string): boolean {
  return /^[ \t]*```[ \t\r]*$/.test(line);
}

function languageOf(info: string): string | undefined {
  return info.length === 0 ? undefined : info.split(/\s+/)[0];
}

/**
 * Split message text into ordered text and fenced-code blocks. Text outside
 * fences is kept byte-for-byte (including whitespace); one newline directly
 * before a closing fence is treated as the fence line break, not as code.
 * An unclosed fence stays plain text so an in-flight reply is never guessed at.
 */
export function parseFencedBlocks(text: string): MessageBlock[] {
  const blocks: MessageBlock[] = [];
  let lineStart = 0;
  let plainStart = 0;

  for (;;) {
    const newline = text.indexOf('\n', lineStart);
    const lineEnd = newline === -1 ? text.length : newline;
    const info = fenceInfo(text.slice(lineStart, lineEnd));

    if (info === null) {
      if (newline === -1) {
        break;
      }
      lineStart = newline + 1;
      continue;
    }

    const bodyStart = newline === -1 ? text.length : newline + 1;
    let scan = bodyStart;
    let closeStart = -1;
    let closeEnd = text.length;

    while (scan <= text.length) {
      const closeNewline = text.indexOf('\n', scan);
      const closeLineEnd = closeNewline === -1 ? text.length : closeNewline;
      if (isClosingFence(text.slice(scan, closeLineEnd))) {
        closeStart = scan;
        closeEnd = closeLineEnd;
        break;
      }
      if (closeNewline === -1) {
        break;
      }
      scan = closeNewline + 1;
    }

    if (closeStart === -1) {
      break;
    }

    if (lineStart > plainStart) {
      blocks.push({ kind: 'text', text: text.slice(plainStart, lineStart) });
    }

    const code = text.slice(bodyStart, closeStart).replace(/\n$/, '');
    const language = languageOf(info);
    blocks.push(
      language === undefined ? { kind: 'code', code } : { kind: 'code', code, language },
    );

    plainStart = closeEnd < text.length ? closeEnd + 1 : text.length;
    lineStart = plainStart;
  }

  if (plainStart < text.length) {
    blocks.push({ kind: 'text', text: text.slice(plainStart) });
  }

  return blocks;
}

/** 16px inline bee: rotating while the agent works, still and subdued at rest. */
export function WorkingBee({ running }: { running: boolean }) {
  return (
    <svg
      viewBox="0 0 20 20"
      aria-hidden="true"
      className={'h-4 w-4 shrink-0 ' + (running ? 'motion-work-bee' : 'opacity-60')}
    >
      <ellipse cx="10" cy="11" rx="5" ry="3.4" fill="var(--color-honey-300)" />
      <path d="M5.5 11h9M7 8.8h6M7 13.2h6" stroke="var(--color-ink-800)" strokeWidth="1.1" />
      <path
        d="M8 7.6C6.7 5.8 4.8 5.4 3.6 6.1c1.5.5 2.3 1.2 4.1 2.2ZM12 7.6c1.3-1.8 3.2-2.2 4.4-1.5-1.5.5-2.3 1.2-4.1 2.2Z"
        fill="var(--color-honey-100)"
        opacity="0.9"
      />
    </svg>
  );
}

/** Three 4px honey dots while an assistant reply has not produced text yet. */
export function TypingDots() {
  return (
    <span className="inline-flex items-center gap-1" aria-hidden="true">
      {TYPING_DOT_DELAYS.map((delay) => (
        <span
          key={delay}
          className="motion-typing-dot h-1 w-1 rounded-full bg-honey-500"
          style={{ animationDelay: delay }}
        />
      ))}
    </span>
  );
}

/** Dark code surface with traffic dots and an optional language label. */
export function CodeBlock({ code, language }: { code: string; language?: string }) {
  const label = language !== undefined && language.trim().length > 0 ? language : null;

  return (
    <div className="overflow-hidden rounded-[8px] bg-ink-800">
      <div className="flex items-center gap-1.5 px-3 pt-2">
        <span className="flex gap-1.5" aria-hidden="true">
          <span className="h-2 w-2 rounded-full bg-signal" />
          <span className="h-2 w-2 rounded-full bg-honey-400" />
          <span className="h-2 w-2 rounded-full bg-verify" />
        </span>
        {label === null ? null : (
          <span className="type-mono-meta ml-auto truncate text-honey-100/80">{label}</span>
        )}
      </div>
      <pre className="type-mono-code overflow-x-auto px-3 pb-3 pt-2 text-honey-100/80">
        <code>{code}</code>
      </pre>
    </div>
  );
}

/** Render message text safely: plain text keeps whitespace, fences become code. */
function MessageBodyImpl({ text }: { text: string }) {
  return (
    <>
      {parseFencedBlocks(text).map((block, index) =>
        block.kind === 'code' ? (
          <div key={`code-${index}`} className="my-2">
            <CodeBlock code={block.code} language={block.language} />
          </div>
        ) : (
          <div
            key={`text-${index}`}
            className="whitespace-pre-wrap text-[12.5px] leading-[18px] text-ink-900"
          >
            {block.text}
          </div>
        ),
      )}
    </>
  );
}

/** Finalized messages keep their identity; streaming text changes re-parse only
 * that message. */
export const MessageBody = memo(MessageBodyImpl);
