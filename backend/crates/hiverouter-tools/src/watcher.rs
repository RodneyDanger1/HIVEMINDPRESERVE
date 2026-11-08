//! Workspace file watching with a 150 ms coalescing window (Phase 12 WP 12.4,
//! `notify 8.2.1`).
//!
//! Contract: a burst of writes inside one 150 ms window produces exactly one
//! `FsChanged` notification per burst (verified by `watcher_debounce`: 5 writes
//! in 50 ms → 1 event). The watcher is an observer only — it never writes to
//! the workspace and holds no durable state.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use hiverouter_contract::Refusal;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::ToolError;

/// Coalescing window for filesystem bursts (blueprint WP 12.4).
pub const WATCH_DEBOUNCE: Duration = Duration::from_millis(150);

/// One coalesced filesystem change notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsChanged {
    /// Paths touched during the burst (workspace-relative when possible).
    pub paths: Vec<PathBuf>,
    /// Whether any event in the burst was a removal/rename (consumer hint).
    pub may_have_removed: bool,
}

/// A live workspace watcher. Dropping it stops the OS watcher and the debounce
/// thread (the receiver reports nothing after the channel closes).
#[derive(Debug)]
pub struct WorkspaceWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<FsChanged>,
    root: PathBuf,
}

impl WorkspaceWatcher {
    /// Starts watching `root` recursively.
    ///
    /// # Errors
    /// [`Refusal::InternalError`] when the root is not a directory, the platform
    /// watcher cannot be created, or the debounce thread cannot start.
    pub fn start(root: impl AsRef<Path>) -> Result<Self, ToolError> {
        let root = root.as_ref().to_path_buf();
        if !root.is_dir() {
            return Err(ToolError::new(
                Refusal::InternalError,
                format!("watch root is not a directory: {}", root.display()),
            ));
        }
        let (raw_sender, raw_receiver) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = notify::recommended_watcher(move |event| {
            // The receiver being gone means the app is shutting down; dropping
            // the event is the correct end-of-life behaviour.
            let _ = raw_sender.send(event);
        })
        .map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot create the filesystem watcher: {error}"),
            )
        })?;
        watcher.watch(&root, RecursiveMode::Recursive).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot watch the workspace root: {error}"),
            )
        })?;

        let (sender, receiver) = mpsc::channel::<FsChanged>();
        let debounce_root = root.clone();
        std::thread::Builder::new()
            .name("hiverouter-fs-debounce".to_owned())
            .spawn(move || debounce_loop(raw_receiver, sender, debounce_root))
            .map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot start the filesystem debounce thread: {error}"),
                )
            })?;

        Ok(Self {
            _watcher: watcher,
            receiver,
            root,
        })
    }

    /// Watched root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Waits up to `timeout` for one coalesced change notification.
    #[must_use]
    pub fn recv_timeout(&self, timeout: Duration) -> Option<FsChanged> {
        self.receiver.recv_timeout(timeout).ok()
    }

    /// Non-blocking poll for one coalesced change notification.
    #[must_use]
    pub fn try_recv(&self) -> Option<FsChanged> {
        self.receiver.try_recv().ok()
    }
}

fn debounce_loop(raw: Receiver<notify::Result<Event>>, out: Sender<FsChanged>, root: PathBuf) {
    loop {
        let first = match raw.recv() {
            Ok(event) => event,
            Err(_) => return,
        };
        let mut changed = FsChanged {
            paths: Vec::new(),
            may_have_removed: false,
        };
        absorb(&first, &root, &mut changed);

        let deadline = Instant::now() + WATCH_DEBOUNCE;
        loop {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                break;
            };
            if remaining.is_zero() {
                break;
            }
            match raw.recv_timeout(remaining) {
                Ok(event) => absorb(&event, &root, &mut changed),
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    if out.send(changed).is_err() {
                        return;
                    }
                    return;
                }
            }
        }

        if !changed.paths.is_empty() && out.send(changed).is_err() {
            return;
        }
    }
}

fn absorb(event: &notify::Result<Event>, root: &Path, changed: &mut FsChanged) {
    let Ok(event) = event else {
        return;
    };
    for path in &event.paths {
        let relative = path.strip_prefix(root).unwrap_or(path);
        if !changed.paths.iter().any(|existing| existing == relative) {
            changed.paths.push(relative.to_path_buf());
        }
    }
    if matches!(
        event.kind,
        notify::EventKind::Remove(_) | notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
    ) {
        changed.may_have_removed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    fn wait_for_first_event(watcher: &WorkspaceWatcher) -> FsChanged {
        // Warm-up write drains the initial registration race deterministically:
        // the first observed event may be the warm-up, which is fine because the
        // burst assertion starts afterwards.
        loop {
            if let Some(event) = watcher.recv_timeout(Duration::from_secs(10)) {
                return event;
            }
            panic!("filesystem watcher produced no event within 10s");
        }
    }

    #[test]
    fn watcher_debounce_five_writes_in_50ms_yield_one_event() {
        let temp = tempfile::tempdir().expect("tempdir");
        let watcher = WorkspaceWatcher::start(temp.path()).expect("watcher");

        // Warm-up: establish that the OS watcher is delivering events.
        fs::write(temp.path().join("warmup.txt"), b"warm").expect("warmup");
        let _ = wait_for_first_event(&watcher);
        // Drain any residual warm-up/registration event so the burst starts from
        // an empty debounce window (Windows can deliver more than one raw event).
        while watcher.recv_timeout(Duration::from_millis(250)).is_some() {}

        // Burst: five writes well inside one 150 ms window. No inter-write
        // sleeps: Windows can otherwise deliver a late raw event after the
        // window and split one burst into two batches.
        for index in 0..5 {
            fs::write(temp.path().join(format!("burst-{index}.txt")), b"x").expect("write");
        }

        let mut batches = 0;
        let mut paths = Vec::new();
        loop {
            match watcher.recv_timeout(Duration::from_millis(600)) {
                Some(event) => {
                    batches += 1;
                    paths.extend(event.paths);
                    if batches > 3 {
                        break;
                    }
                }
                None => break,
            }
        }
        assert_eq!(
            batches, 1,
            "5 writes in 50 ms must coalesce into exactly one notification (got {batches}: {paths:?})"
        );
        for index in 0..5 {
            let expected = PathBuf::from(format!("burst-{index}.txt"));
            assert!(
                paths.contains(&expected),
                "burst path {expected:?} missing from {paths:?}"
            );
        }
    }

    #[test]
    fn watcher_refuses_missing_root() {
        let error = WorkspaceWatcher::start(std::path::Path::new("Z:\\definitely-missing-hiverouter"))
            .expect_err("missing root");
        assert_eq!(error.code(), "E_INTERNAL_ERROR");
    }
}
