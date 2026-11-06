//! Prompt-asset and persona loading (blueprint Phase 3, WP 3.4).
//!
//! `.prompty` files are the portable prompt format (FACT-REGISTER F8): YAML front matter + a markdown body with
//! `system:` / `user:` / `assistant:` role markers. Parsing is delegated to the pinned `prompty 2.1.0` crate; the
//! policy layer here enforces the blueprint rules that prompty itself does not know about:
//!
//! - Licence gate (Rule 19): every template manifest row must carry `{spdx, source_url, fetched_at}` and a licence on
//!   the allowlist, otherwise `E_TEMPLATE_LICENSE_MISSING`.
//! - Reference gate: user-authored templates may not use `${env:...}` or `${file:...}` (`E_TEMPLATE_REF_FORBIDDEN`).
//! - Unknown template -> `E_TEMPLATE_UNKNOWN`; unparseable asset -> `E_TEMPLATE_INVALID`.
//!
//! Role rendering is implemented here over the typed `Agent` (deterministic, pinned by tests) instead of relying on an
//! undocumented pipeline signature.

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use hiverouter_contract::Refusal;
use serde::{Deserialize, Serialize};

/// Licences accepted for prompt/persona content (blueprint Rule 19).
pub const ALLOWED_LICENCES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "CC0-1.0",
];

/// Role marker in the prompt body / rendered message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// System instructions.
    System,
    /// User turn.
    User,
    /// Assistant turn.
    Assistant,
}

impl MessageRole {
    fn from_marker(marker: &str) -> Option<Self> {
        match marker.trim().to_ascii_lowercase().as_str() {
            "system" | "system:" => Some(MessageRole::System),
            "user" | "user:" => Some(MessageRole::User),
            "assistant" | "assistant:" => Some(MessageRole::Assistant),
            _ => None,
        }
    }
}

/// One rendered prompt message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedMessage {
    /// Role the text belongs to.
    pub role: MessageRole,
    /// Rendered text.
    pub content: String,
}

/// One row of the template manifest (`assets/prompts/manifest.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateManifestEntry {
    /// Stable template id referenced by `TEMPLATE_INSERT` and personas.
    pub template_id: String,
    /// File name relative to the manifest directory.
    pub file: String,
    /// SPDX licence id.
    pub spdx: String,
    /// Source URL the asset was fetched from.
    pub source_url: String,
    /// Fetch date (`YYYY-MM-DD`).
    pub fetched_at: String,
}

/// Loaded template asset (manifest row + typed prompty agent).
#[derive(Clone, Debug)]
pub struct LoadedTemplate {
    /// Manifest row.
    pub manifest: TemplateManifestEntry,
    /// Prompt instructions body (markdown, with role markers).
    pub instructions: String,
    /// Prompt name from the front matter.
    pub name: String,
    /// Prompt inputs declared in the front matter.
    pub inputs: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct TemplateRegistryState {
    templates: BTreeMap<String, LoadedTemplate>,
}

/// In-memory template registry loaded from a manifest directory.
#[derive(Clone, Debug, Default)]
pub struct TemplateRegistry {
    state: TemplateRegistryState,
}

impl TemplateRegistry {
    /// Loads every template listed in `manifest.json` inside `directory`.
    ///
    /// # Errors
    /// `E_TEMPLATE_LICENSE_MISSING`, `E_TEMPLATE_REF_FORBIDDEN`, `E_TEMPLATE_INVALID`, or a database/IO refusal.
    pub fn load(directory: &Path) -> Result<Self, Refusal> {
        let manifest_path = directory.join("manifest.json");
        let manifest_text = std::fs::read_to_string(&manifest_path).map_err(|error| {
            tracing::warn!(path = %manifest_path.display(), %error, "template manifest unreadable");
            Refusal::TemplateUnknown
        })?;
        let entries: Vec<TemplateManifestEntry> =
            serde_json::from_str(&manifest_text).map_err(|_| Refusal::TemplateInvalid)?;
        let mut templates = BTreeMap::new();
        for entry in entries {
            if !ALLOWED_LICENCES
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(&entry.spdx))
                || entry.source_url.trim().is_empty()
                || entry.fetched_at.trim().is_empty()
            {
                return Err(Refusal::TemplateLicenseMissing);
            }
            let path = directory.join(&entry.file);
            let loaded = load_template_file(&path, entry)?;
            templates.insert(loaded.manifest.template_id.clone(), loaded);
        }
        Ok(Self {
            state: TemplateRegistryState { templates },
        })
    }

    /// Registers an already-parsed template (tests and embedded assets).
    pub fn insert(&mut self, template: LoadedTemplate) {
        self.state
            .templates
            .insert(template.manifest.template_id.clone(), template);
    }

    /// Looks a template up.
    #[must_use]
    pub fn get(&self, template_id: &str) -> Option<&LoadedTemplate> {
        self.state.templates.get(template_id)
    }

    /// Template ids in the registry.
    #[must_use]
    pub fn template_ids(&self) -> Vec<String> {
        self.state.templates.keys().cloned().collect()
    }

    /// Renders a template with `{{ input }}` substitution and role-marker splitting.
    ///
    /// # Errors
    /// `E_TEMPLATE_UNKNOWN` for an unknown id, `E_TEMPLATE_INVALID` when no role section exists.
    pub fn render(
        &self,
        template_id: &str,
        inputs: &BTreeMap<String, String>,
    ) -> Result<Vec<RenderedMessage>, Refusal> {
        let template = self.get(template_id).ok_or(Refusal::TemplateUnknown)?;
        render_instructions(&template.instructions, inputs)
    }
}

fn load_template_file(
    path: &Path,
    manifest: TemplateManifestEntry,
) -> Result<LoadedTemplate, Refusal> {
    let raw = std::fs::read_to_string(path).map_err(|_| Refusal::TemplateUnknown)?;
    if raw.contains("${env:") || raw.contains("${file:") {
        return Err(Refusal::TemplateRefForbidden);
    }
    // The pinned prompty loader validates the front matter; it can panic on
    // malformed assets, so the call is isolated and mapped to a named refusal.
    let path_owned = path.to_path_buf();
    let parsed = catch_unwind(AssertUnwindSafe(|| {
        prompty::loader::load(path_owned.as_path())
    }))
    .map_err(|_| Refusal::TemplateInvalid)?
    .map_err(|_| Refusal::TemplateInvalid)?;
    let name = if parsed.name.is_empty() {
        manifest.template_id.clone()
    } else {
        parsed.name.clone()
    };
    let instructions = parsed.instructions.unwrap_or_default();
    if instructions.trim().is_empty() {
        return Err(Refusal::TemplateInvalid);
    }
    let inputs = parsed
        .inputs
        .unwrap_or_default()
        .into_iter()
        .map(|property| property.name)
        .collect();
    Ok(LoadedTemplate {
        manifest,
        instructions,
        name,
        inputs,
    })
}

fn render_instructions(
    instructions: &str,
    inputs: &BTreeMap<String, String>,
) -> Result<Vec<RenderedMessage>, Refusal> {
    let mut messages = Vec::new();
    let mut current: Option<(MessageRole, Vec<&str>)> = None;
    for line in instructions.lines() {
        let trimmed = line.trim();
        if let Some(marker) = trimmed.strip_suffix(':') {
            if let Some(role) = MessageRole::from_marker(marker) {
                if let Some((role, lines)) = current.take() {
                    messages.push(RenderedMessage {
                        role,
                        content: lines.join("\n").trim().to_owned(),
                    });
                }
                current = Some((role, Vec::new()));
                continue;
            }
        }
        if let Some((_, lines)) = current.as_mut() {
            lines.push(line);
        }
    }
    if let Some((role, lines)) = current.take() {
        messages.push(RenderedMessage {
            role,
            content: lines.join("\n").trim().to_owned(),
        });
    }
    if messages.is_empty() {
        return Err(Refusal::TemplateInvalid);
    }
    Ok(messages
        .into_iter()
        .map(|message| RenderedMessage {
            role: message.role,
            content: substitute(&message.content, inputs),
        })
        .collect())
}

fn substitute(text: &str, inputs: &BTreeMap<String, String>) -> String {
    let mut rendered = text.to_owned();
    for (key, value) in inputs {
        rendered = rendered.replace(&format!("{{{{ {key} }}}}"), value);
        rendered = rendered.replace(&format!("{{{{{key}}}}}"), value);
    }
    rendered
}

/// Persona tier (blueprint §2a agent tiers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaTier {
    /// Orchestrator tier.
    Hyper,
    /// Sub-hive tier.
    Super,
    /// Executing worker tier.
    Worker,
    /// Tool-invoking tier.
    Tool,
}

/// Typed persona record (blueprint WP 3.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Persona {
    /// Stable persona id.
    pub id: String,
    /// Display name for the UI.
    pub display_name: String,
    /// Agent tier.
    pub tier: PersonaTier,
    /// Template rendered for this persona.
    pub template_id: String,
    /// Tool ids this persona may invoke (empty means none).
    pub allowed_tool_ids: Vec<String>,
    /// Output schema id the child result must validate against.
    pub output_schema_id: Option<String>,
    /// Named refusal policy for this persona (`strict` / `report`).
    pub refusal_policy: String,
}

impl Persona {
    /// Validates the persona record against the template registry.
    ///
    /// # Errors
    /// `E_PERSONA_UNKNOWN` for an invalid id/display name, `E_TEMPLATE_UNKNOWN` when the template is missing.
    pub fn validate(&self, templates: &TemplateRegistry) -> Result<(), Refusal> {
        if self.id.trim().is_empty()
            || self.display_name.trim().is_empty()
            || !matches!(self.refusal_policy.as_str(), "strict" | "report")
        {
            return Err(Refusal::PersonaUnknown);
        }
        if templates.get(&self.template_id).is_none() {
            return Err(Refusal::TemplateUnknown);
        }
        Ok(())
    }
}

/// Loads a persona list from JSON and validates it against the templates.
///
/// # Errors
/// `E_PERSONA_UNKNOWN` / `E_TEMPLATE_UNKNOWN` / `E_TEMPLATE_INVALID` for bad JSON.
pub fn load_personas(
    path: &Path,
    templates: &TemplateRegistry,
) -> Result<Vec<Persona>, Refusal> {
    let text = std::fs::read_to_string(path).map_err(|_| Refusal::PersonaUnknown)?;
    let personas: Vec<Persona> =
        serde_json::from_str(&text).map_err(|_| Refusal::TemplateInvalid)?;
    for persona in &personas {
        persona.validate(templates)?;
    }
    Ok(personas)
}

/// Absolute path helper used by tests and the boot path.
#[must_use]
pub fn assets_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("assets").join("prompts")
}
