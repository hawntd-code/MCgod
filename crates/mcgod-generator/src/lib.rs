//! Generación del código y los recursos de un mod con los generadores de MCreator.
//!
//! Sigue el recorrido de `net.mcreator.generator.Generator`: plantillas base del mod
//! (`base_templates`), plantillas globales por tipo de elemento y por tipo base
//! (`global_templates`), plantillas de cada elemento (`templates`, `list_templates`),
//! archivos de idioma, y después el posproceso de cada archivo según su escritor
//! (Java con imports y formato, JSON con el formato de Gson, bloques de código del
//! usuario).

pub mod conditions;
pub mod config;
pub mod index;
pub mod java;
pub mod json;
pub mod lang;
pub mod mapping;
pub mod model;
pub mod tokens;
pub mod types;
pub mod usercode;
pub mod yaml;

use conditions::CondData;
use config::GeneratorConfig;
use indexmap::IndexMap;
use java::{ImportRules, ImportTree};
use mcgod_ftl::{DirLoader, Engine, Map, Value};
use mcgod_workspace::Workspace;
use model::bean::{Bean, Schema};
use model::{
    Ctx, ElementInfo, Env, Generated, GeneratorObj, JavaConventions, Settings, WorkspaceInfo,
};
pub use model::{Note, NoteCode};
use serde::Serialize;
use serde_json::Value as Json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tokens::Tokens;
use types::{ELEMENT_TYPES, SUPPORTED, base_plural, element_plural};

#[derive(Debug, thiserror::Error)]
pub enum GenError {
    #[error("no se pudieron leer los plugins de generación: {0}")]
    Plugins(String),
    #[error("no existe el generador {0} en los plugins")]
    UnknownGenerator(String),
    #[error("el proyecto no indica su generador (currentGenerator)")]
    NoGenerator,
    #[error("no se pudo escribir {path}: {message}")]
    Write { path: String, message: String },
}

/// Cómo se escribe cada archivo (`GeneratorFile.Writer`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Writer {
    Java,
    Json,
    File,
    Js,
}

impl Writer {
    fn from(def: Option<&str>, path: &Path) -> Writer {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match def.unwrap_or(ext) {
            "java" => Writer::Java,
            "json" => Writer::Json,
            "js" => Writer::Js,
            _ => Writer::File,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedFile {
    pub path: PathBuf,
    /// Ruta dentro del proyecto, con `/`.
    pub relative: String,
    pub writer: Writer,
    #[serde(skip)]
    pub contents: String,
    pub template: String,
    /// Elemento que produjo el archivo; `None` para los archivos del mod.
    pub element: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "detail")]
pub enum SkipReason {
    /// Código bloqueado: la persona lo mantiene a mano (MCreator tampoco lo regenera).
    Locked,
    /// MCgod todavía no genera este tipo.
    Unsupported,
    /// El generador del proyecto no tiene este tipo.
    NotInGenerator,
    /// No se pudo leer su `.mod.json`.
    NoDefinition(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    pub name: String,
    pub element_type: String,
    pub reason: SkipReason,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Generation {
    pub files: Vec<GeneratedFile>,
    /// Elementos que se generaron.
    pub generated: Vec<String>,
    pub skipped: Vec<Skipped>,
    /// Avisos: partes del proyecto que todavía no se generan, JSON inválido…
    pub notes: Vec<Note>,
    /// Plantillas que fallaron. Sus archivos no se escriben.
    pub errors: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteReport {
    pub written: usize,
    pub unchanged: usize,
}

impl Generation {
    /// Escribe los archivos que cambiaron (los iguales no se tocan, así Gradle no
    /// recompila de más).
    pub fn write(&self) -> Result<WriteReport, GenError> {
        let mut report = WriteReport::default();
        for f in &self.files {
            if std::fs::read_to_string(&f.path).is_ok_and(|old| old == f.contents) {
                report.unchanged += 1;
                continue;
            }
            let fail = |e: std::io::Error| GenError::Write {
                path: f.relative.clone(),
                message: e.to_string(),
            };
            if let Some(dir) = f.path.parent() {
                std::fs::create_dir_all(dir).map_err(fail)?;
            }
            std::fs::write(&f.path, &f.contents).map_err(fail)?;
            report.written += 1;
        }
        Ok(report)
    }
}

/// Esquema de datos de un tipo de elemento que MCgod sabe generar.
fn schema_for(kind: &str) -> Option<&'static Schema> {
    match kind {
        "item" => Some(&model::item::ITEM),
        _ => None,
    }
}

/// Pestañas creativas de un elemento y los ítems que aporta.
fn tab_entries(kind: &str, bean: &Bean, name: &str) -> (Vec<String>, Vec<String>) {
    match kind {
        "item" => model::item::tab_entries(bean, name),
        _ => (Vec::new(), Vec::new()),
    }
}

/// Plantilla pendiente de procesar.
struct Pending {
    path: PathBuf,
    def: Json,
    model: Map,
    element: Option<String>,
}

impl Pending {
    fn template(&self) -> String {
        self.def
            .get("template")
            .and_then(yaml::text)
            .unwrap_or_default()
    }
}

/// Agrega una plantilla y quita una anterior con la misma ruta (gana la última, como en MCreator).
fn push_unique(list: &mut Vec<Pending>, p: Pending) {
    list.retain(|x| x.path != p.path);
    list.push(p);
}

pub struct Generator {
    pub config: GeneratorConfig,
}

impl Generator {
    /// Carga un generador (`neoforge-1.21.1`…) de una carpeta de plugins de MCreator.
    pub fn load(plugins_root: &Path, id: &str) -> Result<Generator, GenError> {
        Ok(Generator {
            config: GeneratorConfig::load(plugins_root, id)?,
        })
    }

    /// Reglas del organizador de imports para un proyecto.
    pub fn import_rules(&self, package: &str) -> ImportRules {
        ImportRules {
            project_package: package.to_string(),
            duplicates_whitelist: self.config.duplicates_whitelist(),
            priority: self.config.priority_imports(),
        }
    }

    /// Raíz del código Java del proyecto (`source_root`).
    pub fn source_root(&self, ws: &Workspace) -> PathBuf {
        let raw = self.config.get_str("source_root").unwrap_or_default();
        let root = absolute(&ws.root);
        PathBuf::from(raw.replace("@WORKSPACEROOT", &root.to_string_lossy()))
    }

    /// Genera todo el mod en memoria. Con `tree`, los archivos Java llevan sus imports.
    pub fn generate(
        &self,
        ws: &Workspace,
        tree: Option<&ImportTree>,
    ) -> Result<Generation, GenError> {
        let settings = &ws.data.workspace_settings;
        let modid = settings.modid.clone();
        let package = settings
            .mod_elements_package
            .clone()
            .unwrap_or_else(|| format!("net.mcreator.{modid}"));
        let java_mod_name = tokens::java_mod_name(&modid);
        let settings_json = serde_json::to_value(settings).unwrap_or(Json::Null);
        let roots: Vec<String> = [
            "source_root",
            "res_root",
            "mod_assets_root",
            "mod_data_root",
        ]
        .iter()
        .map(|k| self.config.get_str(k).unwrap_or_default())
        .collect();
        let root = absolute(&ws.root);
        let tokens = Tokens::new(
            &root,
            [&roots[0], &roots[1], &roots[2], &roots[3]],
            &modid,
            &java_mod_name,
            &package,
            settings_json.get("modPicture").and_then(Json::as_str),
            settings_json.get("modIcon").and_then(Json::as_str),
        );

        let mut infos: IndexMap<String, Rc<ElementInfo>> = IndexMap::new();
        for me in &ws.data.mod_elements {
            infos.insert(
                me.name.clone(),
                Rc::new(ElementInfo {
                    name: me.name.clone(),
                    kind: me.element_type.clone(),
                    registry_name: me
                        .registry_name
                        .clone()
                        .unwrap_or_else(|| tokens::registry_name_from_camel_case(&me.name)),
                    locked_code: me.locked_code,
                    compiles: me.compiles,
                    metadata: me.metadata.clone().unwrap_or(Json::Null),
                    folder: me.path.clone(),
                }),
            );
        }

        let env = Rc::new(Env {
            tokens: tokens.clone(),
            mappings: self.config.mappings.clone(),
            elements: infos.clone(),
            generator_id: self.config.id.clone(),
            build_file_version: self.config.get_str("buildfileversion").unwrap_or_default(),
            minecraft_version: self.config.minecraft_version(),
            settings: settings_json,
            notes: RefCell::new(Vec::new()),
        });

        let mut engine = Engine::new(DirLoader {
            roots: self.config.template_roots(),
        });
        let mut out = Generation::default();

        // --- Elementos que se generan ----------------------------------------------
        let mut generated: Vec<Generated> = Vec::new();
        let mut tabs: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();
        for me in &ws.data.mod_elements {
            let info = infos[&me.name].clone();
            let skip = |reason| Skipped {
                name: me.name.clone(),
                element_type: me.element_type.clone(),
                reason,
            };
            if me.locked_code {
                out.skipped.push(skip(SkipReason::Locked));
                continue;
            }
            let (Some(schema), true) = (
                schema_for(&me.element_type),
                SUPPORTED.contains(&me.element_type.as_str()),
            ) else {
                out.skipped.push(skip(SkipReason::Unsupported));
                continue;
            };
            let Some(def) = self.config.definitions.get(&me.element_type) else {
                out.skipped.push(skip(SkipReason::NotInGenerator));
                continue;
            };
            let element = match ws.read_element(&me.name) {
                Ok(e) => e,
                Err(e) => {
                    out.skipped
                        .push(skip(SkipReason::NoDefinition(e.to_string())));
                    continue;
                }
            };
            let bean = Bean {
                schema,
                json: Rc::new(element.definition),
                env: env.clone(),
                me: Some(info.clone()),
            };
            tabs.insert(
                me.name.clone(),
                tab_entries(&me.element_type, &bean, &me.name),
            );
            let data = Value::object(bean);

            let mut base_types = Vec::new();
            if let Some(Json::Array(list)) = def.get("base_types") {
                let empty = Map::new();
                for b in list {
                    let name = match b {
                        Json::Object(o) => {
                            let cond = CondData {
                                primary: &data,
                                extra: &empty,
                            };
                            if conditions::should_skip(b, &cond, &engine, &Map::new()) {
                                continue;
                            }
                            o.get("type").and_then(yaml::text)
                        }
                        other => yaml::text(other),
                    };
                    if let Some(n) = name {
                        base_types.push(base_plural(&n.to_lowercase()));
                    }
                }
            }
            out.generated.push(me.name.clone());
            generated.push(Generated {
                info,
                data,
                base_types,
            });
        }

        if !ws.data.sound_elements.is_empty() {
            env.note(NoteCode::SoundsPending, None);
        }
        if !ws.data.variable_elements.is_empty() {
            env.note(NoteCode::VariablesPending, None);
        }
        if !ws.data.tag_elements.is_empty() {
            env.note(NoteCode::TagsPending, None);
        }

        let tab_order: IndexMap<String, Vec<String>> = ws
            .data
            .tab_element_order
            .as_ref()
            .and_then(Json::as_object)
            .map(|o| {
                o.iter()
                    .map(|(k, v)| {
                        let names = v
                            .as_array()
                            .map(|a| {
                                a.iter()
                                    .filter_map(Json::as_str)
                                    .map(String::from)
                                    .collect()
                            })
                            .unwrap_or_default();
                        (k.clone(), names)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let ctx = Rc::new(Ctx {
            env: env.clone(),
            generated,
            tab_order,
            has_sounds: false,
            has_variables: false,
            tab_entries: tabs,
        });

        // --- Modelo base (BaseDataModelProvider) ------------------------------------
        let mut base = Map::new();
        base.insert(
            "settings".into(),
            Value::object(Settings { env: env.clone() }),
        );
        base.insert("modid".into(), Value::str(modid.as_str()));
        base.insert("JavaModName".into(), Value::str(java_mod_name.as_str()));
        base.insert("package".into(), Value::str(package.as_str()));
        base.insert(
            "generator".into(),
            Value::object(GeneratorObj { ctx: ctx.clone() }),
        );
        base.insert(
            "w".into(),
            Value::object(WorkspaceInfo { ctx: ctx.clone() }),
        );
        base.insert("JavaConventions".into(), Value::object(JavaConventions));
        for (k, v) in base.iter() {
            engine.set_shared(k, v.clone());
        }

        // --- Plantillas del mod -------------------------------------------------------
        let mut pending: Vec<Pending> = Vec::new();
        let elements_of = |pred: &dyn Fn(&Generated) -> bool| -> Vec<Value> {
            ctx.generated
                .iter()
                .filter(|g| pred(g))
                .map(|g| g.data.clone())
                .collect()
        };
        // types.<plural> de los elementos de una lista (vacío para los demás tipos).
        let types_of = |list: &[&Generated]| -> Value {
            let mut m = Map::new();
            for t in ELEMENT_TYPES {
                m.insert(element_plural(t), Value::seq(vec![]));
            }
            for g in list {
                if let Some(Value::Seq(s)) = m.get(&element_plural(&g.info.kind)) {
                    let mut v = s.as_ref().clone();
                    v.push(g.data.clone());
                    m.insert(element_plural(&g.info.kind), Value::seq(v));
                }
            }
            Value::hash(m)
        };

        let mut types = Map::new();
        for t in ELEMENT_TYPES {
            types.insert(element_plural(t), Value::seq(vec![]));
        }
        for b in &self.config.base_types {
            types.insert(format!("base_{b}"), Value::seq(vec![]));
        }

        let w_value = base["w"].clone();
        let mut kinds: Vec<String> = Vec::new();
        for g in &ctx.generated {
            if !kinds.contains(&g.info.kind) {
                kinds.push(g.info.kind.clone());
            }
        }
        for kind in &kinds {
            let group: Vec<&Generated> = ctx
                .generated
                .iter()
                .filter(|g| &g.info.kind == kind)
                .collect();
            let plural = element_plural(kind);
            let list = elements_of(&|g: &Generated| &g.info.kind == kind);
            if let Some(Json::Array(defs)) = self
                .config
                .definitions
                .get(kind)
                .and_then(|d| d.get("global_templates"))
            {
                let mut extra = Map::new();
                extra.insert(plural.clone(), Value::seq(list.clone()));
                extra.insert("types".into(), types_of(&group));
                for p in self.select(defs, &w_value, &extra, &engine, &base, &tokens, None) {
                    push_unique(&mut pending, p);
                }
            }
            types.insert(plural, Value::seq(list));
        }

        let mut bases: Vec<String> = Vec::new();
        for g in &ctx.generated {
            for b in &g.base_types {
                if !bases.contains(b) {
                    bases.push(b.clone());
                }
            }
        }
        for b in &bases {
            let group: Vec<&Generated> = ctx
                .generated
                .iter()
                .filter(|g| g.base_types.contains(b))
                .collect();
            let list = elements_of(&|g: &Generated| g.base_types.contains(b));
            if let Some(Json::Array(defs)) = self
                .config
                .base_definitions
                .get(b)
                .and_then(|d| d.get("global_templates"))
            {
                let mut extra = Map::new();
                extra.insert(b.clone(), Value::seq(list.clone()));
                extra.insert("types".into(), types_of(&group));
                for p in self.select(defs, &w_value, &extra, &engine, &base, &tokens, None) {
                    push_unique(&mut pending, p);
                }
            }
            types.insert(format!("base_{b}"), Value::seq(list));
        }

        // Ningún elemento aporta fragmentos todavía: todas las listas van vacías.
        let mut fragments = Map::new();
        for f in self.config.fragments.keys() {
            fragments.insert(f.clone(), Value::seq(vec![]));
        }

        let mut extra = Map::new();
        extra.insert("types".into(), Value::hash(types));
        extra.insert("fragments".into(), Value::hash(fragments));
        if let Some(Json::Array(defs)) = self.config.yaml.get("base_templates") {
            for p in self.select(defs, &w_value, &extra, &engine, &base, &tokens, None) {
                push_unique(&mut pending, p);
            }
        }
        for p in pending.iter_mut() {
            p.model.insert("variables".into(), Value::seq(vec![]));
            p.model.insert("sounds".into(), Value::seq(vec![]));
            p.model.insert("javamodels".into(), Value::seq(vec![]));
        }

        // --- Plantillas de cada elemento --------------------------------------------
        for g in &ctx.generated {
            let def = &self.config.definitions[&g.info.kind];
            let name = g.info.name.as_str();
            let rn = g.info.registry_name.as_str();
            let mut own: Vec<Pending> = Vec::new();
            let mut model = Map::new();
            model.insert("data".into(), g.data.clone());
            model.insert("registryname".into(), Value::str(rn));
            model.insert("REGISTRYNAME".into(), Value::str(rn.to_uppercase()));
            model.insert("name".into(), Value::str(name));
            let empty = Map::new();

            if let Some(Json::Array(defs)) = def.get("templates") {
                for t in defs {
                    let raw = t.get("name").and_then(yaml::text).unwrap_or_default();
                    let raw = raw.replace("@NAME", name).replace("@registryname", rn);
                    let raw = variable_tokens(&raw, &g.data, None);
                    let Some(path) = tokens.path_in_workspace(&raw) else {
                        continue;
                    };
                    let cond = CondData {
                        primary: &g.data,
                        extra: &empty,
                    };
                    if conditions::should_skip(t, &cond, &engine, &base) {
                        continue;
                    }
                    push_unique(
                        &mut own,
                        Pending {
                            path,
                            def: t.clone(),
                            model: model.clone(),
                            element: Some(name.to_string()),
                        },
                    );
                }
            }

            if let Some(Json::Array(lists)) = def.get("list_templates") {
                for list in lists {
                    let Some(Json::Array(each)) = list.get("forEach") else {
                        continue;
                    };
                    let expr = list
                        .get("listData")
                        .and_then(yaml::text)
                        .unwrap_or_default();
                    let mut expr_model = base.clone();
                    expr_model.insert("data".into(), g.data.clone());
                    let items: Vec<Value> = match engine.eval_expr(&expr, &expr_model) {
                        Ok(Value::Seq(s)) => s.as_ref().clone(),
                        Ok(Value::Hash(h)) => h
                            .iter()
                            .map(|(k, v)| {
                                Value::object(mcgod_ftl::java::Entry {
                                    key: Value::str(k.as_str()),
                                    value: v.clone(),
                                })
                            })
                            .collect(),
                        Ok(_) => Vec::new(),
                        Err(e) => {
                            out.errors.push(format!("{name}: listData {expr}: {e}"));
                            Vec::new()
                        }
                    };
                    for (index, item) in items.iter().enumerate() {
                        for t in each {
                            let raw = t.get("name").and_then(yaml::text).unwrap_or_default();
                            let raw = raw
                                .replace("@NAME", name)
                                .replace("@registryname", rn)
                                .replace("@itemindex", &index.to_string());
                            let raw = variable_tokens(&raw, &g.data, Some(item));
                            let Some(path) = tokens.path_in_workspace(&raw) else {
                                continue;
                            };
                            let cond = CondData {
                                primary: item,
                                extra: &empty,
                            };
                            if conditions::should_skip(t, &cond, &engine, &base) {
                                continue;
                            }
                            let mut m = model.clone();
                            m.insert("data".into(), item.clone());
                            m.insert("itemindex".into(), Value::Num(index as f64));
                            m.insert("parent".into(), g.data.clone());
                            push_unique(
                                &mut own,
                                Pending {
                                    path,
                                    def: t.clone(),
                                    model: m,
                                    element: Some(name.to_string()),
                                },
                            );
                        }
                    }
                }
            }
            for p in own {
                push_unique(&mut pending, p);
            }
        }

        // --- Procesar plantillas -------------------------------------------------------
        let source_root = PathBuf::from(&tokens.source_root);
        let mut rendered: Vec<(Pending, String)> = Vec::new();
        for mut p in pending {
            if let Some(vars) = p.def.get("variables").and_then(yaml::text) {
                for v in vars.split(';') {
                    if let Some((k, val)) = v.split_once('=') {
                        p.model
                            .insert(format!("var_{}", k.trim()), Value::str(val.trim()));
                    }
                }
            }
            let template = p.template();
            match engine.render(&template, &p.model) {
                Ok(code) => rendered.push((p, code)),
                Err(e) => out.errors.push(match &p.element {
                    Some(el) => format!("{el}: {e}"),
                    None => e.to_string(),
                }),
            }
        }

        // Índice de clases: librerías + código del mod en disco + lo que se genera ahora.
        let project_tree = tree.map(|t| {
            let mut t = t.clone();
            t.add_sources(&source_root);
            for (p, _) in &rendered {
                if p.path.extension().is_some_and(|x| x == "java")
                    && let Some(fqn) = java::classpath::source_fqn(&source_root, &p.path)
                {
                    t.add(&fqn);
                }
            }
            t
        });
        if project_tree.is_none() {
            env.note(NoteCode::NoImportIndex, None);
        }
        let rules = self.import_rules(&package);

        for (p, code) in rendered {
            let writer = Writer::from(p.def.get("writer").and_then(Json::as_str), &p.path);
            let current = std::fs::read_to_string(&p.path).ok();
            let contents = match writer {
                Writer::Java => {
                    let code = usercode::process(current.as_deref(), &code, "//");
                    let code = match &project_tree {
                        Some(t) => java::arrange_imports(&code, t, &rules),
                        None => java::format::format(&code),
                    };
                    code.trim().to_string()
                }
                Writer::Js => usercode::process(current.as_deref(), &code, "//"),
                Writer::Json => match json::format(&code) {
                    Ok(j) => j,
                    Err(e) => {
                        env.note(
                            NoteCode::InvalidJson,
                            Some(format!("{}: {e}", relative(&root, &p.path))),
                        );
                        code
                    }
                },
                Writer::File => match p.def.get("usercodeComment").and_then(Json::as_str) {
                    Some(comment) => usercode::process(current.as_deref(), &code, comment),
                    None => code,
                },
            };
            out.files.push(GeneratedFile {
                relative: relative(&root, &p.path),
                path: p.path,
                writer,
                contents,
                template: p
                    .def
                    .get("template")
                    .and_then(yaml::text)
                    .unwrap_or_default(),
                element: p.element,
            });
        }

        // --- Idiomas -----------------------------------------------------------------------
        let mut langs = lang::from_workspace(&ws.data.language_map);
        for g in &ctx.generated {
            if let Some(Json::Array(keys)) = self
                .config
                .definitions
                .get(&g.info.kind)
                .and_then(|d| d.get("localizationkeys"))
            {
                let kctx = lang::KeyContext {
                    name: &g.info.name,
                    registry_name: &g.info.registry_name,
                    modid: &modid,
                    tokens: &tokens,
                };
                lang::apply_element_keys(&mut langs, keys, &g.data, &kctx, &engine, &base);
            }
        }
        if let Some(spec) = self.config.yaml.get("language_file") {
            let folder = spec
                .get("root_folder")
                .and_then(yaml::text)
                .unwrap_or_default();
            let file = spec
                .get("langfile_name")
                .and_then(yaml::text)
                .unwrap_or_else(|| "@langname.json".into());
            if spec.get("format").and_then(Json::as_str) == Some("json") {
                for (code, entries) in &langs {
                    let raw = format!("{folder}{}", file.replace("@langname", code));
                    let Some(path) = tokens.path_in_workspace(&raw) else {
                        continue;
                    };
                    let obj: serde_json::Map<String, Json> = entries
                        .iter()
                        .map(|(k, v)| (k.clone(), Json::String(v.clone())))
                        .collect();
                    out.files.push(GeneratedFile {
                        relative: relative(&root, &path),
                        path,
                        writer: Writer::Json,
                        contents: json::pretty(&Json::Object(obj)),
                        template: "language_file".into(),
                        element: None,
                    });
                }
            } else {
                env.note(NoteCode::LangFormatPending, None);
            }
        }

        out.notes = env.notes.borrow().clone();
        Ok(out)
    }

    /// Plantillas de una lista que se generan (ruta dentro del proyecto y condición cumplida).
    #[allow(clippy::too_many_arguments)]
    fn select(
        &self,
        defs: &[Json],
        primary: &Value,
        extra: &Map,
        engine: &Engine,
        base: &Map,
        tokens: &Tokens,
        element: Option<&str>,
    ) -> Vec<Pending> {
        let mut out: Vec<Pending> = Vec::new();
        for t in defs {
            let raw = t.get("name").and_then(yaml::text).unwrap_or_default();
            let Some(path) = tokens.path_in_workspace(&raw) else {
                continue;
            };
            let cond = CondData { primary, extra };
            if conditions::should_skip(t, &cond, engine, base) {
                continue;
            }
            let mut model = Map::new();
            for (k, v) in extra.iter() {
                if k != "data" {
                    model.insert(k.clone(), v.clone());
                }
            }
            push_unique(
                &mut out,
                Pending {
                    path,
                    def: t.clone(),
                    model,
                    element: element.map(String::from),
                },
            );
        }
        out
    }
}

/// `@[campo]` y `@[item.campo]` en las rutas (`GeneratorTokens.replaceVariableTokens`).
fn variable_tokens(raw: &str, element: &Value, item: Option<&Value>) -> String {
    let re = regex::Regex::new(r"@\[(.*?)]").expect("regex");
    re.replace_all(raw, |c: &regex::Captures| {
        let member = &c[1];
        let empty = Map::new();
        let value = match member.strip_prefix("item.") {
            Some(m) => item.and_then(|i| {
                conditions::value_from(
                    m,
                    &CondData {
                        primary: i,
                        extra: &empty,
                    },
                )
            }),
            None => conditions::value_from(
                member,
                &CondData {
                    primary: element,
                    extra: &empty,
                },
            ),
        };
        value
            .and_then(|v| v.text())
            .unwrap_or_else(|| "null".into())
    })
    .into_owned()
}

fn absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

fn relative(root: &Path, path: &Path) -> String {
    let root = tokens::normalize(root);
    path.strip_prefix(&root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
