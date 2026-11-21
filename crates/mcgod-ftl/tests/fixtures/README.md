# Plantillas de MCreator para pruebas

`mcreator/` tiene copias sin cambios de plantillas del generador NeoForge 1.21.1 de
[MCreator](https://github.com/MCreator/MCreator) (commit `c3d9446`, GPL-3.0 con la excepción de
plantillas `*.ftl` descrita en su `LICENSE.txt`), con finales de línea LF:

- `modbase/neoforge.mods.toml.ftl` y `modbase/mod.java.ftl`: `plugins/generator-1.21.1/neoforge-1.21.1/templates/modbase/`
- `pack.mcmeta.ftl`: `plugins/generator-1.21.1/datapack-1.21.1/templates/`

Las usa `tests/mcreator_templates.rs` para comprobar que el intérprete produce lo mismo que FreeMarker.
