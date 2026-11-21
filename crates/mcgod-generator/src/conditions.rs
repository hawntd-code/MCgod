//! Condiciones de las plantillas en los `.definition.yaml` (`TemplateExpressionParser`):
//! `hasSounds()`, `isMusicDisc`, `!damageCount #= 0`, `"${data.getModels()?has_content}"`,
//! en listas `condition` (todas) o `condition_any` (alguna).

use mcgod_ftl::{Engine, Map, Value};
use serde_json::Value as Json;

/// Datos sobre los que se evalúan las condiciones: el objeto principal (`data`) y
/// otras entradas accesibles como `clave.miembro` (`CompoundConditionData`).
pub struct CondData<'a> {
    pub primary: &'a Value,
    pub extra: &'a Map,
}

/// `true` si la plantilla se debe omitir.
pub fn should_skip(template: &Json, data: &CondData, engine: &Engine, base: &Map) -> bool {
    let (raw, any) = match template.get("condition") {
        Some(c) => (c, false),
        None => match template.get("condition_any") {
            Some(c) => (c, true),
            None => return false,
        },
    };
    let conditions: Vec<String> = match raw {
        Json::Array(list) => list.iter().filter_map(crate::yaml::text).collect(),
        other => crate::yaml::text(other).into_iter().collect(),
    };
    if any {
        !conditions.iter().any(|c| check(c, data, engine, base))
    } else {
        !conditions.iter().all(|c| check(c, data, engine, base))
    }
}

/// Evalúa una condición. Un error cuenta como falso, como en MCreator.
pub fn check(condition: &str, data: &CondData, engine: &Engine, base: &Map) -> bool {
    let (negate, cond) = match condition.strip_prefix('!') {
        Some(rest) => (true, rest),
        None => (false, condition),
    };
    let result = eval(cond, data, engine, base).unwrap_or(false);
    result != negate
}

fn eval(cond: &str, data: &CondData, engine: &Engine, base: &Map) -> Option<bool> {
    if let Some(expr) = cond.strip_prefix("${").and_then(|c| c.strip_suffix('}')) {
        let mut model = base.clone();
        model.insert("data".into(), data.primary.clone());
        for (k, v) in data.extra.iter() {
            model.insert(k.clone(), v.clone());
        }
        return engine.eval_expr(expr, &model).ok()?.as_bool();
    }
    if let Some((field, values)) = cond.split_once("#?=") {
        let n = value_from(field, data)?.as_num()? as i64;
        return Some(
            values
                .trim()
                .split(',')
                .filter_map(|v| v.trim().parse::<i64>().ok())
                .any(|v| v == n),
        );
    }
    if let Some((field, value)) = cond.split_once("#=") {
        let n = value_from(field, data)?.as_num()? as i64;
        return Some(value.trim().parse::<i64>().ok()? == n);
    }
    if let Some((field, value)) = cond.split_once("#>") {
        return Some(value_from(field, data)?.as_num()? > value.trim().parse::<f64>().ok()?);
    }
    if let Some((field, value)) = cond.split_once("#<") {
        return Some(value_from(field, data)?.as_num()? < value.trim().parse::<f64>().ok()?);
    }
    if let Some((field, value)) = cond.split_once("%=") {
        return Some(value_from(field, data)?.text()? == value.trim());
    }
    value_from(cond, data)?.as_bool()
}

/// Campo (`isFood`) o método sin argumentos (`hasSounds()`) de los datos
/// (`TemplateExpressionParser.getValueFrom`). Con `clave.miembro` busca en `extra`.
pub fn value_from(member: &str, data: &CondData) -> Option<Value> {
    let member = member.trim();
    if let Some((key, rest)) = member.split_once('.') {
        let target = data.extra.get(key.trim())?;
        return member_of(target, rest.trim());
    }
    member_of(data.primary, member)
}

/// Miembro de un objeto: un campo o un método sin argumentos.
pub fn member_of(target: &Value, member: &str) -> Option<Value> {
    match member.strip_suffix("()") {
        Some(name) => match target.get(name)? {
            Value::Fn(f) => f(&[]).ok(),
            other => Some(other),
        },
        None => target.get(member),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcgod_ftl::MemLoader;

    fn data() -> Value {
        let mut m = Map::new();
        m.insert("isFood".into(), Value::Bool(true));
        m.insert("damageCount".into(), Value::Num(0.0));
        m.insert("rarity".into(), Value::str("EPIC"));
        m.insert(
            "hasSounds".into(),
            Value::host_fn(|_| Ok(Value::Bool(false))),
        );
        m.insert("models".into(), Value::seq(vec![Value::Num(1.0)]));
        Value::hash(m)
    }

    #[test]
    fn conditions_like_mcreator() {
        let engine = Engine::new(MemLoader::default());
        let primary = data();
        let extra = Map::new();
        let d = CondData {
            primary: &primary,
            extra: &extra,
        };
        let base = Map::new();
        let c = |s: &str| check(s, &d, &engine, &base);
        assert!(c("isFood"));
        assert!(!c("!isFood"));
        assert!(c("damageCount #= 0"));
        assert!(!c("!damageCount #= 0"));
        assert!(c("damageCount #?= 3, 0"));
        assert!(c("rarity %= EPIC"));
        assert!(!c("hasSounds()"));
        assert!(c("${data.models?has_content}"));
        assert!(!c("noSuchField"));
    }

    #[test]
    fn condition_lists() {
        let engine = Engine::new(MemLoader::default());
        let primary = data();
        let extra = Map::new();
        let d = CondData {
            primary: &primary,
            extra: &extra,
        };
        let base = Map::new();
        let all: Json = serde_json::json!({"condition": ["isFood", "hasSounds()"]});
        let any: Json = serde_json::json!({"condition_any": ["isFood", "hasSounds()"]});
        let none: Json = serde_json::json!({"template": "x"});
        assert!(should_skip(&all, &d, &engine, &base));
        assert!(!should_skip(&any, &d, &engine, &base));
        assert!(!should_skip(&none, &d, &engine, &base));
    }
}
