//! Server event sequencing, replay buffer and fan-out.
//!
//! Every server event is published here; `EventBus` assigns the server-global
//! monotonic `seq` required by the transport contract and fans the envelope out
//! to every subscribed session through a bounded broadcast channel. A bounded
//! replay buffer lets a reconnecting client (`hello{last_seq}`) catch up without
//! a full resnapshot when the gap is small.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use hiverouter_contract::{Envelope, Event, Refusal};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Broadcast channel capacity; a lagging session is told to resync, not dropped.
pub const FANOUT_CAPACITY: usize = 1_024;
/// Number of sequenced events retained for reconnect replay.
pub const REPLAY_CAPACITY: usize = 256;

/// One event envelope with its server sequence number.
#[derive(Clone, Debug)]
pub struct SequencedEvent {
    /// Server-global monotonic sequence number (starts at 1).
    pub seq: u64,
    /// Fully built envelope with `seq` populated.
    pub envelope: Envelope,
}

/// Outcome of a reconnect replay request.
#[derive(Debug)]
pub enum Replay {
    /// Client has no prior sequence; send a fresh hello only.
    Fresh,
    /// Client is exactly up to date.
    Current,
    /// Buffered events the client is missing (oldest first).
    Events(Vec<SequencedEvent>),
    /// The gap is too old (or the client claims a future sequence); resnapshot.
    Gap {
        /// First sequence the client can continue from.
        expected_seq: u64,
        /// Sequence the client reported.
        last_seq: u64,
    },
}

/// Sequencer + fan-out bus (one per server process).
pub struct EventBus {
    sender: broadcast::Sender<SequencedEvent>,
    seq: AtomicU64,
    replay: Mutex<VecDeque<SequencedEvent>>,
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("current_seq", &self.current_seq())
            .finish()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Creates an empty bus.
    #[must_use]
    pub fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(FANOUT_CAPACITY);
        Self {
            sender,
            seq: AtomicU64::new(0),
            replay: Mutex::new(VecDeque::with_capacity(REPLAY_CAPACITY)),
        }
    }

    /// Subscribes a session to the live event stream.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<SequencedEvent> {
        self.sender.subscribe()
    }

    /// Publishes an event, assigning the next sequence number and fanning it out.
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the event payload cannot be serialized.
    pub fn publish(&self, event: Event) -> Result<SequencedEvent, Refusal> {
        self.publish_with_correlation(event, None)
    }

    /// Publishes an event and forces the envelope `correlation_id` (ACK/ERROR
    /// chains must share the originating command's correlation id).
    ///
    /// # Errors
    /// `E_SCHEMA_INVALID` when the event payload cannot be serialized.
    pub fn publish_with_correlation(
        &self,
        event: Event,
        correlation_id: Option<Uuid>,
    ) -> Result<SequencedEvent, Refusal> {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        let mut envelope = event.into_envelope()?.with_seq(seq);
        if let Some(correlation_id) = correlation_id {
            envelope = envelope.with_correlation(correlation_id);
        }
        let sequenced = SequencedEvent { seq, envelope };
        {
            let mut replay = self.replay.lock().map_err(|_| Refusal::InternalError)?;
            if replay.len() == REPLAY_CAPACITY {
                replay.pop_front();
            }
            replay.push_back(sequenced.clone());
        }
        // A send failure only means no session is currently subscribed.
        let _ = self.sender.send(sequenced.clone());
        Ok(sequenced)
    }

    /// Current server sequence number (`0` before the first event).
    #[must_use]
    pub fn current_seq(&self) -> u64 {
        self.seq.load(Ordering::SeqCst)
    }

    /// Oldest sequence number still retained for replay, if any.
    #[must_use]
    pub fn oldest_seq(&self) -> Option<u64> {
        self.replay
            .lock()
            .ok()
            .and_then(|replay| replay.front().map(|event| event.seq))
    }

    /// Builds the replay answer for a client that reported `last_seq`.
    #[must_use]
    pub fn replay_after(&self, last_seq: u64) -> Replay {
        let current = self.current_seq();
        if last_seq > current {
            // The client claims a sequence this server has never produced.
            return Replay::Gap {
                expected_seq: current + 1,
                last_seq,
            };
        }
        if last_seq == current {
            return Replay::Current;
        }
        let replay = match self.replay.lock() {
            Ok(replay) => replay,
            Err(poisoned) => poisoned.into_inner(),
        };
        let Some(oldest) = replay.front().map(|event| event.seq) else {
            return Replay::Gap {
                expected_seq: current + 1,
                last_seq,
            };
        };
        if last_seq + 1 < oldest {
            return Replay::Gap {
                expected_seq: oldest,
                last_seq,
            };
        }
        let events = replay
            .iter()
            .filter(|event| event.seq > last_seq)
            .cloned()
            .collect();
        Replay::Events(events)
    }
}

#[cfg(test)]
mod tests {
    use hiverouter_contract::event::EmptyPayload;
    use hiverouter_contract::Event;

    use super::*;

    fn ping() -> Event {
        Event::Ping(EmptyPayload {})
    }

    #[test]
    fn sequences_start_at_one_and_increase() {
        let bus = EventBus::new();
        assert_eq!(bus.current_seq(), 0);
        let first = bus.publish(ping()).expect("publish");
        let second = bus.publish(ping()).expect("publish");
        assert_eq!(first.seq, 1);
        assert_eq!(second.seq, 2);
        assert_eq!(bus.current_seq(), 2);
        assert_eq!(first.envelope.seq, Some(1));
    }

    #[test]
    fn replay_returns_events_after_the_client_sequence() {
        let bus = EventBus::new();
        for _ in 0..3 {
            bus.publish(ping()).expect("publish");
        }
        match bus.replay_after(1) {
            Replay::Events(events) => {
                assert_eq!(events.len(), 2);
                assert_eq!(events[0].seq, 2);
                assert_eq!(events[1].seq, 3);
            }
            other => panic!("unexpected replay: {other:?}"),
        }
        assert!(matches!(bus.replay_after(3), Replay::Current));
    }

    #[test]
    fn replay_gap_when_client_is_ahead_or_buffer_expired() {
        let bus = EventBus::new();
        assert!(matches!(
            bus.replay_after(5),
            Replay::Gap {
                expected_seq: 1,
                last_seq: 5
            }
        ));
        for _ in 0..(REPLAY_CAPACITY + 10) {
            bus.publish(ping()).expect("publish");
        }
        assert!(matches!(
            bus.replay_after(1),
            Replay::Gap { expected_seq, .. } if expected_seq == 11
        ));
    }

    #[tokio::test]
    async fn subscribers_receive_published_events() {
        let bus = EventBus::new();
        let mut first = bus.subscribe();
        let mut second = bus.subscribe();
        let published = bus.publish(ping()).expect("publish");
        let received_first = first.recv().await.expect("first receives");
        let received_second = second.recv().await.expect("second receives");
        assert_eq!(received_first.seq, published.seq);
        assert_eq!(received_second.seq, published.seq);
    }
}
