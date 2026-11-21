//! Tipos de elemento de MCreator (`ModElementTypeLoader`) y sus nombres en plural,
//! que las plantillas usan como `types.items`, `types.livingentities`, etc.

/// Nombre de registro de cada tipo, en el orden en que MCreator los registra.
pub const ELEMENT_TYPES: &[&str] = &[
    "achievement",
    "armor",
    "armortrim",
    "attribute",
    "bannerpattern",
    "biome",
    "block",
    "command",
    "damagetype",
    "dimension",
    "code",
    "enchantment",
    "feature",
    "fluid",
    "function",
    "gamerule",
    "gui",
    "item",
    "itemextension",
    "keybind",
    "livingentity",
    "loottable",
    "overlay",
    "painting",
    "particle",
    "plant",
    "potion",
    "potioneffect",
    "procedure",
    "projectile",
    "recipe",
    "specialentity",
    "structure",
    "tab",
    "tool",
    "villagerprofession",
    "villagertrade",
    "beblock",
    "bebiome",
    "beentity",
    "beitem",
    "bescript",
];

/// Plural de un tipo de elemento (`ModElementType.getPluralName`).
pub fn element_plural(registry_name: &str) -> String {
    match registry_name {
        "livingentity" => "livingentities".into(),
        "specialentity" => "specialentities".into(),
        "beentity" => "beentities".into(),
        other => format!("{}s", other.to_lowercase()),
    }
}

/// Plural de un tipo base (`BaseType.getPluralName`).
pub fn base_plural(name: &str) -> String {
    match name.strip_suffix('y') {
        Some(stem) => format!("{stem}ies"),
        None => format!("{name}s"),
    }
}

/// Tipos base que MCreator registra siempre, además de los que declaran los generadores.
pub const CORE_BASE_TYPES: &[&str] = &["block", "entity", "configuredfeature"];

/// Tipos de elemento que MCgod ya sabe generar. Los demás se informan y se omiten.
pub const SUPPORTED: &[&str] = &["item"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plurals_match_mcreator() {
        assert_eq!(element_plural("item"), "items");
        assert_eq!(element_plural("livingentity"), "livingentities");
        assert_eq!(base_plural("configuredfeature"), "configuredfeatures");
        assert_eq!(base_plural("entity"), "entities");
        assert_eq!(base_plural("blockentity"), "blockentities");
    }
}
