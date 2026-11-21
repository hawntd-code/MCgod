//! Lectura de los YAML de los generadores (`generator.yaml`, `*.definition.yaml`,
//! `mappings/*.yaml`) a `serde_json::Value`, que es con lo que trabaja el resto del crate.

use serde_json::{Map, Number, Value};
use yaml_rust2::{Yaml, YamlLoader};

/// Lee un documento YAML. Un archivo vacío da un mapa vacío, como en MCreator.
pub fn parse(src: &str) -> Result<Value, String> {
    let docs = YamlLoader::load_from_str(src).map_err(|e| e.to_string())?;
    Ok(docs
        .first()
        .map(to_json)
        .unwrap_or(Value::Object(Map::new())))
}

fn to_json(y: &Yaml) -> Value {
    match y {
        Yaml::Real(s) => s
            .parse::<f64>()
            .ok()
            .and_then(Number::from_f64)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(s.clone())),
        Yaml::Integer(i) => Value::Number((*i).into()),
        Yaml::String(s) => Value::String(s.clone()),
        Yaml::Boolean(b) => Value::Bool(*b),
        Yaml::Array(a) => Value::Array(a.iter().map(to_json).collect()),
        Yaml::Hash(h) => Value::Object(h.iter().map(|(k, v)| (key_text(k), to_json(v))).collect()),
        Yaml::Null | Yaml::BadValue | Yaml::Alias(_) => Value::Null,
    }
}

fn key_text(k: &Yaml) -> String {
    match k {
        Yaml::String(s) | Yaml::Real(s) => s.clone(),
        Yaml::Integer(i) => i.to_string(),
        Yaml::Boolean(b) => b.to_string(),
        _ => String::new(),
    }
}

/// Texto de un valor YAML escalar, como `toString()` en Java.
pub fn text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_generator_like_yaml() {
        let v = parse(
            "name: NeoForge for @minecraft (@buildfileversion)\nbuildfileversion: 21.1.265\nimport:\n  - datapack-1.21.1\ntemplates:\n  - template: item/item.java.ftl\n    condition_any:\n      - hasCustomJAVAModel()\n      - \"${data.getModels()?has_content}\"\n",
        )
        .unwrap();
        assert_eq!(v["buildfileversion"], "21.1.265");
        assert_eq!(v["import"][0], "datapack-1.21.1");
        assert_eq!(
            v["templates"][0]["condition_any"][1],
            "${data.getModels()?has_content}"
        );
    }

    #[test]
    fn empty_document_is_empty_map() {
        assert_eq!(parse("").unwrap(), Value::Object(Map::new()));
    }
}
