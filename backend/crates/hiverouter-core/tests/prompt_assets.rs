//! Phase 3 acceptance: prompt-asset / persona schema round-trips and the
//! template render path (blueprint card commands `prompt_schema_roundtrip`,
//! `template_schema_roundtrip`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use hiverouter_contract::Refusal;
use hiverouter_core::prompts::{
    load_personas, load_personas_embedded, MessageRole, Persona, PersonaTier, TemplateManifestEntry,
    TemplateRegistry,
};

fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("prompts")
}

fn temp_manifest(dir: &Path, spdx: &str, file_body: &str) -> PathBuf {
    let manifest = format!(
        r#"[{{"template_id":"t1","file":"t1.prompty","spdx":"{spdx}","source_url":"authored in test","fetched_at":"2026-11-06"}}]"#
    );
    std::fs::write(dir.join("manifest.json"), manifest).expect("write manifest");
    std::fs::write(dir.join("t1.prompty"), file_body).expect("write prompt");
    dir.to_path_buf()
}

#[test]
fn template_schema_roundtrip() {
    let registry = TemplateRegistry::load(&assets()).expect("load template manifest");
    assert_eq!(
        registry.template_ids(),
        vec![
            "agent".to_owned(),
            "chatbot".to_owned(),
            "code-reviewer".to_owned(),
            "content-writer".to_owned(),
            "data-analyst".to_owned(),
            "planner".to_owned(),
            "subagent".to_owned(),
            "superagent".to_owned(),
        ]
    );
    let template = registry.get("planner").expect("planner template");
    assert_eq!(template.name, "planner");
    assert!(template.manifest.spdx == "MIT");
    assert!(template.inputs.iter().any(|input| input == "task"));

    let mut inputs = BTreeMap::new();
    inputs.insert("task".to_owned(), "draw the hive".to_owned());
    let messages = registry.render("planner", &inputs).expect("render");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, MessageRole::System);
    assert!(messages[0].content.contains("planner"));
    assert_eq!(messages[1].role, MessageRole::User);
    assert!(messages[1].content.contains("draw the hive"));
    assert!(!messages[1].content.contains("{{ task }}"), "placeholder not substituted");

    // Error path 1: unknown template id.
    assert_eq!(
        registry.render("nope", &inputs).expect_err("unknown template"),
        Refusal::TemplateUnknown
    );

    // Error path 2: forbidden ${env:} reference in a user template.
    let temp = tempfile::tempdir().expect("temp");
    let body = "---\nname: t1\nmodel:\n  id: deepseek-flash\n---\nsystem:\nKey is ${env:API_KEY}\n";
    temp_manifest(temp.path(), "MIT", body);
    assert_eq!(
        TemplateRegistry::load(temp.path()).expect_err("env ref"),
        Refusal::TemplateRefForbidden
    );

    // Error path 3: licence not on the allowlist.
    let temp = tempfile::tempdir().expect("temp");
    let body = "---\nname: t1\nmodel:\n  id: deepseek-flash\n---\nsystem:\nHello\n";
    temp_manifest(temp.path(), "GPL-3.0", body);
    assert_eq!(
        TemplateRegistry::load(temp.path()).expect_err("licence"),
        Refusal::TemplateLicenseMissing
    );
}

#[test]
fn prompt_schema_roundtrip() {
    let registry = TemplateRegistry::load(&assets()).expect("load templates");
    let personas = load_personas(&assets().join("personas.json"), &registry).expect("personas");
    assert_eq!(personas.len(), 8);
    let persona = &personas[0];
    assert_eq!(persona.tier, PersonaTier::Hyper);
    assert_eq!(persona.template_id, "planner");
    assert_eq!(persona.refusal_policy, "strict");
    assert_eq!(persona.validate(&registry), Ok(()));

    // JSON round-trip of the typed persona record.
    let json = serde_json::to_string(persona).expect("serialize persona");
    let back: Persona = serde_json::from_str(&json).expect("deserialize persona");
    assert_eq!(&back, persona);

    // Invalid persona shape is refused by name.
    let invalid = Persona {
        id: String::new(),
        ..persona.clone()
    };
    assert_eq!(invalid.validate(&registry), Err(Refusal::PersonaUnknown));

    // Persona pointing at a missing template is refused.
    let missing_template = Persona {
        template_id: "does-not-exist".to_owned(),
        ..persona.clone()
    };
    assert_eq!(
        missing_template.validate(&registry),
        Err(Refusal::TemplateUnknown)
    );

    // Manifest row type is part of the schema surface.
    let manifest_row = TemplateManifestEntry {
        template_id: "planner".to_owned(),
        file: "planner.prompty".to_owned(),
        spdx: "MIT".to_owned(),
        source_url: "authored in this repository".to_owned(),
        fetched_at: "2026-11-06".to_owned(),
    };
    let row_json = serde_json::to_value(&manifest_row).expect("manifest row json");
    assert_eq!(row_json["template_id"], "planner");
}

#[test]
fn embedded_personas_load_without_a_data_dir() {
    let registry = TemplateRegistry::load_embedded().expect("embedded templates");
    let personas = load_personas_embedded(&registry).expect("embedded personas");
    assert_eq!(personas.len(), 8);
    assert_eq!(personas[0].id, "planner");
    assert_eq!(personas[0].template_id, "planner");
    assert_eq!(personas[0].validate(&registry), Ok(()));
}
