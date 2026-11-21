//! Lectura de workspaces de MCreator.
//!
//! Un workspace es una carpeta con:
//! - `<modid>.mcreator`: lista de elementos, ajustes del mod, variables, sonidos, tags e idiomas.
//! - `elements/<Nombre>.mod.json`: la definición de cada elemento (`_fv`, `_type`, `definition`).
//!
//! El formato sale de las clases `Workspace`, `WorkspaceSettings`, `ModElement`,
//! `WorkspaceFolderManager` y `GeneratableElement` de MCreator.

pub mod model;

use model::{ElementDefinition, GeneratorId, WorkspaceFile};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const WORKSPACE_EXTENSION: &str = "mcreator";
pub const ELEMENTS_DIR: &str = "elements";

/// Tipos que no guardan `.mod.json`: el código personalizado vive en archivos Java.
const TYPES_WITHOUT_DEFINITION: &[&str] = &["code"];

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("no existe: {0}")]
    NotFound(PathBuf),
    #[error("la carpeta {0} no tiene ningún archivo .mcreator")]
    NoWorkspaceFile(PathBuf),
    #[error("la carpeta tiene varios archivos .mcreator: {0:?}")]
    Ambiguous(Vec<PathBuf>),
    #[error("no se pudo leer {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path} no es un JSON válido de MCreator: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("el proyecto no tiene un elemento llamado {0}")]
    UnknownElement(String),
    #[error("el elemento {0} no tiene archivo de definición")]
    NoDefinition(String),
}

pub type Result<T> = std::result::Result<T, WorkspaceError>;

/// Workspace abierto: ruta de la carpeta, ruta del `.mcreator` y su contenido.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub file: PathBuf,
    pub data: WorkspaceFile,
}

/// Resumen pensado para la interfaz.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    pub root: String,
    pub file: String,
    pub modid: String,
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub generator: Option<GeneratorId>,
    pub mcreator_version: i64,
    pub elements: Vec<ElementSummary>,
    pub variables: usize,
    pub sounds: usize,
    pub tags: usize,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementSummary {
    pub name: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub registry_name: Option<String>,
    pub folder: Option<String>,
    pub compiles: bool,
    pub locked_code: bool,
    /// `false` si falta `elements/<Nombre>.mod.json` en un tipo que lo necesita.
    pub has_definition: bool,
}

impl Workspace {
    /// Abre un workspace a partir del archivo `.mcreator` o de la carpeta que lo contiene.
    pub fn open(path: impl AsRef<Path>) -> Result<Workspace> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(WorkspaceError::NotFound(path.to_path_buf()));
        }
        let file = if path.is_dir() {
            find_workspace_file(path)?
        } else {
            path.to_path_buf()
        };
        let root = file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let data = read_json::<WorkspaceFile>(&file)?;
        Ok(Workspace { root, file, data })
    }

    pub fn elements_dir(&self) -> PathBuf {
        self.root.join(ELEMENTS_DIR)
    }

    pub fn element_file(&self, name: &str) -> PathBuf {
        self.elements_dir().join(format!("{name}.mod.json"))
    }

    /// Lee la definición completa de un elemento.
    pub fn read_element(&self, name: &str) -> Result<ElementDefinition> {
        let element = self
            .data
            .mod_elements
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| WorkspaceError::UnknownElement(name.to_string()))?;
        if TYPES_WITHOUT_DEFINITION.contains(&element.element_type.as_str()) {
            return Err(WorkspaceError::NoDefinition(name.to_string()));
        }
        let file = self.element_file(name);
        if !file.exists() {
            return Err(WorkspaceError::NoDefinition(name.to_string()));
        }
        read_json(&file)
    }

    pub fn summary(&self) -> WorkspaceSummary {
        let s = &self.data.workspace_settings;
        let elements = self
            .data
            .mod_elements
            .iter()
            .map(|e| ElementSummary {
                name: e.name.clone(),
                element_type: e.element_type.clone(),
                registry_name: e.registry_name.clone(),
                folder: e.path.clone(),
                compiles: e.compiles,
                locked_code: e.locked_code,
                has_definition: TYPES_WITHOUT_DEFINITION.contains(&e.element_type.as_str())
                    || self.element_file(&e.name).is_file(),
            })
            .collect();

        WorkspaceSummary {
            root: self.root.display().to_string(),
            file: self.file.display().to_string(),
            modid: s.modid.clone(),
            name: s.mod_name.clone().unwrap_or_else(|| s.modid.clone()),
            version: s.version.clone(),
            author: s.author.clone(),
            description: s.description.clone(),
            generator: s.current_generator.as_deref().map(GeneratorId::parse),
            mcreator_version: self.data.mcreator_version,
            elements,
            variables: self.data.variable_elements.len(),
            sounds: self.data.sound_elements.len(),
            tags: self.data.tag_elements.len(),
            languages: self.data.language_map.keys().cloned().collect(),
        }
    }
}

fn find_workspace_file(dir: &Path) -> Result<PathBuf> {
    let entries = fs::read_dir(dir).map_err(|source| WorkspaceError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    let mut found: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file() && p.extension().and_then(|x| x.to_str()) == Some(WORKSPACE_EXTENSION)
        })
        .collect();
    found.sort();
    match found.len() {
        0 => Err(WorkspaceError::NoWorkspaceFile(dir.to_path_buf())),
        1 => Ok(found.remove(0)),
        _ => Err(WorkspaceError::Ambiguous(found)),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).map_err(|source| WorkspaceError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    // Algunos editores guardan con BOM; Gson lo tolera, serde_json no.
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    serde_json::from_str(text).map_err(|source| WorkspaceError::Json {
        path: path.to_path_buf(),
        source,
    })
}
