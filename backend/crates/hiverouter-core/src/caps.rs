//! Hard spawn caps (blueprint §2a budget invariants).
//!
//! All caps are enforced before a spawn is persisted or dispatched; a refusal is
//! always the named contract code, never a silent clamp.

use hiverouter_contract::Refusal;

/// Maximum spawn depth (Hyper -> Super -> Worker).
pub const MAX_SPAWN_DEPTH: u8 = 3;
/// Maximum children per parent task.
pub const MAX_FANOUT: usize = 8;
/// Maximum agents (tasks) per run.
pub const MAX_AGENTS: u64 = 64;
/// Maximum provider calls / super-steps per run.
pub const MAX_CALLS: u64 = 40;

/// Effective cap configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caps {
    /// Maximum spawn depth.
    pub max_depth: u8,
    /// Maximum fan-out.
    pub max_fanout: usize,
    /// Maximum agents per run.
    pub max_agents: u64,
    /// Maximum provider calls per run.
    pub max_calls: u64,
}

impl Default for Caps {
    fn default() -> Self {
        Self {
            max_depth: MAX_SPAWN_DEPTH,
            max_fanout: MAX_FANOUT,
            max_agents: MAX_AGENTS,
            max_calls: MAX_CALLS,
        }
    }
}

impl Caps {
    /// Checks one spawn against depth/fan-out/total-agent caps.
    ///
    /// `depth` is the child depth (root = 1), `siblings` the number of children
    /// the parent already has, `agents` the run's current task count.
    ///
    /// # Errors
    /// `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT` or `E_AGENT_LIMIT`.
    pub fn check_spawn(&self, depth: u8, siblings: usize, agents: u64) -> Result<(), Refusal> {
        if depth == 0 || depth > self.max_depth {
            return Err(Refusal::SpawnDepth);
        }
        if siblings >= self.max_fanout {
            return Err(Refusal::FanoutLimit);
        }
        if agents >= self.max_agents {
            return Err(Refusal::AgentLimit);
        }
        Ok(())
    }

    /// Checks the provider-call/step counter.
    ///
    /// # Errors
    /// `E_CALL_LIMIT`.
    pub fn check_calls(&self, calls: u64) -> Result<(), Refusal> {
        if calls > self.max_calls {
            return Err(Refusal::CallLimit);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_cap_is_three() {
        let caps = Caps::default();
        assert_eq!(caps.check_spawn(1, 0, 0), Ok(()));
        assert_eq!(caps.check_spawn(3, 0, 0), Ok(()));
        assert_eq!(caps.check_spawn(4, 0, 0), Err(Refusal::SpawnDepth));
        assert_eq!(caps.check_spawn(0, 0, 0), Err(Refusal::SpawnDepth));
    }

    #[test]
    fn fanout_and_agent_caps_are_named() {
        let caps = Caps::default();
        assert_eq!(caps.check_spawn(1, 7, 0), Ok(()));
        assert_eq!(caps.check_spawn(1, 8, 0), Err(Refusal::FanoutLimit));
        assert_eq!(caps.check_spawn(1, 0, 64), Err(Refusal::AgentLimit));
        assert_eq!(caps.check_calls(40), Ok(()));
        assert_eq!(caps.check_calls(41), Err(Refusal::CallLimit));
    }
}
