//! Modelo de datos de las plantillas, como el de `BaseDataModelProvider` de MCreator:
//! `settings`, `generator`, `w`, `JavaConventions` y los elementos (`data`).

pub mod bean;
pub mod item;
pub mod parts;

use crate::mapping::{MCREATOR_PREFIX, Mapper, resolve_mod_namespace};
use crate::tokens::Tokens;
use indexmap::IndexMap;
use mcgod_ftl::{FtlError, HostObject, Map, Value};
use parts::Mappable;
use serde_json::Value as Json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Entrada de un elemento en el proyecto (`ModElement`).
#[derive(Debug, Clone)]
pub struct ElementInfo {
    pub name: String,
    pub kind: String,
    pub registry_name: String,
    pub locked_code: bool,
    pub compiles: bool,
    pub metadata: Json,
    pub folder: Option<String>,
}

/// Datos fijos de una generación: rutas, tablas de nombres, elementos y ajustes.
pub struct Env {
    pub tokens: Tokens,
    pub mappings: HashMap<String, Json>,
    pub elements: IndexMap<String, Rc<ElementInfo>>,
    pub generator_id: String,
    pub build_file_version: String,
    pub minecraft_version: String,
    pub settings: Json,
    /// Avisos encontrados mientras se procesan las plantillas.
    pub notes: RefCell<Vec<Note>>,
}

/// Aviso de la generación, con un código estable que la interfaz traduce.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub code: NoteCode,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NoteCode {
    /// El proyecto tiene sonidos y todavía no se generan.
    SoundsPending,
    /// Variables globales, todavía no se generan.
    VariablesPending,
    /// Etiquetas (tags), todavía no se generan.
    TagsPending,
    /// Un elemento usa un procedure; su llamada se omitió (`detail`: el procedure).
    ProcedurePending,
    /// Sin índice de clases: los archivos Java no llevan imports.
    NoImportIndex,
    /// Una plantilla produjo JSON inválido y se escribió tal cual (`detail`: archivo y error).
    InvalidJson,
    /// El generador usa archivos de idioma `.lang`, que todavía no se escriben.
    LangFormatPending,
    /// Referencia a un elemento que no existe (`detail`: el nombre).
    UnknownElement,
}

impl Env {
    pub fn map(&self, orig: &str, source: &str, table: usize) -> String {
        let rn = |n: &str| self.registry_name(n);
        Mapper {
            mappings: &self.mappings,
            tokens: &self.tokens,
            registry_name: &rn,
        }
        .map(orig, source, table)
    }

    pub fn element(&self, name: &str) -> Option<&Rc<ElementInfo>> {
        self.elements.get(name)
    }

    pub fn registry_name(&self, name: &str) -> Option<String> {
        self.element(name).map(|e| e.registry_name.clone())
    }

    pub fn note(&self, code: NoteCode, detail: Option<String>) {
        let note = Note { code, detail };
        let mut notes = self.notes.borrow_mut();
        if !notes.contains(&note) {
            notes.push(note);
        }
    }
}

/// Nombre simple de un elemento: sin `CUSTOM:`, sin `:Flowing` y sin el sufijo
/// después del último punto (`GeneratorWrapper.getElementPlainName`).
pub fn plain_name(name: &str) -> String {
    let s = name.replace(MCREATOR_PREFIX, "").replace(":Flowing", "");
    match s.rfind('.') {
        Some(i) => s[..i].to_string(),
        None => s,
    }
}

fn method(f: impl Fn(&[Value]) -> Result<Value, FtlError> + 'static) -> Option<Value> {
    Some(Value::host_fn(f))
}

fn constant(v: Value) -> Option<Value> {
    method(move |_| Ok(v.clone()))
}

fn arg_text(args: &[Value], i: usize) -> String {
    args.get(i).and_then(Value::text).unwrap_or_default()
}

/// `getModElement()`: nombre, nombre de registro, tipo, metadatos.
pub struct ModElementObj {
    pub info: Rc<ElementInfo>,
    pub env: Rc<Env>,
}

impl HostObject for ModElementObj {
    fn get(&self, key: &str) -> Option<Value> {
        let i = &self.info;
        match key {
            "getName" | "name" => constant(Value::str(i.name.as_str())),
            "getRegistryName" => constant(Value::str(i.registry_name.as_str())),
            "getRegistryNameUpper" => constant(Value::str(i.registry_name.to_uppercase())),
            "getTypeString" | "getType" => constant(Value::str(i.kind.as_str())),
            "isCodeLocked" => constant(Value::Bool(i.locked_code)),
            "doesCompile" => constant(Value::Bool(i.compiles)),
            "getFolderPath" => constant(Value::str(i.folder.clone().unwrap_or_default())),
            "getMetadata" => {
                let meta = i.metadata.clone();
                method(move |args| {
                    let key = arg_text(args, 0);
                    Ok(meta
                        .get(&key)
                        .filter(|v| !v.is_null())
                        .map(Value::from_json)
                        .unwrap_or(Value::Null))
                })
            }
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "ModElement"
    }

    fn to_text(&self) -> Option<String> {
        Some(self.info.name.clone())
    }
}

/// Elemento que se genera: su objeto de plantilla y los tipos base que aporta.
pub struct Generated {
    pub info: Rc<ElementInfo>,
    pub data: Value,
    pub base_types: Vec<String>,
}

/// Lo que comparten `w` y `generator`.
pub struct Ctx {
    pub env: Rc<Env>,
    pub generated: Vec<Generated>,
    /// Orden de las pestañas creativas (`tab_element_order` del proyecto).
    pub tab_order: IndexMap<String, Vec<String>>,
    pub has_sounds: bool,
    pub has_variables: bool,
    /// Pestañas a las que va cada elemento y lo que aporta a ellas (`MCItem` como `CUSTOM:Nombre`).
    pub tab_entries: HashMap<String, (Vec<String>, Vec<String>)>,
}

/// `WorkspaceInfo` (`w` en las plantillas).
pub struct WorkspaceInfo {
    pub ctx: Rc<Ctx>,
}

impl WorkspaceInfo {
    /// `getCreativeTabMap`: pestaña → ítems, respetando el orden propio de cada pestaña.
    fn creative_tab_map(ctx: &Ctx) -> IndexMap<String, Vec<String>> {
        let mut tab_map: IndexMap<String, Vec<String>> = IndexMap::new();
        let mut custom_without_order: IndexMap<String, Vec<String>> = IndexMap::new();
        for g in &ctx.generated {
            let Some((tabs, items)) = ctx.tab_entries.get(&g.info.name) else {
                continue;
            };
            if items.is_empty() {
                continue;
            }
            for tab in tabs {
                if ctx.tab_order.contains_key(tab) {
                    continue;
                }
                let target = if tab.starts_with(MCREATOR_PREFIX) {
                    &mut custom_without_order
                } else {
                    &mut tab_map
                };
                target
                    .entry(tab.clone())
                    .or_default()
                    .extend(items.iter().cloned());
            }
        }
        for (tab, names) in &ctx.tab_order {
            if let Some(custom) = tab.strip_prefix(MCREATOR_PREFIX)
                && ctx.env.element(custom).is_none_or(|e| e.kind != "tab")
            {
                continue;
            }
            for name in names {
                if let Some((_, items)) = ctx.tab_entries.get(name)
                    && !items.is_empty()
                {
                    tab_map
                        .entry(tab.clone())
                        .or_default()
                        .extend(items.iter().cloned());
                }
            }
        }
        // Las pestañas propias van al final, como en el editor de MCreator.
        let custom: Vec<String> = tab_map
            .keys()
            .filter(|k| k.starts_with(MCREATOR_PREFIX))
            .cloned()
            .collect();
        for k in custom {
            if let Some(v) = tab_map.shift_remove(&k) {
                tab_map.insert(k, v);
            }
        }
        for (k, v) in custom_without_order {
            tab_map.insert(k, v);
        }
        let generated: Vec<&str> = ctx.generated.iter().map(|g| g.info.name.as_str()).collect();
        tab_map.retain(|k, _| match k.strip_prefix(MCREATOR_PREFIX) {
            Some(custom) => generated.contains(&plain_name(custom).as_str()),
            None => true,
        });
        tab_map
    }
}

impl HostObject for WorkspaceInfo {
    fn get(&self, key: &str) -> Option<Value> {
        let ctx = self.ctx.clone();
        let env = ctx.env.clone();
        match key {
            "hasSounds" => constant(Value::Bool(ctx.has_sounds)),
            "hasVariables" => constant(Value::Bool(ctx.has_variables)),
            "hasJavaModels" => constant(Value::Bool(false)),
            "hasItemsInTabs" => constant(Value::Bool(ctx.generated.iter().any(|g| {
                ctx.tab_entries
                    .get(&g.info.name)
                    .is_some_and(|(tabs, items)| !tabs.is_empty() && !items.is_empty())
            }))),
            "getCreativeTabMap" => {
                let map = Self::creative_tab_map(&ctx);
                let mut out = Map::new();
                for (tab, items) in map {
                    out.insert(
                        tab,
                        Value::seq(
                            items
                                .into_iter()
                                .map(|i| Mappable::value(&env, "blocksitems", i))
                                .collect(),
                        ),
                    );
                }
                constant(Value::hash(out))
            }
            "hasProcedure" => method(move |args| {
                // Los procedures todavía no se generan: una referencia a uno no produce código.
                if let Some(name) = args
                    .first()
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.text())
                {
                    env.note(NoteCode::ProcedurePending, Some(name));
                }
                Ok(Value::Bool(false))
            }),
            "hasElementsOfType" => method(move |args| {
                let t = arg_text(args, 0);
                Ok(Value::Bool(ctx.generated.iter().any(|g| g.info.kind == t)))
            }),
            "hasElementsOfBaseType" => method(move |args| {
                let t = crate::types::base_plural(&arg_text(args, 0).to_lowercase());
                Ok(Value::Bool(
                    ctx.generated.iter().any(|g| g.base_types.contains(&t)),
                ))
            }),
            "getElementsOfType" => method(move |args| {
                let t = arg_text(args, 0);
                Ok(Value::seq(
                    ctx.generated
                        .iter()
                        .filter(|g| g.info.kind == t)
                        .map(|g| {
                            Value::object(ModElementObj {
                                info: g.info.clone(),
                                env: env.clone(),
                            })
                        })
                        .collect(),
                ))
            }),
            "getGElementsOfType" => method(move |args| {
                let t = arg_text(args, 0);
                Ok(Value::seq(
                    ctx.generated
                        .iter()
                        .filter(|g| g.info.kind == t)
                        .map(|g| g.data.clone())
                        .collect(),
                ))
            }),
            "hasModElement" => method(move |args| {
                Ok(Value::Bool(
                    env.element(&plain_name(&arg_text(args, 0))).is_some(),
                ))
            }),
            "itemBlock" => {
                method(move |args| Ok(Mappable::value(&env, "blocksitems", arg_text(args, 0))))
            }
            "resolveModNamespace" => method(move |args| {
                Ok(Value::str(resolve_mod_namespace(
                    &arg_text(args, 0),
                    &env.tokens.modid,
                )))
            }),
            "getItemTextureMap" => {
                let mut out = Map::new();
                for g in &ctx.generated {
                    if let Some(t) = g.data.get("texture") {
                        out.insert(g.info.registry_name.clone(), t);
                    }
                }
                constant(Value::hash(out))
            }
            "getWorkspace" => constant(Value::object(WorkspaceObj { ctx })),
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "WorkspaceInfo"
    }
}

/// `w.getWorkspace()` / `generator.getWorkspace()`.
pub struct WorkspaceObj {
    pub ctx: Rc<Ctx>,
}

impl HostObject for WorkspaceObj {
    fn get(&self, key: &str) -> Option<Value> {
        let ctx = self.ctx.clone();
        match key {
            "getModElementByName" => method(move |args| {
                let name = arg_text(args, 0);
                Ok(ctx
                    .env
                    .element(&name)
                    .map(|info| {
                        Value::object(ModElementObj {
                            info: info.clone(),
                            env: ctx.env.clone(),
                        })
                    })
                    .unwrap_or(Value::Null))
            }),
            "containsModElement" => {
                method(move |args| Ok(Value::Bool(ctx.env.element(&arg_text(args, 0)).is_some())))
            }
            "getWorkspaceSettings" => constant(Value::object(Settings {
                env: ctx.env.clone(),
            })),
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "Workspace"
    }
}

/// `GeneratorWrapper` (`generator` en las plantillas).
pub struct GeneratorObj {
    pub ctx: Rc<Ctx>,
}

impl HostObject for GeneratorObj {
    fn get(&self, key: &str) -> Option<Value> {
        let ctx = self.ctx.clone();
        let env = ctx.env.clone();
        match key {
            "map" => method(move |args| {
                let table = args.get(2).and_then(Value::as_num).unwrap_or(0.0) as usize;
                Ok(Value::str(env.map(
                    &arg_text(args, 0),
                    &arg_text(args, 1),
                    table,
                )))
            }),
            "getGeneratorBuildFileVersion" => constant(Value::str(env.build_file_version.as_str())),
            "getGeneratorMinecraftVersion" => constant(Value::str(env.minecraft_version.as_str())),
            "getGeneratorName" => constant(Value::str(env.generator_id.as_str())),
            "getGeneratorFlavor" => constant(Value::str(
                env.generator_id
                    .split('-')
                    .next()
                    .unwrap_or_default()
                    .to_uppercase(),
            )),
            "getElementPlainName" => method(|args| Ok(Value::str(plain_name(&arg_text(args, 0))))),
            "getRegistryNameFromFullName" | "getRegistryNameForModElement" => method(move |args| {
                let name = plain_name(&arg_text(args, 0));
                Ok(Value::str(env.registry_name(&name).unwrap_or_else(|| {
                    env.note(NoteCode::UnknownElement, Some(name.clone()));
                    crate::mapping::UNKNOWN_ELEMENT.into()
                })))
            }),
            "getResourceLocationForModElement" => method(move |args| {
                let name = plain_name(&arg_text(args, 0));
                let rn = env
                    .registry_name(&name)
                    .unwrap_or_else(|| crate::mapping::UNKNOWN_ELEMENT.into());
                Ok(Value::str(format!("{}:{rn}", env.tokens.modid)))
            }),
            "isBlock" => method(move |args| {
                let full = arg_text(args, 0);
                let name = plain_name(&full);
                let has_suffix = full.replace(MCREATOR_PREFIX, "").contains('.');
                Ok(Value::Bool(env.element(&name).is_some_and(|e| {
                    matches!(e.kind.as_str(), "block" | "plant") && !has_suffix
                })))
            }),
            "getWorkspace" => constant(Value::object(WorkspaceObj { ctx })),
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "GeneratorWrapper"
    }
}

/// `WorkspaceSettings` (`settings` en las plantillas).
pub struct Settings {
    pub env: Rc<Env>,
}

impl Settings {
    fn field(&self, name: &str) -> Option<String> {
        self.env
            .settings
            .get(name)
            .and_then(Json::as_str)
            .map(String::from)
    }

    fn set(&self, name: &str) -> Value {
        let mut items: Vec<String> = self
            .env
            .settings
            .get(name)
            .and_then(Json::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Json::as_str)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .unwrap_or_default();
        items.dedup();
        Value::seq(items.into_iter().map(Value::str).collect())
    }
}

fn opt(v: Option<String>) -> Value {
    v.map(Value::str).unwrap_or(Value::Null)
}

impl HostObject for Settings {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "getModID" => constant(Value::str(self.env.tokens.modid.as_str())),
            "getModName" => constant(opt(self.field("modName"))),
            "getVersion" => constant(opt(self.field("version"))),
            "getCleanVersion" => constant(Value::str(clean_version(
                &self.field("version").unwrap_or_default(),
            ))),
            "getDescription" => constant(opt(self.field("description"))),
            "getAuthor" => constant(opt(self.field("author"))),
            "getLicense" => constant(opt(self.field("license"))),
            "getWebsiteURL" => constant(opt(self.field("websiteURL"))),
            "getUpdateURL" => constant(opt(self.field("updateURL"))),
            "getModPicture" => constant(opt(self.field("modPicture"))),
            "getModIcon" => constant(opt(self.field("modIcon"))),
            "getCredits" => constant(opt(self.field("credits"))),
            "getCurrentGenerator" => constant(Value::str(self.env.generator_id.as_str())),
            "getModElementsPackage" => constant(Value::str(self.env.tokens.base_package.as_str())),
            "getJavaModName" => constant(Value::str(self.env.tokens.java_mod_name.as_str())),
            "isServerSideOnly" => constant(Value::Bool(
                self.env
                    .settings
                    .get("serverSideOnly")
                    .and_then(Json::as_bool)
                    .unwrap_or(false),
            )),
            "getRequiredMods" => constant(self.set("requiredMods")),
            "getDependencies" => constant(self.set("dependencies")),
            "getDependants" => constant(self.set("dependants")),
            // Sin APIs de mods registradas: MCreator usa "[0,)" si no conoce la API.
            "getMCreatorDependencies" => constant(Value::seq(vec![])),
            "getMCreatorDependenciesRaw" => constant(self.set("mcreatorDependencies")),
            "getVersionRange" => constant(Value::str("[0,)")),
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "WorkspaceSettings"
    }
}

/// `WorkspaceSettings.getCleanVersion`: la versión en formato SemVer si se puede,
/// si no solo números y puntos, y si no queda nada, `0.0.0`.
pub fn clean_version(version: &str) -> String {
    use regex::Regex;
    let edge = Regex::new(r"(\.$)|(^\.)|(-$)|(^-)|(\+$)|(^\+)").expect("regex");
    let dots = Regex::new(r"\.{2,}").expect("regex");
    let semver = Regex::new(r"^(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*|\d*[A-Za-z][0-9A-Za-z-]*))+(?:-(?:0|[1-9]\d*|\d*[A-Za-z][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|\d*[A-Za-z][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$").expect("regex");

    let mut trimmed = Regex::new(r"[^0-9a-zA-Z.+-]+")
        .expect("regex")
        .replace_all(version, ".")
        .into_owned();
    trimmed = Regex::new(r"-{2,}")
        .expect("regex")
        .replace_all(&trimmed, "-")
        .into_owned();
    trimmed = Regex::new(r"\+{2,}")
        .expect("regex")
        .replace_all(&trimmed, "+")
        .into_owned();
    trimmed = dots.replace_all(&trimmed, ".").into_owned();
    trimmed = edge.replace_all(&trimmed, "").into_owned();
    if semver.is_match(&trimmed) {
        return trimmed;
    }

    let mut clean = Regex::new(r"[^0-9.]+")
        .expect("regex")
        .replace_all(version, ".")
        .into_owned();
    clean = dots.replace_all(&clean, ".").into_owned();
    clean = edge.replace_all(&clean, "").into_owned();
    if clean.is_empty() {
        "0.0.0".into()
    } else {
        clean
    }
}

/// `JavaConventions` de MCreator.
pub struct JavaConventions;

impl HostObject for JavaConventions {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "escapeStringForJava" => method(|args| {
                Ok(match args.first() {
                    None | Some(Value::Null) => Value::Null,
                    Some(v) => Value::str(escape_java(&v.text().unwrap_or_default())),
                })
            }),
            "convertToValidClassName" => method(|args| {
                let src = arg_text(args, 0);
                let mut out = String::new();
                for c in src.chars() {
                    if out.is_empty() {
                        if c.is_ascii_alphabetic() {
                            out.push(c);
                        }
                    } else if c.is_ascii_alphanumeric() {
                        out.push(c);
                    }
                }
                let mut chars = out.chars();
                Ok(Value::str(match chars.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }))
            }),
            "isStringReservedJavaWord" => method(|args| {
                Ok(Value::Bool(
                    crate::java::RESERVED.contains(&arg_text(args, 0).to_lowercase().as_str()),
                ))
            }),
            _ => None,
        }
    }
}

/// `StringEscapeUtils.escapeJava` (commons-text): comillas, barra invertida,
/// caracteres de control y todo lo que no es ASCII como `\uXXXX`.
pub fn escape_java(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 32 || (c as u32) > 0x7f => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{:04X}", unit));
                }
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_version_like_mcreator() {
        assert_eq!(clean_version("1.0.0"), "1.0.0");
        assert_eq!(clean_version("1.0"), "1.0");
        assert_eq!(clean_version("v1.2 beta"), "1.2");
        assert_eq!(clean_version("1.0.0-beta.1"), "1.0.0-beta.1");
        assert_eq!(clean_version("alpha"), "0.0.0");
    }

    #[test]
    fn escape_java_like_commons_text() {
        assert_eq!(escape_java("Rubí \"x\"\n"), "Rub\\u00ED \\\"x\\\"\\n");
        assert_eq!(escape_java("a/b"), "a/b");
    }

    #[test]
    fn plain_names() {
        assert_eq!(plain_name("CUSTOM:RubyOre.block"), "RubyOre");
        assert_eq!(plain_name("CUSTOM:Ruby"), "Ruby");
        assert_eq!(plain_name("CUSTOM:Water:Flowing"), "Water");
    }
}
