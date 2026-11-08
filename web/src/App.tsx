/**
 * App root: one store per mount, honest chrome, nav/tab views, 28 surfaces.
 * Bootstrap connects to the local server; when the socket fails the UI shows
 * a named offline banner and blocks mutations — it never fakes a run.
 */
import { useCallback, useEffect, useState } from 'react';
import { createUiApi } from './api';
import { createUiStore } from './state/store';
import type { NavSection, TabId } from './state/store';
import { UiStoreContext, useUiStore } from './state/context';
import { useAppReducedMotion } from './state/useAppReducedMotion';
import { STRINGS } from './state/strings';
import { NavRail } from './panels/NavRail';
import { Header } from './panels/Header';
import { LibraryPanel } from './panels/LibraryPanel';
import { ControlBar } from './panels/ControlBar';
import { Scrubber } from './panels/Scrubber';
import { ToastRegion } from './panels/ToastRegion';
import { SettingsPanel } from './panels/SettingsPanel';
import { DataPanel } from './panels/DataPanel';
import { AgentsPanel } from './panels/AgentsPanel';
import { ModelsPanel } from './panels/ModelsPanel';
import { RoutingPanel } from './panels/RoutingPanel';
import { ApprovalOverlay } from './panels/ApprovalOverlay';
import { SwarmPanel } from './panels/SwarmPanel';
import { HistoryPanel } from './panels/HistoryPanel';
import { Canvas } from './flow/Canvas';
import { ChatCore } from './chat/ChatCore';

const NAV_SECTIONS = ['workspace', 'agents', 'data', 'models', 'routing', 'settings'] as const;
const TAB_IDS = ['workspace', 'swarm', 'routing', 'history'] as const;

function AppFrame() {
  const nav = useUiStore((state) => state.view.nav);
  const tab = useUiStore((state) => state.view.tab);
  const setNav = useUiStore((state) => state.setNav);
  const setTab = useUiStore((state) => state.setTab);
  const connection = useUiStore((state) => state.connection);
  const connectionRefusal = useUiStore((state) => state.connectionRefusal);
  const apiKind = useUiStore((state) => state.apiKind);
  const run = useUiStore((state) => state.run);
  const reduced = useAppReducedMotion();

  const applyHash = useCallback((): void => {
    const raw = window.location.hash.replace(/^#\/?/, '');
    const [nextNavRaw, nextTabRaw] = raw.split('/');
    const nextNav = (NAV_SECTIONS as readonly string[]).includes(nextNavRaw)
      ? (nextNavRaw as NavSection)
      : null;
    const nextTab = (TAB_IDS as readonly string[]).includes(nextTabRaw)
      ? (nextTabRaw as TabId)
      : null;
    if (nextNav !== null) setNav(nextNav);
    if (nextTab !== null) setTab(nextTab);
  }, [setNav, setTab]);

  useEffect(() => {
    applyHash();
    const onHashChange = (): void => applyHash();
    window.addEventListener('hashchange', onHashChange);
    return () => window.removeEventListener('hashchange', onHashChange);
  }, [applyHash]);

  useEffect(() => {
    const next = `#/${nav}/${tab}`;
    if (window.location.hash === next) return;
    window.history.replaceState(null, '', next);
  }, [nav, tab]);

  useEffect(() => {
    document.documentElement.dataset.reducedMotion = reduced ? 'true' : 'false';
  }, [reduced]);

  const banner =
    connection === 'error' || connection === 'closed'
      ? connectionRefusal?.message ?? STRINGS.connection.offlineBanner
      : connection === 'reconnecting'
        ? STRINGS.connection.reconnectBanner
        : null;
  const skipTarget = nav === 'workspace' && tab === 'workspace' ? '#hive-canvas' : '#main-content';

  const workspaceView =
    tab === 'swarm' ? (
      <SwarmPanel />
    ) : tab === 'routing' ? (
      <RoutingPanel />
    ) : tab === 'history' ? (
      <div className="flex h-full min-h-0">
        <HistoryPanel />
        <div className="relative min-h-0 min-w-0 flex-1">
          {run !== null ? (
            <Canvas />
          ) : (
            <div className="flex h-full items-center justify-center p-6">
              <div className="surface-panel max-w-[420px] rounded-[16px] p-5">
                <div className="type-mono-meta text-[8px] tracking-[0.14em] text-meta">COMMITTED HISTORY</div>
                <h2 className="mt-1 font-display text-[26px] leading-tight text-ink-900">Nothing to replay yet</h2>
                <p className="mt-2 text-[12.5px] leading-[18px] text-ink-500">
                  History is read-only and only appears after a run commits a revision. No graph is
                  shown when there is no authoritative snapshot to replay.
                </p>
                <button
                  type="button"
                  className="type-label mt-4 h-9 rounded-[8px] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400"
                  onClick={() => setTab('workspace')}
                >
                  Go to Workspace
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    ) : (
      <div className="relative h-full min-h-0">
        <Canvas />
        <ChatCore />
      </div>
    );

  const content =
    nav === 'agents' ? (
      <AgentsPanel />
    ) : nav === 'data' ? (
      <DataPanel />
    ) : nav === 'models' ? (
      <ModelsPanel />
    ) : nav === 'routing' ? (
      <RoutingPanel />
    ) : nav === 'settings' ? (
      <SettingsPanel />
    ) : (
      workspaceView
    );

  return (
    <div className="flex h-full min-h-0 flex-col bg-canvas text-ink-900">
      <a className="skip-link" href={skipTarget}>
        {STRINGS.app.skipToCanvas}
      </a>
      <Header />
      <div className="flex min-h-0 flex-1">
        <NavRail />
        <LibraryPanel />
        <main id="main-content" tabIndex={-1} className="relative flex min-w-0 flex-1 flex-col">
          {apiKind === 'mock' ? (
            <div className="type-mono-meta border-b border-amber bg-recessed px-4 py-1 text-ink-800">
              {STRINGS.connection.mock}
            </div>
          ) : null}
          {banner ? (
            <div role="alert" className="type-mono-meta border-b border-signal bg-panel px-4 py-1 text-signal">
              {banner}
            </div>
          ) : null}
          <div className="relative min-h-0 flex-1 overflow-hidden">{content}</div>
          {nav === 'workspace' && tab === 'workspace' ? (
            <div className="absolute inset-x-0 bottom-0 z-30">
              <ControlBar />
              <Scrubber />
            </div>
          ) : null}
          {nav === 'workspace' && tab === 'history' ? (
            <div className="absolute inset-x-0 bottom-0 z-30">
              <Scrubber />
            </div>
          ) : null}
        </main>
      </div>
      <ToastRegion />
      <ApprovalOverlay />
    </div>
  );
}

export function App() {
  const [store] = useState(() => createUiStore(createUiApi()));

  useEffect(() => {
    void store.getState().init();
    return () => {
      store.getState().dispose();
    };
  }, [store]);

  return (
    <UiStoreContext.Provider value={store}>
      <AppFrame />
    </UiStoreContext.Provider>
  );
}
