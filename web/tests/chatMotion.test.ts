import { describe, expect, it } from 'vitest';
import { parseFencedBlocks } from '../src/chat/ChatMotion';

/**
 * Parser contracts for MessageBody's fenced-block splitter. Rendering is a
 * direct map of these blocks, so exact block text is the safety property:
 * no HTML interpretation and no synthesized content.
 */
describe('parseFencedBlocks', () => {
  it('returns no blocks for empty text', () => {
    expect(parseFencedBlocks('')).toEqual([]);
  });

  it('keeps fence-free text as one exact text block', () => {
    const text = 'Line one\n  indented line\nInline ``` ticks stay literal.';
    expect(parseFencedBlocks(text)).toEqual([{ kind: 'text', text }]);
  });

  it('keeps a streamed reply with no fence bytes as one block', () => {
    const text = 'A streamed answer\nwith a second line and trailing space ';
    expect(parseFencedBlocks(text)).toEqual([{ kind: 'text', text }]);
  });

  it('splits surrounding text from a language-tagged code fence', () => {
    const text = 'Run this:\n```tsx\nconst answer = 42;\nconsole.log(answer);\n```\nDone.';
    expect(parseFencedBlocks(text)).toEqual([
      { kind: 'text', text: 'Run this:\n' },
      {
        kind: 'code',
        code: 'const answer = 42;\nconsole.log(answer);',
        language: 'tsx',
      },
      { kind: 'text', text: 'Done.' },
    ]);
  });

  it('handles consecutive fences, an empty body and a missing language', () => {
    const text = '```\n```\nBetween\n```sh\nls -la\n```';
    expect(parseFencedBlocks(text)).toEqual([
      { kind: 'code', code: '' },
      { kind: 'text', text: 'Between\n' },
      { kind: 'code', code: 'ls -la', language: 'sh' },
    ]);
  });

  it('leaves an unclosed fence as plain text without dropping content', () => {
    const text = 'Streaming...\n```tsx\nconst partial = true;';
    expect(parseFencedBlocks(text)).toEqual([{ kind: 'text', text }]);
  });
});
