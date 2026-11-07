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

/// The shipped template manifest, embedded at compile time.
///
/// The release artifact is a single binary: web assets are embedded with
/// `rust-embed` (`static_ui.rs`), so prompt assets are embedded the same way.
/// A missing file is a build error (fail closed before the binary exists); a
/// malformed file still refuses by name at load time.
pub const EMBEDDED_MANIFEST: &str = include_str!("../assets/prompts/manifest.json");

/// The shipped persona manifest, embedded at compile time.
pub const EMBEDDED_PERSONAS: &str = include_str!("../assets/prompts/personas.json");

/// Embedded template bodies, keyed by the manifest `file` field.
pub const EMBEDDED_TEMPLATE_FILES: &[(&str, &str)] = &[
    (
        "planner.prompty",
        include_str!("../assets/prompts/planner.prompty"),
    ),
    (
        "content-writer.prompty",
        include_str!("../assets/prompts/content-writer.prompty"),
    ),
    (
        "data-analyst.prompty",
        include_str!("../assets/prompts/data-analyst.prompty"),
    ),
    (
        "code-reviewer.prompty",
        include_str!("../assets/prompts/code-reviewer.prompty"),
    ),
    (
        "chatbot.prompty",
        include_str!("../assets/prompts/chatbot.prompty"),
    ),
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
///
/// `Default` is the **test-only** empty-registry seam (kept so unit tests can
/// construct an engine without assets). Production constructors load the
/// embedded assets (`load_embedded`) and never silently serve with an empty
/// registry: a load failure is a named refusal surfaced by the boot path.
#[derive(Clone, Debug, Default)]
pub struct TemplateRegistry {
    state: TemplateRegistryState,
}

impl TemplateRegistry {
    /// Loads every template listed in `manifest.json` inside `directory`.
    ///
    /// # Errors
    /// `E_TEMPLATE_LICENSE_MISSING`, `E_TEMPLATE_REF_FORBIDDEN`, `E_TEMPLATE_INVALID`,
    /// or `E_TEMPLATE_UNKNOWN` when the manifest/file cannot be read.
    pub fn load(directory: &Path) -> Result<Self, Refusal> {
        let manifest_path = directory.join("manifest.json");
        let manifest_text = std::fs::read_to_string(&manifest_path).map_err(|error| {
            tracing::warn!(path = %manifest_path.display(), %error, "template manifest unreadable");
            Refusal::TemplateUnknown
        })?;
        Self::build(&manifest_text, |file| {
            let path = directory.join(file);
            std::fs::read_to_string(&path).map_err(|error| {
                tracing::warn!(path = %path.display(), %error, "template file unreadable");
                Refusal::TemplateUnknown
            })
        })
    }

    /// Loads the shipped prompt templates embedded in the binary.
    ///
    /// This is the production constructor seam: the manifest and every
    /// referenced `.prompty` file are compile-time embedded, so a release can
    /// never be missing them silently. Malformed content still refuses by name.
    ///
    /// # Errors
    /// `E_TEMPLATE_INVALID` for unparseable manifest/front matter,
    /// `E_TEMPLATE_UNKNOWN` when the manifest names a file that is not embedded,
    /// `E_TEMPLATE_LICENSE_MISSING` for a licence/source/fetch gap,
    /// `E_TEMPLATE_REF_FORBIDDEN` for `${env:}` / `${file:}` references.
    pub fn load_embedded() -> Result<Self, Refusal> {
        Self::build(EMBEDDED_MANIFEST, |file| {
            EMBEDDED_TEMPLATE_FILES
                .iter()
                .find(|(name, _)| *name == file)
                .map(|(_, body)| (*body).to_owned())
                .ok_or(Refusal::TemplateUnknown)
        })
    }

    /// Builds a registry from manifest text plus a caller-supplied file resolver.
    ///
    /// Exposed so tests can prove the refusal paths (invalid manifest, missing
    /// file, bad licence) against the exact production policy without touching
    /// the embedded assets. Production code uses [`TemplateRegistry::load_embedded`].
    ///
    /// # Errors
    /// Same named refusals as [`TemplateRegistry::load_embedded`].
    pub fn load_from_manifest_text(
        manifest_text: &str,
        files: &[(&str, &str)],
    ) -> Result<Self, Refusal> {
        Self::build(manifest_text, |file| {
            files
                .iter()
                .find(|(name, _)| *name == file)
                .map(|(_, body)| (*body).to_owned())
                .ok_or(Refusal::TemplateUnknown)
        })
    }

    /// Shared loader: manifest JSON -> licence gate -> per-file resolver -> parse.
    fn build(
        manifest_text: &str,
        mut resolve: impl FnMut(&str) -> Result<String, Refusal>,
    ) -> Result<Self, Refusal> {
        let entries: Vec<TemplateManifestEntry> =
            serde_json::from_str(manifest_text).map_err(|_| Refusal::TemplateInvalid)?;
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
            let file = entry.file.clone();
            let raw = resolve(&file)?;
            let source = PathBuf::from(&file);
            let loaded = parse_prompty(&raw, &source, entry)?;
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

fn parse_prompty(
    raw: &str,
    source: &Path,
    manifest: TemplateManifestEntry,
) -> Result<LoadedTemplate, Refusal> {
    if raw.contains("${env:") || raw.contains("${file:") {
        return Err(Refusal::TemplateRefForbidden);
    }
    // The pinned prompty loader validates the front matter; it can panic on
    // malformed assets, so the call is isolated and mapped to a named refusal.
    // `load_from_string` is the same parser the file loader uses, without the
    // filesystem read: embedded bytes and fixture bytes share one policy path.
    let source_owned = source.to_path_buf();
    let parsed = catch_unwind(AssertUnwindSafe(|| {
        prompty::loader::load_from_string(raw, source_owned.as_path())
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
    /// Optional per-profile child cap (must be <= the global fan-out cap).
    #[serde(default)]
    pub max_children: Option<usize>,
    /// Optional per-profile depth cap (must be <= the global depth cap).
    #[serde(default)]
    pub max_depth: Option<u8>,
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
        if self
            .max_children
            .is_some_and(|limit| limit == 0 || limit > crate::caps::MAX_FANOUT)
        {
            return Err(Refusal::PersonaUnknown);
        }
        if self
            .max_depth
            .is_some_and(|limit| limit == 0 || limit > crate::caps::MAX_SPAWN_DEPTH)
        {
            return Err(Refusal::PersonaUnknown);
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
    parse_personas(&text, templates)
}

/// Loads the embedded persona manifest; the single release binary has no
/// `<data_dir>/assets/prompts/personas.json` to read.
///
/// # Errors
/// `E_PERSONA_UNKNOWN` / `E_TEMPLATE_UNKNOWN` / `E_TEMPLATE_INVALID`.
pub fn load_personas_embedded(templates: &TemplateRegistry) -> Result<Vec<Persona>, Refusal> {
    parse_personas(EMBEDDED_PERSONAS, templates)
}

fn parse_personas(text: &str, templates: &TemplateRegistry) -> Result<Vec<Persona>, Refusal> {
    let personas: Vec<Persona> =
        serde_json::from_str(text).map_err(|_| Refusal::TemplateInvalid)?;
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
