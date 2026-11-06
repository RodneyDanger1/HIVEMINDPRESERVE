/**
 * App root: one store per mount, honest chrome, nav/tab views, 28 surfaces.
 * Bootstrap connects to the local server; when the socket fails the UI shows
 * a named offline banner and blocks mutations — it never fakes a run.
 */
import { useEffect, useState } from 'react';
import { createUiApi } from './api';
import { createUiStore } from './state/store';
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
import { DiffDecisionModal } from './panels/DiffDecisionModal';
import { Canvas } from './flow/Canvas';
import { ChatCore } from './chat/ChatCore';

function AppFrame() {
  const nav = useUiStore((state) => state.view.nav);
  const tab = useUiStore((state) => state.view.tab);
  const connection = useUiStore((state) => state.connection);
  const connectionRefusal = useUiStore((state) => state.connectionRefusal);
  const apiKind = useUiStore((state) => state.apiKind);
  const reduced = useAppReducedMotion();

  useEffect(() => {
    document.documentElement.dataset.reducedMotion = reduced ? 'true' : 'false';
  }, [reduced]);

  const banner =
    connection === 'error' || connection === 'closed'
      ? connectionRefusal?.message ?? STRINGS.connection.offlineBanner
      : connection === 'reconnecting'
        ? STRINGS.connection.reconnectBanner
        : null;

  const workspaceView =
    tab === 'routing' ? (
      <RoutingPanel />
    ) : tab === 'history' && nav === 'workspace' ? (
      <div className="relative h-full min-h-0">
        <Canvas />
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
    <div className="flex h-full min-h-0 bg-canvas text-ink-900">
      <a className="skip-link" href="#hive-canvas">
        {STRINGS.app.skipToCanvas}
      </a>
      <NavRail />
      <main className="flex min-w-0 flex-1 flex-col">
        <Header />
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
        {nav === 'workspace' ? <ControlBar /> : null}
        {nav === 'workspace' ? <Scrubber /> : null}
      </main>
      <LibraryPanel />
      <ToastRegion />
      <ApprovalOverlay />
      <DiffDecisionModal />
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
