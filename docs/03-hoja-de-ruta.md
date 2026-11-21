# 03 · Hoja de ruta

Cada fase termina con algo que se puede usar y probar. Las casillas se marcan conforme se completa el trabajo.

## Fase 0 · Cimientos

- [x] Estudiar el código de MCreator ([inventario](01-inventario-mcreator.md))
- [x] Definir arquitectura ([arquitectura](02-arquitectura.md))
- [x] Instalar herramientas: Rust, Node.js, JDK 21, Git, GitHub CLI
- [x] Repositorio con licencia GPL-3.0 y aviso de origen
- [x] App Tauri que abre con el diseño base: árbol de elementos, área de trabajo, panel de IA y consola
- [x] El motor en Rust detecta el entorno (sistema y Java) y la interfaz lo muestra
- [x] Interfaz en español e inglés
- [x] Guía de redacción con prueba automática
- [x] Pruebas de interfaz (Vitest) y del motor (cargo test)
- [x] Integración continua en GitHub (compilar y probar en cada cambio)

## Fase 1 · Núcleo compatible

- [x] `mcgod-workspace`: leer workspaces de MCreator (`.mcreator` y `elements/*.mod.json`) sin perder campos
- [x] Bienvenida solo en el primer arranque; después, pantalla de proyectos con recientes
- [x] Abrir un proyecto: árbol de elementos por tipo, resumen del mod y vista de cada elemento
- [x] `mcgod-gradle`: compilar el mod y abrir Minecraft desde la app, con la consola en vivo y botón para detener
- [x] Proyecto de ejemplo `examples/rubymod` que compila y abre Minecraft con el mod cargado
- [ ] `mcgod-workspace`: guardar cambios con el mismo formato
- [x] `mcgod-ftl`: intérprete FreeMarker con lo que usan las plantillas reales (analiza las 2,212 plantillas de los plugins de MCreator; tres plantillas reales producen el mismo resultado que FreeMarker)
- [ ] `mcgod-generator`: generar el tipo **Item** para NeoForge 1.21.1 (el generador LTS de MCreator)
- [ ] Pruebas golden contra MCreator real

**Resultado:** un ítem creado a mano en un workspace se compila y aparece en el juego.

## Fase 2 · Primer flujo completo con IA

- [ ] Crear proyectos nuevos desde la app
- [ ] Editores visuales de Item, Block, Recipe y Tab, con ayuda en cada opción
- [ ] Editor y visor de texturas
- [ ] Herramientas compartidas de IA (modelo de las 25 de MCreator)
- [ ] Panel de chat: conectar proveedor por API key y ver a la IA trabajar en vivo
- [ ] Servidor MCP con las mismas herramientas

**Resultado:** le pides a la IA "haz un rubí, su bloque y su receta", lo ves construirse y lo pruebas en el juego.

## Fase 3 · Procedures

- [ ] Blockly integrado con los 528 bloques y el JavaScript de MCreator
- [ ] `mcgod-blockly`: procedures → código
- [ ] Triggers globales, variables, dependencias

## Fase 4 · Todos los elementos y generadores

- [ ] Los 37 tipos de elemento de Java Edition
- [ ] Data Pack y Resource Pack
- [ ] Add-On de Bedrock (5 tipos)
- [ ] NeoForge 1.21.1 (LTS)

## Fase 5 · Editores y visores

- [ ] Visor 3D de modelos (OBJ, Java, Blockbench)
- [ ] Editor de animaciones
- [ ] Editor visual de GUIs y overlays
- [ ] Editor de código (Monaco) con elementos bloqueados
- [ ] Búsqueda global, referencias rotas, validación

## Fase 6 · Base de conocimiento y acabado

- [ ] Base de conocimiento: ayuda de MCreator + guías de modding, consultable por la IA y por la persona
- [ ] Traducciones (empezando por español e inglés)
- [ ] Temas, plugins compatibles con MCreator
- [ ] Control de versiones del proyecto, exportar, instaladores y actualizaciones
