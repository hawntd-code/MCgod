//! Pruebas con un workspace de ejemplo hecho a mano siguiendo el formato de MCreator
//! (`tests/fixtures/rubymod`). No lo generó MCreator: cuando tengamos workspaces reales,
//! se agregarán como pruebas de comparación.

use mcgod_workspace::model::{GeneratorId, WorkspaceFile, normalize_element_type};
use mcgod_workspace::{Workspace, WorkspaceError};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rubymod")
}

#[test]
fn opens_from_the_folder_or_from_the_file() {
    let from_dir = Workspace::open(fixture()).unwrap();
    let from_file = Workspace::open(fixture().join("rubymod.mcreator")).unwrap();
    assert_eq!(from_dir.file, from_file.file);
    assert_eq!(from_dir.root, fixture());
    assert_eq!(from_dir.data, from_file.data);
}

#[test]
fn reads_mod_settings() {
    let ws = Workspace::open(fixture()).unwrap();
    let s = &ws.data.workspace_settings;
    assert_eq!(s.modid, "rubymod");
    assert_eq!(s.mod_name.as_deref(), Some("Ruby Mod"));
    assert_eq!(s.current_generator.as_deref(), Some("neoforge-1.21.1"));
    assert_eq!(ws.data.mcreator_version, 202500315219);
    // Campos que el modelo no declara se conservan.
    assert!(s.extra.contains_key("requiredMods"));
}

#[test]
fn summary_lists_elements_in_file_order_with_their_state() {
    let summary = Workspace::open(fixture()).unwrap().summary();
    assert_eq!(summary.name, "Ruby Mod");
    assert_eq!(summary.elements.len(), 7);

    let names: Vec<_> = summary.elements.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Ruby",
            "RubyBlock",
            "RubyOre",
            "RubyPickaxe",
            "RubyBlockRecipe",
            "GlowOnStep",
            "RubyUtils"
        ]
    );

    let by_name = |n: &str| summary.elements.iter().find(|e| e.name == n).unwrap();
    assert_eq!(by_name("Ruby").folder.as_deref(), Some("~/Rubí"));
    assert_eq!(by_name("Ruby").registry_name.as_deref(), Some("ruby"));
    assert!(by_name("Ruby").has_definition);
    // RubyPickaxe no tiene .mod.json en la carpeta de ejemplo.
    assert!(!by_name("RubyPickaxe").has_definition);
    assert!(!by_name("RubyPickaxe").compiles);
    // El código personalizado no usa .mod.json, así que cuenta como completo.
    assert!(by_name("RubyUtils").has_definition);
    assert!(by_name("GlowOnStep").locked_code);

    assert_eq!(summary.variables, 1);
    assert_eq!(summary.sounds, 0);
    assert_eq!(summary.tags, 1);
    assert_eq!(summary.languages, ["en_us", "es_mx"]);
    assert_eq!(
        summary.generator,
        Some(GeneratorId {
            id: "neoforge-1.21.1".into(),
            flavor: "neoforge".into(),
            minecraft: Some("1.21.1".into()),
        })
    );
}

#[test]
fn reads_an_element_definition() {
    let ws = Workspace::open(fixture()).unwrap();
    let ruby = ws.read_element("Ruby").unwrap();
    assert_eq!(ruby.format_version, 92);
    assert_eq!(ruby.element_type, "item");
    assert_eq!(ruby.definition["rarity"], "UNCOMMON");
    assert_eq!(ruby.definition["stackSize"], 64);
    // El orden de los campos se conserva, como lo escribe MCreator.
    let first_keys: Vec<_> = ruby.definition.keys().take(3).cloned().collect();
    assert_eq!(first_keys, ["name", "rarity", "creativeTabs"]);
}

#[test]
fn element_errors_are_specific() {
    let ws = Workspace::open(fixture()).unwrap();
    assert!(
        matches!(ws.read_element("Nope"), Err(WorkspaceError::UnknownElement(n)) if n == "Nope")
    );
    assert!(matches!(
        ws.read_element("RubyPickaxe"),
        Err(WorkspaceError::NoDefinition(_))
    ));
    assert!(matches!(
        ws.read_element("RubyUtils"),
        Err(WorkspaceError::NoDefinition(_))
    ));
}

#[test]
fn round_trip_keeps_every_field() {
    let path = fixture().join("rubymod.mcreator");
    let original: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let parsed: WorkspaceFile = serde_json::from_value(original.clone()).unwrap();
    let written = serde_json::to_value(&parsed).unwrap();
    assert_eq!(written, original);
}

#[test]
fn legacy_types_are_renamed_like_mcreator_does() {
    assert_eq!(normalize_element_type("gun"), "rangeditem");
    assert_eq!(normalize_element_type("mob"), "livingentity");
    assert_eq!(normalize_element_type("item"), "item");

    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("old.mcreator"),
        r#"{"mod_elements":[{"name":"Zombie2","type":"mob"}],"workspaceSettings":{"modid":"old"}}"#,
    )
    .unwrap();
    let ws = Workspace::open(dir.path()).unwrap();
    let el = &ws.data.mod_elements[0];
    assert_eq!(el.element_type, "livingentity");
    // Valores por defecto de MCreator cuando faltan.
    assert!(el.compiles);
    assert!(!el.locked_code);
    assert_eq!(ws.summary().name, "old");
    assert_eq!(ws.summary().generator, None);
}

#[test]
fn accepts_a_utf8_bom() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("bom.mcreator"),
        "\u{feff}{\"workspaceSettings\":{\"modid\":\"bom\"}}",
    )
    .unwrap();
    assert_eq!(Workspace::open(dir.path()).unwrap().summary().modid, "bom");
}

#[test]
fn folder_errors() {
    let empty = tempfile::tempdir().unwrap();
    assert!(matches!(
        Workspace::open(empty.path()),
        Err(WorkspaceError::NoWorkspaceFile(_))
    ));

    let two = tempfile::tempdir().unwrap();
    for name in ["a.mcreator", "b.mcreator"] {
        fs::write(
            two.path().join(name),
            r#"{"workspaceSettings":{"modid":"x"}}"#,
        )
        .unwrap();
    }
    assert!(
        matches!(Workspace::open(two.path()), Err(WorkspaceError::Ambiguous(f)) if f.len() == 2)
    );

    assert!(matches!(
        Workspace::open(empty.path().join("missing")),
        Err(WorkspaceError::NotFound(_))
    ));

    let bad = tempfile::tempdir().unwrap();
    fs::write(bad.path().join("bad.mcreator"), "{ not json").unwrap();
    let err = Workspace::open(bad.path()).unwrap_err();
    assert!(matches!(err, WorkspaceError::Json { .. }));
    assert!(err.to_string().contains("bad.mcreator"));
}

#[test]
fn generator_id_parsing() {
    let g = GeneratorId::parse("datapack-26.3.x");
    assert_eq!(g.flavor, "datapack");
    assert_eq!(g.minecraft.as_deref(), Some("26.3.x"));
    let g = GeneratorId::parse("custom");
    assert_eq!(g.flavor, "custom");
    assert_eq!(g.minecraft, None);
}
