//! Held-out evaluation receipt gate for evolution lessons.
//!
//! A proposal being authorized by four SuperAgents and the operator is not, by
//! itself, evidence that the proposed change was measured. Before an authorized
//! lesson can be appended to `evolution/LESSONS.jsonl`, the operator must place
//! an immutable evaluation receipt at:
//!
//! `data_dir/evolution/eval/<proposal_uuid>/receipt.json`
//!
//! The receipt records the evaluator identity/hash, the pre-registered held-out
//! task set hash and size, baseline/candidate pass rates with confidence
//! intervals, the command, the observation time, and artifact hashes. This
//! module validates the shape and requires the candidate's lower confidence
//! bound to exceed the baseline's upper bound. It does not run the evaluator and
//! does not convert a paper claim into measured evidence.

use std::path::{Path, PathBuf};

use hiverouter_contract::Refusal;
use serde_json::Value;

/// Maximum receipt size accepted from disk (64 KiB).
pub const MAX_RECEIPT_BYTES: u64 = 64 * 1024;
/// Minimum held-out task-set size for a lesson admission.
pub const MIN_TASK_SET: u64 = 8;

/// A verified receipt plus the exact bytes' SHA-256.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedReceipt {
    /// SHA-256 of the receipt bytes.
    pub sha256: String,
    /// Held-out task-set identifier.
    pub task_set_id: String,
    /// Held-out task-set SHA-256.
    pub task_set_sha256: String,
    /// Number of held-out tasks.
    pub task_count: u64,
    /// Baselines/candidate summary retained for the lesson audit line.
    pub baseline_pass_rate: String,
    pub candidate_pass_rate: String,
}

fn receipt_path(data_dir: &Path, proposal_id: &str) -> PathBuf {
    data_dir
        .join("evolution")
        .join("eval")
        .join(proposal_id)
        .join("receipt.json")
}

fn is_hex64(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_safe_rate(value: &Value) -> Option<(f64, f64, f64)> {
    let pass = value.get("pass_rate")?.as_f64()?;
    let lower = value.get("ci_lower")?.as_f64()?;
    let upper = value.get("ci_upper")?.as_f64()?;
    if !(0.0..=1.0).contains(&pass)
        || !(0.0..=1.0).contains(&lower)
        || !(0.0..=1.0).contains(&upper)
        || lower > pass
        || pass > upper
    {
        return None;
    }
    Some((pass, lower, upper))
}

/// Validates the operator-supplied evaluation receipt.
///
/// # Errors
/// `E_APPROVAL_REQUIRED` when the receipt is absent; `E_SCHEMA_INVALID` when it
/// is too large, malformed, mis-bound to another proposal, uses a weak held-out
/// set, or does not show a candidate lower bound above the baseline upper bound.
pub fn verify_evaluation_receipt(
    data_dir: &Path,
    proposal_id: &str,
) -> Result<VerifiedReceipt, Refusal> {
    let path = receipt_path(data_dir, proposal_id);
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Refusal::ApprovalRequired
        } else {
            Refusal::InternalError
        }
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_RECEIPT_BYTES {
        return Err(Refusal::SchemaInvalid);
    }
    let bytes = std::fs::read(&path).map_err(|_| Refusal::InternalError)?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| Refusal::SchemaInvalid)?;
    if value.get("schema").and_then(Value::as_str) != Some("hivemind.evolution-eval.v1")
        || value.get("proposal_id").and_then(Value::as_str) != Some(proposal_id)
    {
        return Err(Refusal::SchemaInvalid);
    }
    let task_set = value.get("task_set").ok_or(Refusal::SchemaInvalid)?;
    let task_set_id = task_set
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or(Refusal::SchemaInvalid)?;
    let task_set_sha256 = task_set
        .get("sha256")
        .and_then(Value::as_str)
        .filter(|value| is_hex64(value))
        .ok_or(Refusal::SchemaInvalid)?;
    let task_count = task_set
        .get("n")
        .and_then(Value::as_u64)
        .filter(|n| *n >= MIN_TASK_SET)
        .ok_or(Refusal::SchemaInvalid)?;
    let evaluator = value.get("evaluator").ok_or(Refusal::SchemaInvalid)?;
    let evaluator_id = evaluator
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or(Refusal::SchemaInvalid)?;
    let evaluator_sha256 = evaluator
        .get("sha256")
        .and_then(Value::as_str)
        .filter(|value| is_hex64(value))
        .ok_or(Refusal::SchemaInvalid)?;
    let baseline = value.get("baseline").ok_or(Refusal::SchemaInvalid)?;
    let candidate = value.get("candidate").ok_or(Refusal::SchemaInvalid)?;
    let (baseline_pass, _, baseline_upper) =
        is_safe_rate(baseline).ok_or(Refusal::SchemaInvalid)?;
    let (candidate_pass, candidate_lower, _) =
        is_safe_rate(candidate).ok_or(Refusal::SchemaInvalid)?;
    if candidate_lower <= baseline_upper {
        return Err(Refusal::SchemaInvalid);
    }
    let observed_at = value
        .get("observed_at")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(Refusal::SchemaInvalid)?;
    let command = value
        .get("command")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(Refusal::SchemaInvalid)?;
    let artifacts = value
        .get("artifacts")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty())
        .ok_or(Refusal::SchemaInvalid)?;
    if artifacts.iter().any(|artifact| {
        artifact
            .get("sha256")
            .and_then(Value::as_str)
            .map_or(true, |hash| !is_hex64(hash))
    }) {
        return Err(Refusal::SchemaInvalid);
    }
    let _ = (evaluator_id, evaluator_sha256, observed_at, command);
    Ok(VerifiedReceipt {
        sha256: hiverouter_tools::sha256_hex(&bytes),
        task_set_id: task_set_id.to_owned(),
        task_set_sha256: task_set_sha256.to_owned(),
        task_count,
        baseline_pass_rate: format!("{baseline_pass:.6}"),
        candidate_pass_rate: format!("{candidate_pass:.6}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_receipt(root: &Path, proposal: &str, candidate_lower: f64) -> PathBuf {
        let path = receipt_path(root, proposal);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        let value = serde_json::json!({
            "schema": "hivemind.evolution-eval.v1",
            "proposal_id": proposal,
            "head_commit": "d8c00fdc208f014b4d26eea51408e1c2e74ed9eb",
            "task_set": { "id": "held-out", "sha256": "a".repeat(64), "n": 8 },
            "evaluator": { "id": "eval", "sha256": "b".repeat(64) },
            "baseline": { "pass_rate": 0.5, "ci_lower": 0.4, "ci_upper": 0.6 },
            "candidate": { "pass_rate": 0.9, "ci_lower": candidate_lower, "ci_upper": 0.95 },
            "observed_at": "2026-11-10T00:00:00Z",
            "command": "run suite",
            "artifacts": [{ "sha256": "c".repeat(64) }]
        });
        std::fs::write(&path, value.to_string()).expect("write");
        path
    }

    #[test]
    fn missing_receipt_requires_approval() {
        let root = tempfile::tempdir().expect("root");
        assert_eq!(
            verify_evaluation_receipt(root.path(), "p-1"),
            Err(Refusal::ApprovalRequired)
        );
    }

    #[test]
    fn valid_receipt_is_bound_to_proposal_and_hashes_the_bytes() {
        let root = tempfile::tempdir().expect("root");
        write_receipt(root.path(), "p-1", 0.8);
        let receipt = verify_evaluation_receipt(root.path(), "p-1").expect("valid");
        assert_eq!(receipt.sha256.len(), 64);
        assert_eq!(receipt.task_count, 8);
        assert_eq!(
            verify_evaluation_receipt(root.path(), "p-2"),
            Err(Refusal::ApprovalRequired)
        );
    }

    #[test]
    fn weak_or_stale_candidate_is_refused() {
        let root = tempfile::tempdir().expect("root");
        write_receipt(root.path(), "p-weak", 0.5);
        assert_eq!(
            verify_evaluation_receipt(root.path(), "p-weak"),
            Err(Refusal::SchemaInvalid)
        );
    }

    #[test]
    fn oversized_receipt_is_refused_before_parse() {
        let root = tempfile::tempdir().expect("root");
        let path = receipt_path(root.path(), "p-big");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(&path, vec![b'{'; (MAX_RECEIPT_BYTES + 1) as usize]).expect("write");
        assert_eq!(
            verify_evaluation_receipt(root.path(), "p-big"),
            Err(Refusal::SchemaInvalid)
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_receipt_is_refused() {
        let root = tempfile::tempdir().expect("root");
        let target = write_receipt(root.path(), "p-target", 0.8);
        let link = receipt_path(root.path(), "p-link");
        std::fs::create_dir_all(link.parent().expect("parent")).expect("dir");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        assert_eq!(
            verify_evaluation_receipt(root.path(), "p-link"),
            Err(Refusal::SchemaInvalid)
        );
    }
}
