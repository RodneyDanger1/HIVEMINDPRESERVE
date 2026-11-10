import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import '@xyflow/react/dist/style.css';
import '@fontsource-variable/inter';
import '@fontsource/playfair-display/600.css';
import { App } from './App';
import './styles/index.css';
import './styles/splash.css';

// The launcher passes a one-time secret in the URL. Send it on same-origin API
// reads; the server requires it when HIVEROUTER_LAUNCH_TOKEN is configured.
const launchToken = (() => {
  try {
    return new URL(window.location.href).searchParams.get('launch');
  } catch {
    return null;
  }
})();
if (launchToken !== null) {
  const originalFetch = window.fetch.bind(window);
  window.fetch = (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    try {
      const rawUrl =
        typeof input === 'string'
          ? input
          : input instanceof URL
            ? input.toString()
            : input.url;
      const target = new URL(rawUrl, window.location.href);
      if (target.origin === window.location.origin && target.pathname.startsWith('/api/')) {
        const headers = new Headers(init?.headers);
        headers.set('x-hivemind-launch', launchToken);
        return originalFetch(input, { ...init, headers });
      }
    } catch {
      // Leave the request untouched when a URL cannot be parsed.
    }
    return originalFetch(input, init);
  };
}

const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element #root is missing from index.html');
}

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
