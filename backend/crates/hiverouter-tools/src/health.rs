//! `/health` and `/diagnose` endpoints (Phase 11 WPs 11.6/11.13).
//!
//! Local-only observability: one axum router the core server merges; no second
//! server, no collector. Sampling uses `sysinfo 0.40.1` and keeps the V1â€“V6
//! null-vs-zero rule: an unavailable sample is `null`, a measured zero is `0`
//! (`children: 0` is a real zero, never `null`).
//!
//! `/diagnose` answers with derived data only: refusal count/codes from the
//! frozen enum, tool counts from the enum-derived census, provider endpoint and
//! price provenance, key presence as a boolean (the key itself is never
//! exposed), DB health and audit directory.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::routing::get;
use axum::{extract::State, Json, Router};
use hiverouter_contract::Refusal;
use hiverouter_provider::{pricing, DEFAULT_BASE_URL};
use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::census::{standard_census, CapabilityCensus};
use crate::error::ToolError;
use crate::tool::{AgentTier, PermissionClass};

/// Health payload schema id.
pub const HEALTH_SCHEMA: &str = "hiverouter.health.v1";
/// Diagnosis payload schema id.
pub const DIAGNOSE_SCHEMA: &str = "hiverouter.diagnose.v1";

/// Path of the health route.
pub const HEALTH_PATH: &str = "/health";
/// Path of the diagnosis route.
pub const DIAGNOSE_PATH: &str = "/diagnose";

/// One health sample (V1â€“V6 honest nulls).
#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    /// Payload schema id.
    pub schema: &'static str,
    /// `ok` when the process is serving.
    pub status: &'static str,
    /// Crate version.
    pub version: &'static str,
    /// Seconds since the server started.
    pub uptime_secs: u64,
    /// Process id.
    pub pid: u32,
    /// Resident set size in bytes (`None` when sampling is unavailable).
    pub rss_bytes: Option<u64>,
    /// Process CPU percent (`None` when sampling is unavailable).
    pub cpu_pct: Option<f32>,
    /// Child tasks currently tracked (a real zero is `0`).
    pub children: usize,
    /// Whether the SQLite layer is healthy.
    pub db_ok: bool,
    /// Whether `DEEPSEEK_API_KEY` is present (the value is never exposed).
    pub provider_key_present: bool,
}

/// Provider facts exposed by `/diagnose` (no secrets).
#[derive(Debug, Clone, Serialize)]
pub struct ProviderDiagnosis {
    /// Documented base URL (F1).
    pub base_url: &'static str,
    /// Documented model ids.
    pub models: Vec<&'static str>,
    /// Price provenance URL.
    pub price_source_url: &'static str,
    /// Price fetch date.
    pub price_fetched_at: &'static str,
    /// Documented run cost cap (decimal string).
    pub run_cost_cap_usd: String,
    /// Documented combined token budget.
    pub token_budget_combined: u64,
    /// Documented call cap.
    pub call_cap: u32,
    /// Documented maximum attempts per call.
    pub max_attempts: u32,
}

/// Refusal census (derived from the frozen enum).
#[derive(Debug, Clone, Serialize)]
pub struct RefusalCensus {
    /// Number of named refusals.
    pub count: usize,
    /// Every stable `E_*` code.
    pub codes: Vec<&'static str>,
}

/// Tool census subset exposed by `/diagnose`.
#[derive(Debug, Clone, Serialize)]
pub struct ToolCensus {
    /// Number of registered tools.
    pub count: usize,
    /// Tool ids.
    pub ids: Vec<String>,
    /// Allowlist grants across tiers.
    pub grants: usize,
}

/// Full diagnosis payload.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnoseReport {
    /// Payload schema id.
    pub schema: &'static str,
    /// Crate version.
    pub version: &'static str,
    /// Seconds since the server started.
    pub uptime_secs: u64,
    /// Whether the SQLite layer is healthy.
    pub db_ok: bool,
    /// Whether the provider key is present (boolean only).
    pub provider_key_present: bool,
    /// Provider facts.
    pub provider: ProviderDiagnosis,
    /// Refusal census.
    pub refusals: RefusalCensus,
    /// Tool census.
    pub tools: ToolCensus,
    /// Full derived capability census.
    pub capability_census: CapabilityCensus,
    /// Audit directory, when configured.
    pub audit_log_dir: Option<String>,
    /// Tier labels (derived from the enum).
    pub agent_tiers: Vec<&'static str>,
    /// Permission class labels (derived from the enum).
    pub permission_classes: Vec<&'static str>,
}

/// Shared state for the health/diagnosis routes.
#[derive(Debug)]
pub struct ServiceState {
    version: &'static str,
    started_at: Instant,
    db_ok: bool,
    provider_key_present: bool,
    children: usize,
    audit_log_dir: Option<PathBuf>,
    census: CapabilityCensus,
    system: Option<System>,
    pid: Pid,
}

impl ServiceState {
    /// Builds the state; the census is derived once (a drift failure refuses
    /// startup rather than serving inconsistent counts).
    ///
    /// # Errors
    /// [`ToolError`] when the capability census has drifted.
    pub fn new(
        version: &'static str,
        db_ok: bool,
        provider_key_present: bool,
        audit_log_dir: Option<PathBuf>,
    ) -> Result<Self, ToolError> {
        Ok(Self {
            version,
            started_at: Instant::now(),
            db_ok,
            provider_key_present,
            children: 0,
            audit_log_dir,
            census: standard_census()?,
            system: System::new().ok(),
            pid: Pid::from_u32(std::process::id()),
        })
    }

    /// Updates the tracked child-task count (operator/task bookkeeping).
    pub fn set_children(&mut self, children: usize) {
        self.children = children;
    }

    /// Updates DB health after a probe.
    pub fn set_db_ok(&mut self, db_ok: bool) {
        self.db_ok = db_ok;
    }

    /// One fresh health sample.
    pub fn sample(&mut self) -> HealthReport {
        let (rss_bytes, cpu_pct) = self.sample_process();
        HealthReport {
            schema: HEALTH_SCHEMA,
            status: "ok",
            version: self.version,
            uptime_secs: self.started_at.elapsed().as_secs(),
            pid: std::process::id(),
            rss_bytes,
            cpu_pct,
            children: self.children,
            db_ok: self.db_ok,
            provider_key_present: self.provider_key_present,
        }
    }

    fn sample_process(&mut self) -> (Option<u64>, Option<f32>) {
        let Some(system) = self.system.as_mut() else {
            return (None, None);
        };
        let pids = [self.pid];
        system.refresh_processes(ProcessesToUpdate::Some(&pids), true);
        match system.process(self.pid) {
            Some(process) => (Some(process.memory()), Some(process.cpu_usage())),
            None => (None, None),
        }
    }

    /// Builds the diagnosis report.
    #[must_use]
    pub fn diagnose(&self) -> DiagnoseReport {
        DiagnoseReport {
            schema: DIAGNOSE_SCHEMA,
            version: self.version,
            uptime_secs: self.started_at.elapsed().as_secs(),
            db_ok: self.db_ok,
            provider_key_present: self.provider_key_present,
            provider: ProviderDiagnosis {
                base_url: DEFAULT_BASE_URL,
                models: hiverouter_provider::Model::ALL
                    .iter()
                    .map(|model| model.id())
                    .collect(),
                price_source_url: pricing::PRICE_SOURCE_URL,
                price_fetched_at: pricing::PRICE_FETCHED_AT,
                run_cost_cap_usd: hiverouter_provider::run_cost_cap_usd().to_string(),
                token_budget_combined: hiverouter_provider::TOKEN_BUDGET_COMBINED,
                call_cap: hiverouter_provider::CALL_CAP,
                max_attempts: hiverouter_provider::MAX_ATTEMPTS,
            },
            refusals: RefusalCensus {
                count: Refusal::ALL.len(),
                codes: Refusal::ALL.iter().map(|refusal| refusal.code()).collect(),
            },
            tools: ToolCensus {
                count: self.census.tool_count,
                ids: self.census.registry_tool_ids.clone(),
                grants: self.census.grant_count(),
            },
            capability_census: self.census.clone(),
            audit_log_dir: self
                .audit_log_dir
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            agent_tiers: AgentTier::all().iter().map(|tier| tier.label()).collect(),
            permission_classes: PermissionClass::all()
                .iter()
                .map(|class| class.label())
                .collect(),
        }
    }

    /// The derived census served by `/diagnose`.
    #[must_use]
    pub fn census(&self) -> &CapabilityCensus {
        &self.census
    }
}

/// Shared handler state.
pub type SharedState = Arc<Mutex<ServiceState>>;

/// The `/health` + `/diagnose` router (core merges this into its single server).
#[must_use]
pub fn router(state: SharedState) -> Router {
    Router::new()
        .route(HEALTH_PATH, get(health_handler))
        .route(DIAGNOSE_PATH, get(diagnose_handler))
        .with_state(state)
}

async fn health_handler(State(state): State<SharedState>) -> Json<HealthReport> {
    let mut state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Json(state.sample())
}

async fn diagnose_handler(State(state): State<SharedState>) -> Json<DiagnoseReport> {
    let state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Json(state.diagnose())
}

/// Convenience for tests and `main`: a state with the standard registry.
///
/// # Errors
/// [`ToolError`] when the census has drifted.
pub fn ready_state(
    version: &'static str,
    db_ok: bool,
    provider_key_present: bool,
) -> Result<SharedState, ToolError> {
    Ok(Arc::new(Mutex::new(ServiceState::new(
        version,
        db_ok,
        provider_key_present,
        None,
    )?)))
}

// The census is built from the standard registry by `ServiceState::new`; this
// module deliberately does not hold a second registry instance.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolId;
    use std::time::Duration;

    async fn serve(state: SharedState) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        let app = router(state);
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    async fn get_json(url: &str) -> (u16, serde_json::Value) {
        let response = reqwest::get(url).await.expect("request");
        let status = response.status().as_u16();
        let body: serde_json::Value = response.json().await.expect("json body");
        (status, body)
    }

    #[tokio::test]
    async fn health_endpoint_returns_200_with_status_json() {
        let state = ready_state("0.1.0", true, false).expect("state");
        let base = serve(state).await;
        let (status, body) = get_json(&format!("{base}{HEALTH_PATH}")).await;
        assert_eq!(status, 200);
        assert_eq!(body["schema"], HEALTH_SCHEMA);
        assert_eq!(body["status"], "ok");
        assert_eq!(body["children"], 0, "a measured zero is 0, not null");
        assert_eq!(body["provider_key_present"], false);
        assert!(
            body.get("rss_bytes").is_some(),
            "the sampling field is present even when its value is null"
        );
        assert_eq!(body["uptime_secs"].as_u64().is_some(), true);
        let _ = Duration::from_millis(1);
    }

    #[tokio::test]
    async fn health_reports_zero_children_as_zero_and_updated_counts() {
        let state = ready_state("0.1.0", true, true).expect("state");
        {
            let mut locked = state.lock().expect("lock");
            locked.set_children(3);
        }
        let base = serve(state).await;
        let (_, body) = get_json(&format!("{base}{HEALTH_PATH}")).await;
        assert_eq!(body["children"], 3);
        assert_eq!(body["provider_key_present"], true);
    }

    #[tokio::test]
    async fn diagnose_endpoint_is_derived_and_secret_free() {
        let state = ready_state("0.1.0", true, false).expect("state");
        let base = serve(state).await;
        let (status, body) = get_json(&format!("{base}{DIAGNOSE_PATH}")).await;
        assert_eq!(status, 200);
        assert_eq!(body["schema"], DIAGNOSE_SCHEMA);
        assert_eq!(
            body["refusals"]["count"].as_u64().expect("count"),
            u64::try_from(Refusal::ALL.len()).expect("fits")
        );
        assert_eq!(
            body["tools"]["count"].as_u64().expect("count"),
            u64::try_from(ToolId::all().len()).expect("fits")
        );
        assert_eq!(
            body["provider"]["base_url"],
            "https://api.deepseek.com",
            "documented endpoint, no /v1"
        );
        assert_eq!(body["provider_key_present"], false);
        let rendered = body.to_string();
        assert!(!rendered.contains("sk-"), "no key material in diagnosis");
        assert!(!rendered.contains("Bearer "));
        assert!(body["capability_census"]["refusal_count"]
            .as_u64()
            .expect("count")
            >= 40);
        assert_eq!(body["agent_tiers"].as_array().expect("tiers").len(), 4);
        assert_eq!(
            body["permission_classes"].as_array().expect("classes").len(),
            4
        );
    }

    #[tokio::test]
    async fn unknown_route_is_404_not_a_silent_pass() {
        let state = ready_state("0.1.0", true, false).expect("state");
        let base = serve(state).await;
        let response = reqwest::get(format!("{base}/not-a-route"))
            .await
            .expect("request");
        assert_eq!(response.status().as_u16(), 404);
    }
}

