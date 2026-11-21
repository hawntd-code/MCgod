//! Configuración de un generador de MCreator: `generator.yaml`, importaciones,
//! definiciones de tipos (`*.definition.yaml`), tipos base (`common.*.yaml`),
//! fragmentos (`*.fragment.yaml`) y tablas de nombres (`mappings/*.yaml`).

use crate::GenError;
use crate::types::{CORE_BASE_TYPES, ELEMENT_TYPES, base_plural};
use crate::yaml;
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct GeneratorConfig {
    /// Nombre del generador, p. ej. `neoforge-1.21.1`.
    pub id: String,
    pub yaml: Value,
    /// Carpetas `plugins/*` donde buscar archivos (como el cargador de plugins de MCreator).
    plugin_dirs: Vec<PathBuf>,
    /// Este generador y los que importa, en orden de prioridad.
    pub chain: Vec<String>,
    /// Definición de cada tipo de elemento que el generador soporta.
    pub definitions: HashMap<String, Value>,
    /// `common.<plural>.yaml`, por plural del tipo base.
    pub base_definitions: IndexMap<String, Value>,
    /// Plurales de todos los tipos base conocidos (para `types.base_*`).
    pub base_types: Vec<String>,
    pub fragments: IndexMap<String, Value>,
    pub mappings: HashMap<String, Value>,
}

impl GeneratorConfig {
    /// Carga el generador `id` desde una carpeta con plugins de MCreator
    /// (cada subcarpeta es un plugin; p. ej. `plugins/generator-1.21.1/neoforge-1.21.1`).
    pub fn load(plugins_root: &Path, id: &str) -> Result<GeneratorConfig, GenError> {
        let mut plugin_dirs: Vec<PathBuf> = std::fs::read_dir(plugins_root)
            .map_err(|e| GenError::Plugins(format!("{}: {e}", plugins_root.display())))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        plugin_dirs.sort();

        let mut cfg = GeneratorConfig {
            id: id.to_string(),
            yaml: Value::Null,
            plugin_dirs,
            chain: vec![id.to_string()],
            definitions: HashMap::new(),
            base_definitions: IndexMap::new(),
            base_types: Vec::new(),
            fragments: IndexMap::new(),
            mappings: HashMap::new(),
        };

        cfg.yaml = cfg
            .read_yaml(&format!("{id}/generator.yaml"))?
            .ok_or_else(|| GenError::UnknownGenerator(id.to_string()))?;

        if let Some(Value::Array(imports)) = cfg.yaml.get("import") {
            for imp in imports {
                let name = match imp {
                    Value::String(s) => Some(s.clone()),
                    Value::Object(o) => {
                        o.get("name").or_else(|| o.get("path")).and_then(yaml::text)
                    }
                    _ => None,
                };
                if let Some(n) = name {
                    cfg.chain.push(n);
                }
            }
        }

        // Definiciones de tipos: solo las del propio generador, como DefinitionsProvider.
        for t in ELEMENT_TYPES {
            if let Some(def) = cfg.read_yaml(&format!("{id}/{t}.definition.yaml"))?
                && def.as_object().is_some_and(|o| !o.is_empty())
            {
                cfg.definitions.insert((*t).to_string(), def);
            }
        }

        // Tipos base: los fijos de MCreator, los que declaran las definiciones y los
        // que tienen un common.<plural>.yaml.
        let mut bases: Vec<String> = CORE_BASE_TYPES.iter().map(|b| base_plural(b)).collect();
        for def in cfg.definitions.values() {
            if let Some(Value::Array(list)) = def.get("base_types") {
                for b in list {
                    let name = match b {
                        Value::Object(o) => o.get("type").and_then(yaml::text),
                        other => yaml::text(other),
                    };
                    if let Some(n) = name {
                        bases.push(base_plural(&n.to_lowercase()));
                    }
                }
            }
        }
        for (name, path) in cfg.files_matching(id, "common.", ".yaml") {
            if let Some(def) = read_yaml_file(&path)? {
                bases.push(name.clone());
                cfg.base_definitions.insert(name, def);
            }
        }
        let mut seen = std::collections::HashSet::new();
        bases.retain(|b| seen.insert(b.clone()));
        cfg.base_types = bases;

        for (name, path) in cfg.files_matching(id, "", ".fragment.yaml") {
            if let Some(def) = read_yaml_file(&path)?
                && def.as_object().is_some_and(|o| !o.is_empty())
            {
                cfg.fragments.insert(name, def);
            }
        }

        cfg.load_mappings()?;
        Ok(cfg)
    }

    /// Carpetas `<generador>/<sub>` existentes, primero las de este generador
    /// y luego las de los importados (`getGeneratorPaths`).
    pub fn paths(&self, sub: &str) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for g in &self.chain {
            for p in &self.plugin_dirs {
                let dir = p.join(g).join(sub);
                if dir.is_dir() {
                    out.push(dir);
                }
            }
        }
        out
    }

    /// Carpetas raíz de las plantillas: `templates` y después `utils`.
    pub fn template_roots(&self) -> Vec<PathBuf> {
        let mut roots = self.paths("templates");
        roots.extend(self.paths("utils"));
        roots
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.yaml.get(key).and_then(yaml::text)
    }

    /// Versión de Minecraft del generador (lo que va después del primer guion).
    pub fn minecraft_version(&self) -> String {
        self.id
            .split_once('-')
            .map(|(_, v)| v.to_string())
            .unwrap_or_default()
    }

    pub fn duplicates_whitelist(&self) -> Vec<String> {
        self.yaml
            .pointer("/import_formatter/duplicates_whitelist")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(yaml::text).collect())
            .unwrap_or_default()
    }

    /// `priority_imports` como clase → nombre completo (`Nullable` → `javax.annotation.Nullable`).
    pub fn priority_imports(&self) -> HashMap<String, String> {
        self.yaml
            .pointer("/import_formatter/priority_imports")
            .and_then(Value::as_object)
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| yaml::text(v).map(|pkg| (k.clone(), format!("{pkg}.{k}"))))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn read_yaml(&self, rel: &str) -> Result<Option<Value>, GenError> {
        for p in &self.plugin_dirs {
            let f = p.join(rel);
            if f.is_file() {
                return read_yaml_file(&f);
            }
        }
        Ok(None)
    }

    /// Archivos `<generador>/<prefix>*<suffix>` de todos los plugins, por nombre intermedio.
    fn files_matching(
        &self,
        generator: &str,
        prefix: &str,
        suffix: &str,
    ) -> Vec<(String, PathBuf)> {
        let mut out: Vec<(String, PathBuf)> = Vec::new();
        for p in &self.plugin_dirs {
            let Ok(entries) = std::fs::read_dir(p.join(generator)) else {
                continue;
            };
            for e in entries.filter_map(|e| e.ok()) {
                let file_name = e.file_name().to_string_lossy().to_string();
                if let Some(mid) = file_name
                    .strip_prefix(prefix)
                    .and_then(|r| r.strip_suffix(suffix))
                    && !mid.is_empty()
                    && !mid.contains('.')
                    && !out.iter().any(|(n, _)| n == mid)
                {
                    out.push((mid.to_string(), e.path()));
                }
            }
        }
        out.sort();
        out
    }

    /// Tablas de nombres: el generador primero; las importadas solo agregan
    /// claves que el generador no define (salvo `_merge_with_existing: false`).
    fn load_mappings(&mut self) -> Result<(), GenError> {
        for dir in self.paths("mappings") {
            let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
                .map_err(|e| GenError::Plugins(format!("{}: {e}", dir.display())))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
                .collect();
            files.sort();
            for f in files {
                let name = f
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let Some(Value::Object(new)) = read_yaml_file(&f)? else {
                    continue;
                };
                let merge = new
                    .get("_merge_with_existing")
                    .and_then(yaml::text)
                    .map(|v| v == "true")
                    .unwrap_or(true);
                match self.mappings.get_mut(&name) {
                    None => {
                        self.mappings.insert(name, Value::Object(new));
                    }
                    Some(Value::Object(existing)) if merge => {
                        let mut merged = new;
                        for (k, v) in existing.iter() {
                            merged.insert(k.clone(), v.clone());
                        }
                        *existing = merged;
                    }
                    Some(_) => {}
                }
            }
        }
        Ok(())
    }
}

fn read_yaml_file(path: &Path) -> Result<Option<Value>, GenError> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| GenError::Plugins(format!("{}: {e}", path.display())))?;
    yaml::parse(&src)
        .map(Some)
        .map_err(|e| GenError::Plugins(format!("{}: {e}", path.display())))
}
