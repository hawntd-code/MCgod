//! Textos traducibles (`LocalizationUtils`): las claves que aporta cada elemento
//! (`localizationkeys` de su definición) se suman al mapa de idiomas del proyecto y
//! se escribe un `<idioma>.json` por idioma.

use crate::conditions::{self, CondData};
use indexmap::IndexMap;
use mcgod_ftl::{Engine, Map, Value};
use serde_json::Value as Json;

pub type LangMap = IndexMap<String, IndexMap<String, String>>;

/// Mapa de idiomas del proyecto (`language_map`), siempre con `en_us`.
pub fn from_workspace(raw: &serde_json::Map<String, Json>) -> LangMap {
    let mut map: LangMap = IndexMap::new();
    for (lang, entries) in raw {
        let mut m = IndexMap::new();
        if let Some(o) = entries.as_object() {
            for (k, v) in o {
                if let Some(s) = v.as_str() {
                    m.insert(k.clone(), s.to_string());
                }
            }
        }
        map.insert(lang.clone(), m);
    }
    map.entry("en_us".into()).or_default();
    map
}

/// `Workspace.setLocalization`: siempre en `en_us`; en los demás idiomas solo si falta.
pub fn set(map: &mut LangMap, key: &str, value: &str) {
    if let Some(en) = map.get_mut("en_us") {
        en.insert(key.to_string(), value.to_string());
    }
    for entries in map.values_mut() {
        entries
            .entry(key.to_string())
            .or_insert_with(|| value.to_string());
    }
}

pub fn remove(map: &mut LangMap, key: &str) {
    for entries in map.values_mut() {
        entries.shift_remove(key);
    }
}

pub struct KeyContext<'a> {
    pub name: &'a str,
    pub registry_name: &'a str,
    pub modid: &'a str,
    pub tokens: &'a crate::tokens::Tokens,
}

fn key_for(template: &str, ctx: &KeyContext, index: Option<usize>) -> String {
    let mut lc1 = ctx.name.to_string();
    if let Some(first) = lc1.get(..1) {
        lc1 = first.to_lowercase() + &lc1[1..];
    }
    let mut k = template
        .replace("@NAME", ctx.name)
        .replace("@modid", ctx.modid)
        .replace("@registryname", ctx.registry_name)
        .replace("@lc1_name", &lc1);
    if let Some(i) = index {
        k = k.replace("@item_index", &i.to_string());
    }
    ctx.tokens.replace(&k)
}

/// Aplica las `localizationkeys` de una definición para un elemento.
pub fn apply_element_keys(
    map: &mut LangMap,
    keys: &[Json],
    element: &Value,
    ctx: &KeyContext,
    engine: &Engine,
    base: &Map,
) {
    let empty = Map::new();
    for def in keys {
        let Some(key_tpl) = def.get("key").and_then(Json::as_str) else {
            continue;
        };
        // (clave, objeto sobre el que se leen condición y valor)
        let mut entries: Vec<(String, Value)> = Vec::new();
        match def.get("fromlist").and_then(Json::as_str) {
            Some(expr) => {
                let mut model = base.clone();
                model.insert("data".into(), element.clone());
                if let Ok(Value::Seq(list)) = engine.eval_expr(expr, &model) {
                    for (i, item) in list.iter().enumerate() {
                        entries.push((key_for(key_tpl, ctx, Some(i)), item.clone()));
                    }
                }
            }
            None => entries.push((key_for(key_tpl, ctx, None), element.clone())),
        }
        for (key, entry) in entries {
            let data = CondData {
                primary: &entry,
                extra: &empty,
            };
            if conditions::should_skip(def, &data, engine, base) {
                remove(map, &key);
                continue;
            }
            let mut value = match def.get("mapto").and_then(Json::as_str) {
                None => entry.text().unwrap_or_default(),
                Some(member) => conditions::member_of(&entry, member)
                    .and_then(|v| v.text())
                    .unwrap_or_default(),
            };
            if let Some(s) = def.get("suffix").and_then(Json::as_str) {
                value.push_str(s);
            }
            if let Some(p) = def.get("prefix").and_then(Json::as_str) {
                value = format!("{p}{value}");
            }
            set(map, &key, &value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_updates_english_and_fills_others() {
        let raw: serde_json::Map<String, Json> = serde_json::from_str(
            r#"{"en_us": {"item.m.ruby": "Old"}, "es_mx": {"item.m.ruby": "Rubí"}}"#,
        )
        .unwrap();
        let mut m = from_workspace(&raw);
        set(&mut m, "item.m.ruby", "Ruby");
        set(&mut m, "item.m.gem", "Gem");
        assert_eq!(m["en_us"]["item.m.ruby"], "Ruby");
        assert_eq!(m["es_mx"]["item.m.ruby"], "Rubí");
        assert_eq!(m["es_mx"]["item.m.gem"], "Gem");
    }
}
