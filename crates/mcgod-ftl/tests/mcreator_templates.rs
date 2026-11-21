//! Plantillas reales de MCreator con un modelo de datos como el de `BaseDataModelProvider`.
//! El resultado esperado aplica las reglas de FreeMarker (recorte de espacios, formatos) a mano.

use mcgod_ftl::{DirLoader, Engine, HostObject, Map, Value};
use std::path::PathBuf;

fn engine() -> Engine {
    Engine::new(DirLoader {
        roots: vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mcreator")],
    })
}

/// `WorkspaceSettings` del mod de ejemplo (examples/rubymod).
struct Settings;
impl HostObject for Settings {
    fn get(&self, key: &str) -> Option<Value> {
        let s = |v: &'static str| Some(Value::host_fn(move |_| Ok(v.into())));
        let empty = || Some(Value::host_fn(|_| Ok(Value::seq(vec![]))));
        match key {
            "getModID" => s("rubymod"),
            "getModName" => s("Ruby Mod"),
            "getCleanVersion" => s("1.0.0"),
            "getLicense" => s("GPL-3.0"),
            "getAuthor" => s("MCgod"),
            "getDescription" => s("Workspace de ejemplo para las pruebas de MCgod"),
            "getUpdateURL" | "getWebsiteURL" | "getModPicture" | "getCredits" => s(""),
            "isServerSideOnly" => Some(Value::host_fn(|_| Ok(false.into()))),
            "getRequiredMods" | "getDependencies" | "getDependants" => empty(),
            _ => None,
        }
    }
}

struct Generator;
impl HostObject for Generator {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "getGeneratorBuildFileVersion" => Some(Value::host_fn(|_| Ok("21.1.265".into()))),
            "getGeneratorMinecraftVersion" => Some(Value::host_fn(|_| Ok("1.21.1".into()))),
            _ => None,
        }
    }
}

/// `WorkspaceInfo` (`w`) de un mod sin sonidos, pestañas ni variables.
struct Workspace;
impl HostObject for Workspace {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "hasSounds" | "hasItemsInTabs" | "hasVariables" => {
                Some(Value::host_fn(|_| Ok(false.into())))
            }
            _ => None,
        }
    }
}

struct JavaConventions;
impl HostObject for JavaConventions {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "escapeStringForJava" => Some(Value::host_fn(|args| {
                let s = args.first().and_then(|v| v.as_str()).unwrap_or("");
                Ok(s.replace('\\', "\\\\").replace('"', "\\\"").into())
            })),
            _ => None,
        }
    }
}

fn model() -> Map {
    let mut types = Map::new();
    types.insert("features".into(), Value::seq(vec![]));
    let mut fragments = Map::new();
    fragments.insert("mixin".into(), Value::seq(vec![]));

    let mut m = Map::new();
    m.insert("settings".into(), Value::object(Settings));
    m.insert("generator".into(), Value::object(Generator));
    m.insert("w".into(), Value::object(Workspace));
    m.insert("JavaConventions".into(), Value::object(JavaConventions));
    m.insert("types".into(), Value::hash(types));
    m.insert("fragments".into(), Value::hash(fragments));
    m.insert("package".into(), "net.mcreator.rubymod".into());
    m.insert("modid".into(), "rubymod".into());
    m.insert("JavaModName".into(), "RubymodMod".into());
    m
}

#[test]
fn neoforge_mods_toml() {
    let out = engine()
        .render("modbase/neoforge.mods.toml.ftl", &model())
        .unwrap();
    let expected = r#"modLoader="javafml"
loaderVersion="[4,)"
license="GPL-3.0"

[[mods]]
modId="rubymod"
version="1.0.0"
displayName="Ruby Mod"
authors="MCgod"
description="Workspace de ejemplo para las pruebas de MCgod"

# Start of user code block mod configuration
# End of user code block mod configuration


# Start of user code block custom mixins
# End of user code block custom mixins

[[dependencies.rubymod]]
    modId="neoforge"
    type="required"
    versionRange="[21.1.265,)"
    ordering="AFTER"
    side="BOTH"

[[dependencies.rubymod]]
    modId="minecraft"
    type="required"
    versionRange="[1.21.1]"
    ordering="AFTER"
    side="BOTH"




# Start of user code block dependencies configuration
# End of user code block dependencies configuration"#;
    assert_eq!(out, expected);
}

#[test]
fn pack_mcmeta() {
    let out = engine().render("pack.mcmeta.ftl", &model()).unwrap();
    // La línea del pack_format tiene texto y un comentario: no se recorta y conserva su espacio final.
    let expected = "{\n    \"pack\": {\n        \"pack_format\": 48, \n        \"description\": \"Workspace de ejemplo para las pruebas de MCgod\"\n    }\n}";
    assert_eq!(out, expected);
}

#[test]
fn main_mod_class_without_elements() {
    let out = engine().render("modbase/mod.java.ftl", &model()).unwrap();
    assert!(
        out.starts_with("package net.mcreator.rubymod;\n\n"),
        "{out}"
    );
    assert!(out.contains("@Mod(\"rubymod\") public class RubymodMod {"));
    assert!(out.contains("public static final String MODID = \"rubymod\";"));
    assert!(out.contains("LogManager.getLogger(RubymodMod.class);"));
    // Sin elementos, javacompress deja vacío el bloque de registros.
    assert!(!out.contains("REGISTRY.register"));
    assert!(out.contains("\t\tmodEventBus.addListener(this::registerNetworking);\n\n\n\t\t// Start of user code block mod init"));
    // Los comentarios FTL desaparecen.
    assert!(!out.contains("<#--") && !out.contains("@formatter"));
    assert!(out.trim_end().ends_with('}'));
}

#[test]
fn main_mod_class_registers_what_the_mod_has() {
    let mut m = model();
    let mut types = Map::new();
    types.insert("features".into(), Value::seq(vec![]));
    types.insert("base_items".into(), Value::seq(vec![Value::str("Ruby")]));
    types.insert(
        "base_blocks".into(),
        Value::seq(vec![Value::str("RubyBlock")]),
    );
    m.insert("types".into(), Value::hash(types));
    let out = engine().render("modbase/mod.java.ftl", &m).unwrap();
    // javacompress junta las líneas de registro, una por línea, sin sangría ni vacías.
    assert!(
        out.contains("RubymodModBlocks.REGISTRY.register(modEventBus);\nRubymodModItems.REGISTRY.register(modEventBus);"),
        "{out}"
    );
}
