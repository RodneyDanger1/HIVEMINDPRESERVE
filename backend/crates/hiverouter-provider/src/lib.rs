//! HiveMind v1 provider crate — Phase 5 (WPs 5.1–5.8).
//!
//! One DeepSeek client with the verified contract (FACT-REGISTER F1), the
//! pre-send budget core (B1–B12), Decimal-only cost accounting (V1–V6) and the
//! offline refusal matrix. There is exactly one provider client and one budget
//! / cost owner in this workspace (CANONICAL C16/C17); no second SDK, no key
//! pools, no live call in tests.
//!
//! Module map:
//! - [`dto`] — documented Chat Completions wire types (requests, responses, SSE)
//! - [`secret`] — `DEEPSEEK_API_KEY` handling (`Zeroizing`, redacted `Debug`)
//! - [`error`] — [`error::ProviderError`] wrapping the frozen [`hiverouter_contract::Refusal`]
//! - [`pricing`] — FACT F1 price rows with source provenance + UTC peak windows
//! - [`budget`] — B1–B12 pre-send enforcement, retry policy, in-flight limiter
//! - [`accounting`] — V1–V6 Decimal cost/token accounting; NULL usage ⇒ INCOMPLETE
//! - [`client`] — the one non-streaming + streaming DeepSeek client
//! - [`tool_assembler`] — index-keyed assembly of streamed tool-call fragments
//!   (validated, deterministic, no execution)
//!
//! # Money rule
//! `f64` is prohibited on every cost path (deny lint below); prices and totals
//! are [`rust_decimal::Decimal`] and a NULL usage never becomes `0.00`.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]
#![warn(missing_docs)]

pub mod accounting;
pub mod budget;
pub mod client;
pub mod dto;
pub mod error;
pub mod pricing;
pub mod secret;
pub mod tool_assembler;

pub use accounting::{CallCost, CostReport, CostStatus, Provenance, RunAccounting};
pub use budget::{
    enforce_single_key, retry_after_from_header_value, run_cost_cap_usd, BudgetLimits,
    BudgetPermit, CallEstimate, InFlightLimiter, RetryPolicy, RunBudget, RunCaps, AGENT_LIMIT,
    CALL_CAP, FANOUT_LIMIT, IN_FLIGHT_LIMIT, MAX_ATTEMPTS, MAX_SPAWN_DEPTH, RETRY_AFTER_CAP,
    RUN_COST_CAP_USD_CENTS, SUPPORTED_KEY_COUNT, TOKEN_BUDGET_COMBINED,
};
pub use client::{
    parse_sse_line, ClientConfig, Completion, DeepSeekClient, SseLine, StreamEvent, StreamHandle,
};
pub use dto::{
    ChatMessage, ChatRequest, ChatResponse, FunctionDefinition, Model, ReasoningEffort, Role,
    Thinking, ToolChoice, ToolSpec, CHAT_COMPLETIONS_PATH, DEFAULT_BASE_URL, MAX_OUTPUT_TOKENS,
};
pub use error::ProviderError;
pub use pricing::{peak_at_unix, Peak, PricingTable};
pub use secret::{ApiKey, API_KEY_ENV};
pub use tool_assembler::{
    AssembledToolCall, ToolCallAssembler, ToolCallAssemblyError, MAX_ARGUMENTS_BYTES,
    MAX_IDENTIFIER_BYTES, MAX_TOOL_CALLS_PER_MESSAGE,
};

/// Crate version from the workspace manifest.
pub const PROVIDER_VERSION: &str = env!("CARGO_PKG_VERSION");
