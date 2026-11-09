/**
 * Remote lane (R1): tree-free fixture principals, devices, grants, explicit
 * light/garage actions and a nonce-bound critical approval. No optimistic state,
 * no remote task row, no network adapter.
 */
import { useCallback, useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStoreApi } from '../state/context';
import { STRINGS } from '../state/strings';
import { EmptyState, PanelTitle } from './StatePack';
import { ConfirmDialog } from './ConfirmDialog';
import { DaemonPanel } from './DaemonPanel';

type Status = 'idle' | 'loading' | 'ok' | 'error';

interface RemoteRow {
  readonly remote_id: string;
  readonly display_name: string;
  readonly owner_kind: string;
  readonly owner_id: string;
  readonly transport_kind: string;
  readonly adapter_kind: string;
  readonly state: string;
  readonly last_seen_at: string | null;
}

interface RemoteSummary {
  readonly remote: RemoteRow;
  readonly device_count: number;
  readonly grant_count: number;
}

interface DeviceRow {
  readonly device_id: string;
  readonly remote_id: string;
  readonly class: string;
  readonly name: string;
  readonly room: string | null;
  readonly state: unknown;
  readonly state_version: number;
  readonly last_seen_at: string | null;
  readonly safety_class: string;
}

interface GrantRow {
  readonly grant_id: string;
  readonly device_id: string;
  readonly capability_id: string;
  readonly action: string;
}

interface AuditRow {
  readonly at: string;
  readonly actor_kind: string;
  readonly action: string;
  readonly outcome: string;
  readonly device_id: string | null;
}

interface RemoteDetail {
  readonly remote: RemoteRow;
  readonly devices: readonly DeviceRow[];
  readonly grants: readonly GrantRow[];
  readonly audit: readonly AuditRow[];
}

interface PendingApproval {
  readonly action_id: string;
  readonly nonce: string;
  readonly device_id: string;
  readonly action: string;
}

function deviceStateLabel(device: DeviceRow): string {
  if (typeof device.state === 'object' && device.state !== null) {
    const value = device.state as Record<string, unknown>;
    if (typeof value.power === 'string') return `power ${value.power}`;
    if (typeof value.position === 'string') return `position ${value.position}`;
    if (typeof value.locked === 'boolean') return value.locked ? 'locked' : 'unlocked';
  }
  return 'state unknown';
}

function defaultCapability(device: DeviceRow): { capability: string; action: string } {
  if (device.class === 'cover') return { capability: 'power', action: 'open' };
  if (device.class === 'lock') return { capability: 'power', action: 'unlock' };
  return { capability: 'power', action: 'toggle' };
}

export function RemotePanel() {
  const storeApi = useUiStoreApi();
  const [status, setStatus] = useState<Status>('idle');
  const [remotes, setRemotes] = useState<readonly RemoteSummary[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<RemoteDetail | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [name, setName] = useState('Fixture remote');
  const [deviceName, setDeviceName] = useState('Study light');
  const [deviceClass, setDeviceClass] = useState('light');
  const [pendingApproval, setPendingApproval] = useState<PendingApproval | null>(null);
  const [confirmRevoke, setConfirmRevoke] = useState(false);
  const [busy, setBusy] = useState(false);

  const token = (): string | null => storeApi.getState().session?.token ?? null;

  const loadRemotes = useCallback(async (): Promise<void> => {
    setStatus('loading');
    try {
      const response = await fetch(`${apiBaseUrl()}/api/remotes`, { headers: { Accept: 'application/json' } });
      if (!response.ok) throw new Error(`remote list HTTP ${response.status}`);
      const body = (await response.json()) as { remotes?: unknown };
      const rows: RemoteSummary[] = [];
      if (Array.isArray(body.remotes)) {
        for (const raw of body.remotes) {
          if (typeof raw !== 'object' || raw === null) continue;
          const item = raw as Record<string, unknown>;
          if (typeof item.remote !== 'object' || item.remote === null) continue;
          const remote = item.remote as RemoteRow;
          if (typeof remote.remote_id !== 'string') continue;
          rows.push({
            remote,
            device_count: typeof item.device_count === 'number' ? item.device_count : 0,
            grant_count: typeof item.grant_count === 'number' ? item.grant_count : 0,
          });
        }
      }
      setRemotes(rows);
      setStatus('ok');
      setSelectedId((current) => current ?? rows[0]?.remote.remote_id ?? null);
    } catch {
      setRemotes([]);
      setStatus('error');
    }
  }, []);

  const loadDetail = useCallback(async (remoteId: string): Promise<void> => {
    try {
      const response = await fetch(`${apiBaseUrl()}/api/remotes/${encodeURIComponent(remoteId)}`, {
        headers: { Accept: 'application/json' },
      });
      if (!response.ok) throw new Error(`remote detail HTTP ${response.status}`);
      const body = (await response.json()) as RemoteDetail;
      setDetail(body);
    } catch {
      setDetail(null);
    }
  }, []);

  useEffect(() => {
    void loadRemotes();
  }, [loadRemotes]);

  useEffect(() => {
    if (selectedId !== null) void loadDetail(selectedId);
  }, [selectedId, loadDetail]);

  const write = async (path: string, body: unknown): Promise<Record<string, unknown>> => {
    const auth = token();
    if (!auth) throw new Error('Session token unavailable');
    const response = await fetch(`${apiBaseUrl()}${path}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
        'x-hivemind-token': auth,
      },
      body: JSON.stringify(body),
    });
    const payload = (await response.json().catch(() => ({}))) as Record<string, unknown>;
    if (!response.ok) {
      throw new Error(typeof payload.code === 'string' ? payload.code : `HTTP ${response.status}`);
    }
    return payload;
  };

  const createRemote = async (): Promise<void> => {
    if (busy || name.trim().length === 0) return;
    setBusy(true);
    try {
      const created = await write('/api/remotes', { display_name: name.trim(), transport_kind: 'in_process' });
      setNotice('Remote created.');
      setSelectedId(typeof created.remote_id === 'string' ? created.remote_id : null);
      await loadRemotes();
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'remote create refused');
    } finally {
      setBusy(false);
    }
  };

  const createDevice = async (): Promise<void> => {
    if (busy || selectedId === null || deviceName.trim().length === 0) return;
    setBusy(true);
    try {
      const safety = deviceClass === 'light' || deviceClass === 'switch' ? 'reversible' : 'critical';
      await write(`/api/remotes/${encodeURIComponent(selectedId)}/devices`, {
        class: deviceClass,
        name: deviceName.trim(),
        safety_class: safety,
      });
      setNotice('Device registered.');
      await loadDetail(selectedId);
      await loadRemotes();
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'device create refused');
    } finally {
      setBusy(false);
    }
  };

  const grantDevice = async (device: DeviceRow): Promise<void> => {
    if (busy || selectedId === null) return;
    const capability = defaultCapability(device);
    setBusy(true);
    try {
      await write(`/api/remotes/${encodeURIComponent(selectedId)}/grants`, {
        device_id: device.device_id,
        capability_id: capability.capability,
        action: capability.action,
      });
      setNotice(`Granted ${capability.action}.`);
      await loadDetail(selectedId);
      await loadRemotes();
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'grant refused');
    } finally {
      setBusy(false);
    }
  };

  const act = async (device: DeviceRow): Promise<void> => {
    if (busy) return;
    const capability = defaultCapability(device);
    setBusy(true);
    try {
      const payload = await write(`/api/devices/${encodeURIComponent(device.device_id)}/actions`, {
        capability_id: capability.capability,
        action: capability.action,
        params: {},
        idempotency_key: crypto.randomUUID(),
      });
      if (payload.status === 'awaiting_approval') {
        const nonce = payload.approval_nonce;
        const actionId = payload.action_id;
        if (typeof nonce === 'string' && typeof actionId === 'string') {
          setPendingApproval({
            action_id: actionId,
            nonce,
            device_id: device.device_id,
            action: capability.action,
          });
          setNotice('Critical action awaiting explicit approval.');
        }
      } else {
        setNotice(`Action ${String(payload.status)}.`);
      }
      await loadDetail(device.remote_id);
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'action refused');
    } finally {
      setBusy(false);
    }
  };

  const resolve = async (approve: boolean): Promise<void> => {
    if (busy || pendingApproval === null) return;
    setBusy(true);
    try {
      const payload = await write(`/api/device-actions/${encodeURIComponent(pendingApproval.action_id)}/resolve`, {
        nonce: pendingApproval.nonce,
        decision: approve ? 'approve' : 'deny',
      });
      setNotice(`Approval result: ${String(payload.status)}.`);
      setPendingApproval(null);
      if (selectedId !== null) await loadDetail(selectedId);
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'approval refused');
    } finally {
      setBusy(false);
    }
  };

  const revokeRemote = async (): Promise<void> => {
    if (busy || selectedId === null) return;
    setBusy(true);
    try {
      await write(`/api/remotes/${encodeURIComponent(selectedId)}/revoke`, {});
      setConfirmRevoke(false);
      setNotice('Remote revoked; grants and actions are now refused.');
      await loadRemotes();
      await loadDetail(selectedId);
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'revoke refused');
    } finally {
      setBusy(false);
    }
  };

  const heartbeatRemote = async (): Promise<void> => {
    if (busy || selectedId === null) return;
    setBusy(true);
    try {
      const remote = await write(
        `/api/remotes/${encodeURIComponent(selectedId)}/heartbeat`,
        {},
      );
      setNotice(`Heartbeat accepted; state ${String(remote.state)}.`);
      await loadRemotes();
      await loadDetail(selectedId);
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'heartbeat refused');
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="h-full min-h-0 overflow-y-auto p-4">
      <div className="mx-auto max-w-[1120px]">
        <PanelTitle>{STRINGS.panels.remoteTitle}</PanelTitle>
        <p className="type-body-l mb-3 max-w-[720px] text-ink-500">
          Tree-free principals outside the run DAG. They do not consume run agents/calls and never become task cards.
          Fixture devices only; no network adapter is enabled in this build.
        </p>

        {notice ? <p role="status" className="type-mono-meta mb-2 text-meta">{notice}</p> : null}
        {status === 'loading' ? <p className="type-mono-meta text-meta">Loading remote lane…</p> : null}
        {status === 'error' ? <p role="alert" className="type-mono-meta text-signal">Remote lane unavailable; no card invented.</p> : null}

        <section className="surface-panel rounded-[var(--radius-card)] p-3">
          <h2 className="type-label text-ink-900">Create fixture remote</h2>
          <div className="mt-2 flex flex-wrap gap-2">
            <input
              className="type-mono-code h-8 min-w-[180px] flex-1 rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 text-ink-900"
              value={name}
              maxLength={120}
              onChange={(event) => setName(event.target.value)}
              aria-label="Remote display name"
            />
            <button
              type="button"
              className="type-label h-8 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400 disabled:opacity-50"
              disabled={busy}
              onClick={() => void createRemote()}
            >
              Create remote
            </button>
          </div>
        </section>

        {remotes.length === 0 && status === 'ok' ? <EmptyState message="No remotes enrolled." /> : null}

        <ul className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
          {remotes.map((summary) => {
            const selected = summary.remote.remote_id === selectedId;
            return (
              <li key={summary.remote.remote_id}>
                <button
                  type="button"
                  aria-pressed={selected}
                  onClick={() => setSelectedId(summary.remote.remote_id)}
                  className={
                    'surface-panel w-full rounded-[var(--radius-card)] p-3 text-left transition-colors ' +
                    (selected ? 'border-honey-500' : 'hover:border-honey-400')
                  }
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="type-label text-ink-900">{summary.remote.display_name}</span>
                    <span className="type-mono-meta text-meta">{summary.remote.state}</span>
                  </div>
                  <div className="type-mono-meta mt-1 text-meta">
                    {summary.remote.owner_kind}:{summary.remote.owner_id} · {summary.remote.adapter_kind} · {summary.device_count} devices · {summary.grant_count} grants
                  </div>
                </button>
              </li>
            );
          })}
        </ul>

        {detail ? (
          <div className="mt-4 grid gap-3 lg:grid-cols-[1.2fr_1fr]">
            <section className="surface-panel rounded-[var(--radius-card)] p-3">
              <div className="flex items-center justify-between gap-2">
                <h2 className="type-label text-ink-900">Devices</h2>
                {detail.remote.state === 'revoked' ? (
                  <span className="type-mono-meta text-[11px] text-signal">revoked</span>
                ) : (
                  <div className="flex items-center gap-1">
                    <button
                      type="button"
                      className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 hover:bg-honey-100 disabled:opacity-50"
                      disabled={busy}
                      onClick={() => void heartbeatRemote()}
                    >
                      Heartbeat
                    </button>
                    <button
                      type="button"
                      className="type-label h-8 rounded-[var(--radius-button)] border border-signal px-3 text-signal hover:bg-recessed disabled:opacity-50"
                      disabled={busy}
                      onClick={() => setConfirmRevoke(true)}
                    >
                      Revoke remote
                    </button>
                  </div>
                )}
              </div>
              <div className="mt-2 flex flex-wrap gap-2">
                <input
                  className="type-mono-code h-8 min-w-[160px] flex-1 rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 text-ink-900"
                  value={deviceName}
                  onChange={(event) => setDeviceName(event.target.value)}
                  aria-label="Device name"
                />
                <select
                  className="type-mono-code h-8 rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 text-ink-900"
                  value={deviceClass}
                  onChange={(event) => setDeviceClass(event.target.value)}
                  aria-label="Device class"
                >
                  <option value="light">light</option>
                  <option value="switch">switch</option>
                  <option value="cover">garage cover</option>
                  <option value="lock">lock</option>
                </select>
                <button
                  type="button"
                  className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 hover:bg-honey-100 disabled:opacity-50"
                  disabled={busy}
                  onClick={() => void createDevice()}
                >
                  Add device
                </button>
              </div>
              <ul className="mt-3 space-y-2">
                {detail.devices.map((device) => {
                  const capability = defaultCapability(device);
                  const grant = detail.grants.some(
                    (row) =>
                      row.device_id === device.device_id &&
                      row.capability_id === capability.capability &&
                      row.action === capability.action,
                  );
                  return (
                    <li key={device.device_id} className="rounded-[var(--radius-md)] border border-hairline bg-panel-alt p-2">
                      <div className="flex items-center justify-between gap-2">
                        <span className="type-mono-code text-ink-900">{device.name}</span>
                        <span className="type-mono-meta text-meta">{device.class} · {device.safety_class}</span>
                      </div>
                      <div className="type-mono-meta mt-1 text-meta">
                        {deviceStateLabel(device)} · v{device.state_version}
                      </div>
                      <div className="mt-2 flex gap-1">
                        {!grant ? (
                          <button
                            type="button"
                            className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 hover:bg-honey-100 disabled:opacity-50"
                            disabled={busy}
                            onClick={() => void grantDevice(device)}
                          >
                            Grant {capability.action}
                          </button>
                        ) : (
                          <button
                            type="button"
                            className="type-label h-8 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400 disabled:opacity-50"
                            disabled={busy || pendingApproval !== null}
                            onClick={() => void act(device)}
                          >
                            {capability.action === 'toggle'
                              ? deviceStateLabel(device).includes('on')
                                ? 'Turn off'
                                : 'Turn on'
                              : capability.action === 'open'
                                ? 'Open garage'
                                : 'Unlock'}
                          </button>
                        )}
                      </div>
                    </li>
                  );
                })}
              </ul>
            </section>

            <section className="surface-panel rounded-[var(--radius-card)] p-3">
              <h2 className="type-label text-ink-900">Audit</h2>
              {pendingApproval ? (
                <div className="mt-2 rounded-[var(--radius-md)] border border-honey-500 bg-honey-100 p-2">
                  <div className="type-label text-ink-900">Critical approval required</div>
                  <p className="type-body-l mt-1 text-ink-700">
                    {pendingApproval.action} · device {pendingApproval.device_id.slice(0, 12)}. Approve once; replay is refused.
                  </p>
                  <div className="mt-2 flex gap-2">
                    <button
                      type="button"
                      className="type-label h-8 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400 disabled:opacity-50"
                      disabled={busy}
                      onClick={() => void resolve(true)}
                    >
                      Approve
                    </button>
                    <button
                      type="button"
                      className="type-label h-8 rounded-[var(--radius-button)] border border-signal px-3 text-signal hover:bg-recessed disabled:opacity-50"
                      disabled={busy}
                      onClick={() => void resolve(false)}
                    >
                      Deny
                    </button>
                  </div>
                </div>
              ) : null}
              {detail.audit.length === 0 ? <EmptyState message="No remote audit rows yet." /> : null}
              <ul className="mt-2 space-y-1">
                {detail.audit.slice(0, 20).map((row, index) => (
                  <li key={`${row.at}-${index}`} className="type-mono-meta text-meta">
                    {row.at} · {row.action} · {row.outcome}
                    {row.device_id ? ` · ${row.device_id.slice(0, 12)}` : ''}
                  </li>
                ))}
              </ul>
            </section>
          </div>
        ) : null}
        <DaemonPanel />
      </div>
      <ConfirmDialog
        open={confirmRevoke}
        title="Revoke remote"
        body="Immediately invalidates this remote’s grants, leases and pending actions. In-flight physical actions may already have reached the device; the audit remains."
        confirmLabel="Revoke"
        danger
        onConfirm={() => void revokeRemote()}
        onCancel={() => setConfirmRevoke(false)}
      />
    </div>
  );
}
