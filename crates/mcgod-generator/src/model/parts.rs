//! Piezas de los elementos (`net.mcreator.element.parts`): valores traducibles
//! (`MappableElement`), texturas (`TextureHolder`) y referencias a procedures.

use super::Env;
use mcgod_ftl::{FtlError, HostObject, Value};
use std::rc::Rc;

fn method(f: impl Fn(&[Value]) -> Result<Value, FtlError> + 'static) -> Option<Value> {
    Some(Value::host_fn(f))
}

fn constant(v: Value) -> Option<Value> {
    method(move |_| Ok(v.clone()))
}

fn opt_str(s: &Option<String>) -> Value {
    s.as_deref().map(Value::str).unwrap_or(Value::Null)
}

/// Valor que se traduce con una tabla de `mappings` (`MappableElement`): pestañas,
/// animaciones, ítems y bloques de Minecraft, etc. Se escribe con su valor traducido.
pub struct Mappable {
    pub value: Option<String>,
    pub source: String,
    pub env: Rc<Env>,
}

impl Mappable {
    pub fn value(env: &Rc<Env>, source: &str, value: impl Into<String>) -> Value {
        Value::object(Mappable {
            value: Some(value.into()),
            source: source.to_string(),
            env: env.clone(),
        })
    }

    fn mapped(&self, table: usize) -> String {
        match &self.value {
            Some(v) => self.env.map(v, &self.source, table),
            None => String::new(),
        }
    }
}

impl HostObject for Mappable {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "getUnmappedValue" => constant(opt_str(&self.value)),
            "getMappedValue" => {
                let value = self.value.clone();
                let source = self.source.clone();
                let env = self.env.clone();
                method(move |args| {
                    let table = args.first().and_then(Value::as_num).unwrap_or(0.0) as usize;
                    Ok(match &value {
                        Some(v) => Value::str(env.map(v, &source, table)),
                        None => Value::Null,
                    })
                })
            }
            "toString" => constant(Value::str(self.mapped(0))),
            "isEmpty" => constant(Value::Bool(self.value.as_deref().is_none_or(str::is_empty))),
            "getMappingSource" => constant(Value::str(self.source.as_str())),
            "asTagEntry" => {
                let v = self.value.clone().unwrap_or_default();
                let tag = v
                    .strip_prefix("TAG:")
                    .map(|t| crate::mapping::resolve_mod_namespace(t, &self.env.tokens.modid))
                    .unwrap_or(v);
                constant(Value::str(tag))
            }
            "isValidReference" => {
                let valid = match self.value.as_deref() {
                    None | Some("") => false,
                    Some(v) => match v.strip_prefix(crate::mapping::MCREATOR_PREFIX) {
                        Some(name) => self.env.element(&crate::model::plain_name(name)).is_some(),
                        None => true,
                    },
                };
                constant(Value::Bool(valid))
            }
            "getDataListEntryType" => constant(Value::str("")),
            "isManaged" => constant(Value::Bool(false)),
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "valor traducible"
    }

    fn to_text(&self) -> Option<String> {
        Some(self.mapped(0))
    }
}

/// Textura de un elemento (`TextureHolder`): `"ruby"` o `"minecraft:diamond"`.
pub struct Texture {
    pub raw: Option<String>,
    pub modid: String,
}

impl Texture {
    fn namespace(&self) -> String {
        match &self.raw {
            None => String::new(),
            Some(t) => match t.split_once(':') {
                Some((ns, _)) => ns.to_string(),
                None => self.modid.clone(),
            },
        }
    }

    fn name(&self) -> String {
        match &self.raw {
            None => String::new(),
            Some(t) => match t.split_once(':') {
                Some((_, n)) => n.to_string(),
                None => t.clone(),
            },
        }
    }

    fn raw_name(&self) -> String {
        match &self.raw {
            Some(t) if !t.trim().is_empty() => t.clone(),
            _ => String::new(),
        }
    }
}

impl HostObject for Texture {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "format" => {
                let (ns, name) = (self.namespace(), self.name());
                method(move |args| {
                    // String.formatted con dos %s: espacio de nombres y nombre.
                    let pattern = args.first().and_then(Value::text).unwrap_or_default();
                    let mut out = String::new();
                    let mut values = [ns.as_str(), name.as_str()].into_iter();
                    let mut rest = pattern.as_str();
                    while let Some(i) = rest.find("%s") {
                        out.push_str(&rest[..i]);
                        out.push_str(values.next().unwrap_or(""));
                        rest = &rest[i + 2..];
                    }
                    out.push_str(rest);
                    Ok(Value::str(out))
                })
            }
            "namespace" => constant(Value::str(self.namespace())),
            "name" => constant(Value::str(self.name())),
            "getRawTextureName" | "toString" => constant(Value::str(self.raw_name())),
            "isEmpty" => constant(Value::Bool(self.raw_name().is_empty())),
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "textura"
    }

    fn to_text(&self) -> Option<String> {
        Some(self.raw_name())
    }

    fn is_empty(&self) -> bool {
        self.raw_name().is_empty()
    }
}

/// Referencia a un procedure, con o sin valor fijo (`Procedure`, `LogicProcedure`,
/// `NumberProcedure`, `StringListProcedure`…).
pub struct ProcedureRef {
    pub name: Option<String>,
    pub fixed: Value,
}

impl HostObject for ProcedureRef {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "getName" => constant(opt_str(&self.name)),
            "name" => Some(opt_str(&self.name)),
            "getFixedValue" => constant(self.fixed.clone()),
            "fixedValue" => Some(self.fixed.clone()),
            // Los procedures todavía no se generan: no tienen dependencias conocidas.
            "getDependencies" => constant(Value::seq(vec![])),
            "getReturnValueType" => constant(Value::Null),
            _ => None,
        }
    }

    fn type_name(&self) -> &str {
        "procedure"
    }

    fn to_text(&self) -> Option<String> {
        self.fixed.text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_formats_like_texture_holder() {
        let t = Texture {
            raw: Some("ruby".into()),
            modid: "rubymod".into(),
        };
        let Some(Value::Fn(f)) = t.get("format") else {
            panic!("format no es método")
        };
        assert_eq!(
            f(&[Value::str("%s:item/%s")]).unwrap(),
            Value::str("rubymod:item/ruby")
        );
        let vanilla = Texture {
            raw: Some("minecraft:diamond".into()),
            modid: "rubymod".into(),
        };
        assert_eq!(vanilla.namespace(), "minecraft");
        assert_eq!(vanilla.name(), "diamond");
        assert!(
            Texture {
                raw: Some(" ".into()),
                modid: "x".into()
            }
            .is_empty()
        );
    }
}
