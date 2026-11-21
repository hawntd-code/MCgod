//! Genera `examples/rubymod` con el generador NeoForge 1.21.1 de `plugins/` y revisa
//! cada archivo. El índice de clases es un subconjunto escrito a mano de las clases
//! de Minecraft y NeoForge que usa el código generado.

use mcgod_generator::java::ImportTree;
use mcgod_generator::{Generation, Generator, NoteCode, SkipReason, Writer};
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tree() -> ImportTree {
    let mut t = ImportTree::default();
    for fqn in [
        "net.minecraft.world.item.Item",
        "net.minecraft.world.item.Rarity",
        "net.minecraft.world.item.CreativeModeTab",
        "net.minecraft.world.item.CreativeModeTabs",
        "net.minecraft.core.registries.Registries",
        "net.neoforged.neoforge.registries.DeferredRegister",
        "net.neoforged.neoforge.registries.DeferredItem",
        "net.neoforged.neoforge.event.BuildCreativeModeTabContentsEvent",
        "net.neoforged.bus.api.SubscribeEvent",
        "net.neoforged.bus.api.IEventBus",
        "net.neoforged.fml.common.EventBusSubscriber",
        "net.neoforged.fml.common.Mod",
        "net.neoforged.fml.ModContainer",
        "org.apache.logging.log4j.Logger",
        "org.apache.logging.log4j.LogManager",
        "org.apache.logging.log4j.Level",
        "java.lang.String",
        "java.util.List",
        "java.awt.List",
    ] {
        t.add(fqn);
    }
    t
}

fn generate() -> Generation {
    let ws = mcgod_workspace::Workspace::open(repo().join("examples/rubymod")).unwrap();
    let generator = Generator::load(&repo().join("plugins"), "neoforge-1.21.1").unwrap();
    generator.generate(&ws, Some(&tree())).unwrap()
}

fn file<'a>(g: &'a Generation, rel: &str) -> &'a str {
    &g.files
        .iter()
        .find(|f| f.relative == rel)
        .unwrap_or_else(|| {
            panic!(
                "no se generó {rel}; hay: {:?}",
                g.files.iter().map(|f| &f.relative).collect::<Vec<_>>()
            )
        })
        .contents
}

#[test]
fn generates_the_ruby_item_and_the_mod_base() {
    let g = generate();
    assert!(g.errors.is_empty(), "{:?}", g.errors);
    assert_eq!(g.generated, vec!["Ruby"]);

    let mut paths: Vec<&str> = g.files.iter().map(|f| f.relative.as_str()).collect();
    paths.sort();
    assert_eq!(
        paths,
        vec![
            "src/main/java/net/mcreator/rubymod/RubymodMod.java",
            "src/main/java/net/mcreator/rubymod/init/RubymodModItems.java",
            "src/main/java/net/mcreator/rubymod/init/RubymodModTabs.java",
            "src/main/java/net/mcreator/rubymod/item/RubyItem.java",
            "src/main/resources/META-INF/neoforge.mods.toml",
            "src/main/resources/assets/rubymod/lang/en_us.json",
            "src/main/resources/assets/rubymod/lang/es_mx.json",
            "src/main/resources/assets/rubymod/models/item/ruby.json",
            "src/main/resources/pack.mcmeta",
        ]
    );
    let writers: Vec<Writer> = g
        .files
        .iter()
        .filter(|f| f.relative.ends_with(".java"))
        .map(|f| f.writer)
        .collect();
    assert!(writers.iter().all(|w| *w == Writer::Java));
}

#[test]
fn ruby_item_class() {
    let g = generate();
    assert_eq!(
        file(&g, "src/main/java/net/mcreator/rubymod/item/RubyItem.java"),
        "package net.mcreator.rubymod.item;\n\n\
import net.minecraft.world.item.Rarity;\n\
import net.minecraft.world.item.Item;\n\n\
public class RubyItem extends Item {\n\
\tpublic RubyItem() {\n\
\t\tsuper(new Item.Properties().fireResistant().rarity(Rarity.UNCOMMON));\n\
\t}\n\
}"
    );
}

#[test]
fn items_registry_and_creative_tab() {
    let g = generate();
    let items = file(
        &g,
        "src/main/java/net/mcreator/rubymod/init/RubymodModItems.java",
    );
    assert!(items.starts_with("/*\n *    MCreator note: This file will be REGENERATED on each build.\n */\n\npackage net.mcreator.rubymod.init;\n\n"), "{items}");
    assert!(items.contains(
        "import net.mcreator.rubymod.item.RubyItem;\nimport net.mcreator.rubymod.RubymodMod;"
    ));
    assert!(items.contains("\tpublic static final DeferredItem<Item> RUBY;\n"));
    assert!(items.contains("\t\tRUBY = REGISTRY.register(\"ruby\", RubyItem::new);\n"));
    assert!(items.contains("// Start of user code block custom items"));

    let tabs = file(
        &g,
        "src/main/java/net/mcreator/rubymod/init/RubymodModTabs.java",
    );
    assert!(
        tabs.contains("if (tabData.getTabKey() == CreativeModeTabs.INGREDIENTS) {"),
        "{tabs}"
    );
    assert!(tabs.contains("tabData.accept(RubymodModItems.RUBY.get());"));
    // RubymodModItems está en el mismo paquete: no se importa.
    assert!(!tabs.contains("import net.mcreator.rubymod.init."));

    let main = file(&g, "src/main/java/net/mcreator/rubymod/RubymodMod.java");
    assert!(main.contains("RubymodModItems.REGISTRY.register(modEventBus);"));
    assert!(main.contains("RubymodModTabs.REGISTRY.register(modEventBus);"));
    assert!(main.contains("import net.mcreator.rubymod.init.RubymodModItems;"));
    assert!(!main.contains("import java.lang.String;"));
    assert!(!main.contains("java.awt"));
}

#[test]
fn model_and_language_files() {
    let g = generate();
    assert_eq!(
        file(
            &g,
            "src/main/resources/assets/rubymod/models/item/ruby.json"
        ),
        "{\n  \"parent\": \"item/generated\",\n  \"textures\": {\n    \"layer0\": \"rubymod:item/ruby\"\n  }\n}"
    );
    let en = file(&g, "src/main/resources/assets/rubymod/lang/en_us.json");
    assert!(en.contains("\"item.rubymod.ruby\": \"Ruby\""), "{en}");
    let es = file(&g, "src/main/resources/assets/rubymod/lang/es_mx.json");
    assert!(es.contains("\"item.rubymod.ruby\": \"Rubí\""), "{es}");
}

#[test]
fn reports_what_it_skips() {
    let g = generate();
    let reason = |name: &str| {
        g.skipped
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.reason.clone())
    };
    assert_eq!(reason("RubyBlock"), Some(SkipReason::Unsupported));
    assert_eq!(reason("GlowOnStep"), Some(SkipReason::Locked));
    assert_eq!(reason("RubyUtils"), Some(SkipReason::Locked));
    let codes: Vec<NoteCode> = g.notes.iter().map(|n| n.code).collect();
    assert!(codes.contains(&NoteCode::VariablesPending));
    assert!(codes.contains(&NoteCode::TagsPending));
    assert!(!codes.contains(&NoteCode::NoImportIndex));
}

#[test]
fn writes_only_what_changed() {
    let dir = tempfile::tempdir().unwrap();
    let src = repo().join("examples/rubymod");
    for f in ["rubymod.mcreator", "elements/Ruby.mod.json"] {
        let to = dir.path().join(f);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(src.join(f), to).unwrap();
    }
    let ws = mcgod_workspace::Workspace::open(dir.path()).unwrap();
    let generator = Generator::load(&repo().join("plugins"), "neoforge-1.21.1").unwrap();
    let first = generator
        .generate(&ws, Some(&tree()))
        .unwrap()
        .write()
        .unwrap();
    assert_eq!(first.written, 9);
    // La segunda vez el código de usuario y los imports se leen del disco y no cambia nada.
    let second = generator
        .generate(&ws, Some(&tree()))
        .unwrap()
        .write()
        .unwrap();
    assert_eq!((second.written, second.unchanged), (0, 9));
}
