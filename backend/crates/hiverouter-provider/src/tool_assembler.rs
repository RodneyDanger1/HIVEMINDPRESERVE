//! Deterministic assembly of streamed tool-call fragments.
//!
//! The provider client forwards tool calls as fragments
//! ([`StreamEvent::ToolCallDelta`]): the call `id` and function `name` are
//! documented to arrive on the first fragment, and the JSON-encoded
//! `arguments` string is delivered as one or more fragments that must be
//! concatenated in arrival order.
//!
//! [`ToolCallAssembler`] owns the per-message assembly state and is the only
//! component that merges those fragments. It is a pure, fail-closed validation
//! state machine at the provider boundary:
//!
//! - fragments are merged per `index`, with the final calls sorted by `index`;
//! - an identical repeated `id`/`name` is idempotent (the provider documents
//!   them as first-fragment-only, so this tolerates benign repeats), while a
//!   conflicting `id`/`name` is refused;
//! - two calls that resolve to the same id are refused (never merged);
//! - a call that is still missing its id, name or arguments when the stream is
//!   finished is refused as truncated;
//! - the concatenated arguments must parse as a JSON **object**;
//! - call count, identifier size and argument size are bounded.
//!
//! The module performs **no tool resolution, no approval and no execution**.
//! Its output [`AssembledToolCall`] is a typed value; only a future core
//! adapter may decide whether a call is allowed to run (the W10 design keeps
//! the loop and the registry outside this crate).
//!
//! Errors carry the frozen refusal codes without adding new contract variants:
//! structural/truncation/identity failures map to
//! [`Refusal::ProviderInvalidResponse`], argument-shape failures map to
//! [`Refusal::ToolSchema`].

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use serde_json::Value;

use hiverouter_contract::Refusal;

use crate::client::StreamEvent;

/// Maximum number of distinct tool calls accepted in one assistant message.
pub const MAX_TOOL_CALLS_PER_MESSAGE: usize = 8;

/// Maximum bytes of one tool-call id or function name.
pub const MAX_IDENTIFIER_BYTES: usize = 128;

/// Maximum bytes of the concatenated JSON arguments of one tool call.
pub const MAX_ARGUMENTS_BYTES: usize = 1024 * 1024;

/// One complete tool call assembled from streamed fragments.
///
/// This is a value, not a capability: the assembler never executes anything and
/// the type exposes no method that could run a tool.
///
/// Derives `Eq` because the core tool loop stores these calls in an
/// `Eq`-derived `StreamedReply` (`hiverouter-core/src/execution.rs`); every field
/// is `Eq` (including `serde_json::Value` in the pinned 1.0.145).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledToolCall {
    /// Index of the call in the assistant message (also the assembly key).
    pub index: u32,
    /// Provider tool-call id (unique within the message).
    pub id: String,
    /// Function name exactly as the model requested it.
    pub name: String,
    /// Parsed JSON arguments; always a JSON object.
    pub arguments: Value,
}

/// Why a tool-call assembly was refused.
///
/// Structural and truncation failures map to
/// [`Refusal::ProviderInvalidResponse`]; argument-shape failures map to
/// [`Refusal::ToolSchema`]. No new refusal code is introduced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCallAssemblyError {
    /// More distinct calls arrived than the per-message bound allows.
    TooManyCalls {
        /// Number of distinct calls that would have existed.
        count: usize,
        /// Configured bound.
        limit: usize,
    },
    /// A later fragment carried a different id for the same index.
    ConflictingId {
        /// Call index.
        index: u32,
    },
    /// A later fragment carried a different function name for the same index.
    ConflictingName {
        /// Call index.
        index: u32,
    },
    /// Two different indices resolved to the same tool-call id.
    DuplicateId {
        /// The duplicated id.
        id: String,
        /// Index the id was first seen at.
        first_index: u32,
        /// Index that duplicated the id.
        second_index: u32,
    },
    /// An identifier was empty or contained control characters.
    InvalidIdentifier {
        /// Call index.
        index: u32,
        /// `"id"` or `"name"`.
        field: &'static str,
    },
    /// An identifier exceeded [`MAX_IDENTIFIER_BYTES`].
    IdentifierTooLong {
        /// Call index.
        index: u32,
        /// `"id"` or `"name"`.
        field: &'static str,
        /// Observed byte length.
        bytes: usize,
    },
    /// The stream finished before the call's id arrived.
    MissingId {
        /// Call index.
        index: u32,
    },
    /// The stream finished before the call's function name arrived.
    MissingName {
        /// Call index.
        index: u32,
    },
    /// The stream finished before any arguments fragment arrived.
    MissingArguments {
        /// Call index.
        index: u32,
    },
    /// The concatenated arguments exceeded [`MAX_ARGUMENTS_BYTES`].
    ArgumentsTooLarge {
        /// Call index.
        index: u32,
        /// Bytes that would have resulted.
        bytes: usize,
        /// Configured bound.
        limit: usize,
    },
    /// The concatenated arguments were not valid JSON.
    InvalidArgumentsJson {
        /// Call index.
        index: u32,
        /// Parser detail (no argument content).
        detail: String,
    },
    /// The arguments were valid JSON but not an object.
    ArgumentsNotAnObject {
        /// Call index.
        index: u32,
    },
}

impl ToolCallAssemblyError {
    /// The frozen refusal this assembly failure maps to.
    #[must_use]
    pub fn refusal(&self) -> Refusal {
        match self {
            Self::InvalidArgumentsJson { .. } | Self::ArgumentsNotAnObject { .. } => {
                Refusal::ToolSchema
            }
            Self::TooManyCalls { .. }
            | Self::ConflictingId { .. }
            | Self::ConflictingName { .. }
            | Self::DuplicateId { .. }
            | Self::InvalidIdentifier { .. }
            | Self::IdentifierTooLong { .. }
            | Self::MissingId { .. }
            | Self::MissingName { .. }
            | Self::MissingArguments { .. }
            | Self::ArgumentsTooLarge { .. } => Refusal::ProviderInvalidResponse,
        }
    }

    /// The stable refusal code (`E_*`) for this failure.
    #[must_use]
    pub fn code(&self) -> &'static str {
        self.refusal().code()
    }
}

impl fmt::Display for ToolCallAssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyCalls { count, limit } => write!(
                f,
                "tool-call assembly refused: {count} calls exceed the per-message limit {limit}"
            ),
            Self::ConflictingId { index } => write!(
                f,
                "tool-call assembly refused: call {index} received two different ids"
            ),
            Self::ConflictingName { index } => write!(
                f,
                "tool-call assembly refused: call {index} received two different function names"
            ),
            Self::DuplicateId {
                id,
                first_index,
                second_index,
            } => write!(
                f,
                "tool-call assembly refused: id {id:?} appears at indices {first_index} and {second_index}"
            ),
            Self::InvalidIdentifier { index, field } => write!(
                f,
                "tool-call assembly refused: call {index} has an empty or control-character {field}"
            ),
            Self::IdentifierTooLong {
                index,
                field,
                bytes,
            } => write!(
                f,
                "tool-call assembly refused: call {index} {field} is {bytes} bytes (limit {MAX_IDENTIFIER_BYTES})"
            ),
            Self::MissingId { index } => write!(
                f,
                "tool-call assembly refused: call {index} is truncated (missing id)"
            ),
            Self::MissingName { index } => write!(
                f,
                "tool-call assembly refused: call {index} is truncated (missing function name)"
            ),
            Self::MissingArguments { index } => write!(
                f,
                "tool-call assembly refused: call {index} is truncated (no arguments fragments)"
            ),
            Self::ArgumentsTooLarge {
                index,
                bytes,
                limit,
            } => write!(
                f,
                "tool-call assembly refused: call {index} arguments are {bytes} bytes (limit {limit})"
            ),
            Self::InvalidArgumentsJson { index, detail } => write!(
                f,
                "tool-call assembly refused: call {index} arguments are not valid JSON ({detail})"
            ),
            Self::ArgumentsNotAnObject { index } => write!(
                f,
                "tool-call assembly refused: call {index} arguments are not a JSON object"
            ),
        }
    }
}

impl std::error::Error for ToolCallAssemblyError {}

#[derive(Debug, Default)]
struct PendingCall {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
    argument_fragments: usize,
}

/// Deterministic assembler for streamed tool-call fragments.
///
/// Feed it every [`StreamEvent`] from one assistant message via
/// [`Self::push_event`] (or individual fragments via [`Self::push_fragment`]),
/// then call [`Self::finish`] once the stream completed. `finish` returns the
/// complete calls sorted by index; nothing is executed.
#[derive(Debug, Default)]
pub struct ToolCallAssembler {
    pending: BTreeMap<u32, PendingCall>,
}

impl ToolCallAssembler {
    /// Creates an empty assembler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct call indices seen so far.
    #[must_use]
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Consumes one stream event.
    ///
    /// Returns `Ok(true)` when the event was a tool-call fragment (consumed and
    /// validated), `Ok(false)` for any other event (`Delta`, `Finished`, ...).
    ///
    /// # Errors
    /// Returns [`ToolCallAssemblyError`] when the fragment is malformed, adds a
    /// conflicting identifier, exceeds a bound, or introduces a new index past
    /// [`MAX_TOOL_CALLS_PER_MESSAGE`].
    pub fn push_event(&mut self, event: &StreamEvent) -> Result<bool, ToolCallAssemblyError> {
        match event {
            StreamEvent::ToolCallDelta {
                index,
                id,
                name,
                arguments,
            } => {
                self.push_fragment(*index, id.as_deref(), name.as_deref(), arguments.as_deref())?;
                Ok(true)
            }
            StreamEvent::Delta { .. } | StreamEvent::Finished { .. } => Ok(false),
        }
    }

    /// Merges one tool-call fragment for `index`.
    ///
    /// `id` and `name` are documented to arrive on the first fragment; identical
    /// repeats are tolerated, conflicting values are refused. `arguments`
    /// fragments are concatenated in arrival order.
    ///
    /// # Errors
    /// Same failure set as [`Self::push_event`].
    pub fn push_fragment(
        &mut self,
        index: u32,
        id: Option<&str>,
        name: Option<&str>,
        arguments: Option<&str>,
    ) -> Result<(), ToolCallAssemblyError> {
        if !self.pending.contains_key(&index) && self.pending.len() >= MAX_TOOL_CALLS_PER_MESSAGE {
            return Err(ToolCallAssemblyError::TooManyCalls {
                count: self.pending.len().saturating_add(1),
                limit: MAX_TOOL_CALLS_PER_MESSAGE,
            });
        }

        let pending = self.pending.entry(index).or_default();

        if let Some(id) = id {
            validate_identifier(index, "id", id)?;
            match &pending.id {
                Some(existing) if existing != id => {
                    return Err(ToolCallAssemblyError::ConflictingId { index });
                }
                Some(_) => {}
                None => pending.id = Some(id.to_owned()),
            }
        }

        if let Some(name) = name {
            validate_identifier(index, "name", name)?;
            match &pending.name {
                Some(existing) if existing != name => {
                    return Err(ToolCallAssemblyError::ConflictingName { index });
                }
                Some(_) => {}
                None => pending.name = Some(name.to_owned()),
            }
        }

        if let Some(fragment) = arguments {
            let new_len = pending.arguments.len().saturating_add(fragment.len());
            if new_len > MAX_ARGUMENTS_BYTES {
                return Err(ToolCallAssemblyError::ArgumentsTooLarge {
                    index,
                    bytes: new_len,
                    limit: MAX_ARGUMENTS_BYTES,
                });
            }
            pending.arguments.push_str(fragment);
            pending.argument_fragments = pending.argument_fragments.saturating_add(1);
        }

        Ok(())
    }

    /// Validates every pending call and returns them sorted by index.
    ///
    /// # Errors
    /// Returns the first truncation, duplicate-id or argument-shape failure in
    /// index order. An empty assembler returns an empty vector (a text-only
    /// reply is not an error).
    pub fn finish(self) -> Result<Vec<AssembledToolCall>, ToolCallAssemblyError> {
        let mut calls = Vec::with_capacity(self.pending.len());
        let mut seen_ids: HashMap<String, u32> = HashMap::with_capacity(self.pending.len());

        for (index, pending) in self.pending {
            let id = pending.id.ok_or(ToolCallAssemblyError::MissingId { index })?;
            let name = pending
                .name
                .ok_or(ToolCallAssemblyError::MissingName { index })?;
            if pending.argument_fragments == 0 {
                return Err(ToolCallAssemblyError::MissingArguments { index });
            }
            let arguments: Value = serde_json::from_str(&pending.arguments).map_err(|error| {
                ToolCallAssemblyError::InvalidArgumentsJson {
                    index,
                    detail: error.to_string(),
                }
            })?;
            if !arguments.is_object() {
                return Err(ToolCallAssemblyError::ArgumentsNotAnObject { index });
            }
            if let Some(first_index) = seen_ids.insert(id.clone(), index) {
                return Err(ToolCallAssemblyError::DuplicateId {
                    id,
                    first_index,
                    second_index: index,
                });
            }
            calls.push(AssembledToolCall {
                index,
                id,
                name,
                arguments,
            });
        }

        Ok(calls)
    }
}

fn validate_identifier(
    index: u32,
    field: &'static str,
    value: &str,
) -> Result<(), ToolCallAssemblyError> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(ToolCallAssemblyError::InvalidIdentifier { index, field });
    }
    if value.len() > MAX_IDENTIFIER_BYTES {
        return Err(ToolCallAssemblyError::IdentifierTooLong {
            index,
            field,
            bytes: value.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::StreamEvent;

    fn fragment(
        index: u32,
        id: Option<&str>,
        name: Option<&str>,
        arguments: Option<&str>,
    ) -> StreamEvent {
        StreamEvent::ToolCallDelta {
            index,
            id: id.map(ToOwned::to_owned),
            name: name.map(ToOwned::to_owned),
            arguments: arguments.map(ToOwned::to_owned),
        }
    }

    /// Pins the cross-crate `Eq` contract: the core 1.2 tool loop stores these
    /// calls in `StreamedReply`, which derives `Eq`. If a future field is not
    /// `Eq`, this provider test fails before the workspace build reaches core.
    #[test]
    fn assembled_tool_call_satisfies_the_core_eq_contract() {
        fn requires_eq<T: Eq>() {}
        requires_eq::<AssembledToolCall>();
        requires_eq::<ToolCallAssemblyError>();
    }

    #[test]
    fn merges_fragments_by_index_in_order_and_sorts_output() {
        let mut assembler = ToolCallAssembler::new();
        // Index 1 arrives before index 0; index 0 arguments are split in two.
        assembler
            .push_event(&fragment(1, Some("call_b"), Some("file_list"), Some("{}")))
            .expect("valid fragment");
        assembler
            .push_event(&fragment(0, Some("call_a"), Some("file_read"), Some("{\"path\":")))
            .expect("valid fragment");
        assembler
            .push_event(&fragment(0, None, None, Some("\"a.txt\"}")))
            .expect("valid fragment");

        let calls = assembler.finish().expect("complete assembly");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].index, 0, "output is sorted by index");
        assert_eq!(calls[0].id, "call_a");
        assert_eq!(calls[0].name, "file_read");
        assert_eq!(calls[0].arguments, serde_json::json!({"path": "a.txt"}));
        assert_eq!(calls[1].index, 1);
        assert_eq!(calls[1].id, "call_b");
        assert_eq!(calls[1].name, "file_list");
        assert_eq!(calls[1].arguments, serde_json::json!({}));
    }

    #[test]
    fn identical_repeated_id_and_name_are_idempotent() {
        let mut assembler = ToolCallAssembler::new();
        assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), Some("{}"))
            .expect("first fragment");
        assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), None)
            .expect("identical repeats are tolerated");
        let calls = assembler.finish().expect("assembled");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_a");
    }

    #[test]
    fn conflicting_id_is_refused() {
        let mut assembler = ToolCallAssembler::new();
        assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), Some("{}"))
            .expect("first fragment");
        let error = assembler
            .push_fragment(0, Some("call_b"), None, None)
            .expect_err("a different id for the same index must be refused");
        assert_eq!(error, ToolCallAssemblyError::ConflictingId { index: 0 });
        assert_eq!(error.code(), "E_PROVIDER_INVALID_RESPONSE");
    }

    #[test]
    fn conflicting_name_is_refused() {
        let mut assembler = ToolCallAssembler::new();
        assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), Some("{}"))
            .expect("first fragment");
        let error = assembler
            .push_fragment(0, None, Some("shell_exec"), None)
            .expect_err("a different name for the same index must be refused");
        assert_eq!(error, ToolCallAssemblyError::ConflictingName { index: 0 });
    }

    #[test]
    fn duplicate_ids_across_calls_are_refused() {
        let mut assembler = ToolCallAssembler::new();
        assembler
            .push_fragment(0, Some("call_same"), Some("file_read"), Some("{}"))
            .expect("first call");
        assembler
            .push_fragment(1, Some("call_same"), Some("file_list"), Some("{}"))
            .expect("second call");
        let error = assembler.finish().expect_err("duplicate id must be refused");
        assert_eq!(
            error,
            ToolCallAssemblyError::DuplicateId {
                id: "call_same".to_owned(),
                first_index: 0,
                second_index: 1,
            }
        );
        assert_eq!(error.code(), "E_PROVIDER_INVALID_RESPONSE");
    }

    #[test]
    fn truncated_calls_are_refused() {
        let mut assembler = ToolCallAssembler::new();
        assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), None)
            .expect("partial call");
        let error = assembler.finish().expect_err("no arguments is truncated");
        assert_eq!(error, ToolCallAssemblyError::MissingArguments { index: 0 });
    }

    #[test]
    fn missing_id_and_name_are_refused_at_finish() {
        let mut missing_id = ToolCallAssembler::new();
        missing_id
            .push_fragment(0, None, Some("file_read"), Some("{}"))
            .expect("partial call");
        assert_eq!(
            missing_id.finish().expect_err("missing id"),
            ToolCallAssemblyError::MissingId { index: 0 }
        );

        let mut missing_name = ToolCallAssembler::new();
        missing_name
            .push_fragment(1, Some("call_a"), None, Some("{}"))
            .expect("partial call");
        assert_eq!(
            missing_name.finish().expect_err("missing name"),
            ToolCallAssemblyError::MissingName { index: 1 }
        );
    }

    #[test]
    fn invalid_arguments_json_maps_to_tool_schema() {
        let mut assembler = ToolCallAssembler::new();
        assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), Some("{not-json"))
            .expect("fragment accepted");
        let error = assembler.finish().expect_err("bad JSON is refused");
        assert!(matches!(
            error,
            ToolCallAssemblyError::InvalidArgumentsJson { index: 0, .. }
        ));
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
        assert_eq!(error.refusal(), Refusal::ToolSchema);
    }

    #[test]
    fn non_object_arguments_are_refused() {
        let mut assembler = ToolCallAssembler::new();
        assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), Some("\"just-a-string\""))
            .expect("fragment accepted");
        let error = assembler.finish().expect_err("non-object is refused");
        assert_eq!(error, ToolCallAssemblyError::ArgumentsNotAnObject { index: 0 });
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
    }

    #[test]
    fn empty_or_control_identifiers_are_refused() {
        let mut assembler = ToolCallAssembler::new();
        let error = assembler
            .push_fragment(0, Some(""), Some("file_read"), Some("{}"))
            .expect_err("empty id");
        assert_eq!(
            error,
            ToolCallAssemblyError::InvalidIdentifier {
                index: 0,
                field: "id"
            }
        );

        let error = assembler
            .push_fragment(0, Some("call_a"), Some("bad\tname"), Some("{}"))
            .expect_err("control character in name");
        assert_eq!(
            error,
            ToolCallAssemblyError::InvalidIdentifier {
                index: 0,
                field: "name"
            }
        );
    }

    #[test]
    fn oversized_identifier_is_refused() {
        let long = "x".repeat(MAX_IDENTIFIER_BYTES + 1);
        let mut assembler = ToolCallAssembler::new();
        let error = assembler
            .push_fragment(0, Some(&long), Some("file_read"), Some("{}"))
            .expect_err("oversized id");
        assert_eq!(
            error,
            ToolCallAssemblyError::IdentifierTooLong {
                index: 0,
                field: "id",
                bytes: MAX_IDENTIFIER_BYTES + 1,
            }
        );
    }

    #[test]
    fn too_many_distinct_calls_are_refused() {
        let mut assembler = ToolCallAssembler::new();
        for index in 0..u32::try_from(MAX_TOOL_CALLS_PER_MESSAGE).expect("fits") {
            assembler
                .push_fragment(index, Some("call_x"), Some("file_list"), Some("{}"))
                .expect("bounded call");
        }
        let error = assembler
            .push_fragment(
                u32::try_from(MAX_TOOL_CALLS_PER_MESSAGE).expect("fits"),
                Some("call_y"),
                Some("file_list"),
                Some("{}"),
            )
            .expect_err("ninth call is refused");
        assert_eq!(
            error,
            ToolCallAssemblyError::TooManyCalls {
                count: MAX_TOOL_CALLS_PER_MESSAGE + 1,
                limit: MAX_TOOL_CALLS_PER_MESSAGE,
            }
        );
    }

    #[test]
    fn oversized_arguments_are_refused_before_unbounded_growth() {
        let large = "x".repeat(MAX_ARGUMENTS_BYTES + 1);
        let mut assembler = ToolCallAssembler::new();
        let error = assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), Some(&large))
            .expect_err("over the argument bound");
        assert!(matches!(
            error,
            ToolCallAssemblyError::ArgumentsTooLarge { index: 0, .. }
        ));
    }

    #[test]
    fn empty_assembler_finishes_clean_for_text_only_replies() {
        let calls = ToolCallAssembler::new().finish().expect("no calls is not an error");
        assert!(calls.is_empty());
    }

    #[test]
    fn push_event_consumes_tool_fragments_and_ignores_other_events() {
        let mut assembler = ToolCallAssembler::new();
        let consumed = assembler
            .push_event(&StreamEvent::Delta {
                content: "hello".to_owned(),
                reasoning_content: None,
            })
            .expect("delta is ignored");
        assert!(!consumed);
        let consumed = assembler
            .push_event(&fragment(0, Some("call_a"), Some("file_list"), Some("{}")))
            .expect("tool fragment");
        assert!(consumed);
        assert_eq!(assembler.pending_len(), 1);
        let finished = assembler
            .push_event(&StreamEvent::Finished {
                usage: None,
                finish_reason: Some("tool_calls".to_owned()),
            })
            .expect("finished is ignored");
        assert!(!finished);
        let calls = assembler.finish().expect("assembled");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "file_list");
    }

    #[test]
    fn error_messages_do_not_embed_argument_content() {
        let mut assembler = ToolCallAssembler::new();
        assembler
            .push_fragment(0, Some("call_a"), Some("file_read"), Some("{\"secret\":super-secret"))
            .expect("fragment accepted");
        let error = assembler.finish().expect_err("bad JSON");
        let rendered = error.to_string();
        assert!(
            !rendered.contains("super-secret"),
            "assembly errors must not echo arguments"
        );
    }
}
