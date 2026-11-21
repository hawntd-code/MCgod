//! Intérprete compatible con FreeMarker para las plantillas de generación de MCreator.
//!
//! Implementa el subconjunto de FreeMarker 2.3 que usan los generadores de MCreator
//! (`if`, `list`, `assign`, `macro`, `function`, `include`, `import`, `sep`, built-ins,
//! lambdas, rangos, `?interpret`…) con la misma configuración que MCreator
//! (`DefaultFreemarkerConfiguration`): números con `0.##########`, booleanos `true`/`false`,
//! sin escapado automático y con recorte de espacios en líneas que solo tienen etiquetas.

pub mod ast;
mod builtins;
mod eval;
pub mod expr;
pub mod lexer;
mod parser;
pub mod value;

pub use value::{HostObject, Map, Value};

use ast::Body;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, thiserror::Error)]
pub enum FtlError {
    /// Un valor que no existe (FreeMarker lo llama "undefined"). `!` y `??` lo atrapan.
    #[error("{0}")]
    Missing(String),
    #[error("{0}")]
    Eval(String),
    #[error("{template}, línea {line}: {message}")]
    At {
        template: String,
        line: u32,
        message: String,
    },
    #[error("no existe la plantilla {0}")]
    NotFound(String),
}

impl FtlError {
    /// Agrega la plantilla y la línea, si todavía no las tiene.
    pub fn at(self, template: &str, line: u32) -> FtlError {
        match self {
            FtlError::Missing(m) | FtlError::Eval(m) => FtlError::At {
                template: template.to_string(),
                line,
                message: m,
            },
            other => other,
        }
    }
}

/// De dónde salen las plantillas (por nombre relativo a la raíz de plantillas).
pub trait Loader {
    fn load(&self, name: &str) -> Option<String>;
}

/// Busca las plantillas en varias carpetas, en orden (como el `MultiTemplateLoader` de MCreator).
pub struct DirLoader {
    pub roots: Vec<std::path::PathBuf>,
}

impl Loader for DirLoader {
    fn load(&self, name: &str) -> Option<String> {
        self.roots
            .iter()
            .find_map(|r| std::fs::read_to_string(r.join(name)).ok())
            // Las plantillas de MCreator usan LF; una copia hecha con Git en Windows puede traer CRLF.
            .map(|s| s.replace("\r\n", "\n"))
    }
}

/// Plantillas en memoria (para pruebas).
#[derive(Default)]
pub struct MemLoader(pub HashMap<String, String>);

impl Loader for MemLoader {
    fn load(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

pub struct Template {
    pub name: Rc<str>,
    pub body: Body,
}

pub struct Engine {
    loader: Box<dyn Loader>,
    cache: RefCell<HashMap<String, Rc<Template>>>,
    /// Variables disponibles en todas las plantillas (como las de `BaseDataModelProvider`).
    pub shared: Map,
}

impl Engine {
    pub fn new(loader: impl Loader + 'static) -> Engine {
        let mut shared = Map::new();
        shared.insert(
            "javacompress".into(),
            Value::directive(|_, body| Ok(java_compress(&body))),
        );
        Engine {
            loader: Box::new(loader),
            cache: RefCell::new(HashMap::new()),
            shared,
        }
    }

    pub fn set_shared(&mut self, name: &str, value: Value) {
        self.shared.insert(name.to_string(), value);
    }

    pub fn template(&self, name: &str) -> Result<Rc<Template>, FtlError> {
        if let Some(t) = self.cache.borrow().get(name) {
            return Ok(t.clone());
        }
        let src = self
            .loader
            .load(name)
            .ok_or_else(|| FtlError::NotFound(name.to_string()))?;
        let t = self.parse(name, &src)?;
        self.cache.borrow_mut().insert(name.to_string(), t.clone());
        Ok(t)
    }

    pub fn parse(&self, name: &str, src: &str) -> Result<Rc<Template>, FtlError> {
        let name: Rc<str> = name.into();
        let body = parser::parse_template(&name, src).map_err(|e| FtlError::At {
            template: name.to_string(),
            line: e.line,
            message: e.message,
        })?;
        Ok(Rc::new(Template { name, body }))
    }

    /// Procesa la plantilla `name` con los datos dados.
    pub fn render(&self, name: &str, data: &Map) -> Result<String, FtlError> {
        let t = self.template(name)?;
        self.run(&t, data)
    }

    /// Procesa un texto de plantilla que no viene del cargador.
    pub fn render_str(&self, name: &str, src: &str, data: &Map) -> Result<String, FtlError> {
        let t = self.parse(name, src)?;
        self.run(&t, data)
    }

    fn run(&self, t: &Template, data: &Map) -> Result<String, FtlError> {
        let mut run = eval::Run::new(self, data, t.name.clone());
        run.hoist(&t.body);
        run.exec(&t.body)?;
        Ok(run.out.pop().unwrap_or_default())
    }
}

/// Port de `JavaCompressDirective` de MCreator: reduce cada tramo de espacios a uno
/// (salto de línea si el tramo tenía uno, espacio si no), sin tocar el contenido de
/// los textos entre comillas, y quita los espacios del principio.
pub fn java_compress(input: &str) -> String {
    #[derive(PartialEq)]
    enum S {
        Text,
        Ws,
        Quote,
        Escaped,
    }
    let mut out = String::with_capacity(input.len());
    let mut state = S::Text;
    let mut ws_has_break = false;
    let mut seen = false;
    let flush_ws = |out: &mut String, state: &mut S, ws_has_break: &mut bool, seen: bool| {
        if *state == S::Ws {
            if seen {
                out.push(if *ws_has_break { '\n' } else { ' ' });
            }
            *ws_has_break = false;
            *state = S::Text;
        }
    };
    for c in input.chars() {
        match state {
            S::Text => {
                if c.is_whitespace() {
                    state = S::Ws;
                    ws_has_break = c == '\n' || c == '\r';
                } else if c == '"' {
                    out.push(c);
                    state = S::Quote;
                } else {
                    out.push(c);
                    seen = true;
                }
            }
            S::Ws => {
                if c.is_whitespace() {
                    if c == '\n' || c == '\r' {
                        ws_has_break = true;
                    }
                } else {
                    flush_ws(&mut out, &mut state, &mut ws_has_break, seen);
                    out.push(c);
                    if c == '"' {
                        state = S::Quote;
                    } else {
                        seen = true;
                    }
                }
            }
            S::Quote => {
                out.push(c);
                if c == '\\' {
                    state = S::Escaped;
                } else if c == '"' {
                    state = S::Text;
                    seen = true;
                }
            }
            S::Escaped => {
                out.push(c);
                state = S::Quote;
            }
        }
    }
    flush_ws(&mut out, &mut state, &mut ws_has_break, seen);
    out
}
