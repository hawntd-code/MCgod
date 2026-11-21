# 03 · Hoja de ruta

Cada fase termina con algo que se puede usar y probar. Las casillas se marcan conforme se completa el trabajo.

## Fase 0 · Cimientos

- [x] Estudiar el código de MCreator ([inventario](01-inventario-mcreator.md))
- [x] Definir arquitectura ([arquitectura](02-arquitectura.md))
- [x] Instalar herramientas: Rust, Node.js, JDK 21, Git, GitHub CLI
- [x] Repositorio con licencia GPL-3.0 y aviso de origen
- [x] App Tauri que abre con el diseño base: árbol de elementos, área de trabajo, panel de IA y consola
- [x] El motor en Rust detecta el entorno (sistema y Java) y la interfaz lo muestra
- [ ] Integración continua en GitHub (compilar y probar en cada cambio)

## Fase 1 · Núcleo compatible

- [ ] `mcgod-workspace`: abrir y guardar workspaces de MCreator sin perder datos
- [ ] `mcgod-ftl`: intérprete FreeMarker con lo que usan las plantillas reales
- [ ] `mcgod-generator`: generar el tipo **Item** para NeoForge 26.3
- [ ] Pruebas golden contra MCreator real
- [ ] `mcgod-gradle`: compilar el mod y abrir Minecraft, con la consola en vivo

**Resultado:** un ítem creado a mano en un workspace se compila y aparece en el juego.

## Fase 2 · Primer flujo completo con IA

- [ ] Crear y abrir proyectos desde la app
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
