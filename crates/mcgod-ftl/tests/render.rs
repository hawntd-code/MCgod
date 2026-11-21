//! Comportamiento del intérprete, característica por característica.
//! Los resultados esperados siguen las reglas de FreeMarker 2.3 con la configuración de MCreator.

use mcgod_ftl::{Engine, FtlError, HostObject, Map, MemLoader, Value};
use std::collections::HashMap;

fn engine(files: &[(&str, &str)]) -> Engine {
    let mut m = HashMap::new();
    for (k, v) in files {
        m.insert(k.to_string(), v.to_string());
    }
    Engine::new(MemLoader(m))
}

fn render(src: &str, data: Map) -> String {
    engine(&[])
        .render_str("prueba.ftl", src, &data)
        .unwrap_or_else(|e| panic!("{e}"))
}

fn data(pairs: &[(&str, Value)]) -> Map {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

fn json(s: &str) -> Value {
    Value::from_json(&serde_json::from_str(s).unwrap())
}

#[test]
fn interpolation_uses_mcreator_number_and_boolean_formats() {
    let d = data(&[
        ("n", 64.0.into()),
        ("f", 1.2.into()),
        ("b", true.into()),
        ("s", "hola".into()),
    ]);
    assert_eq!(
        render("${n} ${f} ${b} ${s} ${1.0} ${1/3}", d),
        "64 1.2 true hola 1 0.3333333333"
    );
}

#[test]
fn if_elseif_else_and_comparisons() {
    let src = "<#if n gt 10>grande<#elseif n == 5>cinco<#else>otro</#if>";
    assert_eq!(render(src, data(&[("n", 20.0.into())])), "grande");
    assert_eq!(render(src, data(&[("n", 5.0.into())])), "cinco");
    assert_eq!(render(src, data(&[("n", 1.0.into())])), "otro");
    assert_eq!(
        render(
            "<#if (n > 3) && !(n >= 9)>ok</#if>",
            data(&[("n", 4.0.into())])
        ),
        "ok"
    );
    assert_eq!(
        render(
            "<#if s == \"A\" || s != \"B\">ok</#if>",
            data(&[("s", "A".into())])
        ),
        "ok"
    );
}

#[test]
fn list_with_sep_loop_builtins_else_and_break() {
    let d = data(&[("xs", json(r#"["a","b","c"]"#)), ("empty", json("[]"))]);
    assert_eq!(
        render("<#list xs as x>${x}<#sep>, </#list>", d.clone()),
        "a, b, c"
    );
    assert_eq!(
        render(
            "<#list xs as x>${x?index}${x?has_next?then('+','.')}</#list>",
            d.clone()
        ),
        "0+1+2."
    );
    assert_eq!(
        render(
            "<#list xs as x>${x?counter}<#if x?is_first>!</#if></#list>",
            d.clone()
        ),
        "1!23"
    );
    assert_eq!(
        render("<#list empty as x>${x}<#else>nada</#list>", d.clone()),
        "nada"
    );
    assert_eq!(
        render(
            "<#list xs as x><#if x == 'b'><#break></#if>${x}</#list>",
            d.clone()
        ),
        "a"
    );
    assert_eq!(
        render(
            "<#list 1..3 as i>${i}</#list> <#list 0..<3 as i>${i}</#list>",
            d
        ),
        "123 012"
    );
}

#[test]
fn list_over_a_hash_with_key_and_value() {
    let d = data(&[("m", json(r#"{"x":1,"y":2}"#))]);
    assert_eq!(render("<#list m as k, v>${k}=${v};</#list>", d), "x=1;y=2;");
}

#[test]
fn assign_local_global_capture_and_compound_operators() {
    let src = r#"<#assign a = 1 b = "x"><#assign a += 2><#assign b += "y"><#assign c>cap${a}</#assign>${a} ${b} ${c}"#;
    assert_eq!(render(src, Map::new()), "3 xy cap3");
    let src = "<#macro m><#local v = 'local'>${v}</#macro><#assign v = 'ns'><@m/> ${v}";
    assert_eq!(render(src, Map::new()), "local ns");
    let src = "<#macro m><#global g = 'G'></#macro><@m/>${g}";
    assert_eq!(render(src, Map::new()), "G");
}

#[test]
fn macros_with_named_positional_defaults_nested_and_catch_all() {
    let src = r#"<#macro box title size=2 extra...>[${title}:${size}<#list extra as k, v> ${k}=${v}</#list>|<#nested>]</#macro><@box title="a"/> <@box title="b" size=5 color="red">in</@box>"#;
    assert_eq!(render(src, Map::new()), "[a:2|] [b:5 color=red|in]");
    let src = "<#macro pair a b>${a}-${b}</#macro><@pair 1, 'x'/>";
    assert_eq!(render(src, Map::new()), "1-x");
    // <#nested> ve las variables de quien llama, no las de la macro.
    let src = "<#macro twice><#local x = 'macro'><#nested><#nested></#macro><#list ['A'] as x><@twice>${x}</@twice></#list>";
    assert_eq!(render(src, Map::new()), "AA");
    // Una macro se puede usar antes de su definición.
    assert_eq!(
        render("<@later/><#macro later>tarde</#macro>", Map::new()),
        "tarde"
    );
    // <#return> corta la macro.
    assert_eq!(
        render("<#macro m>a<#return>b</#macro><@m/>", Map::new()),
        "a"
    );
}

#[test]
fn functions_return_values_and_can_recurse() {
    let src = "<#function fact n><#if n lte 1><#return 1></#if><#return n * fact(n - 1)></#function>${fact(5)}";
    assert_eq!(render(src, Map::new()), "120");
    let src = "<#function greet name='mundo'><#return 'hola ' + name></#function>${greet()} ${greet('MC')}";
    assert_eq!(render(src, Map::new()), "hola mundo hola MC");
}

#[test]
fn include_is_relative_and_shares_the_namespace() {
    let e = engine(&[
        (
            "item/item.java.ftl",
            "<#include \"../util.ftl\">${helper()}|${shared}",
        ),
        (
            "util.ftl",
            "<#assign shared = 'S'><#function helper><#return 'H'></#function>",
        ),
    ]);
    assert_eq!(e.render("item/item.java.ftl", &Map::new()).unwrap(), "H|S");
}

#[test]
fn import_creates_a_namespace() {
    let e = engine(&[
        (
            "main.ftl",
            "<#import \"lib/ns.ftl\" as lib><@lib.hi who='x'/> ${lib.twice(2)} ${lib.v}",
        ),
        (
            "lib/ns.ftl",
            "<#assign v = 'V'><#macro hi who>hi ${who}</#macro><#function twice n><#return n*2></#function>",
        ),
    ]);
    assert_eq!(e.render("main.ftl", &Map::new()).unwrap(), "hi x 4 V");
}

#[test]
fn default_and_exists_operators_handle_missing_paths() {
    let d = data(&[("a", json(r#"{"b":{"c":"ok"}}"#))]);
    assert_eq!(
        render(
            "${a.b.c!'no'} ${a.x!'no'} ${(a.x.y)!'no'} ${nope!}",
            d.clone()
        ),
        "ok no no "
    );
    assert_eq!(
        render("${a.b???c} ${a.zz???c} ${nada??}", d),
        "true false false"
    );
}

#[test]
fn then_only_evaluates_the_chosen_branch() {
    assert_eq!(
        render("${true?then('si', missing.value)}", Map::new()),
        "si"
    );
}

#[test]
fn string_builtins() {
    let d = data(&[("s", "net.mcreator.rubymod.item".into())]);
    let r = render(
        "${s?keep_before('.')}|${s?keep_after('.')}|${s?keep_before_last('.')}|${s?keep_after_last('.')}|${s?replace('.', '/')}|${s?upper_case}|${s?starts_with('net')?c}|${s?contains('ruby')?c}|${s?remove_beginning('net.')}|${s?remove_ending('.item')}|${s?last_index_of('.')}|${s?substring(4, 12)}",
        d,
    );
    assert_eq!(
        r,
        "net|mcreator.rubymod.item|net.mcreator.rubymod|item|net/mcreator/rubymod/item|NET.MCREATOR.RUBYMOD.ITEM|true|true|mcreator.rubymod.item|net.mcreator.rubymod|20|mcreator"
    );
    assert_eq!(
        render(
            "${'  x '?trim}|${'a,b'?split(',')?size}|${'12.5'?number + 1}|${'Hola'?lower_case}",
            Map::new()
        ),
        "x|2|13.5|hola"
    );
    // En FreeMarker, las expresiones regulares van en textos crudos r'…' o con \\ doble.
    assert_eq!(
        render(
            r"${'item_42'?matches(r'item_\d+')?c} ${'x'?matches('\\d')?c}",
            Map::new()
        ),
        "true false"
    );
    assert_eq!(
        render("${'say \"hi\"'?c}", Map::new()),
        "\"say \\\"hi\\\"\""
    );
}

#[test]
fn sequence_builtins_and_lambdas() {
    let d = data(&[
        (
            "xs",
            json(r#"[{"n":3,"ok":true},{"n":1,"ok":false},{"n":7,"ok":true}]"#),
        ),
        ("ns", json("[4,2,9]")),
    ]);
    assert_eq!(render("${xs?filter(x -> x.ok)?size}", d.clone()), "2");
    assert_eq!(
        render("${ns?min} ${ns?max} ${ns?first} ${ns?join('-')}", d.clone()),
        "2 9 4 4-2-9"
    );
    assert_eq!(
        render("${ns?seq_contains(2)?c} ${ns?seq_contains(5)?c}", d.clone()),
        "true false"
    );
    assert_eq!(
        render(
            "<#list ns?chunk(2) as c>[${c?join(',')}]</#list>",
            d.clone()
        ),
        "[4,2][9]"
    );
    assert_eq!(
        render(
            "${ns[1]} ${ns[1..]?join(',')} ${(ns + [1])?size}",
            d.clone()
        ),
        "2 2,9 4"
    );
    assert_eq!(
        render(
            "${xs?has_content?c} ${[]?has_content?c} ${''?has_content?c} ${missing?has_content?c}",
            d
        ),
        "true false false false"
    );
}

#[test]
fn number_and_type_builtins() {
    assert_eq!(
        render(
            "${3.7?int} ${3.7?floor} ${2.5?c} ${1000000?c} ${true?string('sí','no')}",
            Map::new()
        ),
        "3 3 2.5 1000000 sí"
    );
    assert_eq!(
        render(
            "${true?is_boolean?c} ${'x'?is_string?c} ${1?is_string?c}",
            Map::new()
        ),
        "true true false"
    );
}

#[test]
fn hash_literals_merge_and_eval_json() {
    let r = render(
        r#"<#assign h = {"a": 1} + {"b": 2}>${h.a}${h["b"]} ${'{"k":[1,2]}'?eval_json.k?size}"#,
        Map::new(),
    );
    assert_eq!(r, "12 2");
}

#[test]
fn interpret_runs_in_the_callers_context() {
    let d = data(&[("tpl", "valor=${var}".into())]);
    assert_eq!(
        render("<#list ['x','y'] as var><@tpl?interpret/>;</#list>", d),
        "valor=x;valor=y;"
    );
}

struct Settings;
impl HostObject for Settings {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "getModID" => Some(Value::host_fn(|_| Ok("rubymod".into()))),
            "isServerSideOnly" => Some(Value::host_fn(|_| Ok(false.into()))),
            "getVersionRange" => Some(Value::host_fn(|args| {
                Ok(format!("[{}]", args[0].as_str().unwrap_or("?")).into())
            })),
            "version" => Some("1.0".into()),
            _ => None,
        }
    }
}

#[test]
fn host_objects_expose_methods_and_fields() {
    let d = data(&[("settings", Value::object(Settings))]);
    let r = render(
        "${settings.getModID()} ${settings.isServerSideOnly()?c} ${settings.getVersionRange('jei')} ${settings.version}",
        d,
    );
    assert_eq!(r, "rubymod false [jei] 1.0");
}

#[test]
fn javacompress_collapses_whitespace_outside_strings() {
    let src =
        "<@javacompress>\n    a(\n        b,   c\n    );\n    \"keep   this\"\n</@javacompress>";
    assert_eq!(render(src, Map::new()), "a(\nb, c\n);\n\"keep   this\"\n");
}

#[test]
fn whitespace_stripping_in_java_like_templates() {
    let src =
        "class A {\n\t<#if x>\n\tint a;\n\t</#if>\n\t<#list ys as y>\n\tint ${y};\n\t</#list>\n}\n";
    let d = data(&[("x", true.into()), ("ys", json(r#"["b","c"]"#))]);
    assert_eq!(
        render(src, d),
        "class A {\n\tint a;\n\tint b;\n\tint c;\n}\n"
    );
}

#[test]
fn errors_point_to_the_template_and_line() {
    let e = engine(&[])
        .render_str("t.ftl", "uno\ndos ${nada.campo}", &Map::new())
        .unwrap_err();
    let msg = e.to_string();
    assert!(matches!(e, FtlError::At { line: 2, .. }), "{msg}");
    assert!(msg.contains("t.ftl") && msg.contains("nada"), "{msg}");

    let e = engine(&[])
        .render_str("t.ftl", "<#if x>abierto", &Map::new())
        .unwrap_err();
    assert!(e.to_string().contains("falta </#if>"), "{e}");

    let e = engine(&[])
        .render_str("t.ftl", "<#switch x></#switch>", &Map::new())
        .unwrap_err();
    assert!(e.to_string().contains("no soportada"), "{e}");

    let e = engine(&[])
        .render("no-existe.ftl", &Map::new())
        .unwrap_err();
    assert!(matches!(e, FtlError::NotFound(_)));
}

#[test]
fn templates_are_cached_after_the_first_load() {
    let e = engine(&[("a.ftl", "${1+1}")]);
    assert_eq!(e.render("a.ftl", &Map::new()).unwrap(), "2");
    assert_eq!(e.render("a.ftl", &Map::new()).unwrap(), "2");
}
