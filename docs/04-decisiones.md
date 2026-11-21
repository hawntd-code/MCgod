# 04 · Registro de decisiones

Cada decisión importante del proyecto, con fecha y motivo.

## 2026-11-20 · Reescribir MCreator en Rust, no hacerle ingeniería inversa

MCreator es código abierto (GPL-3.0), así que leemos su código directamente. La herramienta REA no aplica: no soporta aplicaciones Java/JAR.

## 2026-11-20 · Licencia GPL-3.0

Usamos código y plantillas de MCreator, así que el proyecto es obra derivada y se publica como GPL-3.0. El proyecto no tiene fines de lucro.

## 2026-11-20 · No usar el nombre ni el logo de MCreator

"MCreator" y "Pylo" son marcas registradas. Su licencia pide que las distribuciones propias no las usen como nombre, pero sí que indiquen que derivan de MCreator (ver [NOTICE.md](../NOTICE.md)).

## 2026-11-20 · Paridad completa con MCreator

Todos los tipos de elemento, todos los generadores (NeoForge, Data Pack, Resource Pack, Bedrock) y todas las funciones. Se logra reutilizando los datos de sus plugins (ver [arquitectura](02-arquitectura.md)).

## 2026-11-20 · Tauri para la interfaz

Motor en Rust; interfaz en React + TypeScript mostrada en WebView2. Se eligió sobre Dioxus (todo Rust, pero menos maduro) y egui/Slint (nativo, pero obligaba a rehacer Blockly desde cero). Blockly y los visores 3D son JavaScript y funcionan directo en el WebView.

## 2026-11-20 · Interfaz moderna con ayuda en cada opción

Interfaz rediseñada, no una copia de la de MCreator. Cada opción muestra qué hace, porque en MCreator hay que ir a buscarlo.

## 2026-11-20 · Panel de IA integrado y MCP

Panel de chat a la derecha donde cada persona conecta su proveedor de IA por API key y ve a la IA trabajar en vivo. Las mismas herramientas se exponen por MCP para clientes externos.
