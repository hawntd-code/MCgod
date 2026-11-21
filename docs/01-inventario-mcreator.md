# 01 · Inventario de MCreator

Radiografía del código fuente de MCreator que usamos como referencia para la reescritura.

- **Repositorio:** https://github.com/MCreator/MCreator (GPL-3.0)
- **Commit estudiado:** `c3d9446e4050a4ece89dd0fbdc64591c6e9a1f1a` (15 nov 2026)
- **Copia local:** `reference/MCreator/` (no se sube a este repositorio)

## 1. Tamaño del programa

MCreator está hecho en Java, con Swing (FlatLaf) para la interfaz y un Chromium embebido (JCEF) para mostrar Blockly.

| Paquete Java (`src/main/java/net/mcreator`) | Archivos | Líneas | Qué hace |
|---|---:|---:|---|
| `ui` | 626 | 98 241 | Toda la interfaz: editores de elementos, vistas, diálogos, MCP |
| `element` | 233 | 20 463 | Modelo de los 37 tipos de elemento de mod (+5 de Bedrock) |
| `blockly` | 112 | 10 912 | Traducción de bloques de procedures a código |
| `generator` | 53 | 6 988 | Motor de generación (carga generadores, ejecuta plantillas) |
| `workspace` | 42 | 6 812 | Formato del proyecto en disco, referencias, carpetas |
| `io` | 40 | 4 102 | Archivos, red, servidor MCP base |
| `util` | 42 | 3 942 | Utilidades |
| `minecraft` | 16 | 3 741 | Datos de Minecraft (data lists, mapeos) |
| `java` | 21 | 2 912 | Análisis de código Java del proyecto |
| `plugin` | 30 | 2 133 | Sistema de plugins y eventos |
| `preferences` | 21 | 1 512 | Preferencias |
| `gradle` | 10 | 777 | Integración con Gradle (Tooling API) |
| **Total** | | **≈ 162 500** | 60 % es interfaz |

Hay además 35 archivos de pruebas en `src/test`.

## 2. Lo más valioso son los datos

Casi todo lo que MCreator "sabe hacer" no está en el código Java, sino en archivos de datos dentro de `plugins/`. Ese es el punto clave de nuestra estrategia (ver [02-arquitectura.md](02-arquitectura.md)).

| Plugin | Archivos | Contenido |
|---|---:|---|
| `generator-1.21.1` | 1 169 | Generadores NeoForge 1.21.1 (LTS), Data Pack y Resource Pack · 1 020 plantillas `.ftl` |
| `generator-26.3.x` | 1 200 | Generadores NeoForge 26.3, Data Pack y Resource Pack · 1 047 plantillas `.ftl` |
| `generator-addon-26.5x` | 147 | Add-On de Bedrock Edition (experimental) · 119 plantillas `.ftl` |
| `mcreator-core` | 3 425 | Bloques de procedures, listas de datos de Minecraft, plantillas de elementos, triggers |
| `mcreator-localization` | 10 882 | Traducciones y **textos de ayuda** de cada opción |
| `mcreator-themes` | 743 | Temas e íconos |
| `mcreator-link` | 76 | Integración con hardware (Arduino, Raspberry Pi) |

### Generadores (todo lo que hay que soportar)

| Generador | Estado en MCreator |
|---|---|
| NeoForge 1.21.1 | LTS |
| NeoForge 26.3 | estable |
| Data Pack (1.21.1 y 26.3.x) | estable |
| Resource Pack (1.21.1 y 26.3.x) | estable |
| Add-On Bedrock 26.5x | experimental |

### `mcreator-core` por dentro

| Carpeta | Archivos | Para qué sirve |
|---|---:|---|
| `procedures` | 528 | Definición JSON de cada bloque de procedure |
| `datalists` | 1 915 | Listas de Minecraft: ítems, bloques, entidades, sonidos, partículas… |
| `templates` | 615 | Plantillas de elementos prefabricados |
| `features` | 107 | Bloques de features de generación del mundo |
| `triggers` | 65 | Eventos globales que disparan procedures |
| `aitasks` | 43 | Bloques de IA de criaturas (no es IA generativa) |
| `jstriggers` / `jsontriggers` | 55 | Triggers de Bedrock y de logros |
| `blockly` | 19 | JavaScript propio de MCreator para Blockly (campos y extensiones) |

## 3. Cómo funciona la generación de código

1. Cada generador tiene un `generator.yaml` con rutas, tareas de Gradle y plantillas base.
2. Cada tipo de elemento tiene un `<tipo>.definition.yaml` que lista qué plantillas usar, con qué condiciones y dónde escribir el resultado.
3. Las plantillas son FreeMarker (`.ftl`). Leen los campos del elemento (`data.stackSize`) y llaman métodos de Java.

Lo que las plantillas del generador NeoForge 26.3 piden al código Java:

| Dependencia | Cantidad |
|---|---:|
| Métodos distintos sobre el elemento (`data.hasInventory()`, …) | 78 |
| Métodos de ayuda (`generator.map`, `w.hasProcedure`, `thelper.clamp`, …) | 29 |
| Funciones FreeMarker definidas en `.ftl` | 44 |
| Macros FreeMarker definidas en `.ftl` | 82 |
| Usos de `?interpret` (plantillas dinámicas) | 14 |

Built-ins de FreeMarker más usados: `has_content` (275), `size` (68), `index` (50), `replace` (43), `then` (38), `lower_case` (33), `filter` (31), `starts_with` (23).

Después de generar, MCreator ordena imports y formatea el Java con JBoss Roaster.

## 4. Cómo guarda un proyecto

- `<modid>.mcreator`: archivo JSON (Gson) con la lista de elementos, variables, sonidos, tags, idiomas, carpetas y la configuración (`workspaceSettings.currentGenerator`).
- `elements/<Nombre>.mod.json`: un JSON por elemento con todos sus campos.
- Guardado atómico (escribe `.lock` y lo mueve) y copias de seguridad con fecha.

## 5. Ya tiene MCP (desde 2026)

`ui/mcp` expone **25 herramientas** por HTTP: crear/editar elementos con esquema, buscar referencias, bloques y plantillas de Blockly, compilar, abrir el juego, leer consola y logs, crear texturas, importar modelos, editar archivos del proyecto, variables, tags, data lists y textos de ayuda.

Lo que **no** tiene: chat de IA integrado, conexión a proveedores de IA, ni base de conocimiento para la IA. Eso es lo que este proyecto agrega.

## 6. Editores de elementos

`ui/modgui` tiene un editor por tipo de elemento: Achievement, Armor, ArmorTrim, Attribute, BannerPattern, Biome, Block, Command, CustomElement, CustomGUI, DamageType, Dimension, Enchantment, Feature, Fluid, Function, GameRule, Item, ItemExtension, KeyBind, LivingEntity, LootTable, Overlay, Painting, Particle, Plant, Potion, PotionEffect, Procedure, Projectile, Recipe, SpecialEntity, Structure, Tab, Tool, VillagerProfession, VillagerTrade. Bedrock: BEBiome, BEBlock, BEEntity, BEItem, BEScript.

Otros módulos de interfaz: editor de GUI visual (`wysiwyg`), editor de código (`ide`, RSyntaxTextArea), creador de texturas y armaduras, visores de modelos, búsqueda, validación, control de versiones (JGit), consola de Gradle.

## 7. Ayuda de cada opción

`mcreator-localization/help/default/` tiene **729 archivos Markdown** en inglés, uno por opción de cada editor (por ejemplo, 120 para Block y 78 para Entity). En español hay 165 (es_ES), 174 (es_AR) y 180 (es_CL). Es la semilla de la base de conocimiento.
