import { createElement, type ReactElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';

import type { UiApi } from '../src/api/client';
import {
  ChatCore,
  MAX_RENDERED_CHAT_MESSAGES,
  visibleChatMessages,
} from '../src/chat/ChatCore';
import { UiStoreContext } from '../src/state/context';
import { createUiStore, type ChatMessage, type UiState } from '../src/state/store';
import { STRINGS } from '../src/state/strings';

/**
 * Chat streaming surface: bounded transcript, stable transcript label, and no
 * whole-transcript live region that would announce every token.
 */

const SESSION = { token: 'test-token', schema: 'hiverouter.v1', last_seq: 0, ws: '/ws' };

function fakeApi(): UiApi {
  return {
    kind: 'ws',
    async open() {
      return SESSION;
    },
    close() {
      /* no session to close in a static render */
    },
    async send() {
      /* commands are not sent by static renders */
    },
    subscribe() {
      return () => undefined;
    },
    subscribeStatus() {
      return () => undefined;
    },
    subscribeClientError() {
      return () => undefined;
    },
  } as unknown as UiApi;
}

function renderWith(node: ReactElement, state: Partial<UiState> = {}): string {
  const store = createUiStore(fakeApi());
  store.setState({ connection: 'open' as const, chat: { pinned: true, visible: true }, ...state });
  if (Object.keys(state).length > 0) {
    store.setState(state);
  }
  (store as unknown as { getInitialState: () => UiState }).getInitialState = () =>
    store.getState();
  return renderToStaticMarkup(
    createElement(UiStoreContext.Provider, { value: store }, node),
  );
}

const countOf = (source: string, needle: string): number => source.split(needle).length - 1;

function message(index: number): ChatMessage {
  return {
    id: `m-${index}`,
    role: index % 2 === 0 ? 'user' : 'assistant',
    text: `message-${String(index).padStart(3, '0')}`,
    runId: 'run-1',
    streaming: false,
  };
}

describe('ChatCore transcript cap', () => {
  it('returns the same array under the cap and the newest slice above it', () => {
    const small: readonly ChatMessage[] = [message(0), message(1)];
    expect(visibleChatMessages(small)).toBe(small);
    expect(visibleChatMessages([])).toEqual([]);

    const large = Array.from({ length: MAX_RENDERED_CHAT_MESSAGES + 2 }, (_, index) =>
      message(index),
    );
    const visible = visibleChatMessages(large);
    expect(visible).toHaveLength(MAX_RENDERED_CHAT_MESSAGES);
    expect(visible[0]?.id).toBe('m-2');
    expect(visible[visible.length - 1]?.id).toBe(`m-${large.length - 1}`);
  });

  it('renders an explicit cap label and only the newest messages', () => {
    const total = MAX_RENDERED_CHAT_MESSAGES + 5;
    const messages = Array.from({ length: total }, (_, index) => message(index));
    const markup = renderWith(createElement(ChatCore), { messages });

    expect(markup).toContain(`Showing the latest ${MAX_RENDERED_CHAT_MESSAGES} of ${total} messages`);
    expect(markup).toContain('5 earlier messages are hidden.');
    expect(markup).toContain('message-005');
    expect(markup).toContain(`message-${String(total - 1).padStart(3, '0')}`);
    expect(markup).not.toContain('message-004');
  });

  it('names the transcript once and keeps the token stream out of aria-live', () => {
    const markup = renderWith(createElement(ChatCore), {
      messages: [message(0), message(1)],
    });
    // Section and scroll container must not share one accessible name.
    expect(countOf(markup, `aria-label="${STRINGS.chat.aria}"`)).toBe(1);
    expect(countOf(markup, 'aria-label="Chat transcript"')).toBe(1);
    // Only the live connection/stream status remains live; the transcript itself
    // is not a live region, so REPLY_DELTA does not re-announce every token.
    expect(countOf(markup, 'aria-live=')).toBe(1);
  });
});
