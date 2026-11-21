//! Traducción de nombres de MCreator a código (`NameMapper`): `MATERIALS` →
//! `CreativeModeTabs.INGREDIENTS`, `none` → `UseAnim.NONE`, `CUSTOM:Ruby` → la
//! plantilla `_mcreator_map_template` del mapping.

use crate::tokens::Tokens;
use crate::yaml;
use serde_json::Value;
use std::collections::HashMap;

pub const MCREATOR_PREFIX: &str = "CUSTOM:";
pub const EXTERNAL_PREFIX: &str = "EXTERNAL:";
pub const MOD_NAMESPACE_PREFIX: &str = "mod:";
pub const UNKNOWN_ELEMENT: &str = "deleted_mod_element";

pub struct Mapper<'a> {
    pub mappings: &'a HashMap<String, Value>,
    pub tokens: &'a Tokens,
    /// Nombre de registro de un elemento del proyecto por su nombre.
    pub registry_name: &'a dyn Fn(&str) -> Option<String>,
}

/// `mod:algo` → `<modid>:algo`.
pub fn resolve_mod_namespace(rl: &str, modid: &str) -> String {
    match rl.strip_prefix(MOD_NAMESPACE_PREFIX) {
        Some(rest) => format!("{modid}:{rest}"),
        None => rl.to_string(),
    }
}

impl Mapper<'_> {
    pub fn map(&self, orig: &str, source: &str, table: usize) -> String {
        if orig.is_empty() {
            return String::new();
        }
        let Some(mapping) = self.mappings.get(source).and_then(Value::as_object) else {
            return orig.to_string();
        };

        if let Some(rest) = orig.strip_prefix(EXTERNAL_PREFIX) {
            return rest.to_string();
        }

        match mapping.get("_bypass_prefix") {
            Some(Value::String(p)) => {
                if let Some(rest) = orig.strip_prefix(p.as_str()) {
                    return format!("{p}{}", resolve_mod_namespace(rest, &self.tokens.modid));
                }
            }
            Some(Value::Array(list)) => {
                for p in list.iter().filter_map(Value::as_str) {
                    if let Some(rest) = orig.strip_prefix(p) {
                        return format!("{p}{}", resolve_mod_namespace(rest, &self.tokens.modid));
                    }
                }
            }
            _ => {}
        }

        if let Some(custom) = orig.strip_prefix(MCREATOR_PREFIX) {
            let mut name = custom.to_string();
            let template: Option<String> = match mapping.get("_mcreator_map_template") {
                Some(Value::String(s)) => Some(s.clone()),
                Some(Value::Array(list)) => list.get(table).and_then(yaml::text),
                Some(Value::Object(by_suffix)) => {
                    let sep = mapping
                        .get("_suffix_separator")
                        .and_then(yaml::text)
                        .unwrap_or_else(|| ".".into());
                    let (lookup, suffix) = match custom.rfind(sep.as_str()) {
                        Some(i) if !custom[i + sep.len()..].is_empty() => (
                            custom[i + sep.len()..].to_string(),
                            Some(custom[i..].to_string()),
                        ),
                        _ => ("_default".to_string(), None),
                    };
                    if let Some(suf) = suffix {
                        name = custom
                            .strip_suffix(suf.as_str())
                            .unwrap_or(custom)
                            .to_string();
                    }
                    match by_suffix.get(&lookup) {
                        Some(Value::String(s)) => Some(s.clone()),
                        Some(Value::Array(list)) => list.get(table).and_then(yaml::text),
                        _ => None,
                    }
                }
                _ => None,
            };
            // Sin plantilla, MCreator devuelve el nombre original completo (con `CUSTOM:`).
            let Some(tpl) = template else {
                return orig.to_string();
            };
            let mut out = self.tokens.replace(
                &tpl.replace("@NAME", &name)
                    .replace("@UPPERNAME", &name.to_uppercase())
                    .replace("@name", &name.to_lowercase()),
            );
            if tpl.contains("@CamelCaseName") {
                out = out.replace("@CamelCaseName", &snake_to_camel(&name));
            }
            if tpl.contains("@registryname") || tpl.contains("@REGISTRYNAME") {
                let rn = (self.registry_name)(&name).unwrap_or_else(|| UNKNOWN_ELEMENT.into());
                out = out
                    .replace("@registryname", &rn)
                    .replace("@REGISTRYNAME", &rn.to_uppercase());
            }
            return out;
        }

        lookup(mapping, orig, table)
            .or_else(|| lookup(mapping, "_default", table))
            .unwrap_or_else(|| orig.to_string())
    }
}

fn lookup(mapping: &serde_json::Map<String, Value>, key: &str, table: usize) -> Option<String> {
    match mapping.get(key) {
        Some(Value::Array(list)) => list.get(table).and_then(yaml::text),
        Some(v) if table == 0 => yaml::text(v),
        _ => None,
    }
}

/// `ruby_block` → `RubyBlock` (`StringUtils.snakeToCamel`).
pub fn snake_to_camel(s: &str) -> String {
    s.split('_')
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn tokens() -> Tokens {
        Tokens::new(
            Path::new("/ws"),
            [
                "@WORKSPACEROOT/src",
                "@WORKSPACEROOT/res",
                "@RESROOT/a",
                "@RESROOT/d",
            ],
            "rubymod",
            "RubymodMod",
            "net.mcreator.rubymod",
            None,
            None,
        )
    }

    #[test]
    fn maps_like_name_mapper() {
        let mut mappings = HashMap::new();
        mappings.insert(
            "tabs".to_string(),
            crate::yaml::parse("_default: CreativeModeTabs.BUILDING_BLOCKS\nMATERIALS: CreativeModeTabs.INGREDIENTS\n").unwrap(),
        );
        mappings.insert(
            "blocksitems".to_string(),
            crate::yaml::parse("_bypass_prefix:\n  - \"TAG:\"\n_default:\n  - Blocks.AIR\n  - air\nItems.DIAMOND:\n  - Items.DIAMOND\n  - diamond\n").unwrap(),
        );
        mappings.insert(
            "projectiles".to_string(),
            crate::yaml::parse("_mcreator_map_template:\n  - \"@NAMEEntity\"\n  - \"@JavaModNameEntities.@REGISTRYNAME.get()\"\n").unwrap(),
        );
        let t = tokens();
        let rn = |n: &str| (n == "RubyArrow").then(|| "ruby_arrow".to_string());
        let m = Mapper {
            mappings: &mappings,
            tokens: &t,
            registry_name: &rn,
        };
        assert_eq!(
            m.map("MATERIALS", "tabs", 0),
            "CreativeModeTabs.INGREDIENTS"
        );
        assert_eq!(m.map("NOPE", "tabs", 0), "CreativeModeTabs.BUILDING_BLOCKS");
        assert_eq!(m.map("Items.DIAMOND", "blocksitems", 1), "diamond");
        assert_eq!(m.map("TAG:mod:gems", "blocksitems", 0), "TAG:rubymod:gems");
        assert_eq!(m.map("CUSTOM:Ruby", "blocksitems", 0), "CUSTOM:Ruby");
        assert_eq!(
            m.map("CUSTOM:RubyArrow", "projectiles", 1),
            "RubymodModEntities.RUBY_ARROW.get()"
        );
        assert_eq!(m.map("EXTERNAL:x.Y", "tabs", 0), "x.Y");
        assert_eq!(m.map("anything", "missing_source", 0), "anything");
    }
}
