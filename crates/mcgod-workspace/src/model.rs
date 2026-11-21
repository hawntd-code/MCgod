//! Estructuras del archivo `<modid>.mcreator`, tal como lo escribe MCreator con Gson.
//!
//! Los nombres de campo mezclan `snake_case` y `camelCase` porque así están en las clases
//! Java de MCreator (`Workspace`, `WorkspaceSettings`, `ModElement`). Cada estructura guarda
//! los campos que no conoce en `extra`, para no perder datos al volver a escribir el archivo.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Contenido completo de `<modid>.mcreator`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceFile {
    #[serde(default)]
    pub mod_elements: Vec<ModElement>,
    #[serde(default)]
    pub variable_elements: Vec<Value>,
    #[serde(default)]
    pub sound_elements: Vec<Value>,
    /// Llaves con la forma `TIPO:espacio:nombre`, p. ej. `ITEMS:minecraft:planks`.
    #[serde(default)]
    pub tag_elements: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_element_order: Option<Value>,
    /// Idioma → (clave → texto). MCreator siempre incluye `en_us`.
    #[serde(default)]
    pub language_map: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(
        rename = "foldersRoot",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub folders_root: Option<Value>,
    #[serde(rename = "workspaceSettings")]
    pub workspace_settings: WorkspaceSettings,
    #[serde(rename = "mcreatorVersion", default)]
    pub mcreator_version: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Ajustes del mod (`net.mcreator.workspace.settings.WorkspaceSettings`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub modid: String,
    #[serde(rename = "modName", default, skip_serializing_if = "Option::is_none")]
    pub mod_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(
        rename = "websiteURL",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub website_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(rename = "serverSideOnly", default)]
    pub server_side_only: bool,
    #[serde(
        rename = "currentGenerator",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub current_generator: Option<String>,
    #[serde(
        rename = "modElementsPackage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mod_elements_package: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Entrada de un elemento en `mod_elements` (`net.mcreator.workspace.elements.ModElement`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModElement {
    pub name: String,
    #[serde(rename = "type", deserialize_with = "de_element_type")]
    pub element_type: String,
    #[serde(default = "yes")]
    pub compiles: bool,
    #[serde(default)]
    pub locked_code: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    /// Carpeta del árbol de MCreator, p. ej. `~/Minerales`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn yes() -> bool {
    true
}

/// MCreator renombró dos tipos con los años; su lector hace esta misma conversión.
pub fn normalize_element_type(raw: &str) -> &str {
    match raw {
        "gun" => "rangeditem",
        "mob" => "livingentity",
        other => other,
    }
}

fn de_element_type<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let raw = String::deserialize(d)?;
    Ok(normalize_element_type(&raw).to_string())
}

/// Contenido de `elements/<Nombre>.mod.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementDefinition {
    /// Versión del formato del elemento (`_fv`).
    #[serde(rename = "_fv", default)]
    pub format_version: i64,
    #[serde(rename = "_type")]
    pub element_type: String,
    pub definition: Map<String, Value>,
}

/// Generador elegido en el proyecto, p. ej. `neoforge-1.21.1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorId {
    pub id: String,
    /// `neoforge`, `forge`, `fabric`, `datapack`, `resourcepack`, `addon`…
    pub flavor: String,
    pub minecraft: Option<String>,
}

impl GeneratorId {
    pub fn parse(id: &str) -> GeneratorId {
        match id.split_once('-') {
            Some((flavor, version)) if !version.is_empty() => GeneratorId {
                id: id.to_string(),
                flavor: flavor.to_string(),
                minecraft: Some(version.to_string()),
            },
            _ => GeneratorId {
                id: id.to_string(),
                flavor: id.to_string(),
                minecraft: None,
            },
        }
    }
}
