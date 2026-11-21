//! Elementos y sus partes como objetos de plantilla, a partir del JSON de `.mod.json`.
//!
//! MCreator lee cada `.mod.json` con Gson a una clase Java (`Item`, `Block`…) y las
//! plantillas leen sus campos y métodos. Aquí cada clase es un `Schema`: la lista de
//! campos con su tipo y su valor por defecto, más una función con sus métodos.

use super::parts::{Mappable, ProcedureRef, Texture};
use super::{ElementInfo, Env, ModElementObj};
use mcgod_ftl::{HostObject, Value};
use serde_json::{Map as JsonMap, Value as Json};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub enum Kind {
    /// Número; el valor es el que deja el constructor de la clase (o el inicial del editor).
    Num(f64),
    Bool(bool),
    Str(Option<&'static str>),
    /// `MappableElement` con su tabla de `mappings` y un valor por defecto.
    Mappable(&'static str, Option<&'static str>),
    MappableList(&'static str),
    /// `TextureHolder`; `true` si puede ser nulo.
    Texture(bool),
    Procedure,
    StrList,
    /// `Map<String, Procedure>`.
    ProcedureMap,
    /// Lista de objetos con su propio esquema.
    Beans(&'static Schema),
    /// Cualquier JSON, tal cual.
    Json,
}

pub struct Field {
    pub name: &'static str,
    pub kind: Kind,
}

pub type Methods = fn(&Bean, &str) -> Option<Value>;

pub struct Schema {
    pub class: &'static str,
    pub fields: &'static [Field],
    pub methods: Option<Methods>,
}

impl Schema {
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }
}

/// Objeto Java leído de JSON. Si es un elemento, `me` tiene su entrada del proyecto.
pub struct Bean {
    pub schema: &'static Schema,
    pub json: Rc<JsonMap<String, Json>>,
    pub env: Rc<Env>,
    pub me: Option<Rc<ElementInfo>>,
}

impl Bean {
    pub fn value(&self, name: &str) -> Value {
        match self.schema.field(name) {
            Some(f) => convert(f.kind, self.json.get(name), &self.env),
            None => self
                .json
                .get(name)
                .map(Value::from_json)
                .unwrap_or(Value::Null),
        }
    }

    pub fn num(&self, name: &str) -> f64 {
        self.value(name).as_num().unwrap_or(0.0)
    }

    pub fn flag(&self, name: &str) -> bool {
        self.value(name).as_bool().unwrap_or(false)
    }

    /// Texto de un campo de texto (no traduce los `MappableElement`).
    pub fn text(&self, name: &str) -> Option<String> {
        match self.value(name) {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    /// Valor sin traducir de un `MappableElement`.
    pub fn unmapped(&self, name: &str) -> Option<String> {
        match self.value(name) {
            Value::Object(o) => match o.get("getUnmappedValue") {
                Some(Value::Fn(f)) => f(&[]).ok().and_then(|v| v.text()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn list_len(&self, name: &str) -> usize {
        match self.value(name) {
            Value::Seq(s) => s.len(),
            Value::Hash(h) => h.len(),
            _ => 0,
        }
    }
}

fn getter_field(key: &str) -> Option<String> {
    let rest = key.strip_prefix("get").or_else(|| key.strip_prefix("is"))?;
    let mut c = rest.chars();
    let first = c.next()?;
    if !first.is_uppercase() {
        return None;
    }
    Some(first.to_lowercase().collect::<String>() + c.as_str())
}

impl HostObject for Bean {
    fn get(&self, key: &str) -> Option<Value> {
        if let Some(m) = self.schema.methods
            && let Some(v) = m(self, key)
        {
            return Some(v);
        }
        if key == "getModElement"
            && let Some(me) = &self.me
        {
            let obj = Value::object(ModElementObj {
                info: me.clone(),
                env: self.env.clone(),
            });
            return Some(Value::host_fn(move |_| Ok(obj.clone())));
        }
        if self.schema.field(key).is_some() || self.json.contains_key(key) {
            return Some(self.value(key));
        }
        // Getters de Java: getTexture() → texture, isFood() → isFood/food.
        if let Some(field) = getter_field(key) {
            let name = if self.schema.field(&field).is_some() || self.json.contains_key(&field) {
                Some(field)
            } else if key.starts_with("is") && self.schema.field(key).is_some() {
                Some(key.to_string())
            } else {
                None
            };
            if let Some(name) = name {
                let v = self.value(&name);
                return Some(Value::host_fn(move |_| Ok(v.clone())));
            }
        }
        None
    }

    fn type_name(&self) -> &str {
        self.schema.class
    }
}

/// Valor de plantilla de un campo según su tipo, con el valor por defecto si falta.
pub fn convert(kind: Kind, json: Option<&Json>, env: &Rc<Env>) -> Value {
    let json = json.filter(|j| !j.is_null());
    match kind {
        Kind::Num(d) => Value::Num(json.and_then(Json::as_f64).unwrap_or(d)),
        Kind::Bool(d) => Value::Bool(json.and_then(Json::as_bool).unwrap_or(d)),
        Kind::Str(d) => match json {
            Some(Json::String(s)) => Value::str(s.as_str()),
            Some(other) => Value::from_json(other),
            None => d.map(Value::str).unwrap_or(Value::Null),
        },
        Kind::Mappable(source, default) => {
            match json.and_then(mappable_value).or(default.map(String::from)) {
                Some(v) => Mappable::value(env, source, v),
                None => Value::Null,
            }
        }
        Kind::MappableList(source) => Value::seq(
            json.and_then(Json::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(mappable_value)
                        .map(|v| Mappable::value(env, source, v))
                        .collect()
                })
                .unwrap_or_default(),
        ),
        Kind::Texture(nullable) => {
            let raw = json.and_then(|j| match j {
                Json::String(s) => Some(s.clone()),
                Json::Object(o) => o.get("texture").and_then(Json::as_str).map(String::from),
                _ => None,
            });
            if raw.is_none() && nullable {
                Value::Null
            } else {
                Value::object(Texture {
                    raw,
                    modid: env.tokens.modid.clone(),
                })
            }
        }
        Kind::Procedure => json.map(procedure).unwrap_or(Value::Null),
        Kind::StrList => Value::seq(
            json.and_then(Json::as_array)
                .map(|a| a.iter().map(Value::from_json).collect())
                .unwrap_or_default(),
        ),
        Kind::ProcedureMap => {
            let mut m = mcgod_ftl::Map::new();
            if let Some(Json::Object(o)) = json {
                for (k, v) in o {
                    m.insert(k.clone(), procedure(v));
                }
            }
            Value::hash(m)
        }
        Kind::Beans(schema) => Value::seq(
            json.and_then(Json::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Json::as_object)
                        .map(|o| {
                            Value::object(Bean {
                                schema,
                                json: Rc::new(o.clone()),
                                env: env.clone(),
                                me: None,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
        ),
        Kind::Json => json.map(Value::from_json).unwrap_or(Value::Null),
    }
}

/// `"MATERIALS"` o `{"value": "MATERIALS"}` (formato anterior de MCreator).
fn mappable_value(j: &Json) -> Option<String> {
    match j {
        Json::String(s) => Some(s.clone()),
        Json::Object(o) => o.get("value").and_then(Json::as_str).map(String::from),
        _ => None,
    }
}

fn procedure(j: &Json) -> Value {
    match j {
        Json::Object(o) => Value::object(ProcedureRef {
            name: o
                .get("name")
                .and_then(Json::as_str)
                .filter(|s| !s.is_empty())
                .map(String::from),
            fixed: o
                .get("fixedValue")
                .map(Value::from_json)
                .unwrap_or(Value::Null),
        }),
        Json::String(s) => Value::object(ProcedureRef {
            name: Some(s.clone()),
            fixed: Value::Null,
        }),
        other => Value::object(ProcedureRef {
            name: None,
            fixed: Value::from_json(other),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getters_map_to_fields() {
        assert_eq!(getter_field("getTexture").as_deref(), Some("texture"));
        assert_eq!(getter_field("isFood").as_deref(), Some("food"));
        assert_eq!(getter_field("getmodels"), None);
        assert_eq!(getter_field("hasInventory"), None);
    }
}
