//! Tipo de elemento Item (`net.mcreator.element.types.Item`).
//!
//! Los valores por defecto son los del constructor de la clase; si el constructor no
//! pone uno, se usa el valor inicial del editor de MCreator (`@Numeric(init = …)`),
//! para que un `.mod.json` escrito a mano o por una IA genere un ítem válido.

use super::bean::{Bean, Field, Kind, Schema};
use mcgod_ftl::Value;

const fn f(name: &'static str, kind: Kind) -> Field {
    Field { name, kind }
}

pub static STATE_ENTRY: Schema = Schema {
    class: "Item.StateEntry",
    fields: &[
        f("renderType", Kind::Num(0.0)),
        f("texture", Kind::Texture(false)),
        f("customModelName", Kind::Str(Some("Normal"))),
        f("stateMap", Kind::Json),
    ],
    methods: Some(model_methods),
};

pub static ANIMATION_ENTRY: Schema = Schema {
    class: "Item.AnimationEntry",
    fields: &[
        f("animation", Kind::Mappable("animations", None)),
        f("speed", Kind::Num(1.0)),
        f("condition", Kind::Procedure),
    ],
    methods: None,
};

pub static ATTRIBUTE_MODIFIER: Schema = Schema {
    class: "AttributeModifierEntry",
    fields: &[
        f("attribute", Kind::Mappable("attributes", None)),
        f("amount", Kind::Num(0.0)),
        f("operation", Kind::Str(Some("ADD_VALUE"))),
        f(
            "equipmentSlot",
            Kind::Mappable("equipmentslots", Some("default")),
        ),
    ],
    methods: None,
};

pub static ITEM: Schema = Schema {
    class: "Item",
    fields: &[
        f("renderType", Kind::Num(0.0)),
        f("texture", Kind::Texture(false)),
        f("customModelName", Kind::Str(Some("Normal"))),
        f("guiTexture", Kind::Texture(true)),
        f("customProperties", Kind::ProcedureMap),
        f("states", Kind::Beans(&STATE_ENTRY)),
        f("animations", Kind::Beans(&ANIMATION_ENTRY)),
        f("name", Kind::Str(None)),
        f("rarity", Kind::Str(Some("COMMON"))),
        f("creativeTabs", Kind::MappableList("tabs")),
        f("stackSize", Kind::Num(64.0)),
        f("enchantability", Kind::Num(0.0)),
        f("useDuration", Kind::Num(0.0)),
        f("toolType", Kind::Num(1.0)),
        f("damageCount", Kind::Num(0.0)),
        f("recipeRemainder", Kind::Mappable("blocksitems", None)),
        f("destroyAnyBlock", Kind::Bool(false)),
        f("immuneToFire", Kind::Bool(false)),
        f("isPiglinCurrency", Kind::Bool(false)),
        f("repairItems", Kind::MappableList("blocksitems")),
        f("stayInGridWhenCrafting", Kind::Bool(false)),
        f("damageOnCrafting", Kind::Bool(false)),
        f("enableMeleeDamage", Kind::Bool(false)),
        f("damageVsEntity", Kind::Num(4.0)),
        f("attackSpeed", Kind::Num(1.6)),
        f("specialInformation", Kind::Procedure),
        f("glowCondition", Kind::Procedure),
        f("guiBoundTo", Kind::Str(None)),
        f("openGUIOnRightClick", Kind::Procedure),
        f("inventorySize", Kind::Num(9.0)),
        f("inventoryStackSize", Kind::Num(99.0)),
        f("onRightClickedInAir", Kind::Procedure),
        f("onRightClickedOnBlock", Kind::Procedure),
        f("onCrafted", Kind::Procedure),
        f("onEntityHitWith", Kind::Procedure),
        f("onItemInInventoryTick", Kind::Procedure),
        f("onItemInUseTick", Kind::Procedure),
        f("onStoppedUsing", Kind::Procedure),
        f("onEntitySwing", Kind::Procedure),
        f("onDroppedByPlayer", Kind::Procedure),
        f("onFinishUsingItem", Kind::Procedure),
        f("everyTickWhileUsing", Kind::Procedure),
        f("onItemEntityDestroyed", Kind::Procedure),
        f("enableRanged", Kind::Bool(false)),
        f("shootConstantly", Kind::Bool(false)),
        f("rangedItemChargesPower", Kind::Bool(false)),
        f("projectile", Kind::Mappable("projectiles", None)),
        f("projectileDisableAmmoCheck", Kind::Bool(false)),
        f("onRangedItemUsed", Kind::Procedure),
        f("rangedUseCondition", Kind::Procedure),
        f("isFood", Kind::Bool(false)),
        f("nutritionalValue", Kind::Num(4.0)),
        f("saturation", Kind::Num(0.3)),
        f("eatResultItem", Kind::Mappable("blocksitems", None)),
        f("isMeat", Kind::Bool(false)),
        f("isAlwaysEdible", Kind::Bool(false)),
        f(
            "animation",
            Kind::Mappable("itemuseanimations", Some("none")),
        ),
        f("isMusicDisc", Kind::Bool(false)),
        f("musicDiscMusic", Kind::Mappable("sounds", None)),
        f("musicDiscDescription", Kind::Str(None)),
        f("musicDiscLengthInTicks", Kind::Num(100.0)),
        f("musicDiscAnalogOutput", Kind::Num(0.0)),
        f("providedBannerPatterns", Kind::StrList),
        f("attributeModifiers", Kind::Beans(&ATTRIBUTE_MODIFIER)),
    ],
    methods: Some(item_methods),
};

fn constant(v: Value) -> Option<Value> {
    Some(Value::host_fn(move |_| Ok(v.clone())))
}

fn flag(v: bool) -> Option<Value> {
    constant(Value::Bool(v))
}

/// Tipo de modelo (`Item.decodeModelType`): 0 integrado, 1 JSON, 2 OBJ, 3 Java.
fn model_kind(b: &Bean) -> (i64, String) {
    (
        b.num("renderType") as i64,
        b.text("customModelName").unwrap_or_else(|| "Normal".into()),
    )
}

/// Métodos de modelo que comparten `Item` y `Item.StateEntry`.
fn model_methods(b: &Bean, key: &str) -> Option<Value> {
    let (kind, custom) = model_kind(b);
    match key {
        "hasNormalModel" => flag(kind == 0 && custom == "Normal"),
        "hasToolModel" => flag(kind == 0 && custom == "Tool"),
        "hasRangedItemModel" => flag(kind == 0 && custom == "Ranged item"),
        "hasCustomJSONModel" => flag(kind == 1),
        "hasCustomOBJModel" => flag(kind == 2),
        "hasCustomJAVAModel" => flag(kind == 3),
        "getTextureMap" => constant(Value::hash(Default::default())),
        _ => None,
    }
}

fn item_methods(b: &Bean, key: &str) -> Option<Value> {
    if let Some(v) = model_methods(b, key) {
        return Some(v);
    }
    let is_food = b.flag("isFood");
    let animation = b.unmapped("animation").unwrap_or_else(|| "none".into());
    let eat_result = b.unmapped("eatResultItem").unwrap_or_default();
    match key {
        "hasGUITexture" => flag(match b.value("guiTexture") {
            Value::Object(t) => !t.is_empty(),
            _ => false,
        }),
        "hasInventory" => flag(b.text("guiBoundTo").is_some_and(|s| !s.is_empty())),
        "hasNonDefaultAnimation" => flag(if is_food {
            animation != "eat"
        } else {
            animation != "none"
        }),
        "hasCustomFoodConsumable" => flag(
            is_food
                && !(b.num("useDuration") == 32.0 && (animation == "eat" || animation == "drink")),
        ),
        "hasEatResultItem" => flag(is_food && !eat_result.is_empty()),
        "hasCustomEatResultItem" => flag(is_food && eat_result.starts_with("CUSTOM:")),
        "hasBannerPatterns" => flag(b.list_len("providedBannerPatterns") > 0),
        // Los patrones de estandarte todavía no se generan.
        "getPatternDescription" => constant(Value::str("")),
        "getModels" => {
            // Solo los estados que tienen alguna propiedad (Item.getModels).
            let states = match b.value("states") {
                Value::Seq(s) => s
                    .iter()
                    .filter(|st| {
                        st.get("stateMap")
                            .is_some_and(|m| matches!(m, Value::Hash(h) if !h.is_empty()))
                    })
                    .cloned()
                    .collect(),
                _ => Vec::new(),
            };
            constant(Value::seq(states))
        }
        "getRepairItemsAsStringList" => {
            let list = if b.num("damageCount") == 0.0 {
                Vec::new()
            } else {
                match b.value("repairItems") {
                    Value::Seq(s) => s
                        .iter()
                        .filter_map(|m| match m.get("getUnmappedValue") {
                            Some(Value::Fn(f)) => f(&[]).ok(),
                            _ => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                }
            };
            constant(Value::seq(list))
        }
        "getCreativeTabs" => constant(b.value("creativeTabs")),
        "getSpecialInfoProcedure" => constant(b.value("specialInformation")),
        "getFixedSpecialInformation" => {
            // ISpecialInfoHolder: los textos fijos, si no hay procedure.
            let info = b.value("specialInformation");
            let named = info.get("name").is_some_and(|n| !n.is_null());
            let fixed = match info.get("fixedValue") {
                Some(Value::Seq(s)) if !named => s.as_ref().clone(),
                _ => Vec::new(),
            };
            constant(Value::seq(fixed))
        }
        _ => None,
    }
}

/// Pestañas creativas del ítem y lo que aporta a ellas (`ITabContainedElement`).
pub fn tab_entries(b: &Bean, name: &str) -> (Vec<String>, Vec<String>) {
    let tabs = match b.value("creativeTabs") {
        Value::Seq(s) => s
            .iter()
            .filter_map(|t| match t.get("getUnmappedValue") {
                Some(Value::Fn(f)) => f(&[]).ok().and_then(|v| v.text()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    (tabs, vec![format!("CUSTOM:{name}")])
}
