# MCgod

Programa para crear mods de Minecraft, con un asistente de IA dentro. Es una variante de [MCreator](https://mcreator.net/) reescrita en Rust: interfaz nueva, un panel de chat donde conectas tu propio proveedor de IA y un servidor MCP para que otros agentes trabajen sobre tu mod.

> **Estado:** fase 1 en curso. La app abre proyectos de MCreator y muestra sus elementos agrupados por tipo, con el detalle de cada uno, en español e inglés. Todavía no edita, compila ni crea mods. Ver la [hoja de ruta](docs/03-hoja-de-ruta.md).

## Objetivos

- **Paridad con MCreator:** los 37 tipos de elemento, los procedures con bloques y todos sus generadores (NeoForge, Data Pack, Resource Pack y Add-Ons de Bedrock).
- **Menos memoria:** motor en Rust e interfaz en WebView2, en lugar de Java con Swing y un Chromium embebido.
- **Asistente de IA en la app:** panel lateral como el de VS Code o JetBrains. Conectas Anthropic, OpenAI, Gemini, DeepSeek, Ollama o cualquier servicio compatible con OpenAI usando tu propia clave, y ves cada cambio que hace la IA mientras lo hace.
- **MCP:** las mismas herramientas del chat, disponibles para Claude Code, Codex, Cursor y otros clientes.
- **Ayuda en cada opción:** cada campo explica qué hace, y una base de conocimiento de modding que también consulta la IA.

## Documentación

| Documento | Contenido |
|---|---|
| [01 · Inventario de MCreator](docs/01-inventario-mcreator.md) | Radiografía del código de MCreator: tamaño, plugins, generadores, formato de proyecto |
| [02 · Arquitectura](docs/02-arquitectura.md) | Cómo está construido MCgod y por qué |
| [03 · Hoja de ruta](docs/03-hoja-de-ruta.md) | Fases del proyecto y su avance |
| [04 · Decisiones](docs/04-decisiones.md) | Registro de decisiones con fecha y motivo |
| [05 · Guía de redacción](docs/05-guia-de-redaccion.md) | Cómo se escriben los textos del proyecto y qué frases están prohibidas |

## Ejecutar en desarrollo (Windows)

Requisitos: [Rust](https://rustup.rs/) estable, [Node.js](https://nodejs.org/) LTS, [JDK 21](https://adoptium.net/) y WebView2 (ya viene con Windows 11).

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
```

Pruebas:

```powershell
npm --prefix app test
cargo test --workspace
```

Para estudiar el código de referencia de MCreator:

```powershell
git clone --depth 1 https://github.com/MCreator/MCreator.git reference/MCreator
```

## Estructura

```
app/              Interfaz (React + TypeScript + Tailwind)
app/src-tauri/    Puente entre interfaz y motor, ajustes de la app (Rust + Tauri 2)
crates/           Piezas del motor en Rust (hoy: mcgod-workspace, lector de proyectos de MCreator)
docs/             Documentación del proyecto
scripts/          Utilidades de desarrollo
reference/        Código de MCreator para consulta (no se versiona)
```

## Licencia

GPL-3.0. MCgod deriva de MCreator, de Pylo y sus colaboradores; ver [NOTICE.md](NOTICE.md). No es un producto oficial de Minecraft ni de MCreator.
